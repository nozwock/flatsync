/// Represents the type of a Flatpak remote.
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
    /// Represents a static Flatpak remote.
    #[default]
    Static,
    /// Represents a USB Flatpak remote.
    Usb,
    /// Represents a LAN Flatpak remote.
    Lan,
}

/// Converts a `libflatpak::RemoteType` into a `FlatpakRemoteType`.
impl From<libflatpak::RemoteType> for FlatpakRemoteType {
    /// Converts the given `libflatpak::RemoteType` value into a `FlatpakRemoteType`.
    ///
    /// # Arguments
    ///
    /// * `value` - The `libflatpak::RemoteType` value to convert.
    ///
    /// # Returns
    ///
    /// The converted `FlatpakRemoteType` value.
    fn from(value: libflatpak::RemoteType) -> Self {
        match value {
            libflatpak::RemoteType::Static => Self::Static,
            libflatpak::RemoteType::Usb => Self::Usb,
            libflatpak::RemoteType::Lan => Self::Lan,
            _ => unimplemented!(),
        }
    }
}
