use std::fmt;

/// Represents the kind of Flatpak installation. This is either a user or system installation.
#[derive(
    Hash, Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq, PartialOrd, Ord,
)]
pub enum FlatpakInstallationKind {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "default")]
    System,
}

impl FlatpakInstallationKind {
    pub fn try_from_str(s: &str) -> Result<Self, crate::Error> {
        match s {
            "user" => Ok(Self::User),
            "default" => Ok(Self::System),
            _ => Err(crate::Error::InvalidFlatpakInstallationKind(s.into())),
        }
    }
}

impl fmt::Display for FlatpakInstallationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::System => write!(f, "default"),
        }
    }
}
