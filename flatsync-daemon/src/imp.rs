use crate::{
    data_sinks::{data_sink::DataSink, GitHubGistDataSink},
    Error,
};
use ashpd::desktop::background::Background;
use libflatsync_common::{config, FlatpakInstallationPayload};
use log::{info, trace};
use std::path::Path;
use tokio::fs;

pub struct Impl {
    sink: Box<dyn DataSink + 'static + Send + Sync>,
}

impl Impl {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {
            sink: Box::new(GitHubGistDataSink::new().await?),
        })
    }

    pub async fn set_gist_secret(&self, secret: &str) -> Result<(), Error> {
        self.sink.set_secret(secret).await
    }

    pub fn set_gist_id(&self, id: &str) {
        self.sink.set_sink_id(id);
    }

    pub fn autosync(&self) -> bool {
        self.sink.autosync()
    }

    pub fn set_autosync(&self, autosync: bool) {
        self.sink.set_autosync(autosync)
    }

    pub fn autosync_timer(&self) -> u32 {
        self.sink.autosync_timer()
    }

    pub fn set_autosync_timer(&self, timer: u32) {
        self.sink.set_autosync_timer(timer);
    }

    pub async fn post_gist(&self) -> Result<(), Error> {
        let payload = FlatpakInstallationPayload::new_from_system()
            .map_err(Error::FlatpakInstallationQueryFailure)?;
        if !self.sink.is_initialised() {
            return Err(Error::GistIdMissing);
        }

        self.sink.update(payload).await?;
        Ok(())
    }

    pub async fn create_gist(&self) -> Result<String, Error> {
        if self.sink.is_initialised() {
            return Err(Error::GistAlreadyInitialized(self.sink.sink_id()));
        }

        info!("Creating new gist...");
        let payload = FlatpakInstallationPayload::new_from_system()
            .map_err(Error::FlatpakInstallationQueryFailure)?;
        trace!("Current gist payload: {:?}", payload);
        self.sink.create(payload).await?;
        info!("Done creating new gist.");
        Ok(self.sink.sink_id())
    }

    pub async fn fetch_gist(&self) -> Result<Option<FlatpakInstallationPayload>, Error> {
        let val = if self.sink.is_initialised() {
            Some(self.sink.fetch().await?)
        } else {
            None
        };

        Ok(val)
    }

    async fn autostart_file_sanbox(&self, install: bool) -> Result<(), Error> {
        // `dbus_activatable` has to be set to false, otherwise this doesn't work for some reason.
        // I guess this has something to do with the fact that in our D-Bus service file we call `app.drey.FlatSync.Daemon` instead of `app.drey.FlatSync`?
        Background::request()
            .reason("Enable autostart for FlatSync's daemon")
            .auto_start(install)
            .command(&["flatsync-daemon"])
            .dbus_activatable(false)
            .send()
            .await?;

        Ok(())
    }

    async fn autostart_file_native(&self, install: bool) -> Result<(), Error> {
        let autostart_desktop_file = Path::new(config::AUTOSTART_DESKTOP_FILE_PATH);
        let desktop_file_name = autostart_desktop_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let mut autostart_user_folder = glib::user_config_dir();
        autostart_user_folder.push("autostart");
        let mut autostart_file = autostart_user_folder.clone();
        autostart_file.push(desktop_file_name);
        if install {
            if !autostart_user_folder.exists() {
                fs::create_dir_all(&autostart_user_folder).await?;
            }
            fs::copy(autostart_desktop_file, autostart_file).await?;
        } else if autostart_file.exists() {
            fs::remove_file(autostart_file).await?;
        }

        Ok(())
    }

    pub async fn autostart_file(&self, install: bool) -> Result<(), Error> {
        // We currently still need the non-Portal version of this for native builds, as those don't work properly with the Portal APIs.
        if ashpd::is_sandboxed().await {
            self.autostart_file_sanbox(install).await?;
        } else {
            self.autostart_file_native(install).await?;
        }

        Ok(())
    }
}
