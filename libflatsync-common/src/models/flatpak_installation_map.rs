pub use crate::models::{FlatpakInstallation, FlatpakInstallationKind};
use libflatpak::{gio, prelude::*};
use std::collections::BTreeMap;

#[derive(Debug, Clone, diff_derive::Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
#[repr(transparent)]
#[serde(transparent)]
pub struct FlatpakInstallationMap(pub BTreeMap<FlatpakInstallationKind, FlatpakInstallation>);

impl FlatpakInstallationMap {
    pub fn available_installations() -> Result<Self, crate::Error> {
        let mut ret: BTreeMap<_, _> = match libflatpak::system_installations(gio::Cancellable::NONE)
        {
            Ok(v) => v
                .into_iter()
                .map(|item| {
                    (
                        FlatpakInstallationKind::try_from_str(item.id().unwrap().as_str()).unwrap(),
                        item.into(),
                    )
                })
                .collect(),
            Err(e) => return Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        };

        let user_inst = match FlatpakInstallation::user_installation() {
            Ok(inst) => (
                FlatpakInstallationKind::try_from_str(inst.id.as_str()).unwrap(),
                inst,
            ),
            Err(e) => return Err(e),
        };

        ret.insert(user_inst.0, user_inst.1);

        Ok(Self(ret))
    }

    pub fn get(&self, kind: FlatpakInstallationKind) -> Option<&FlatpakInstallation> {
        self.0.get(&kind)
    }
}
