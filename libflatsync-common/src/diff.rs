use crate::{error::Error, raw};
use diff_derive::Diff;
use libflatpak::{
    gio::{self, traits::FileExt},
    glib::{self, Cast},
    traits::{InstallationExt, InstalledRefExt, RefExt, RemoteExt},
    Installation,
};
use std::{collections::HashMap, path::PathBuf};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Diff, serde::Serialize, serde::Deserialize,
)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub enum FlatpakRefKind {
    #[default]
    App,
    Runtime,
}

impl From<libflatpak::RefKind> for FlatpakRefKind {
    fn from(value: libflatpak::RefKind) -> Self {
        match value {
            libflatpak::RefKind::App => Self::App,
            libflatpak::RefKind::Runtime => Self::Runtime,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub struct FlatpakRef {
    pub kind: FlatpakRefKind,
    pub ref_: String,
    pub id: String,
    pub arch: String,
    pub branch: String,
    pub commit: String,
    pub origin: String,
    // AppStream metadata specific fields
    pub name: Option<String>,
    pub version: Option<String>,
    pub license: Option<String>,
    pub summary: Option<String>,
    pub oars: Option<String>,
}

impl<O: glib::IsA<libflatpak::InstalledRef>> From<O> for FlatpakRef {
    #[must_use]
    fn from(value: O) -> Self {
        let value = value.upcast();

        Self {
            kind: value.kind().into(),
            ref_: value.format_ref_cached().unwrap().into(),
            id: value.name().unwrap().into(),
            arch: value.arch().unwrap().into(),
            branch: value.branch().unwrap().into(),
            commit: value.commit().unwrap().into(),
            origin: value.origin().unwrap().into(),
            name: value.appdata_name().map(|s| s.into()),
            version: value.appdata_version().map(|s| s.into()),
            license: value.appdata_license().map(|s| s.into()),
            summary: value.appdata_summary().map(|s| s.into()),
            oars: value.appdata_content_rating_type().map(|s| s.into()),
        }
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Diff, serde::Serialize, serde::Deserialize,
)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub enum FlatpakRemoteType {
    #[default]
    Static,
    Usb,
    Lan,
}

impl From<libflatpak::RemoteType> for FlatpakRemoteType {
    fn from(value: libflatpak::RemoteType) -> Self {
        match value {
            libflatpak::RemoteType::Static => Self::Static,
            libflatpak::RemoteType::Usb => Self::Usb,
            libflatpak::RemoteType::Lan => Self::Lan,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone, Diff, PartialEq, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub struct FlatpakRemote {
    pub type_: FlatpakRemoteType,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub collection_id: Option<String>,
    pub gpg_verify: bool,
    pub url: Option<String>,
    pub prio: i32,
}

impl<O: glib::IsA<libflatpak::Remote>> From<O> for FlatpakRemote {
    #[must_use]
    fn from(value: O) -> Self {
        let value = value.upcast();

        Self {
            type_: value.type_().into(),
            name: value.name().unwrap().into(),
            title: value.title().map(|s| s.into()),
            description: value.description().map(|s| s.into()),
            collection_id: value.collection_id().map(|s| s.into()),
            gpg_verify: value.is_gpg_verify(),
            url: match &value.url().map(|s| s.to_string()).unwrap()[..] {
                // Map empty slices to Option<String>::None
                "" => None,
                s => Some(s.into()),
            },
            prio: value.prio(),
        }
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Diff, serde::Serialize, serde::Deserialize,
)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub enum FlatpakInstallationStorageType {
    #[default]
    Default,
    Network,
    Mmc,
    Sdcard,
    HardDisk,
}

impl From<libflatpak::StorageType> for FlatpakInstallationStorageType {
    fn from(value: libflatpak::StorageType) -> Self {
        match value {
            libflatpak::StorageType::Default => Self::Default,
            libflatpak::StorageType::HardDisk => Self::HardDisk,
            libflatpak::StorageType::Sdcard => Self::Sdcard,
            libflatpak::StorageType::Mmc => Self::Mmc,
            libflatpak::StorageType::Network => Self::Network,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Default, Clone, Diff, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub fn user_installation() -> Result<Self, Error> {
        match Installation::new_user(gio::Cancellable::NONE) {
            Ok(item) => Ok(item.into()),
            Err(e) => Err(Error::FlatpakInstallationQueryFailure(e)),
        }
    }
}

#[derive(Debug, Clone, Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
#[repr(transparent)]
#[serde(transparent)]
pub struct FlatpakInstallationMap(pub HashMap<String, FlatpakInstallation>);

impl FlatpakInstallationMap {
    pub fn available_installations() -> Result<Self, Error> {
        Ok(Self(raw::installations().map(|v| {
            v.into_iter()
                .map(|item| (item.id().unwrap().into(), item.into()))
                .collect()
        })?))
    }
}
