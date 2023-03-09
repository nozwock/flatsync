use libflatpak::{gio, traits::*};
use std::future::pending;
use zbus::{dbus_interface, ConnectionBuilder};

struct Daemon {}

#[derive(zbus::DBusError, Debug)]
enum Error {
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.CouldntQueryInstalledFlatpaks")]
    CouldntQueryInstalledFlatpaks,
}

#[dbus_interface(name = "app.drey.FlatSync.Daemon1")]
impl Daemon {
    // Can be `async` as well.
    fn get_installed_user_flatpaks(&self) -> Result<Vec<String>, Error> {
        let refs = libflatpak::Installation::new_user(gio::Cancellable::NONE)
            .and_then(|i| {
                i.list_installed_refs_by_kind(libflatpak::RefKind::App, gio::Cancellable::NONE)
            })
            .map_err(|_| Error::CouldntQueryInstalledFlatpaks)?;
        Ok(refs
            .into_iter()
            .filter_map(|r| r.name())
            .map(|n| n.to_string())
            .collect())
    }
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = Daemon {};
    let _con = ConnectionBuilder::session()?
        .name("app.drey.FlatSync.Daemon")?
        .serve_at("/app/drey/FlatSync/Daemon", daemon)?
        .build()
        .await?;

    println!("Started daemon. Press Ctrl+C to exit.");

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
