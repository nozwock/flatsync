#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    diff_derive::Diff,
    serde::Serialize,
    serde::Deserialize,
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
