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
        if imp.gist_secret().await.is_err() {
            println!("No secret found");
            continue;
        }
        imp.post_gist().await?;
        interval.tick().await;
        if let Some(remote) = imp.fetch_gist().await? {
            let mut local = FlatpakInstallationMap::available_installations()?;
            let diff = remote.diff(&local);
            local.apply(&diff);
            // Resolve the diff, or print it for now
            // println!("{:#?}", local);
        }
    }
}
