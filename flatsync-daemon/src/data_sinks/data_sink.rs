use std::collections::HashMap;

use crate::{settings::Settings, Error};
use async_trait::async_trait;
use libflatsync_common::FlatpakInstallationPayload;

pub static FILE_NAME: &str = "flatsync.json";

#[async_trait]
pub trait DataSink {
    /// Create a new data sink with the given payload.
    ///
    /// # Returns
    /// The id of the newly created data source.
    async fn create(&self, payload: FlatpakInstallationPayload) -> Result<(), Error>;
    /// Fetch the data from the data sink.
    async fn fetch(&self) -> Result<FlatpakInstallationPayload, Error>;
    /// Update the data sink with the given payload.
    async fn update(&self, payload: FlatpakInstallationPayload) -> Result<(), Error>;

    async fn set_secret(&self, secret: &str) -> Result<(), Error> {
        self.keyring().unlock().await?;
        let name = self.sink_name();
        self.keyring()
            .create_item(
                &format!("{} token", name),
                HashMap::from([("purpose", format!("{}-secret", name).as_ref())]),
                secret,
                true,
            )
            .await?;
        Ok(())
    }

    async fn secret(&self) -> Result<String, Error> {
        self.keyring().unlock().await?;
        let mut item = self
            .keyring()
            .search_items(HashMap::from([(
                "purpose",
                format!("{}-secret", self.sink_name()).as_ref(),
            )]))
            .await?;
        let item = item.pop().ok_or(Error::KeychainEntryNotFound)?;
        let secret = item.secret().await?;
        Ok(std::str::from_utf8(&secret)?.to_string())
    }

    fn is_initialised(&self) -> bool {
        !self.sink_id().is_empty()
    }
    fn sink_id(&self) -> String {
        Settings::instance().get(&format!("{}-id", self.sink_name()))
    }
    fn set_sink_id(&self, id: &str) {
        Settings::instance()
            .set(&format!("{}-id", self.sink_name()), id)
            .unwrap();
    }

    fn keyring(&self) -> &oo7::Keyring;
    fn sink_name(&self) -> &'static str;
}
