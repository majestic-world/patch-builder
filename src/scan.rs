//! A Scan: compares the Source with the Update tree's current Manifest and plans the next Build,
//! without writing anything.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;

use crate::comparison::{ComparedFile, FileStatus};
use crate::error::{BuildError, io_at};
use crate::manifest::{self, Entry, HashAlgorithm, Manifest};

/// What the next Build publishes, as found by the last Scan.
#[derive(Debug)]
pub struct Plan {
    pub source: PathBuf,
    pub update_tree: PathBuf,
    pub algorithm: HashAlgorithm,
    /// Every Source file with its status against `previous`, in path order.
    pub files: Vec<ComparedFile>,
    /// Paths listed in `previous` that left the Source; the Build deletes their Archives.
    pub removed: Vec<String>,
    /// The Update tree's Manifest at Scan time; `None` before its first Build.
    pub previous: Option<Manifest>,
}

impl Plan {
    /// Manifest entries for the scanned Source, in path order.
    pub fn entries(&self) -> Vec<Entry> {
        self.files
            .iter()
            .map(|file| Entry {
                path: file.path.clone(), size: file.size, hash: file.hash.clone(), crc32: file.crc32.clone(),
            })
            .collect()
    }

    /// Whether the Build must write a new Manifest.
    pub fn changes_manifest(&self) -> bool {
        self.previous.as_ref().is_none_or(|previous| {
            previous.hash_algorithm != self.algorithm || previous.files != self.entries()
        })
    }

    /// Manifest version the Update tree holds once this Plan is built.
    pub fn next_version(&self) -> u64 {
        match &self.previous {
            None => 1,
            Some(previous) if self.changes_manifest() => previous.version + 1,
            Some(previous) => previous.version,
        }
    }

    /// Whether building this Plan would change the Update tree at all.
    pub fn has_work(&self) -> bool {
        self.changes_manifest()
            || !self.removed.is_empty()
            || self.files.iter().any(|file| file.status != FileStatus::Unchanged)
    }

    /// The Plan a fresh Scan finds right after this Plan was built into `published`.
    pub fn into_published(mut self, published: Manifest) -> Self {
        for file in &mut self.files {
            file.status = FileStatus::Unchanged;
        }
        self.removed.clear();
        self.previous = Some(published);
        self
    }
}

pub fn read_manifest(update_tree: &Path) -> Result<Option<Manifest>, BuildError> {
    manifest::read(update_tree).map_err(|error| BuildError::Manifest { path: manifest::path_in(update_tree), error })
}

/// Hashes every Source file and compares it with the Update tree's current Manifest.
///
/// `on_progress(done, total)` reports Source bytes hashed; it is called from worker threads.
pub fn run(source: &Path, update_tree: &Path, algorithm: HashAlgorithm, on_progress: &(dyn Fn(u64, u64) + Sync)) -> Result<Plan, BuildError> {
    check_folders(source, update_tree)?;
    let found = walk(source)?;
    let previous = read_manifest(update_tree)?;
    let previous_entries: HashMap<&str, &Entry> =
        previous.iter().flat_map(|manifest| &manifest.files).map(|entry| (entry.path.as_str(), entry)).collect();

    let total = found.iter().map(|file| file.size).sum();
    let done = AtomicU64::new(0);
    on_progress(0, total);

    // Largest files first, so a big file never starts last and leaves the other cores idle.
    let mut by_size: Vec<&FoundFile> = found.iter().collect();
    by_size.sort_unstable_by_key(|file| std::cmp::Reverse(file.size));
    let mut files = by_size
        .par_iter()
        .map(|file| {
            let previous_entry = previous_entries.get(file.path.as_str());
            let previous_algorithm = previous_entry
                .filter(|entry| entry.size == file.size)
                .and_then(|_| previous.as_ref().map(|manifest| manifest.hash_algorithm));
            let need_blake3 = algorithm.blake3() || previous_algorithm.is_some_and(HashAlgorithm::blake3);
            let need_crc32 = algorithm.crc32() || previous_algorithm.is_some_and(HashAlgorithm::crc32);
            let hashes = hash_file(&file.absolute, need_blake3, need_crc32).map_err(io_at(&file.absolute))?;
            let status = match previous_entry {
                None => FileStatus::New,
                Some(entry)
                    if entry.size == file.size
                        && hashes.matches_entry(entry, previous_algorithm.expect("entry belongs to a manifest")) =>
                {
                    let archive = manifest::archive_path(update_tree, &file.path);
                    if archive.is_file()
                        && (previous_algorithm != Some(HashAlgorithm::Crc32) || !algorithm.blake3()
                            || archive_matches_blake3(
                                &archive,
                                hashes.blake3.as_deref().expect("BLAKE3 was requested"),
                            ))
                    {
                        FileStatus::Unchanged
                    } else {
                        FileStatus::Changed
                    }
                }
                // The file's size or its previous hash differs.
                Some(_) => FileStatus::Changed,
            };
            on_progress(done.fetch_add(file.size, Ordering::Relaxed) + file.size, total);
            let (hash, crc32) = hashes.selected(algorithm);
            Ok(ComparedFile { path: file.path.clone(), status, size: file.size, hash, crc32 })
        })
        .collect::<Result<Vec<_>, BuildError>>()?;
    files.sort_unstable_by(|a, b| a.path.cmp(&b.path));

    let current: HashSet<&str> = found.iter().map(|file| file.path.as_str()).collect();
    let removed = previous
        .iter()
        .flat_map(|manifest| &manifest.files)
        .filter(|entry| !current.contains(entry.path.as_str()))
        .map(|entry| entry.path.clone())
        .collect();

    Ok(Plan { source: source.to_owned(), update_tree: update_tree.to_owned(), algorithm, files, removed, previous })
}

#[derive(Debug)]
pub(crate) struct Hashes {
    blake3: Option<String>,
    crc32: Option<String>,
}

impl Hashes {
    fn matches_entry(&self, entry: &Entry, algorithm: HashAlgorithm) -> bool {
        match algorithm {
            HashAlgorithm::Blake3 => self.blake3.as_deref() == Some(entry.hash.as_str()),
            HashAlgorithm::Crc32 => self.crc32.as_deref() == Some(entry.hash.as_str()),
            HashAlgorithm::Both => {
                self.blake3.as_deref() == Some(entry.hash.as_str())
                    && self.crc32.as_deref().zip(entry.crc32.as_deref()).is_some_and(|(a, b)| a == b)
            }
        }
    }

    pub(crate) fn selected(self, algorithm: HashAlgorithm) -> (String, Option<String>) {
        match algorithm {
            HashAlgorithm::Blake3 => (self.blake3.expect("BLAKE3 was requested"), None),
            HashAlgorithm::Crc32 => (self.crc32.expect("CRC32 was requested"), None),
            HashAlgorithm::Both => (self.blake3.expect("BLAKE3 was requested"), self.crc32),
        }
    }
}

pub(crate) struct Hashers {
    blake3: Option<blake3::Hasher>,
    crc32: Option<crc32fast::Hasher>,
}

impl Hashers {
    pub(crate) fn new(blake3: bool, crc32: bool) -> Self {
        Self {
            blake3: blake3.then(blake3::Hasher::new),
            crc32: crc32.then(crc32fast::Hasher::new),
        }
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) {
        if let Some(hasher) = &mut self.blake3 {
            hasher.update(bytes);
        }
        if let Some(hasher) = &mut self.crc32 {
            hasher.update(bytes);
        }
    }

    pub(crate) fn finish(self) -> Hashes {
        Hashes {
            blake3: self.blake3.map(|hasher| hasher.finalize().to_hex().to_string()),
            crc32: self.crc32.map(|hasher| format!("{:08x}", hasher.finalize())),
        }
    }
}

fn hash_file(path: &Path, need_blake3: bool, need_crc32: bool) -> io::Result<Hashes> {
    let mut hashers = Hashers::new(need_blake3, need_crc32);
    if need_blake3 && !need_crc32 {
        // Keep the optimized mmap and parallel path for BLAKE3-only scans.
        hashers.blake3.as_mut().expect("BLAKE3 was requested").update_mmap_rayon(path)?;
    } else {
        let mut file = fs::File::open(path)?;
        let mut buffer = [0; 64 * 1024];
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            hashers.update(&buffer[..read]);
        }
    }
    Ok(hashers.finish())
}

/// CRC32 alone cannot prove an existing archive matches a newly selected BLAKE3 digest.
pub(crate) fn archive_matches_blake3(path: &Path, expected: &str) -> bool {
    let check = || -> Option<bool> {
        let mut archive = zip::ZipArchive::new(fs::File::open(path).ok()?).ok()?;
        if archive.len() != 1 {
            return Some(false);
        }
        let mut entry = archive.by_index(0).ok()?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0; 64 * 1024];
        loop {
            let read = entry.read(&mut buffer).ok()?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Some(hasher.finalize().to_hex().as_str() == expected)
    };
    check().unwrap_or(false)
}

fn check_folders(source: &Path, update_tree: &Path) -> Result<(), BuildError> {
    if !source.is_dir() {
        return Err(BuildError::SourceNotAFolder(source.to_owned()));
    }
    let source = fs::canonicalize(source).map_err(io_at(source))?;
    let update_tree = resolve(update_tree)?;
    if source.starts_with(&update_tree) || update_tree.starts_with(&source) {
        return Err(BuildError::OverlappingFolders);
    }
    Ok(())
}

/// Absolute form of a path that may not exist yet: its nearest existing ancestor, canonicalized.
fn resolve(path: &Path) -> Result<PathBuf, BuildError> {
    match fs::canonicalize(path) {
        Ok(resolved) => Ok(resolved),
        Err(error) if error.kind() == io::ErrorKind::NotFound => match (path.parent(), path.file_name()) {
            (Some(parent), Some(name)) if !parent.as_os_str().is_empty() => Ok(resolve(parent)?.join(name)),
            _ => Err(BuildError::Io { path: path.to_owned(), error }),
        },
        Err(error) => Err(BuildError::Io { path: path.to_owned(), error }),
    }
}

struct FoundFile {
    /// Relative to the Source root, `/`-separated.
    path: String,
    absolute: PathBuf,
    size: u64,
}

/// Every regular file under `source`, sorted by relative path.
fn walk(source: &Path) -> Result<Vec<FoundFile>, BuildError> {
    let mut files = Vec::new();
    let mut folders = vec![source.to_owned()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).map_err(io_at(&folder))? {
            let entry = entry.map_err(io_at(&folder))?;
            let absolute = entry.path();
            let file_type = entry.file_type().map_err(io_at(&absolute))?;
            if file_type.is_symlink() {
                return Err(BuildError::SymbolicLink(absolute));
            }
            if file_type.is_dir() {
                folders.push(absolute);
                continue;
            }
            let size = entry.metadata().map_err(io_at(&absolute))?.len();
            let path = relative_path(source, &absolute)?;
            files.push(FoundFile { path, absolute, size });
        }
    }
    files.sort_unstable_by(|a, b| a.path.cmp(&b.path));

    let mut seen: HashMap<String, &str> = HashMap::with_capacity(files.len());
    for file in &files {
        if let Some(first) = seen.insert(file.path.to_lowercase(), &file.path) {
            return Err(BuildError::CaseCollision { first: first.to_owned(), second: file.path.clone() });
        }
    }
    Ok(files)
}

fn relative_path(source: &Path, absolute: &Path) -> Result<String, BuildError> {
    let relative = absolute.strip_prefix(source).expect("walked entries live under the Source");
    let mut path = String::new();
    for component in relative.components() {
        let name = component.as_os_str().to_str().ok_or_else(|| BuildError::NonUnicodePath(absolute.to_owned()))?;
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(name);
    }
    Ok(path)
}

#[cfg(test)]
#[path = "../tests/unit/scan.rs"]
mod tests;
