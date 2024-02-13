use clap::Subcommand;
use libflatsync_common::dbus::DaemonProxy;
use libflatsync_common::providers::github::{GitHubProvider, GH_APP_INSTALLATION_URL};
use libflatsync_common::providers::oauth_client::OauthClientDeviceFlow;
use libflatsync_common::providers::providers_list::Providers;
use log::info;
use std::process;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize the FlatSync daemon, store the credentials in the keyring, and back up the Flatpak list for the first time
    Init {
        #[arg(long)]
        #[arg(default_value = "github")]
        provider: Providers,

        #[arg(long)]
        gist_id: Option<String>,
    },
    Autostart {
        /// Whether to install the autostart file
        #[arg(long, default_value_t = false)]
        uninstall: bool,
    },
    /// Start the syncing process manually
    SyncNow,
    /// Controls the Autosync Behaviour
    Autosync {
        #[arg(long, default_value_t = true)]
        get_autosync: bool,
        #[arg(long)]
        set_autosync: Option<bool>,
    },
    /// Controls how Often Sync happens in Minutes
    AutosyncTimer {
        #[arg(long, default_value_t = true)]
        get_autosync_timer: bool,
        #[arg(long)]
        set_autosync_timer: Option<u32>,
    },
}

pub async fn init(
    proxy: &DaemonProxy<'_>,
    provider: Providers,
    gist_id: Option<String>,
) -> Result<(), zbus::Error> {
    info!("Initializing FlatSync daemon");

    match provider {
        Providers::Github => init_for_github(proxy, gist_id).await?,
    }

    Ok(())
}

async fn init_for_github(
    proxy: &DaemonProxy<'_>,
    gist_id: Option<String>,
) -> Result<(), zbus::Error> {
    // Initialize OAuth Device Flow
    let github = GitHubProvider::new();

    let device_auth_res = github.device_code().await.unwrap();

    println!("Please install FlatSync as a GitHub app to your account **first** by following this link: {:?}.\nThis is required because GitHub's API doesn't allow us to interact with Gists without additional permission via a GitHub App installation.", GH_APP_INSTALLATION_URL);

    println!(
        "Afterwards, please visit {:?} and enter the following code: {:?}",
        &device_auth_res.verification_uri().to_string(),
        &device_auth_res.user_code().secret().to_string()
    );

    let token_pair =
        serde_json::to_string(&github.register_device(device_auth_res).await.unwrap()).unwrap();

    proxy.set_gist_secret(token_pair.as_str()).await?;

    if let Some(id) = gist_id {
        proxy.set_gist_id(id.as_ref()).await?;
    } else {
        let id = proxy.create_gist().await?;
        info!(
            "Successfully created a Flatsync list with GitHub Gist ID: {:?}",
            id
        );
    }

    Ok(())
}

pub fn handle_daemon_error(error: zbus::Error) {
    eprintln!("Something Went Wrong, is the Daemon running?\n {}", error);
    process::exit(1);
}
