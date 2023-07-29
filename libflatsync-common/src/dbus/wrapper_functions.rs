use log::info;

use crate::dbus::DaemonProxy;

impl DaemonProxy<'_> {
    pub async fn init<S: AsRef<str>>(
        &self,
        token: S,
        gist_id: Option<String>,
    ) -> Result<(), zbus::Error> {
        self.set_gist_secret(token.as_ref()).await?;

        if let Some(id) = gist_id {
            self.set_gist_id(id.as_ref()).await?;
        } else {
            let id = self.create_gist().await?;
            info!("Successfully created a Flatsync list with id: {:?}", id);
        }

        Ok(())
    }
}
