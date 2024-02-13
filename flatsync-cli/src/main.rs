use gio::prelude::*;
use glib::FromVariant;
use std::process;

use libflatsync_common::config::APP_ID;

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
            Err(error) => handle_daemon_error(error),
        },
        Commands::Autosync {
            get_autosync,
            set_autosync,
        } => {
            if get_autosync {
                match proxy.autosync().await {
                    Ok(autosync) => println!("Autosync: {}", autosync),
                    Err(error) => handle_daemon_error(error),
                }
            }
            if let Some(new_setting) = set_autosync {
                match proxy.set_autosync(new_setting).await {
                    Ok(_) => println!("Setting Autosync to {}", new_setting),
                    Err(error) => handle_daemon_error(error),
                }
            }
        }
        Commands::AutosyncTimer {
            get_autosync_timer,
            set_autosync_timer,
        } => {
            if get_autosync_timer {
                match proxy.autosync_timer().await {
                    Ok(autosync_timer) => println!("Autosync Timer: {}", autosync_timer),
                    Err(error) => handle_daemon_error(error),
                }
            }
            if let Some(new_timer) = set_autosync_timer {
                let autosync_timer_key = gio::Settings::new(APP_ID)
                    .settings_schema()
                    .unwrap()
                    .key("autosync-timer");
                let new_timer_variant = glib::Variant::from(new_timer);

                if autosync_timer_key.range_check(&new_timer_variant) {
                    match proxy.set_autosync_timer(new_timer).await {
                        Ok(_) => println!("Setting Autosync Timer to {}", new_timer),
                        Err(error) => handle_daemon_error(error),
                    }
                } else {
                    let range_variant = autosync_timer_key.range().child_value(1).child_value(0);
                    let range = <(u32, u32)>::from_variant(&range_variant).unwrap();

                    eprintln!(
                        "Value {} is out of range. Range is {}-{}.",
                        new_timer, range.0, range.1
                    );
                    process::exit(1);
                }
            }
        }
    }

    Ok(())
}
