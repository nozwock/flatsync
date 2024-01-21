use crate::{
    error::Error,
    models::{FlatpakInstallation, FlatpakInstallationKind, FlatpakInstallationMap},
};
use chrono::{DateTime, Utc};
use log::trace;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FlatpakInstallationPayload {
    pub installations: FlatpakInstallationMap,
    pub altered_at: DateTime<Utc>,
}

impl FlatpakInstallationPayload {
    pub fn new_from_system() -> Result<FlatpakInstallationPayload, Error> {
        let installations = FlatpakInstallationMap::available_installations()?;
        let altered_at = Utc::now();

        Ok(Self {
            installations,
            altered_at,
        })
    }

    /// ## `installations_from_file()`
    /// Reads the local installations file and returns a `FlatpakInstallationPayload` from it.
    /// Returns an `Error` if the file doesn't exist or if it fails to read it.
    ///
    /// * `file_path` - The path to the local installations file.
    pub fn new_from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Error> {
        let path = file_path.as_ref();
        if !path.exists() {
            return Err(Error::FlatpakInstallationFileFailure(
                "File doesn't exist".into(),
            ));
        }

        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let file_payload: FlatpakInstallationPayload = serde_json::from_reader(reader)
            .map_err(|e| Error::FlatpakInstallationFileFailure(e.to_string()))?;

        trace!("Read from file '{:?}': {:?}", path, file_payload);

        Ok(file_payload)
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, file_path: &P) -> Result<(), Error> {
        let serialized = serde_json::to_string(self).map_err(|e| {
            Error::FlatpakInstallationFileFailure(format!(
                "Failed to serialize local payload: {}",
                e
            ))
        })?;

        trace!(
            "Writing to file '{:?}': {:?}",
            file_path.as_ref(),
            serialized
        );

        std::fs::write(file_path, serialized)
            .map_err(|e| Error::FlatpakInstallationFileFailure(e.to_string()))?;

        Ok(())
    }

    pub fn installations(&self, kind: FlatpakInstallationKind) -> Option<&FlatpakInstallation> {
        self.installations.get(kind)
    }
}
