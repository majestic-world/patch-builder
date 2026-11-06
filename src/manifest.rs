//! The Manifest: the JSON document at the root of an Update tree that Launchers read.

use std::fs;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "manifest.json";
/// Suffix that turns a Source file path into its Archive path.
pub const ARCHIVE_SUFFIX: &str = ".zip";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    #[default]
    #[serde(rename = "blake3")]
    Blake3,
    #[serde(rename = "crc32")]
    Crc32,
    #[serde(rename = "blake3+crc32")]
    Both,
}

impl HashAlgorithm {
    pub fn blake3(self) -> bool {
        matches!(self, Self::Blake3 | Self::Both)
    }

    pub fn crc32(self) -> bool {
        matches!(self, Self::Crc32 | Self::Both)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Grows by one whenever the file list or selected hash algorithm changes.
    pub version: u64,
    pub hash_algorithm: HashAlgorithm,
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
    /// CRC32 of the uncompressed Source file when both algorithms are selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crc32: Option<String>,
}

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Invalid(serde_json::Error),
}

pub fn path_in(update_tree: &Path) -> PathBuf {
    update_tree.join(FILE_NAME)
}

/// Where the Archive of the Source file at `path` lives inside the Update tree.
pub fn archive_path(update_tree: &Path, path: &str) -> PathBuf {
    let mut archive = update_tree.to_owned();
    archive.extend(path.split('/'));
    archive.as_mut_os_string().push(ARCHIVE_SUFFIX);
    archive
}

/// The Update tree's current Manifest, or `None` before its first Build.
pub fn read(update_tree: &Path) -> Result<Option<Manifest>, ReadError> {
    let bytes = match fs::read(path_in(update_tree)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(ReadError::Io(error)),
    };
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(ReadError::Invalid)?;
    if manifest.files.iter().any(|entry| {
        (manifest.hash_algorithm == HashAlgorithm::Both) != entry.crc32.is_some()
    }) {
        return Err(ReadError::Invalid(<serde_json::Error as serde::de::Error>::custom(
            "CRC32 field does not match the manifest hash algorithm",
        )));
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
