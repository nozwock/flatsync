use std::collections::HashMap;
use zbus::dbus_interface;
use crate::imp::Error;
use gtk::glib;
use std::path::Path;
use tokio::fs;
use flatsync::config;

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
    async fn install_autostart_file(&mut self) {
        self.install_autostart_file_imp().await;
    }
}

impl Daemon {
    pub async fn new() -> Result<Self, Error> {
        let keyring = oo7::Keyring::new().await?;
        Ok(Self { keyring })
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

    async fn install_autostart_file_imp(&mut self) -> Result<(), tokio::io::Error> {
        let autostart_desktop_file = Path::new(config::AUTOSTART_DESKTOP_FILE_PATH);
        let desktop_file_name = autostart_desktop_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let mut autostart_user_folder = glib::user_config_dir();
        autostart_user_folder.push("autostart");
        if !autostart_user_folder.exists() {
            fs::create_dir_all(&autostart_user_folder).await;
        }
        autostart_user_folder.push(&desktop_file_name);
        if !autostart_user_folder.exists() {
            fs::copy(autostart_desktop_file, autostart_user_folder).await;
        }
        Ok(())
    }
}
