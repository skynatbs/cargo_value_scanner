use std::fs;
use std::io;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde_json::Error as SerdeError;

use crate::domain::app_state::PersistedState;

const APP_QUALIFIER: &str = "com";
const APP_ORG: &str = "CargoValueScanner";
const APP_NAME: &str = "CargoValueScanner";

fn data_file() -> Option<PathBuf> {
    ProjectDirs::from(APP_QUALIFIER, APP_ORG, APP_NAME)
        .map(|dirs| dirs.config_dir().join("state.json"))
}

pub fn load_persisted_state() -> Option<PersistedState> {
    let path = data_file()?;
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_persisted_state(state: &PersistedState) -> Result<(), PersistSaveError> {
    let path = data_file().ok_or(PersistSaveError::StorageUnavailable)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(state)?;
    fs::write(path, json)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PersistSaveError {
    #[error("storage directory unavailable")]
    StorageUnavailable,
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] SerdeError),
}
