use clap::Parser;
use zbus::{dbus_proxy, Connection, Result};

#[dbus_proxy(
    interface = "app.drey.FlatSync.Daemon1",
    default_service = "app.drey.FlatSync.Daemon",
    default_path = "/app/drey/FlatSync/Daemon"
)]
trait Daemon {
    async fn set_gist_secret(&self, secret: &str) -> Result<()>;
    async fn install_autostart_file(&self) -> Result<()>;
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    gist_secret: String,

    #[arg(short, long, default_value_t = false)]
    autostart: bool,
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<()> {
    let connection = Connection::session().await?;
    let proxy = DaemonProxy::new(&connection).await?;
    let args = Args::parse();
    proxy.set_gist_secret(&args.gist_secret).await?;
    if args.autostart {
        proxy.install_autostart_file().await?;
    }

    Ok(())
}
