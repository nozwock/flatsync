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
