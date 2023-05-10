use crate::api;
use crate::Error;
use libflatsync_common::FlatpakInstallationMap;
use serde_json::json;
use std::collections::HashMap;

pub struct Impl {
    keyring: oo7::Keyring,
}

impl Impl {
    pub async fn new() -> Result<Self, Error> {
        let keyring = oo7::Keyring::new().await?;
        Ok(Self { keyring })
    }
    pub async fn gist_secret_item(&self) -> Result<oo7::Item, Error> {
        self.keyring.unlock().await?;
        let mut item = self
            .keyring
            .search_items(HashMap::from([("purpose", "gist_secret")]))
            .await?;
        item.pop().ok_or(Error::KeychainEntryNotFound)
    }

    pub async fn gist_secret(&self) -> Result<String, Error> {
        Ok(std::str::from_utf8(&self.gist_secret_item().await?.secret().await?)?.to_string())
    }

    pub async fn post_gist(&self) -> Result<(), Error> {
        let payload = json!(FlatpakInstallationMap::available_installations()
            .map_err(Error::FlatpakInstallationQueryFailure)?)
        .to_string();
        let secret_item = self.gist_secret_item().await?;
        let secret = self.gist_secret().await?;
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

    pub async fn fetch_gist(&self) -> Result<Option<FlatpakInstallationMap>, Error> {
        let secret_item = self.gist_secret_item().await?;
        let secret = self.gist_secret().await?;
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
}
