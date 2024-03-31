pub use crate::models::{FlatpakInstallation, FlatpakInstallationKind};
use anyhow::Context;
use libflatpak::{gio, prelude::*};
use std::collections::BTreeMap;

/// Maps `FlatpakInstallationKind` to `FlatpakInstallation`, so that we can easily access the user and system installations.
#[derive(Debug, Clone, diff_derive::Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
#[repr(transparent)]
#[serde(transparent)]
pub struct FlatpakInstallationMap(pub BTreeMap<FlatpakInstallationKind, FlatpakInstallation>);

impl FlatpakInstallationMap {
    /// Queries the system for available Flatpak installations.
    pub fn available_installations() -> Result<Self, crate::Error> {
        let mut ret = libflatpak::system_installations(gio::Cancellable::NONE)
            .map_err(crate::Error::FlatpakInstallationQueryFailure)?
            .into_iter()
            .map(|item| {
                item.id()
                    .with_context(|| format!("No installation id for {item}"))
                    .map_err(crate::Error::other)
                    .and_then(|id| FlatpakInstallationKind::try_from_str(id.as_str()))
                    .map(|i| (i, item.into()))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;

        let inst = FlatpakInstallation::user_installation()?;

        ret.insert(
            FlatpakInstallationKind::try_from_str(inst.id.as_str())?,
            inst,
        );

        Ok(Self(ret))
    }

    pub fn get(&self, kind: FlatpakInstallationKind) -> Option<&FlatpakInstallation> {
        self.0.get(&kind)
    }
}
