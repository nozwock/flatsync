use crate::DaemonProxy;

impl DaemonProxy<'_> {
    pub(crate) async fn autostart(&self, uninstall: bool) -> anyhow::Result<()> {
        // we pass !uninstall as `autostart_file()` expects `install: bool`
        self.autostart_file(!uninstall).await?;

        Ok(())
    }
}
