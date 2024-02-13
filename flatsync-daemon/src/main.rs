use libflatpak::gio::prelude::*;
use libflatpak::prelude::*;
use log::{debug, error, info, warn};
use zbus::ConnectionBuilder;

extern crate futures_executor;

use libflatsync_common::config::APP_ID;

mod context;
mod data_sinks;
mod dbus;
mod error;
pub use error::DBusError;
pub use error::Error;
mod imp;
mod settings;

pub struct ManualSync;

pub enum MessageType {
    FlatpakInstallationChanged,
    TimeToPoll(Option<ManualSync>),
    TimerChanged,
}

async fn poll_remote(
    ctx: &mut context::Context,
    imp: &imp::Impl,
    manual_sync: bool,
) -> Result<(), Error> {
    let network_is_metered = gio::NetworkMonitor::default().is_network_metered();
    let power_saver_is_enabled = gio::PowerProfileMonitor::get_default().is_power_saver_enabled();

    if (network_is_metered || power_saver_is_enabled) && !manual_sync {
        if network_is_metered {
            debug!("Network is metered, skipping remote poll...");
        }
        if power_saver_is_enabled {
            debug!("Power Saver is enabled, skipping remote poll...");
        }

        return Ok(());
    }

    let res = imp.fetch_gist().await;
    match res {
        Ok(Some(remote)) => {
            // if the local and remote are the same, we don't need to do anything
            if ctx.installations_changed(&remote) {
                // compare `altered_at` values to determine which version is newer
                // this is done so that when a user does not have an active internet connection and alters their installations, the changes are not falsely overwritten from remote
                let altered_remote = remote.altered_at;
                let altered_local = ctx.local_altered_at();

                debug!(
                    "Remote altered at: {:?}\nLocal altered at: {:?}",
                    altered_remote, altered_local
                );

                // local is newer, so we push to remote
                // otherwise, we apply the remote's changes
                if altered_local > altered_remote {
                    info!("Local is newer, updating remote...");
                    imp.post_gist().await?;
                    info!("Pushed local changes to remote");
                } else {
                    info!("Remote is newer, updating local state...");
                    ctx.sync_to_system(&remote)?;
                    info!("Updated local state");
                }
            }
        }
        Ok(None) => {
            debug!("Fetching remote returned empty result");
        }
        Err(e) => {
            debug!("Error fetching remote: {:?}", e);
            if let Error::HttpFailure(e) = e {
                if e.is_timeout() {
                    warn!("Connection timed out trying to fetch remote, are you online?");
                }
            }
            return Ok(());
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let (sender_flatpak_installation_changed, mut reciever) =
        tokio::sync::mpsc::channel::<MessageType>(1);

    let daemon = dbus::Daemon::new(sender_flatpak_installation_changed.clone()).await?;

    let _con = ConnectionBuilder::session()?
        .name("app.drey.FlatSync.Daemon")?
        .serve_at("/app/drey/FlatSync/Daemon", daemon)?
        .build()
        .await?;

    info!("Started daemon. Press Ctrl+C to exit");

    let imp = imp::Impl::new().await?;

    let mut ctx = context::Context::new()?;

    let interval = tokio::time::interval(std::time::Duration::from_secs(
        imp.autosync_timer() as u64 * 60,
    ));

    let sender_timer_changed = sender_flatpak_installation_changed.clone();

    let settings = gio::Settings::new(APP_ID);

    settings.connect_changed(Some("autosync-timer"), move |_, _| {
        debug!("Timer Changed");
        match futures_executor::block_on(sender_timer_changed.send(MessageType::TimerChanged)) {
            Ok(_) => debug!("Timer Updated"),
            Err(_) => debug!("Failed to Send Timer Update"),
        };
    });
    // The Setting Key only emits the `changed` signal if it has been read after the listener has been setted up
    settings.get::<u32>("autosync-timer");

    // We need a second sender to send a signal for polling the remote every X seconds (defined by the interval above)
    let sender_remote_poll_interval = sender_flatpak_installation_changed.clone();

    // This task takes care of monitoring the flatpak installations for changes by using event listeners provided by libflatpak
    tokio::task::spawn_blocking(move || {
        let monitor_user = libflatpak::Installation::new_user(libflatpak::gio::Cancellable::NONE)
            .unwrap()
            .create_monitor(libflatpak::gio::Cancellable::NONE)
            .unwrap();

        let monitor_system =
            libflatpak::Installation::new_system(libflatpak::gio::Cancellable::NONE)
                .unwrap()
                .create_monitor(libflatpak::gio::Cancellable::NONE)
                .unwrap();

        // Since we need to create 2 different monitors, one for user, one for system, we need to clone the sender here once more
        let sender_system_installation_changed = sender_flatpak_installation_changed.clone();

        monitor_user.connect_changed(move |_, _, _, _| {
            sender_flatpak_installation_changed
                .blocking_send(MessageType::FlatpakInstallationChanged)
                .unwrap();
        });

        monitor_system.connect_changed(move |_, _, _, _| {
            sender_system_installation_changed
                .blocking_send(MessageType::FlatpakInstallationChanged)
                .unwrap();
        });

        let glib_loop = glib::MainLoop::new(None, false);

        glib_loop.run();
    });

    let mut handle = Some(manager_timer_thread(
        interval,
        sender_remote_poll_interval.clone(),
        None,
    ));

    loop {
        // We listen for a new message, which can either indicate local installation changes or timed polling of the remote
        if let Some(msg) = reciever.recv().await {
            // Since we always poll the remote in both message cases, we just check if the message indicates local installation changes
            // If so, we update the app's local state to reflect the changes, and poll the remote afterwards
            if matches!(msg, MessageType::FlatpakInstallationChanged) {
                ctx.refresh_local_installations()?;
            }

            if matches!(msg, MessageType::TimerChanged) {
                handle = Some(manager_timer_thread(
                    tokio::time::interval(std::time::Duration::from_secs(
                        imp.autosync_timer() as u64 * 60,
                    )),
                    sender_remote_poll_interval.clone(),
                    handle,
                ));
            }

            let manual_sync = matches!(msg, MessageType::TimeToPoll(Some(ManualSync)));

            if (imp.autosync() || manual_sync) && !matches!(msg, MessageType::TimerChanged) {
                if let Err(e) = poll_remote(&mut ctx, &imp, manual_sync).await {
                    error!("{}", e.to_string());
                }
            }
        }
    }
}

fn manager_timer_thread(
    mut interval: tokio::time::Interval,
    sender: tokio::sync::mpsc::Sender<MessageType>,
    previous_thread: Option<tokio::task::JoinHandle<()>>,
) -> tokio::task::JoinHandle<()> {
    if let Some(handle) = previous_thread {
        handle.abort();
    }
    tokio::task::spawn(async move {
        loop {
            interval.tick().await;
            sender.send(MessageType::TimeToPoll(None)).await.unwrap();
        }
    })
}
