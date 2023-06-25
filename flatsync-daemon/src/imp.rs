use crate::api::CreateGistResponse;
use crate::Error;
use crate::{api, settings::Settings};
use ashpd::desktop::background::Background;
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

    async fn get_gist_secret_item(&self) -> Result<oo7::Item, Error> {
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
        let secret = self.get_gist_secret().await?;
        let gist_id = Settings::instance().get_gist_id();
        match gist_id.as_ref() {
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
                Settings::instance().set_gist_id(&resp.id);
            }
        }
        Ok(())
    }

    pub async fn create_gist(&self, public: bool) -> Result<CreateGistResponse, Error> {
        let secret = self.get_gist_secret().await?;
        let gist_id = Settings::instance().get_gist_id();
        match gist_id {
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

                Settings::instance().set_gist_id(&resp.id);

                Ok(resp)
            }
            Some(id) => Err(Error::GistAlreadyInitialized(id)),
        }
    }

    pub async fn fetch_gist(&self) -> Result<Option<FlatpakInstallationMap>, Error> {
        let secret = self.get_gist_secret().await?;
        let gist_id = Settings::instance().get_gist_id();
        Ok(match gist_id {
            Some(gist_id) => {
                let request = api::FetchGist::new(gist_id);
                Some(request.fetch(&secret).await?)
            }
            // Wait for upload of a gist.
            None => None,
        })
    }

    async fn autostart_file_sanbox(&self, install: bool) -> Result<(), Error> {
        // `dbus_activatable` has to be set to false, otherwise this doesn't work for some reason.
        // I guess this has something to do with the fact that in our D-Bus service file we call `app.drey.FlatSync.Daemon` instead of `app.drey.FlatSync`?
        Background::request()
            .reason("Enable autostart for FlatSync's daemon")
            .auto_start(install)
            .command(&["flatsync-daemon"])
            .dbus_activatable(false)
            .send()
            .await?;

        Ok(())
    }

    async fn autostart_file_native(&self, install: bool) -> Result<(), Error> {
        let autostart_desktop_file = Path::new(config::AUTOSTART_DESKTOP_FILE_PATH);
        let desktop_file_name = autostart_desktop_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let mut autostart_user_folder = glib::user_config_dir();
        autostart_user_folder.push("autostart");
        let mut autostart_file = autostart_user_folder.clone();
        autostart_file.push(desktop_file_name);
        if install {
            if !autostart_user_folder.exists() {
                fs::create_dir_all(&autostart_user_folder).await?;
            }
            fs::copy(autostart_desktop_file, autostart_file).await?;
        } else if autostart_file.exists() {
            fs::remove_file(autostart_file).await?;
        }

        Ok(())
    }

    pub async fn autostart_file(&self, install: bool) -> Result<(), Error> {
        // We currently still need the non-Portal version of this for native builds, as those don't work properly with the Portal APIs.
        if ashpd::is_sandboxed().await {
            self.autostart_file_sanbox(install).await?;
        } else {
            self.autostart_file_native(install).await?;
        }

        Ok(())
    }
}
