use crate::Error;
use diff::Diff;
use libflatsync_common::FlatpakInstallationPayload;
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

    /// ## `update_local_installations()`
    /// Gets the current local installations and compares them to the `local_installations` member.
    ///
    /// Updates the `local_installations` member of the `Context` struct aswell as the local file's content, should they differ.
    pub fn update_local_installations(&mut self) -> Result<(), Error> {
        let cur =
            FlatpakInstallationPayload::new().map_err(Error::FlatpakInstallationQueryFailure)?;

        if self.installations_changed(&cur) {
            debug!(
                "Local installations changed, timestamps:\nCurrent: {:?}\nMember: {:?}",
                cur.altered_at, self.local_installations.altered_at
            );
            self.local_installations = cur;
            Self::installations_to_file(&self.local_installations)?;
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
}
