use crate::models::FlatpakRefKind;
use libflatpak::{glib, prelude::*};

/// Represents a Flatpak reference. This is a subset of the `libflatpak::InstalledRef` struct which can be diffed and serialized.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    Hash,
    diff_derive::Diff,
    serde::Serialize,
    serde::Deserialize,
)]
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

/// Converts a `libflatpak::InstalledRef` into a `FlatpakRef` struct.
///
/// # Arguments
///
/// * `value` - The value to convert into a `FlatpakRef`.
///
/// # Returns
///
/// The converted `FlatpakRef` struct.
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
