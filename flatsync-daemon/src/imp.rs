use crate::api;
use libflatsync_common::{FlatpakInstallationDiff, FlatpakInstallationMap};
use serde_json::json;
use std::collections::HashMap;

pub struct Impl {
    keyring: oo7::Keyring,
    diff: Vec<FlatpakInstallationDiff>,
}

impl Impl {
    pub async fn new() -> Result<Self, crate::Error> {
        let keyring = oo7::Keyring::new().await?;
        Ok(Self {
            keyring,
            diff: vec![],
        })
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

    pub async fn create_gist(&self, public: bool) -> Result<String, crate::Error> {
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

    pub async fn post_gist(&self) -> Result<(), crate::Error> {
        let installations = match FlatpakInstallationMap::available_installations() {
            Ok(map) => map,
            Err(e) => return Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        };
        let secret_item = self.gist_secret_item().await?;
        let secret = self.gist_secret().await?;
        let mut attributes = secret_item.attributes().await?;
        match attributes.get("gist_id") {
            Some(gist_id) => {
                let request = api::UpdateGist::new(json!(installations).to_string());
                request.post(&secret, gist_id).await?;
            }
            None => {
                let request = api::CreateGist::new(
                    "Installed Flatpaks and its remote repositories".to_string(),
                    false,
                    json!(installations).to_string(),
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
}
