use zbus::{dbus_proxy, Result};

#[dbus_proxy(
    interface = "app.drey.FlatSync.Daemon0",
    default_service = "app.drey.FlatSync.Daemon",
    default_path = "/app/drey/FlatSync/Daemon"
)]
pub trait Daemon {
    async fn set_gist_secret(&self, secret: &str) -> Result<()>;
    async fn create_gist(&self) -> Result<String>;
    async fn post_gist(&self) -> Result<()>;
    async fn set_gist_id(&self, id: &str) -> Result<()>;
    async fn autostart_file(&self, install: bool) -> Result<()>;
}