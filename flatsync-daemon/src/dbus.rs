use std::collections::HashMap;
use zbus::dbus_interface;

pub struct Daemon {
    keyring: oo7::Keyring,
}

#[derive(zbus::DBusError, Debug)]
enum DBusError {
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.CouldntQueryInstalledFlatpaks")]
    CouldntQueryInstalledFlatpaks,
    #[dbus_error(name = "app.drey.FlatSync.Daemon.Error.InvalidSecret")]
    InvalidSecret,
}

#[dbus_interface(name = "app.drey.FlatSync.Daemon1")]
impl Daemon {
    async fn set_gist_secret(&mut self, secret: &str) -> Result<(), DBusError> {
        if secret.is_empty() {
            return Err(DBusError::InvalidSecret);
        }
        self.set_gist_secret_imp(secret)
            .await
            .map_err(|_| DBusError::InvalidSecret)
    }
}

impl Daemon {
    pub async fn new() -> Self {
        let keyring = oo7::Keyring::new().await;
        Self {
            keyring: keyring.unwrap(),
        }
    }

    async fn set_gist_secret_imp(&mut self, secret: &str) -> Result<(), oo7::Error> {
        self.keyring.unlock().await?;
        self.keyring
            .create_item(
                "GitHub Gists token",
                HashMap::from([("purpose", "gist_secret")]),
                secret,
                true,
            )
            .await?;
        Ok(())
    }
}
