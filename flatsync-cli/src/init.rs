use log::info;

use crate::DaemonProxy;

impl DaemonProxy<'_> {
    pub(crate) async fn init<S: AsRef<str>>(&self, token: S, public: bool) -> anyhow::Result<()> {
        self.set_gist_secret(token.as_ref()).await?;

        let id = self.create_gist(public).await?;
        info!("Successfully created a Flatsync list with id: {:?}", id);

        Ok(())
    }
}
