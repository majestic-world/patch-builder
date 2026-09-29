//! A Scan: compares the Source with the Update tree's current Manifest and plans the next Build,
//! without writing anything.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;

use crate::comparison::{ComparedFile, FileStatus};
use crate::error::{BuildError, io_at};
use crate::manifest::{self, Entry, Manifest};

/// What the next Build publishes, as found by the last Scan.
#[derive(Debug)]
pub struct Plan {
    pub source: PathBuf,
    pub update_tree: PathBuf,
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
            .map(|file| Entry { path: file.path.clone(), size: file.size, hash: file.hash.clone() })
            .collect()
    }

    /// Whether the Build must write a new Manifest (its file list differs from `previous`).
    pub fn changes_manifest(&self) -> bool {
        self.previous.as_ref().is_none_or(|previous| previous.files != self.entries())
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
        self.previous.is_none()
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
pub fn run(source: &Path, update_tree: &Path, on_progress: &(dyn Fn(u64, u64) + Sync)) -> Result<Plan, BuildError> {
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
            let hash = hash_file(&file.absolute).map_err(io_at(&file.absolute))?;
            let status = match previous_entries.get(file.path.as_str()) {
                None => FileStatus::New,
                Some(entry)
                    if entry.size == file.size
                        && entry.hash == hash
                        && manifest::archive_path(update_tree, &file.path).is_file() =>
                {
                    FileStatus::Unchanged
                }
                // Also covers a listed file whose Archive went missing: the Update tree needs it back.
                Some(_) => FileStatus::Changed,
            };
            on_progress(done.fetch_add(file.size, Ordering::Relaxed) + file.size, total);
            Ok(ComparedFile { path: file.path.clone(), status, size: file.size, hash })
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

    Ok(Plan { source: source.to_owned(), update_tree: update_tree.to_owned(), files, removed, previous })
}

pub fn hash_file(path: &Path) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    // Memory-maps large files and hashes their chunks on the shared thread pool.
    hasher.update_mmap_rayon(path)?;
    Ok(hasher.finalize().to_hex().to_string())
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
