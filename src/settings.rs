//! Folders, hash selection, and appearance remembered between sessions in the user's app-data folder.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::manifest::HashAlgorithm;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub update_tree: String,
    #[serde(default)]
    pub hash_algorithm: HashAlgorithm,
    #[serde(default)]
    pub dark_theme: bool,
}

/// `%APPDATA%\Patch Builder\settings.json` on Windows; the platform config folder elsewhere.
fn file() -> io::Result<PathBuf> {
    let config = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no app-data folder for this user"))?;
    Ok(config.join("Patch Builder").join("settings.json"))
}

/// Saved settings; a first run, or a file this version cannot read, uses defaults.
pub fn load() -> Settings {
    file().and_then(fs::read).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok()).unwrap_or_default()
}

pub fn save(settings: &Settings) -> io::Result<()> {
    let file = file()?;
    if let Some(folder) = file.parent() {
        fs::create_dir_all(folder)?;
    }
    fs::write(file, serde_json::to_vec_pretty(settings)?)
}

#[cfg(test)]
#[path = "../tests/unit/settings.rs"]
mod tests;
