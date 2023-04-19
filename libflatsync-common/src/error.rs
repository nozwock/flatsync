use libflatpak::glib;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}", 0)]
    FlatpakInstallationQueryFailure(glib::Error),
}
