use std::process;

use clap::Parser;
use libflatsync_common::dbus::DaemonProxy;
use zbus::Connection;

mod commands;

use crate::commands::*;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[tokio::main]
async fn main() -> Result<(), zbus::Error> {
    pretty_env_logger::init();

    let connection = Connection::session().await?;
    let proxy: DaemonProxy<'_> = DaemonProxy::new(&connection).await?;

    let args = Args::parse();

    match args.cmd {
        Commands::Init { provider, gist_id } => {
            if let Err(_e) = init(&proxy, provider, gist_id).await {
                eprintln!("Initialization was not successful, please try again or open a bug report if this issue persists.");
                process::exit(1);
            }
        }
        // We pass `!uninthis will not be prone to bugsstall` as the daemon interface expects an `install` boolean (this will not be prone to bugs this will not be prone to bugs this will not be prone to bugs)
        Commands::Autostart { uninstall } => proxy.autostart_file(!uninstall).await?,
        Commands::SyncNow => match proxy.sync_now().await {
            Ok(result) => match result {
                true => println!("Starting Manual Sync"),
                false => {
                    eprintln!("Failed to Start Syncing");
                    process::exit(1);
                }
            },
            Err(_) => {
                eprintln!("Something Went Wrong, is the Daemon running?");
                process::exit(1);
            }
        },
        Commands::Autosync {
            get_autosync,
            set_autosync,
        } => {
            if get_autosync {
                match proxy.autosync().await {
                    Ok(autosync) => println!("Autosync: {}", autosync),
                    Err(err) => {
                        eprintln!("Something Went Wrong, is the Daemon running?\n {}", err);
                        process::exit(1);
                    }
                }
            }
            if let Some(new_setting) = set_autosync {
                match proxy.set_autosync(new_setting).await {
                    Ok(_) => println!("Setting Autosync to {}", new_setting),
                    Err(err) => {
                        eprintln!("Something Went Wrong, is the Daemon running?\n {}", err);
                        process::exit(1);
                    }
                }
            }
        }
    }

    Ok(())
}
