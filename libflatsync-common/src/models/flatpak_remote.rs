use crate::models::FlatpakRemoteType;
use libflatpak::{glib, prelude::*};

#[derive(
    Debug, Default, Clone, diff_derive::Diff, PartialEq, serde::Serialize, serde::Deserialize,
)]
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
            type_: libflatpak::prelude::RemoteExt::type_(&value).into(),
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

impl From<&FlatpakRemote> for libflatpak::Remote {
    fn from(remote: &FlatpakRemote) -> Self {
        let ret = libflatpak::Remote::new(&remote.name);
        if let Some(val) = &remote.title {
            ret.set_title(val);
        }
        if let Some(val) = &remote.description {
            ret.set_description(val);
        }
        if let Some(val) = &remote.url {
            ret.set_url(val);
        }

        ret.set_collection_id(remote.collection_id.as_deref());
        ret.set_gpg_verify(remote.gpg_verify);
        ret.set_prio(remote.prio);

        ret
    }
}
