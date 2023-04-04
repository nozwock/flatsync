use std::{thread, time::Duration};

use zbus::ConnectionBuilder;

mod api;
mod dbus;
mod error;
mod traits;
pub(crate) use error::{DBusError, Error};

use log::info;

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

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
