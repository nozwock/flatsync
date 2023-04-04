use diff::Diff;
use libflatsync_common::{
    FlatpakInstallation, FlatpakInstallationDiff, FlatpakInstallationMap,
    FlatpakInstallationMapDiff,
};

pub trait FlatpakPatch {
    type Base: Diff<Repr = Self>;

    fn apply(self, base: Self::Base) -> Result<(), crate::Error>;
}

impl FlatpakPatch for FlatpakInstallationMapDiff {
    type Base = FlatpakInstallationMap;

    fn apply(self, mut base: Self::Base) -> Result<(), crate::Error> {
        self.0.removed.into_iter().try_for_each(|s| libflatpak::);

        self.0.altered.into_iter().try_for_each(|(k, v)| {
            v.apply(base.0.entry(k).or_insert(Default::default()).clone())
        })?;

        Ok(())
    }
}

impl FlatpakPatch for FlatpakInstallationDiff {
    type Base = FlatpakInstallation;

    fn apply(self, base: Self::Base) -> Result<(), crate::Error> {
        todo!()
    }
}
