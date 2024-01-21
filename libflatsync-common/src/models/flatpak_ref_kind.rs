/// Flatpak reference kind, used to differentiate between app and runtime refs.
/// Subset of the `libflatpak::RefKind` enum which can be diffed and serialized.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    diff_derive::Diff,
    serde::Serialize,
    serde::Deserialize,
)]
#[diff(attr(#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]))]
pub enum FlatpakRefKind {
    #[default]
    App,
    Runtime,
}

/// Converts a `libflatpak::RefKind` into a `FlatpakRefKind`.
impl From<libflatpak::RefKind> for FlatpakRefKind {
    /// Converts the given `libflatpak::RefKind` value into a `FlatpakRefKind` value.
    ///
    /// # Arguments
    ///
    /// * `value` - The `libflatpak::RefKind` value to convert.
    ///
    /// # Returns
    ///
    /// The converted `FlatpakRefKind` value.
    fn from(value: libflatpak::RefKind) -> Self {
        match value {
            libflatpak::RefKind::App => Self::App,
            libflatpak::RefKind::Runtime => Self::Runtime,
            _ => unimplemented!(),
        }
    }
}
