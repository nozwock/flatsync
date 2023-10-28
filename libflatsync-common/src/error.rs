use libflatpak::glib;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", 0)]
    FlatpakInstallationQueryFailure(glib::Error),
    #[error("Got invalid Flatpak installation kind: {0}")]
    InvalidFlatpakInstallationKind(String),
    #[error("Error while interacting with local Flatpak installation file: {0}")]
    FlatpakInstallationFileFailure(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error while dealing with reqwest and OAuth2: {0}")]
    OAuth2ReqwestFailure(String),
}
