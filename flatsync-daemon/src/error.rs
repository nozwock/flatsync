#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while querying Flatpak installations: {}", 0)]
    FlatpakInstallationQueryFailure(libflatsync_common::error::Error),
    #[error("FlatSync gist is already initialized with ID: {}", 0)]
    // GistAlreadyInitialized(String),
    // #[error("Gist uninitialized yet no gist ID was provided when attempting to sync")]
    // GistIdMissing,
    // #[error("Encountered error with the keychain: {}", 0)]
    KeychainFailure(#[from] oo7::Error),
    #[error("The specified keychain entry could not be found")]
    KeychainEntryNotFound,
    #[error("Failed to interpret UTF-8 sequence: {}", 0)]
    Utf8Failure(#[from] std::str::Utf8Error),
    #[error("Encountered HTTP error: {}", 0)]
    HttpFailure(#[from] reqwest::Error),
}
