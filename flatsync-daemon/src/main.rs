use diff::Diff;
use libflatsync_common::FlatpakInstallationMap;
use log::info;
use zbus::ConnectionBuilder;

mod api;
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

    let imp = imp::Impl::new().await?;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

    loop {
        interval.tick().await;
        if imp.get_gist_secret().await.is_err() {
            println!("No secret found");
            continue;
        }
        if let Some(remote) = imp.fetch_gist().await? {
            let local = FlatpakInstallationMap::available_installations()?;
            let diff = remote.diff(&local);

            // if the local and remote are the same, we don't need to do anything
            if !diff.0.altered.is_empty() || !diff.0.removed.is_empty() {
                // TODO: Apply diff
                // TODO: Also check if we're newer or if remote is newer
                // ? Maybe just create a file with all the installed Flatpaks, and then compare the API response `updated_at` with the file's `mtime`?
                // ! Otherwise, we may be able to use Flatpaks `flatpak history` if this is exposed to our used library (the function lists modifications (update, install, uninstall) with timestamps)
                imp.post_gist().await?;
            }
            // Resolve the diff, or print it for now
            println!("{:#?}", diff);
        }
    }
}
