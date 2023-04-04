use diff::Diff;
use libflatsync_common::{config, FlatpakInstallationMap};
use log::{debug, info};
use serde_json::json;
use tap::Tap;

use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use zbus::dbus_interface;

use crate::api;

pub struct Daemon {
    keyring: oo7::Keyring,
}

#[dbus_interface(name = "app.drey.FlatSync.Daemon0")]
impl Daemon {
    async fn set_gist_secret(&mut self, secret: &str) -> Result<(), crate::DBusError> {
        if secret.is_empty() {
            return Err(crate::DBusError::InvalidSecret);
        }
        self.set_gist_secret_imp(secret)
            .await
            .map_err(|_| crate::DBusError::InvalidSecret)
    }

    /// ## `CreateGist(...)`
    /// Create a remote gist with the list of local Flatpak installations and get the gist file ID
    ///
    /// Parameters:
    /// - `bool`: Whether the gist will be publicly viewable
    async fn create_gist(&mut self, public: bool) -> Result<String, crate::DBusError> {
        self.create_gist_imp(public)
            .await
            .map_err(|e| crate::DBusError::GistCreateFailure(e.to_string()))
    }

    /// ## `SyncGist(...)`
    /// Synchronize local Flatpak installations with the remote gist specified in the `id` parameter and get the diff in a JSON-formatted string
    ///
    /// Parameters:
    /// - `id` (nullable): Gist file ID to synchronize against
    async fn sync_gist(&mut self, id: &str) -> Result<String, crate::DBusError> {
        let id = match id {
            "" => None,
            otherwise => Some(otherwise.into()),
        };

        self.sync_gist_imp(id)
            .await
            .map_err(|e| crate::DBusError::GistSyncFailure(e.to_string()))
    }

    /// ## `UpdateGist(..)`
    /// Update the remote gist with the list of local Flatpak installations
    async fn update_gist(&self) -> Result<(), crate::DBusError> {
        self.update_gist_imp()
            .await
            .map_err(|e| crate::DBusError::GistUpdateFailure(e.to_string()))
            .tap(|r| {
                if r.is_ok() {
                    info!("Gist successfully updated")
                }
            })
    }

    /// ## `ApplyGist(..)`
    /// Apply changes listed in the gist to Flatpak installations
    async fn apply_gist(&self) -> Result<(), crate::DBusError> {
        self.apply_gist_imp()
            .await
            .map_err(|e| crate::DBusError::GistApplyFailure(e.to_string()))
            .tap(|r| {
                if r.is_ok() {
                    info!("Gist successfully applied")
                }
            })
    }

    async fn install_autostart_file(&mut self) -> Result<(), crate::DBusError> {
        self.install_autostart_file_imp()
            .await
            .map_err(|_| crate::DBusError::AutoStart)
    }
}

impl Daemon {
    pub async fn new() -> Result<Self, crate::Error> {
        let keyring = oo7::Keyring::new().await?;
        Ok(Self { keyring })
    }

    async fn set_gist_secret_imp(&mut self, secret: &str) -> Result<(), oo7::Error> {
        self.keyring.unlock().await?;
        self.keyring
            .create_item(
                "GitHub Gists token",
                HashMap::from([("purpose", "gist_secret")]),
                secret,
                true,
            )
            .await?;
        Ok(())
    }

    pub async fn gist_secret_item(&self) -> Result<oo7::Item, crate::Error> {
        self.keyring.unlock().await?;
        let mut item = self
            .keyring
            .search_items(HashMap::from([("purpose", "gist_secret")]))
            .await?;
        item.pop().ok_or(crate::Error::KeychainEntryNotFound)
    }

    pub async fn gist_secret(&self) -> Result<String, crate::Error> {
        Ok(std::str::from_utf8(&self.gist_secret_item().await?.secret().await?)?.to_string())
    }

    async fn create_gist_imp(&mut self, public: bool) -> Result<String, crate::Error> {
        let installations = match FlatpakInstallationMap::available_installations() {
            Ok(map) => map,
            Err(e) => return Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        };

        let secret_item = self.gist_secret_item().await?;
        let secret = self.gist_secret().await?;
        let mut attrs = secret_item.attributes().await?;

        match attrs.get("gist_id") {
            Some(id) => Err(crate::Error::GistAlreadyInitialized(id.clone())),
            None => {
                let resp = api::CreateGist::new(
                    "List of installed Flatpaks".into(),
                    public,
                    json!(installations).to_string(),
                )
                .post(&secret)
                .await?;

                attrs.insert("gist_id".to_string(), resp.id.clone());

                secret_item
                    .set_attributes(
                        attrs
                            .iter()
                            .map(|(k, v)| (k.as_ref(), v.as_ref()))
                            .collect(),
                    )
                    .await?;

                Ok(resp.id)
            }
        }
    }

    async fn sync_gist_imp(&mut self, id: Option<String>) -> Result<String, crate::Error> {
        let local_map = match FlatpakInstallationMap::available_installations() {
            Ok(map) => map,
            Err(e) => return Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        };

        let gh_token = self.gist_secret().await?;

        let id = match id {
            Some(id) => id,
            None => {
                match self
                    .gist_secret_item()
                    .await?
                    .attributes()
                    .await?
                    .get("gist_id")
                {
                    Some(id) => id.to_owned(),
                    None => return Err(crate::Error::GistIdMissing),
                }
            }
        };

        let remote_map = api::GetGist::new(id).get(gh_token).await?;

        Ok(json!(remote_map.diff(&local_map)).to_string())
    }

    async fn update_gist_imp(&self) -> Result<(), crate::Error> {
        let installations = match FlatpakInstallationMap::available_installations() {
            Ok(map) => map,
            Err(e) => return Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        };

        let secret_item = self.gist_secret_item().await?;
        let secret = self.gist_secret().await?;

        match secret_item.attributes().await?.get("gist_id") {
            Some(id) => Ok(api::UpdateGist::new(json!(installations).to_string())
                .post(&secret, id)
                .await?),
            None => Err(crate::Error::GistIdMissing),
        }
    }

    async fn apply_gist_imp(&self) -> Result<(), crate::Error> {
        todo!()
    }

    async fn install_autostart_file_imp(&mut self) -> Result<(), tokio::io::Error> {
        let autostart_desktop_file = Path::new(config::AUTOSTART_DESKTOP_FILE_PATH);
        let desktop_file_name = autostart_desktop_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let mut autostart_user_folder = glib::user_config_dir();
        autostart_user_folder.push("autostart");
        if !autostart_user_folder.exists() {
            fs::create_dir_all(&autostart_user_folder).await?;
        }
        autostart_user_folder.push(desktop_file_name);
        if !autostart_user_folder.exists() {
            fs::copy(autostart_desktop_file, autostart_user_folder).await?;
        }
        Ok(())
    }
}
