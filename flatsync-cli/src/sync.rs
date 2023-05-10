use libflatsync_common::FlatpakInstallationMapDiff;
use log::debug;

use crate::DaemonProxy;

impl DaemonProxy<'_> {
    pub(crate) async fn sync(&self, id: Option<String>) -> anyhow::Result<()> {
        let diff: FlatpakInstallationMapDiff =
            serde_json::from_str(&self.sync_gist(&id.unwrap_or_default()).await?)?;

        debug!("{diff:#?}");

        Ok(())
    }
}

#[derive(Debug, clap::Subcommand)]
pub(crate) enum SyncCommands {
    /// Apply the synchronization diff. Use `--prefer` to override the picker
    Apply {
        #[arg(value_enum, short = 'p', long)]
        prefer: Option<PreferStrategy>,
    },
}

impl SyncCommands {
    #[inline(always)]
    pub async fn route(self, proxy: &crate::DaemonProxy<'_>) -> anyhow::Result<()> {
        match self {
            SyncCommands::Apply { prefer } => match prefer {
                Some(p) => p.route(proxy).await,
                None => todo!(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub(crate) enum PreferStrategy {
    Local,
    Remote,
}

impl PreferStrategy {
    #[inline(always)]
    pub async fn route(self, proxy: &crate::DaemonProxy<'_>) -> anyhow::Result<()> {
        match self {
            PreferStrategy::Local => Ok(proxy.update_gist().await?),
            PreferStrategy::Remote => Ok(proxy.apply_gist().await?),
        }
    }
}
