use libflatpak::{gio, traits::*};
use std::collections::HashMap;
use std::future::pending;
use zbus::{dbus_interface, ConnectionBuilder};

struct Daemon {
    keyring: oo7::Keyring,
}

#[derive(zbus::DBusError, Debug)]
enum Error {
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.CouldntQueryInstalledFlatpaks")]
    CouldntQueryInstalledFlatpaks,
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.InvalidSecret")]
    InvalidSecret,
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

    async fn set_gist_secret(&mut self, secret: &str) -> Result<(), Error> {
        if secret.is_empty() {
            return Err(Error::InvalidSecret);
        }
        self.keyring.unlock().await;
        self.keyring
            .create_item(
                "GitHub Gists token",
                HashMap::from([("gist_secret", secret)]),
                b"secret",
                true,
            )
            .await;
        self.keyring.lock().await;
        Ok(())
    }
}

impl Daemon {
    async fn new() -> Self {
        let keyring = oo7::Keyring::new().await;
        Self {
            keyring: keyring.unwrap(),
        }
    }
}

// Although we use `async-std` here, you can use any async runtime of choice.
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let daemon = Daemon::new().await;
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
