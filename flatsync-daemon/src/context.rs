use crate::Error;
use diff::Diff;
use libflatpak::{gio, prelude::*};
use libflatsync_common::{
    FlatpakInstallationKind, FlatpakInstallationPayload, FlatpakRef, FlatpakRemote,
};
use log::{debug, trace};
use std::{collections::HashSet, path::PathBuf};

/// ## `Context`
/// Holds variables that are used throughout the daemon's lifetime.
pub struct Context {
    local_installations: FlatpakInstallationPayload,
}

impl Context {
    pub fn new() -> Result<Self, Error> {
        Self::init_local_installations_file()?;

        let local_installations_file_path = Self::get_local_installations_file();
        let local_installations =
            match FlatpakInstallationPayload::new_from_file(local_installations_file_path) {
                Ok(payload) => payload,
                Err(_e) => FlatpakInstallationPayload::new_from_system()
                    .map_err(Error::FlatpakInstallationQueryFailure)?,
            };

        Ok(Self {
            local_installations,
        })
    }

    /// ## `refresh_local_installations()`
    /// Gets the current local installations and compares them to the `local_installations` member.
    ///
    /// Updates the `local_installations` member of the `Context` struct aswell as the local file's content, should they differ.
    pub fn refresh_local_installations(&mut self) -> Result<(), Error> {
        let cur = FlatpakInstallationPayload::new_from_system()
            .map_err(Error::FlatpakInstallationQueryFailure)?;

        if self.installations_changed(&cur) {
            debug!(
                "Local installations changed, timestamps:\nCurrent: {:?}\nMember: {:?}",
                cur.altered_at, self.local_installations.altered_at
            );
            self.set_cache_and_file(cur)?;
        }

        Ok(())
    }

    /// ## `installations_changed()`
    /// Used to check whether or not our local installations differ from the `installations` member of a given `FlatpakInstallationPayload`.
    ///
    /// Returns `true` if the `installations` members are different, `false` otherwise.
    ///
    /// * `other` - The `FlatpakInstallationPayload` to compare against.
    ///
    /// # Example
    /// ```rust no_run
    /// use libflatsync_common::FlatpakInstallationPayload;
    /// use flatsync_daemon::context::Context;
    ///
    /// fn fetch_some_other_payload() -> FlatpakInstallationPayload {
    ///   // ...
    /// }
    ///
    /// fn main() {
    ///    let ctx = Context::new().unwrap();
    ///    let other = fetch_some_other_payload();
    ///
    ///    if ctx.installations_changed(other) {
    ///       // the local installations differ from the remote ones
    ///    }
    /// }
    pub fn installations_changed(&self, other: &FlatpakInstallationPayload) -> bool {
        let diff = self
            .local_installations
            .installations
            .diff(&other.installations);

        trace!("Diff: {:#?}", diff);

        !diff.0.altered.is_empty() || !diff.0.removed.is_empty()
    }

    pub fn local_altered_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.local_installations.altered_at
    }

    pub fn sync_to_system(&mut self, remote: &FlatpakInstallationPayload) -> Result<(), Error> {
        self.install_to_system(remote)?;
        self.uninstall_from_system(remote)?;

        let mut local = FlatpakInstallationPayload::new_from_system()
            .map_err(Error::FlatpakInstallationQueryFailure)?;
        local.altered_at = remote.altered_at;
        log::debug!("Done updating local state, refreshing cache");
        self.set_cache_and_file(local)?;
        Ok(())
    }

    /// ## `init_local_installations_file()`
    /// Creates the local installations file (and user data dir) if it doesn't exist.
    ///
    /// Also populates it with the current local installations if it's empty.
    fn init_local_installations_file() -> Result<(), Error> {
        let flatsync_user_data_dir = Self::get_user_flatsync_dir();

        if !flatsync_user_data_dir.exists() {
            std::fs::create_dir_all(&flatsync_user_data_dir)?;
        }

        let local_installations_file_path = Self::get_local_installations_file();

        let populate_file = || -> Result<(), Error> {
            let current = FlatpakInstallationPayload::new_from_system()
                .map_err(Error::FlatpakInstallationQueryFailure)?;
            current
                .write_to_file(&local_installations_file_path)
                .map_err(|e| Error::FlatpakInstallationFileFailure(e.to_string()))?;
            Ok(())
        };

        match std::fs::File::open(&local_installations_file_path) {
            Ok(file) => {
                if file.metadata()?.len() == 0 {
                    populate_file()?;
                }
            }
            Err(e) => {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(Error::FlatpakInstallationFileFailure(e.to_string()));
                }

                populate_file()?;
            }
        }
        Ok(())
    }

    fn get_local_installations_file() -> PathBuf {
        let mut flatsync_user_data_file = Self::get_user_flatsync_dir();
        flatsync_user_data_file.push("flatsync.json");

        flatsync_user_data_file
    }

    fn get_user_flatsync_dir() -> PathBuf {
        let mut flatsync_user_data_dir = glib::user_data_dir();
        flatsync_user_data_dir.push("flatsync");

        flatsync_user_data_dir
    }

    fn set_cache_and_file(&mut self, payload: FlatpakInstallationPayload) -> Result<(), Error> {
        self.local_installations = payload;
        self.local_installations
            .write_to_file(&Self::get_local_installations_file())
            .map_err(|e| Error::FlatpakInstallationFileFailure(e.to_string()))?;
        Ok(())
    }

    fn is_installed(&self, kind: FlatpakInstallationKind, id: &str) -> Result<bool, Error> {
        let binding = FlatpakInstallationPayload::new_from_system().unwrap();
        let apps = match binding.installations(kind) {
            Some(v) => v,
            None => return Err(Error::FlatpakNoSuchInstallation),
        };

        Ok(apps.refs.iter().any(|ref_| ref_.ref_ == id))
    }

    fn run_in_transaction(
        &self,
        installation: &libflatpak::Installation,
        kind: FlatpakInstallationKind,
        ref_: &FlatpakRef,
        func: impl FnOnce(&libflatpak::Transaction) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let transaction =
            libflatpak::Transaction::for_installation(installation, gio::Cancellable::NONE)
                .unwrap();
        transaction.add_default_dependency_sources();
        // Since we're a background application, we don't want to annoy the user
        transaction.set_no_interaction(true);

        if func(&transaction).is_err() {
            return Ok(());
        }

        log::trace!(
            "Operations for ref {} with kind {}: {:?}",
            ref_.id,
            kind,
            transaction
                .operations()
                .iter()
                .map(|s| s.get_ref().unwrap().into())
                .collect::<Vec<String>>()
        );

        transaction
            .run(gio::Cancellable::NONE)
            .map_err(|e| Error::FlatpakTransactionFailure(e.to_string()))
    }

    fn install_ref(
        &self,
        installation: &libflatpak::Installation,
        kind: FlatpakInstallationKind,
        ref_: &FlatpakRef,
    ) -> Result<(), Error> {
        self.run_in_transaction(installation, kind, ref_, |transaction| {
            if let Err(e) = transaction.add_install(&ref_.origin, &ref_.ref_, &[]) {
                log::error!("Couldn't install reference {}: {}", ref_.ref_, e);
                return Err(Error::FlatpakInstallationFailed(
                    ref_.ref_.clone(),
                    e.to_string(),
                ));
            }

            if let Err(e) = transaction.add_update(&ref_.ref_, &[], Some(&ref_.commit)) {
                log::error!(
                    "Couldn't select the commit {}, falling back to latest: {}",
                    &ref_.commit,
                    e
                );
                return Ok(());
            }

            Ok(())
        })
    }

    fn uninstall_ref(
        &self,
        installation: &libflatpak::Installation,
        kind: FlatpakInstallationKind,
        ref_: &FlatpakRef,
    ) -> Result<(), Error> {
        self.run_in_transaction(installation, kind, ref_, |transaction| {
            if let Err(e) = transaction.add_uninstall(&ref_.ref_) {
                log::error!("Couldn't uninstall reference {}: {}", ref_.ref_, e);
                return Err(Error::FlatpakUninstallationFailed(
                    ref_.ref_.clone(),
                    e.to_string(),
                ));
            }

            Ok(())
        })
    }

    fn add_remote(
        &self,
        remote: &FlatpakRemote,
        installation: &libflatpak::Installation,
    ) -> Result<(), Error> {
        // No need to install local repositories...
        if let Some(true) = remote.url.as_ref().map(|u| u.starts_with("file://")) {
            return Ok(());
        }

        log::debug!("Adding remote {}", &remote.name);

        let flatpak_remote: libflatpak::Remote = remote.into();
        // FIXME: This is a hack to get around the fact that we don't have a GPG key for the
        // Flatpak repo.
        flatpak_remote.set_gpg_verify(false);
        installation
            .add_remote(&flatpak_remote, true, gio::Cancellable::NONE)
            .map_err(|e| Error::FlatpakRemoteAddFailed(remote.name.clone(), e.to_string()))?;
        installation
            .update_remote_sync(&remote.name, gio::Cancellable::NONE)
            .map_err(|e| Error::FlatpakRemoteRefreshFailed(remote.name.clone(), e.to_string()))?;

        Ok(())
    }

    fn get_user_install_path() -> PathBuf {
        let mut user_path = glib::home_dir();
        user_path.push(".local");
        user_path.push("share");
        user_path.push("flatpak");
        user_path
    }

    fn get_user_or_system_installation(kind: FlatpakInstallationKind) -> libflatpak::Installation {
        match kind {
            FlatpakInstallationKind::User => {
                // User Installation
                let file = gio::File::for_path(Self::get_user_install_path());

                libflatpak::Installation::for_path(&file, true, gio::Cancellable::NONE).unwrap()
            }
            FlatpakInstallationKind::System => {
                libflatpak::Installation::new_system(gio::Cancellable::NONE).unwrap()
            }
        }
    }

    fn install_for_kind(
        &self,
        remote: &FlatpakInstallationPayload,
        kind: FlatpakInstallationKind,
    ) -> Result<(), Error> {
        let remote_installations = match remote.installations(kind) {
            Some(e) => Ok(e),
            None => Err(Error::FlatpakNoSuchInstallation),
        }?;

        let installation = Self::get_user_or_system_installation(kind);

        for remote in &remote_installations.remotes {
            self.add_remote(remote, &installation)?;
        }

        for ref_ in &remote_installations.refs {
            if self.is_installed(kind, &ref_.ref_)? {
                trace!("Ref {} is already installed, skipping", ref_.ref_);
                continue;
            }
            log::trace!("Installing ref {}", ref_.ref_);
            self.install_ref(&installation, kind, ref_)?;
        }

        Ok(())
    }

    fn uninstall_for_kind(
        &self,
        remote: &FlatpakInstallationPayload,
        kind: FlatpakInstallationKind,
    ) -> Result<(), Error> {
        let remote_installations_for_kind = match remote.installations(kind) {
            Some(k) => k,
            None => return Err(Error::FlatpakNoSuchInstallation),
        };

        let local_installations_for_kind = self
            .local_installations
            .installations(kind)
            .ok_or(Error::FlatpakNoSuchInstallation)?;

        let remote_refs_temp: HashSet<FlatpakRef> = remote_installations_for_kind
            .refs
            .clone()
            .into_iter()
            .collect();

        let to_uninstall = local_installations_for_kind
            .refs
            .iter()
            .filter(|ref_| !remote_refs_temp.contains(ref_))
            .collect::<Vec<_>>();

        let installation = Self::get_user_or_system_installation(kind);

        for ref_ in to_uninstall {
            if !self.is_installed(kind, &ref_.ref_)? {
                log::trace!("Ref {} is already uninstalled, skipping", ref_.ref_);
                continue;
            }
            log::trace!("Uninstalling ref {}", ref_.ref_);
            self.uninstall_ref(&installation, kind, ref_)?;
        }

        Ok(())
    }

    fn install_to_system(&mut self, remote: &FlatpakInstallationPayload) -> Result<(), Error> {
        self.install_for_kind(remote, FlatpakInstallationKind::System)?;
        self.install_for_kind(remote, FlatpakInstallationKind::User)?;

        Ok(())
    }

    fn uninstall_from_system(&self, remote: &FlatpakInstallationPayload) -> Result<(), Error> {
        self.uninstall_for_kind(remote, FlatpakInstallationKind::System)?;
        self.uninstall_for_kind(remote, FlatpakInstallationKind::User)?;

        Ok(())
    }
}
