use crate::DaemonProxy;

impl DaemonProxy<'_> {
    pub(crate) async fn init<S: AsRef<str>>(&self, token: S) -> anyhow::Result<()> {
        Ok(self.set_gist_secret(token.as_ref()).await?)
    }
}
