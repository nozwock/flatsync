use log::{error, info, trace};
use zbus::ConnectionBuilder;

mod api;
mod context;
mod dbus;
mod error;
pub use error::DBusError;
pub use error::Error;
mod imp;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let daemon = dbus::Daemon::new().await?;

    let _con = ConnectionBuilder::session()?
        .name("app.drey.FlatSync.Daemon")?
        .serve_at("/app/drey/FlatSync/Daemon", daemon)?
        .build()
        .await?;

    info!("Started daemon. Press Ctrl+C to exit");

    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    let imp = imp::Impl::new().await?;

    let mut ctx = context::Context::new()?;

    loop {
        interval.tick().await;
        if imp.get_gist_secret().await.is_err() {
            error!("No secret found");
            continue;
        }

        ctx.update_local_installations()?;

        let res = imp.fetch_gist().await;

        match res {
            Ok(Some(remote)) => {
                // if the local and remote are the same, we don't need to do anything
                if ctx.installations_changed(&remote) {
                    // compare `altered_at` values to determine which version is newer
                    // this is done so that when a user does not have an active internet connection and alters their installations, the changes are not falsely overwritten from remote
                    let altered_remote = remote.altered_at;
                    let altered_local = ctx.get_local_altered_at();

                    // local is newer, so we push to remote
                    // otherwise, we apply the remote's changes
                    if altered_local > altered_remote {
                        info!("Local is newer, updating remote...");
                        imp.post_gist().await?;
                        info!("Pushed local changes to remote")
                    } else {
                        info!("Remote is newer");
                        // TODO: Apply diff
                    }
                }
            }
            Err(e) => {
                // TODO: Filter for different error types
                trace!("{:?}", e);
                continue;
            }
            _ => {}
        }
    }
}
