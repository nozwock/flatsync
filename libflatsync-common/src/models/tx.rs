use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    installations: Vec<InstallationTx>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InstallationTx {
    New {
        id: String,
        path: PathBuf,
    },
    Modify {
        id: String,
        remotes: Vec<RemoteTx>,
        refs: Vec<RefTx>,
    },
    Remove {
        id: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RemoteTx {
    New {
        type_: super::FlatpakRemoteType,
        name: String,
        url: String,
        gpg_verify: bool,
        prio: i32,
        title: Option<String>,
        description: Option<String>,
        collection_id: Option<String>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum RefTx {
    /// Installations
    New {
        ref_: String,
        origin: String,
        name: Option<String>,
        version: Option<String>,
        license: Option<String>,
        oars: Option<String>,
    },
    /// Updates
    Update {
        ref_: String,
        commit: String,
        name: Option<String>,
        version: Option<String>,
        license: Option<String>,
        oars: Option<String>,
    },
    /// Removals/uninstalls
    Remove {
        ref_: String,
        name: Option<String>,
        version: Option<String>,
        license: Option<String>,
        oars: Option<String>,
    },
}
