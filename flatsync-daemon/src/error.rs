#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while querying Flatpak installations: {0}")]
    FlatpakInstallationQueryFailure(libflatsync_common::Error),
    #[error("Error while retrieving installation")]
    FlatpakNoSuchInstallation,
    #[error("Error while interacting with local Flatpak installation file: {0}")]
    FlatpakInstallationFileFailure(String),
    #[error("Error while installating Flatpak reference '{0}': {1}")]
    FlatpakInstallationFailed(String, String),
    #[error("Error while uninstalling Flatpak reference '{0}': {1}")]
    FlatpakUninstallationFailed(String, String),
    #[error("Error while adding Flatpak remote '{0}': {1}")]
    FlatpakRemoteAddFailed(String, String),
    #[error("Error while refreshing Flatpak remote '{0}': {1}")]
    FlatpakRemoteRefreshFailed(String, String),
    #[error("Flatpak is already installed")]
    FlatpakAlreadyInstalled,
    #[error("Transaction failed: {0}")]
    FlatpakTransactionFailure(String),
    #[error("FlatSync gist is already initialized with ID: {0}")]
    GistAlreadyInitialized(String),
    #[error("Gist uninitialized yet no gist ID was provided when attempting to sync")]
    GistIdMissing,
    #[error("Encountered error with the keychain: {0}")]
    KeychainFailure(#[from] oo7::Error),
    #[error("The specified keychain entry could not be found")]
    KeychainEntryNotFound,
    #[error("Failed to interpret UTF-8 sequence: {0}")]
    Utf8Failure(#[from] std::str::Utf8Error),
    #[error("Encountered HTTP error: {0}")]
    HttpFailure(#[from] reqwest::Error),
    #[error("Missing files in gist")]
    MissingGistFiles,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ASHPD error: {0}")]
    AshpdFailure(#[from] ashpd::Error),
}

#[derive(zbus::DBusError, Debug)]
pub enum DBusError {
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.GistCreateFailure")]
    GistCreateFailure(String),
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.GistUpdateFailure")]
    GistUpdateFailure(String),
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.GistSyncFailure")]
    GistSyncFailure(String),
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.GistApplyFailure")]
    GistApplyFailure(String),
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.InvalidSecret")]
    InvalidSecret,
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.AutoStartFailure")]
    AutoStartFailure,
}
