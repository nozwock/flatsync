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

impl From<libflatpak::RefKind> for FlatpakRefKind {
    fn from(value: libflatpak::RefKind) -> Self {
        match value {
            libflatpak::RefKind::App => Self::App,
            libflatpak::RefKind::Runtime => Self::Runtime,
            _ => unimplemented!(),
        }
    }
}
