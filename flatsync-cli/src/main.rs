use clap::{Parser, Subcommand};
use zbus::{dbus_proxy, Connection, Result};

mod init;
mod sync;
use sync::SyncCommands;

#[dbus_proxy(
    interface = "app.drey.FlatSync.Daemon1",
    default_service = "app.drey.FlatSync.Daemon",
    default_path = "/app/drey/FlatSync/Daemon"
)]
trait Daemon {
    async fn set_gist_secret(&self, secret: &str) -> Result<()>;
    async fn create_gist(&self, public: bool) -> Result<String>;
    async fn sync_gist(&self, id: &str) -> Result<String>;
    async fn install_autostart_file(&self) -> Result<()>;
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    autostart: bool,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize the FlatSync daemon and store the credentials in the keyring
    Init {
        #[arg(value_name = "API_TOKEN")]
        token: String,
    },
    /// Synchronize with the gist file
    Sync {
        /// Specify the gist file ID to synchronize with
        #[arg(short, long)]
        id: Option<String>,

        #[command(subcommand)]
        cmd: Option<SyncCommands>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let connection = Connection::session().await?;
    let proxy = DaemonProxy::new(&connection).await?;

    let args = Args::parse();

    match args.cmd {
        Commands::Init { token } => proxy.init(token).await?,
        Commands::Sync { id, cmd } => match cmd {
            Some(_) => todo!(),
            None => proxy.sync(id).await?,
        },
    }

    if args.autostart {
        proxy.install_autostart_file().await?;
    }

    Ok(())
}
