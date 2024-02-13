use crate::{imp::Impl, DBusError};
use log::{debug, info};
use zbus::dbus_interface;

use crate::ManualSync;
use crate::MessageType;

pub struct Daemon {
    imp: Impl,
    sender: tokio::sync::mpsc::Sender<MessageType>,
}

impl Daemon {
    pub async fn new(sender: tokio::sync::mpsc::Sender<MessageType>) -> Result<Self, crate::Error> {
        let imp = Impl::new().await?;
        Ok(Self { imp, sender })
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
    async fn create_gist(&mut self) -> Result<String, DBusError> {
        self.imp.create_gist().await.map_err(|e| {
            debug!("Error creating gist: {:?}", e);
            DBusError::GistCreateFailure(e.to_string())
        })
    }

    /// ## `UpdateGist(..)`
    /// Update the remote gist with the list of local Flatpak installations
    async fn post_gist(&self) -> Result<(), DBusError> {
        self.imp
            .post_gist()
            .await
            .map_err(|e| DBusError::GistUpdateFailure(e.to_string()))?;
        info!("Gist successfully updated");
        Ok(())
    }

    async fn set_gist_id(&self, id: &str) -> Result<(), DBusError> {
        self.imp.set_gist_id(id);

        Ok(())
    }

    async fn sync_now(&self) -> Result<bool, DBusError> {
        info!("Starting Manual Sync");
        match self
            .sender
            .send(MessageType::TimeToPoll(Some(ManualSync)))
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Err(DBusError::SendError),
        }
    }

    async fn autosync(&self) -> Result<bool, DBusError> {
        Ok(self.imp.autosync())
    }

    async fn set_autosync(&self, autosync: bool) -> Result<(), DBusError> {
        self.imp.set_autosync(autosync);
        Ok(())
    }

    async fn autosync_timer(&self) -> Result<u32, DBusError> {
        Ok(self.imp.autosync_timer())
    }

    async fn set_autosync_timer(&self, timer: u32) -> Result<(), DBusError> {
        self.imp.set_autosync_timer(timer);
        Ok(())
    }

    async fn autostart_file(&mut self, install: bool) -> Result<(), DBusError> {
        self.imp
            .autostart_file(install)
            .await
            .map_err(|_| DBusError::AutoStartFailure)
    }
}
