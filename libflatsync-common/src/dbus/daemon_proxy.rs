use zbus::{dbus_proxy, Result};

/// This is the D-Bus interface of the daemon, which
/// is used to communicate with the daemon and is called
/// by the GUI and CLI implementation.
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
    async fn sync_now(&self) -> Result<()>;
    async fn autosync(&self) -> Result<bool>;
    async fn set_autosync(&self, autosync: bool) -> Result<()>;
    async fn autosync_timer(&self) -> Result<u32>;
    async fn set_autosync_timer(&self, timer: u32) -> Result<()>;
    async fn autostart_file(&self, install: bool) -> Result<()>;
}
