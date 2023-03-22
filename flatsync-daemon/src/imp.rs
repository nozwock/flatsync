use crate::api;
use libflatpak::{gio, traits::*};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error doing stuff with keychain")]
    KeychainError(#[from] oo7::Error),
    #[error("The specified keychain entry could not be found")]
    KeychainEntryNotFound(),
    #[error("Error querying installed flatpaks")]
    CouldntQueryInstalledFlatpaks(#[from] libflatpak::glib::Error),
}

pub struct Impl {
    keyring: oo7::Keyring,
}

impl Impl {
    pub async fn new() -> Self {
        let keyring = oo7::Keyring::new().await;
        Self {
            keyring: keyring.unwrap(),
        }
    }

    // Can be `async` as well.
    pub fn get_installed_user_flatpaks(&self) -> Result<Vec<String>, Error> {
        let refs = libflatpak::Installation::new_user(gio::Cancellable::NONE).and_then(|i| {
            i.list_installed_refs_by_kind(libflatpak::RefKind::App, gio::Cancellable::NONE)
        })?;
        Ok(refs
            .into_iter()
            .filter_map(|r| r.name())
            .map(|n| n.to_string())
            .collect())
    }

    pub fn get_installed_system_flatpaks(&self) -> Result<Vec<String>, Error> {
        let refs = libflatpak::Installation::new_system(gio::Cancellable::NONE).and_then(|i| {
            i.list_installed_refs_by_kind(libflatpak::RefKind::App, gio::Cancellable::NONE)
        })?;
        Ok(refs
            .into_iter()
            .filter_map(|r| r.name())
            .map(|n| n.to_string())
            .collect())
    }

    pub fn serialise_json(&self) -> Result<serde_json::Value, Error> {
        let installed_flatpaks_user = self.get_installed_user_flatpaks().unwrap();
        let installed_flatpaks_system = self.get_installed_system_flatpaks().unwrap();
        let json_data = json!({
            "user": &installed_flatpaks_user,
            "system": &installed_flatpaks_system
        });
        Ok(json_data)
    }

    pub async fn get_gist_secret_item(&self) -> Result<oo7::Item, Error> {
        self.keyring.unlock().await?;
        let mut item = self
            .keyring
            .search_items(HashMap::from([("purpose", "gist_secret")]))
            .await?;
        if item.is_empty() {
            // panic!("No secret found");
            return Err(Error::KeychainEntryNotFound());
        }
        Ok(item.pop().unwrap())
    }

    pub async fn get_gist_secret(&self) -> Result<String, Error> {
        Ok(
            std::str::from_utf8(&self.get_gist_secret_item().await?.secret().await.unwrap())
                .unwrap()
                .to_string(),
        )
    }

    pub async fn post_gist(&self) {
        let json_data = self.serialise_json().unwrap();
        let secret_item = self.get_gist_secret_item().await.unwrap();
        let secret = self.get_gist_secret().await.unwrap();
        let mut attributes = secret_item.attributes().await.unwrap();
        if attributes.contains_key("gist_id") {
            let gist_id = attributes.get("gist_id").unwrap();
            let request = api::UpdateGist::new(json_data.to_string());
            request.post(&secret, gist_id).await.unwrap();
        } else {
            let request = api::CreateGist::new(
                "List of installed flatpaks".to_string(),
                false,
                json_data.to_string(),
            );
            let resp = request.post(&secret).await.unwrap();
            attributes.insert("gist_id".to_string(), resp.id);
            secret_item
                .set_attributes(
                    attributes
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect(),
                )
                .await
                .unwrap();
        }
    }
}
