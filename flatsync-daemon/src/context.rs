use crate::Error;
use diff::Diff;
use libflatpak::{gio, prelude::*};
use libflatsync_common::{
    FlatpakInstallationKind, FlatpakInstallationPayload, FlatpakRef, FlatpakRemote,
};
use log::{debug, trace};
use std::path::PathBuf;

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
            match Self::installations_from_file(local_installations_file_path) {
                Ok(payload) => payload,
                Err(_e) => FlatpakInstallationPayload::new()
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
        let cur =
            FlatpakInstallationPayload::new().map_err(Error::FlatpakInstallationQueryFailure)?;

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

    pub fn get_local_altered_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.local_installations.altered_at
    }

    /// ## `installations_to_file()`
    /// Writes the given `FlatpakInstallationPayload` to the local installations file.
    ///
    /// Returns an `Error` if it fails to serialize the given `FlatpakInstallationPayload` or if it fails to write to the file.
    fn installations_to_file(
        flatpak_installation_payload: &FlatpakInstallationPayload,
    ) -> Result<(), Error> {
        let serialized = serde_json::to_string(flatpak_installation_payload).map_err(|e| {
            Error::FlatpakInstallationFileFailure(format!(
                "Failed to serialize local payload: {}",
                e
            ))
        })?;
        let local_installations_file_path = Self::get_local_installations_file();

        trace!("Writing to file: {:?}", local_installations_file_path);

        std::fs::write(local_installations_file_path, serialized)
            .map_err(|e| Error::FlatpakInstallationFileFailure(e.to_string()))?;

        Ok(())
    }

    /// ## `installations_from_file()`
    /// Reads the local installations file and returns a `FlatpakInstallationPayload` from it.
    /// Returns an `Error` if the file doesn't exist or if it fails to read it.
    ///
    /// * `file_path` - The path to the local installations file.
    fn installations_from_file(file_path: PathBuf) -> Result<FlatpakInstallationPayload, Error> {
        if !file_path.exists() {
            return Err(Error::FlatpakInstallationFileFailure(
                "File doesn't exist".into(),
            ));
        }

        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        let file_payload: FlatpakInstallationPayload = serde_json::from_reader(reader)
            .map_err(|e| Error::FlatpakInstallationFileFailure(e.to_string()))?;

        trace!("Read from file: {:?}", file_payload);

        Ok(file_payload)
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
            let current = FlatpakInstallationPayload::new()
                .map_err(Error::FlatpakInstallationQueryFailure)?;
            Self::installations_to_file(&current)?;
            Ok(())
        };

        match std::fs::File::open(local_installations_file_path) {
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
        Self::installations_to_file(&self.local_installations)?;
        Ok(())
    }

    fn is_installed(&self, kind: FlatpakInstallationKind, id: &str) -> Result<bool, Error> {
        let apps = self.local_installations.installations(kind).unwrap();
        Ok(apps.refs.iter().any(|ref_| ref_.ref_ == id))
    }

    fn install_ref(
        &self,
        installation: &libflatpak::Installation,
        kind: FlatpakInstallationKind,
        ref_: &FlatpakRef,
    ) -> Result<(), Error> {
        let transaction =
            libflatpak::Transaction::for_installation(installation, gio::Cancellable::NONE)
                .unwrap();
        transaction.add_default_dependency_sources();
        // Since we're a background application, we don't want to annoy the user
        transaction.set_no_interaction(true);

        if self.is_installed(kind, &ref_.ref_)? {
            return Ok(());
        }

        if let Err(e) = transaction.add_install(&ref_.origin, &ref_.ref_, &[]) {
            log::error!("Couldn't install reference {}: {}", ref_.ref_, e);
            return Ok(());
        }

        if let Err(e) = transaction.add_update(&ref_.ref_, &[], Some(&ref_.commit)) {
            log::error!(
                "Couldn't select the commit {}, falling back to latest: {}",
                &ref_.commit,
                e
            );
        }

        log::debug!(
            "Operations for ref {}: {:?}",
            ref_.id,
            transaction
                .operations()
                .iter()
                .map(|s| s.get_ref().unwrap().as_str().to_string())
                .collect::<Vec<String>>()
        );

        transaction.run(gio::Cancellable::NONE).map_err(|e| {
            Error::FlatpakInstallationFailed(
                ref_.name.as_deref().unwrap().to_string(),
                e.to_string(),
            )
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

    fn install_for_kind(
        &self,
        remote: &FlatpakInstallationPayload,
        kind: FlatpakInstallationKind,
    ) -> Result<(), Error> {
        let remote_installations = match remote.installations.get(kind) {
            Some(e) => Ok(e),
            None => Err(Error::FlatpakNoSuchInstallation),
        }?;

        let installation = match kind {
            FlatpakInstallationKind::User => {
                // User Installation
                let mut user_path = glib::home_dir();
                user_path.push(".local");
                user_path.push("share");
                user_path.push("flatpak");
                let file = gio::File::for_path(user_path);

                libflatpak::Installation::for_path(&file, true, gio::Cancellable::NONE).unwrap()
            }
            FlatpakInstallationKind::System => {
                libflatpak::Installation::new_system(gio::Cancellable::NONE).unwrap()
            }
        };

        for remote in &remote_installations.remotes {
            self.add_remote(remote, &installation)?;
        }

        for ref_ in &remote_installations.refs {
            self.install_ref(&installation, kind, ref_)?;
        }

        Ok(())
    }

    pub fn install_to_system(&mut self, remote: &FlatpakInstallationPayload) -> Result<(), Error> {
        self.install_for_kind(remote, FlatpakInstallationKind::System)?;
        self.install_for_kind(remote, FlatpakInstallationKind::User)?;

        let mut local =
            FlatpakInstallationPayload::new().map_err(Error::FlatpakInstallationQueryFailure)?;
        local.altered_at = remote.altered_at;
        log::debug!("Done updating local state, refreshing cache");
        self.set_cache_and_file(local)?;
        Ok(())
    }
}
