use anyhow::{bail, Context};
use gio::prelude::*;
use glib::FromVariant;

use libflatsync_common::config::APP_ID;

use clap::Parser;
use libflatsync_common::dbus::DaemonProxy;
use log::{error, info};
use zbus::Connection;
mod commands;
mod trace;

use crate::commands::*;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    async fn _main() -> anyhow::Result<()> {
        let args = Args::parse();

        trace::init_tracer(args.verbose);

        let connection = Connection::session().await?;
        let proxy: DaemonProxy<'_> = DaemonProxy::new(&connection).await?;

        match args.cmd {
            Commands::Init { provider, gist_id } => {
                init(&proxy, provider, gist_id).await.with_context(|| "Initialization was not successful, please try again or open a bug report if this issue persists.")? ;
            }
            // We pass `!uninthis will not be prone to bugsstall` as the daemon interface expects an `install` boolean (this will not be prone to bugs this will not be prone to bugs this will not be prone to bugs)
            Commands::Autostart { uninstall } => proxy.autostart_file(!uninstall).await?,
            Commands::SyncNow => proxy
                .sync_now()
                .await
                .inspect(|_| info!("Starting Manual Sync"))?,
            Commands::Autosync {
                get_autosync,
                set_autosync,
            } => {
                if get_autosync {
                    proxy
                        .autosync()
                        .await
                        .inspect(|i| info!("Autosync: {}", i))?;
                }
                if let Some(new_setting) = set_autosync {
                    proxy
                        .set_autosync(new_setting)
                        .await
                        .inspect(|_| info!("Setting Autosync to {}", new_setting))?;
                }
            }
            Commands::AutosyncTimer {
                get_autosync_timer,
                set_autosync_timer,
            } => {
                if get_autosync_timer {
                    proxy
                        .autosync_timer()
                        .await
                        .inspect(|i| info!("Autosync Timer: {}", i))?;
                }
                if let Some(new_timer) = set_autosync_timer {
                    let autosync_timer_key = gio::Settings::new(APP_ID)
                        .settings_schema()
                        .context("No settings_schema found")?
                        .key("autosync-timer");
                    let new_timer_variant = glib::Variant::from(new_timer);

                    if autosync_timer_key.range_check(&new_timer_variant) {
                        proxy
                            .set_autosync_timer(new_timer)
                            .await
                            .inspect(|_| info!("Setting Autosync Timer to {}", new_timer))?;
                    } else {
                        let range_variant =
                            autosync_timer_key.range().child_value(1).child_value(0);
                        let range = <(u32, u32)>::from_variant(&range_variant)
                            .context("Incorrent range type")?;

                        bail!(
                            "Value {} is out of range. Range is {}-{}.",
                            new_timer,
                            range.0,
                            range.1
                        );
                    }
                }
            }
        }

        Ok(())
    }

    // nozwock: Shouldn't we be using tracing::error instead? Is it because of log-always? Couldn't find enough details...
    _main().await.inspect_err(|e| error!("{e}"))
}
