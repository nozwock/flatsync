use std::{collections::HashMap, path::PathBuf};

use diff_derive::Diff;
use libflatpak::{
    gio::{self, traits::FileExt},
    glib::{self, Cast},
    traits::{InstallationExt, InstalledRefExt, RefExt, RemoteExt},
    Installation,
};

pub mod tx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub enum FlatpakRefKind {
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

#[derive(Debug, Clone, PartialEq, Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub struct FlatpakRef {
    kind: FlatpakRefKind,
    ref_: String,
    id: String,
    arch: String,
    branch: String,
    commit: String,
    origin: String,
    // AppStream metadata specific fields
    name: Option<String>,
    version: Option<String>,
    license: Option<String>,
    summary: Option<String>,
    oars: Option<String>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub enum FlatpakRemoteType {
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

#[derive(Debug, Clone, Diff, PartialEq, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub struct FlatpakRemote {
    type_: FlatpakRemoteType,
    name: String,
    title: Option<String>,
    description: Option<String>,
    collection_id: Option<String>,
    gpg_verify: bool,
    /// Note: local remotes have empty URLs
    url: String,
    prio: i32,
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
            url: value.url().unwrap().into(),
            prio: value.prio(),
        }
    }
}

#[derive(Debug, Clone, Diff, PartialEq, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub struct FlatpakInstallation {
    id: String,
    path: PathBuf,
    refs: Vec<FlatpakRef>,
    remotes: Vec<FlatpakRemote>,
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

#[derive(Debug, Clone, Diff, serde::Serialize, serde::Deserialize)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
#[repr(transparent)]
#[serde(transparent)]
pub struct FlatpakInstallationMap(HashMap<String, FlatpakInstallation>);

impl FlatpakInstallationMap {
    pub fn available_installations() -> Result<Self, crate::Error> {
        let mut ret: HashMap<_, _> = match libflatpak::system_installations(gio::Cancellable::NONE)
        {
            Ok(v) => v
                .into_iter()
                .map(|item| (item.id().unwrap().into(), item.into()))
                .collect(),
            Err(e) => return Err(crate::Error::FlatpakInstallationQueryFailure(e)),
        };

        let user_inst = match FlatpakInstallation::user_installation() {
            Ok(inst) => (inst.id.to_owned(), inst),
            Err(e) => return Err(e),
        };

        ret.insert(user_inst.0, user_inst.1);

        Ok(Self(ret))
    }
}

// FlatpakInstallationMapDiff is the expansion of `#[derive(Diff, ..)] pub struct FlatpakInstallationMap`
impl FlatpakInstallationMapDiff {}
