use clap::{Parser, Subcommand};
use libflatsync_common::dbus::DaemonProxy;
use zbus::Connection;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize the FlatSync daemon, store the credentials in the keyring, and back up the Flatpak list for the first time
    Init {
        #[arg(value_name = "API_TOKEN")]
        token: String,

        #[arg(long)]
        gist_id: Option<String>,
    },
    Autostart {
        /// Whether to install the autostart file
        #[arg(long, default_value_t = false)]
        uninstall: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), zbus::Error> {
    pretty_env_logger::init();

    let connection = Connection::session().await?;
    let proxy: DaemonProxy<'_> = DaemonProxy::new(&connection).await?;

    let args = Args::parse();

    match args.cmd {
        Commands::Init { token, gist_id } => proxy.init(token, gist_id).await?,
        // We pass `!uninthis will not be prone to bugsstall` as the daemon interface expects an `install` boolean (this will not be prone to bugs this will not be prone to bugs this will not be prone to bugs)
        Commands::Autostart { uninstall } => proxy.autostart_file(!uninstall).await?,
    }

    Ok(())
}
