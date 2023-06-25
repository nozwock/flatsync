use clap::{Parser, Subcommand};
use zbus::{dbus_proxy, Connection, Result};

mod autostart;
mod init;
mod sync;
use sync::SyncCommands;

#[dbus_proxy(
    interface = "app.drey.FlatSync.Daemon0",
    default_service = "app.drey.FlatSync.Daemon",
    default_path = "/app/drey/FlatSync/Daemon"
)]
trait Daemon {
    async fn set_gist_secret(&self, secret: &str) -> Result<()>;
    async fn create_gist(&self, public: bool) -> Result<String>;
    async fn sync_gist(&self, id: &str) -> Result<String>;
    async fn update_gist(&self) -> Result<()>;
    async fn apply_gist(&self) -> Result<()>;
    async fn autostart_file(&self, install: bool) -> Result<()>;
    async fn set_gist_id(&self, id: &str) -> Result<()>;
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize the FlatSync daemon, store the credentials in the keyring, and back up the Flatpak list for the first time
    Init {
        /// Whether to make the list publicly available for viewing by others
        #[arg(long, default_value_t = false)]
        public: bool,

        #[arg(value_name = "API_TOKEN")]
        token: String,

        #[arg(long)]
        gist_id: Option<String>,
    },
    /// Synchronize with the gist file
    Sync {
        /// Specify the gist file ID to synchronize with
        #[arg(short, long)]
        id: Option<String>,

        #[command(subcommand)]
        cmd: Option<SyncCommands>,
    },
    Autostart {
        /// Whether to install the autostart file
        #[arg(long, default_value_t = false)]
        uninstall: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let connection = Connection::session().await?;
    let proxy = DaemonProxy::new(&connection).await?;

    let args = Args::parse();

    match args.cmd {
        Commands::Init {
            token,
            public,
            gist_id,
        } => proxy.init(token, public, gist_id).await?,
        Commands::Sync { id, cmd } => match cmd {
            Some(cmd) => cmd.route(&proxy).await?,
            None => proxy.sync(id).await?,
        },
        Commands::Autostart { uninstall } => proxy.autostart(uninstall).await?,
    }

    Ok(())
}
