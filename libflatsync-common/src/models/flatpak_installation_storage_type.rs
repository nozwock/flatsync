/// Represents the storage type for a Flatpak installation. This is a subset of the `libflatpak::StorageType` enum which can be diffed and serialized.
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
    /// The default storage type.
    #[default]
    Default,
    /// Network storage type.
    Network,
    /// MMC storage type.
    Mmc,
    /// SD card storage type.
    Sdcard,
    /// Hard disk storage type.
    HardDisk,
}

impl From<libflatpak::StorageType> for FlatpakInstallationStorageType {
    /// Converts a `libflatpak::StorageType` to `FlatpakInstallationStorageType`.
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
