use crate::models::{FlatpakInstallationStorageType, FlatpakRef, FlatpakRemote};
use libflatpak::{gio, glib, prelude::*, Installation};
use std::path::PathBuf;

#[derive(
    Debug, Default, Clone, diff_derive::Diff, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub struct FlatpakInstallation {
    pub id: String,
    pub path: PathBuf,
    pub display_name: Option<String>,
    pub priority: i32,
    pub storage_type: FlatpakInstallationStorageType,
    pub refs: Vec<FlatpakRef>,
    pub remotes: Vec<FlatpakRemote>,
}

impl<O: glib::IsA<libflatpak::Installation>> From<O> for FlatpakInstallation {
    fn from(value: O) -> Self {
        let value = value.upcast();

        Self {
            id: value.id().unwrap().into(),
            path: match value.path() {
                Some(f) => f.path().unwrap(),
                None => Default::default(),
            },
            display_name: value.display_name().map(|s| s.into()),
            priority: value.priority(),
            storage_type: value.storage_type().into(),
            refs: match value.list_installed_refs(gio::Cancellable::NONE) {
                Ok(v) => v.into_iter().map(|item| item.into()).collect(),
                Err(_) => vec![],
            },
            remotes: match value.list_remotes(gio::Cancellable::NONE) {
                Ok(v) => v.into_iter().map(|item| item.into()).collect(),
                Err(_) => vec![],
            },
        }
    }
}

impl FlatpakInstallation {
    pub fn user_installation() -> Result<Self, crate::Error> {
        match Installation::new_user(gio::Cancellable::NONE) {
            Ok(item) => Ok(item.into()),
            Err(e) => Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        }
    }
}
