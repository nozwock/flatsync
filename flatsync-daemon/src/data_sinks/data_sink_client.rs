use crate::Error;
use async_trait::async_trait;
use libflatsync_common::providers::oauth_client::TokenPair;
use std::collections::HashMap;

pub enum SecretType {
    // `Simple` never gets constructed, but we should keep it here to allow for a reference and future, less-specific implementations
    #[allow(dead_code)]
    Simple(String),
    OAuth(TokenPair),
}
#[async_trait]
pub trait DataSinkClient {
    async fn keyring(&self) -> oo7::Keyring {
        oo7::Keyring::new().await.unwrap()
    }

    fn sink_name(&self) -> &'static str;

    async fn secret_raw(&self) -> Result<String, Error> {
        let keyring = self.keyring().await;
        let mut item = keyring
            .search_items(HashMap::from([(
                "purpose",
                format!("{}-secret", self.sink_name()).as_ref(),
            )]))
            .await?;
        let item = item.pop().ok_or(Error::KeychainEntryNotFound)?;
        let secret = item.secret().await?;
        Ok(std::str::from_utf8(&secret)?.to_string())
    }

    async fn secret(&self) -> Result<SecretType, Error> {
        Ok(SecretType::Simple(self.secret_raw().await?))
    }
}
