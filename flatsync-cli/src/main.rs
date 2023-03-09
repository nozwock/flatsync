use zbus::{dbus_proxy, Connection, Result};

#[dbus_proxy(
    interface = "app.drey.FlatSync.Daemon1",
    default_service = "app.drey.FlatSync.Daemon",
    default_path = "/app/drey/FlatSync/Daemon"
)]
trait Daemon {
    async fn get_installed_user_flatpaks(&self) -> Result<Vec<String>>;
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;

    // `dbus_proxy` macro creates `MyGreaterProxy` based on `Notifications` trait.
    let proxy = DaemonProxy::new(&connection).await?;
    let refs = proxy.get_installed_user_flatpaks().await?;
    for r in refs {
        println!("{}", r);
    }

    Ok(())
}
