use crate::api::CreateGistResponse;
use crate::imp::Impl;
use crate::DBusError;
use log::info;
use tap::Tap;
use zbus::dbus_interface;

pub struct Daemon {
    imp: Impl,
}

impl Daemon {
    pub async fn new() -> Result<Self, crate::Error> {
        let imp = Impl::new().await?;
        Ok(Self { imp })
    }
}

#[dbus_interface(name = "app.drey.FlatSync.Daemon0")]
impl Daemon {
    async fn set_gist_secret(&mut self, secret: &str) -> Result<(), DBusError> {
        if secret.is_empty() {
            return Err(DBusError::InvalidSecret);
        }
        self.imp
            .set_gist_secret(secret)
            .await
            .map_err(|_| DBusError::InvalidSecret)
    }

    /// ## `CreateGist(...)`
    /// Create a remote gist with the list of local Flatpak installations and get the gist file ID
    ///
    /// Parameters:
    /// - `bool`: Whether the gist will be publicly viewable
    async fn create_gist(&mut self, public: bool) -> Result<CreateGistResponse, DBusError> {
        self.imp
            .create_gist(public)
            .await
            .map_err(|e| DBusError::GistCreateFailure(e.to_string()))
    }

    /// ## `UpdateGist(..)`
    /// Update the remote gist with the list of local Flatpak installations
    async fn post_gist(&self) -> Result<(), DBusError> {
        self.imp
            .post_gist()
            .await
            .map_err(|e| DBusError::GistUpdateFailure(e.to_string()))
            .tap(|r| {
                if r.is_ok() {
                    info!("Gist successfully updated")
                }
            })
    }

    async fn install_autostart_file(&mut self) -> Result<(), DBusError> {
        self.imp
            .install_autostart_file()
            .await
            .map_err(|_| DBusError::AutoStartFailure)
    }
}
