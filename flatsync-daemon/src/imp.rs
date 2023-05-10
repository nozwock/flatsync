use crate::api;
use crate::api::CreateGistResponse;
use crate::Error;
use libflatsync_common::{config, FlatpakInstallationMap};
use std::{collections::HashMap, path::Path};
use tokio::fs;

pub struct Impl {
    keyring: oo7::Keyring,
}

impl Impl {
    pub async fn new() -> Result<Self, Error> {
        let keyring = oo7::Keyring::new().await?;
        Ok(Self { keyring })
    }

    pub async fn get_gist_secret_item(&self) -> Result<oo7::Item, Error> {
        self.keyring.unlock().await?;
        let mut item = self
            .keyring
            .search_items(HashMap::from([("purpose", "gist_secret")]))
            .await?;
        item.pop().ok_or(Error::KeychainEntryNotFound)
    }

    pub async fn get_gist_secret(&self) -> Result<String, Error> {
        Ok(std::str::from_utf8(&self.get_gist_secret_item().await?.secret().await?)?.to_string())
    }

    pub async fn set_gist_secret(&self, secret: &str) -> Result<(), Error> {
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

    pub async fn post_gist(&self) -> Result<(), Error> {
        let payload = FlatpakInstallationMap::available_installations()
            .map_err(Error::FlatpakInstallationQueryFailure)?;
        let secret_item = self.get_gist_secret_item().await?;
        let secret = self.get_gist_secret().await?;
        let mut attributes = secret_item.attributes().await?;
        match attributes.get("gist_id") {
            Some(gist_id) => {
                // TODO check if the gist exists
                let request = api::UpdateGist::new(payload);
                request.post(&secret, gist_id).await?;
            }
            None => {
                let request = api::CreateGist::new(
                    "Installed Flatpaks and its remote repositories".to_string(),
                    false,
                    payload,
                );
                let resp = request.post(&secret).await?;
                attributes.insert("gist_id".to_string(), resp.id);
                secret_item
                    .set_attributes(
                        attributes
                            .iter()
                            .map(|(k, v)| (k.as_str(), v.as_str()))
                            .collect(),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn create_gist(&self, public: bool) -> Result<CreateGistResponse, Error> {
        let secret_item = self.get_gist_secret_item().await?;
        let secret = self.get_gist_secret().await?;
        let mut attributes = secret_item.attributes().await?;

        match attributes.get("gist_id") {
            None => {
                let installations = FlatpakInstallationMap::available_installations()
                    .map_err(Error::FlatpakInstallationQueryFailure)?;
                let resp = api::CreateGist::new(
                    "List of installed Flatpaks".into(),
                    public,
                    installations,
                )
                .post(&secret)
                .await?;

                attributes.insert("gist_id".to_string(), resp.id.clone());

                secret_item
                    .set_attributes(
                        attributes
                            .iter()
                            .map(|(k, v)| (k.as_str(), v.as_str()))
                            .collect(),
                    )
                    .await?;

                Ok(resp)
            }
            Some(id) => Err(Error::GistAlreadyInitialized(id.to_string())),
        }
    }

    pub async fn fetch_gist(&self) -> Result<Option<FlatpakInstallationMap>, Error> {
        let secret_item = self.get_gist_secret_item().await?;
        let secret = self.get_gist_secret().await?;
        let attributes = secret_item.attributes().await?;
        Ok(match attributes.get("gist_id") {
            Some(gist_id) => {
                let request = api::FetchGist::new(gist_id);
                Some(request.fetch(&secret).await?)
            }
            // Wait for upload of a gist.
            None => None,
        })
    }

    pub async fn install_autostart_file(&self) -> Result<(), Error> {
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
