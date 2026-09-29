//! The Manifest: the JSON document at the root of an Update tree that Launchers read.

use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "manifest.json";
pub const HASH_ALGORITHM: &str = "blake3";
/// Suffix that turns a Source file path into its Archive path.
pub const ARCHIVE_SUFFIX: &str = ".zip";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Grows by one whenever the file list changes, so Launchers can skip hashing an up-to-date install.
    pub version: u64,
    pub hash_algorithm: String,
    /// Sorted by path.
    pub files: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Source file path relative to the Source root, `/`-separated; its Archive is `path` + `.zip`.
    pub path: String,
    /// Size of the uncompressed Source file, in bytes.
    pub size: u64,
    /// Hash of the uncompressed Source file, lowercase hex.
    pub hash: String,
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Invalid(serde_json::Error),
    UnsupportedHash(String),
}

pub fn path_in(update_tree: &Path) -> PathBuf {
    update_tree.join(FILE_NAME)
}

/// The Update tree's current Manifest, or `None` before its first Build.
pub fn read(update_tree: &Path) -> Result<Option<Manifest>, ReadError> {
    let bytes = match fs::read(path_in(update_tree)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ReadError::Io(error)),
    };
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(ReadError::Invalid)?;
    if manifest.hash_algorithm != HASH_ALGORITHM {
        return Err(ReadError::UnsupportedHash(manifest.hash_algorithm));
    }
    Ok(Some(manifest))
}

/// Replaces the Manifest in one step, so a Launcher never reads a half-written file.
pub fn write(update_tree: &Path, manifest: &Manifest) -> io::Result<()> {
    let target = path_in(update_tree);
    let staging = update_tree.join(format!("{FILE_NAME}.tmp"));
    let mut writer = BufWriter::new(fs::File::create(&staging)?);
    serde_json::to_writer_pretty(&mut writer, manifest)?;
    writer.write_all(b"\n")?;
    writer.into_inner().map_err(io::IntoInnerError::into_error)?.sync_all()?;
    fs::rename(&staging, &target)
}
