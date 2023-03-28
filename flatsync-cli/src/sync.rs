use libflatsync_common::FlatpakInstallationMapDiff;

use crate::DaemonProxy;

impl DaemonProxy<'_> {
    pub(crate) async fn sync(&self, id: Option<String>) -> anyhow::Result<()> {
        let diff: FlatpakInstallationMapDiff =
            serde_json::from_str(&self.sync_gist(&id.unwrap_or_default()).await?)?;

        println!("{diff:#?}");

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

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub(crate) enum PreferStrategy {
    Local,
    Remote,
}
