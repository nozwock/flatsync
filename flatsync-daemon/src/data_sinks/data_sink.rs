use std::collections::HashMap;

use crate::{settings::Settings, Error};
use async_trait::async_trait;
use libflatsync_common::FlatpakInstallationPayload;
use log::debug;

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

    fn is_initialised(&self) -> bool {
        !self.sink_id().is_empty()
    }

    fn sink_id(&self) -> String {
        Settings::instance().get(&format!("{}-id", self.sink_name()))
    }

    fn set_sink_id(&self, id: &str) {
        debug!("Setting new sink id: {}", id);
        Settings::instance()
            .set(&format!("{}-id", self.sink_name()), id)
            .unwrap();
    }

    async fn set_secret(&self, secret: &str) -> Result<(), Error> {
        let keyring = self.keyring().await;
        keyring.unlock().await?;
        let name = self.sink_name();
        keyring
            .create_item(
                &format!("{} token", name),
                HashMap::from([("purpose", format!("{}-secret", name).as_ref())]),
                secret,
                true,
            )
            .await?;
        Ok(())
    }

    async fn keyring(&self) -> oo7::Keyring {
        oo7::Keyring::new().await.unwrap()
    }

    fn sink_name(&self) -> &'static str;
}
