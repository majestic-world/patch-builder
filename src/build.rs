//! A Build: turns a Source into an Update tree, re-zipping only the files that differ
//! from the Update tree's current Manifest.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::comparison::{ComparedFile, FileStatus};
use crate::manifest::{self, ARCHIVE_SUFFIX, Entry, HASH_ALGORITHM, Manifest};

pub struct Report {
    /// Every Source file with its status against the previous Manifest, in path order.
    pub files: Vec<ComparedFile>,
    /// Archives deleted because their Source file is gone.
    pub removed_archives: u64,
    pub manifest_version: u64,
    /// False when the file list is unchanged and the previous Manifest was kept as is.
    pub manifest_written: bool,
}

#[derive(Debug)]
pub enum BuildError {
    SourceNotAFolder(PathBuf),
    OverlappingFolders,
    SymbolicLink(PathBuf),
    NonUnicodePath(PathBuf),
    /// Two Source files whose paths differ only in letter case; Windows installs cannot hold both.
    CaseCollision { first: String, second: String },
    Manifest { path: PathBuf, error: manifest::ReadError },
    Io { path: PathBuf, error: io::Error },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNotAFolder(path) => write!(f, "The Source {} is not a folder.", path.display()),
            Self::OverlappingFolders => f.write_str("The Source and the Update tree must not be inside each other."),
            Self::SymbolicLink(path) => {
                write!(f, "{} is a symbolic link; the Source must hold regular files only.", path.display())
            }
            Self::NonUnicodePath(path) => write!(f, "{} has a name that is not valid Unicode.", path.display()),
            Self::CaseCollision { first, second } => write!(
                f,
                "{first} and {second} differ only in letter case; Windows installs cannot hold both."
            ),
            Self::Manifest { path, error } => match error {
                manifest::ReadError::Io(error) => write!(f, "Cannot read {}: {error}", path.display()),
                manifest::ReadError::Invalid(error) => {
                    write!(f, "{} is not a valid Manifest: {error}", path.display())
                }
                manifest::ReadError::UnsupportedHash(algorithm) => write!(
                    f,
                    "{} uses the {algorithm} hash; this Build only reads {HASH_ALGORITHM} Manifests.",
                    path.display()
                ),
            },
            Self::Io { path, error } => write!(f, "{}: {error}", path.display()),
        }
    }
}

impl std::error::Error for BuildError {}

fn io_at(path: &Path) -> impl FnOnce(io::Error) -> BuildError + '_ {
    move |error| BuildError::Io { path: path.to_owned(), error }
}

struct SourceFile {
    /// Relative to the Source root, `/`-separated.
    path: String,
    absolute: PathBuf,
    size: u64,
}

/// Runs a Build in place: the Update tree's current Manifest is the previous Build.
///
/// `on_progress(done, total)` reports Source bytes processed; it is called from worker threads.
pub fn run(
    source: &Path,
    update_tree: &Path,
    on_progress: &(dyn Fn(u64, u64) + Sync),
) -> Result<Report, BuildError> {
    check_folders(source, update_tree)?;
    let files = scan(source)?;
    let previous = manifest::read(update_tree)
        .map_err(|error| BuildError::Manifest { path: manifest::path_in(update_tree), error })?;
    let previous_entries: HashMap<&str, &Entry> =
        previous.iter().flat_map(|manifest| &manifest.files).map(|entry| (entry.path.as_str(), entry)).collect();

    let total = files.iter().map(|file| file.size).sum();
    let done = AtomicU64::new(0);
    on_progress(0, total);

    // Largest files first, so a big file never starts last and leaves the other cores idle.
    let mut by_size: Vec<&SourceFile> = files.iter().collect();
    by_size.sort_unstable_by_key(|file| std::cmp::Reverse(file.size));
    let mut compared = by_size
        .par_iter()
        .map(|file| {
            let compared = process(file, update_tree, previous_entries.get(file.path.as_str()).copied())?;
            on_progress(done.fetch_add(file.size, Ordering::Relaxed) + file.size, total);
            Ok(compared)
        })
        .collect::<Result<Vec<_>, BuildError>>()?;
    compared.sort_unstable_by(|a, b| a.path.cmp(&b.path));

    let current: HashSet<&str> = files.iter().map(|file| file.path.as_str()).collect();
    let mut removed_archives = 0;
    for entry in previous.iter().flat_map(|manifest| &manifest.files) {
        if !current.contains(entry.path.as_str()) && remove_archive(update_tree, &entry.path)? {
            removed_archives += 1;
        }
    }

    let entries: Vec<Entry> = compared
        .iter()
        .map(|file| Entry { path: file.path.clone(), size: file.size, hash: file.hash.clone() })
        .collect();
    let (manifest_version, manifest_written) = match previous {
        Some(previous) if previous.files == entries => (previous.version, false),
        previous => {
            let version = previous.map_or(1, |previous| previous.version + 1);
            let manifest = Manifest { version, hash_algorithm: HASH_ALGORITHM.to_owned(), files: entries };
            // Written last: Launchers only see the new Manifest once every Archive it lists exists.
            manifest::write(update_tree, &manifest).map_err(io_at(&manifest::path_in(update_tree)))?;
            (version, true)
        }
    };

    Ok(Report { files: compared, removed_archives, manifest_version, manifest_written })
}

fn check_folders(source: &Path, update_tree: &Path) -> Result<(), BuildError> {
    if !source.is_dir() {
        return Err(BuildError::SourceNotAFolder(source.to_owned()));
    }
    fs::create_dir_all(update_tree).map_err(io_at(update_tree))?;
    let source = fs::canonicalize(source).map_err(io_at(source))?;
    let update_tree = fs::canonicalize(update_tree).map_err(io_at(update_tree))?;
    if source.starts_with(&update_tree) || update_tree.starts_with(&source) {
        return Err(BuildError::OverlappingFolders);
    }
    Ok(())
}

/// Every regular file under `source`, sorted by relative path.
fn scan(source: &Path) -> Result<Vec<SourceFile>, BuildError> {
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
            files.push(SourceFile { path, absolute, size });
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
    let relative = absolute.strip_prefix(source).expect("scanned entries live under the Source");
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

fn archive_path(update_tree: &Path, path: &str) -> PathBuf {
    let mut archive = update_tree.to_owned();
    archive.extend(path.split('/'));
    archive.as_mut_os_string().push(ARCHIVE_SUFFIX);
    archive
}

fn process(file: &SourceFile, update_tree: &Path, previous: Option<&Entry>) -> Result<ComparedFile, BuildError> {
    let hash = hash_file(&file.absolute).map_err(io_at(&file.absolute))?;
    let archive = archive_path(update_tree, &file.path);
    let status = match previous {
        None => FileStatus::New,
        Some(entry) if entry.size == file.size && entry.hash == hash && archive.is_file() => FileStatus::Unchanged,
        // Also covers a listed file whose Archive went missing: the Update tree needs it back.
        Some(_) => FileStatus::Changed,
    };
    if status != FileStatus::Unchanged {
        write_archive(file, &archive)?;
    }
    Ok(ComparedFile { path: file.path.clone(), status, size: file.size, hash })
}

fn hash_file(path: &Path) -> io::Result<String> {
    let mut hasher = blake3::Hasher::new();
    // Memory-maps large files and hashes their chunks on the shared thread pool.
    hasher.update_mmap_rayon(path)?;
    Ok(hasher.finalize().to_hex().to_string())
}

/// Zips one Source file under its own name, then moves the result over the old Archive.
fn write_archive(file: &SourceFile, archive: &Path) -> Result<(), BuildError> {
    let folder = archive.parent().expect("Archives live inside the Update tree");
    fs::create_dir_all(folder).map_err(io_at(folder))?;
    let mut staging = archive.as_os_str().to_owned();
    staging.push(".tmp");
    let staging = PathBuf::from(staging);
    let name = file.path.rsplit('/').next().unwrap_or(&file.path);

    let written = (|| -> io::Result<()> {
        let mut zip = ZipWriter::new(BufWriter::new(File::create(&staging)?));
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .large_file(file.size >= u64::from(u32::MAX));
        zip.start_file(name, options)?;
        io::copy(&mut BufReader::with_capacity(1 << 20, File::open(&file.absolute)?), &mut zip)?;
        zip.finish()?.flush()?;
        fs::rename(&staging, archive)
    })();
    written.map_err(|error| {
        // The staging file is incomplete; losing it is harmless and the error below says why.
        let _ = fs::remove_file(&staging);
        BuildError::Io { path: archive.to_owned(), error }
    })
}

/// Deletes the Archive of a file that left the Source, plus any folders it leaves empty.
fn remove_archive(update_tree: &Path, path: &str) -> Result<bool, BuildError> {
    let archive = archive_path(update_tree, path);
    match fs::remove_file(&archive) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(BuildError::Io { path: archive, error }),
    }
    let mut folder = archive.parent();
    while let Some(current) = folder.filter(|current| *current != update_tree) {
        // Stops at the first folder that still holds something.
        if fs::remove_dir(current).is_err() {
            break;
        }
        folder = current.parent();
    }
    Ok(true)
}

#[cfg(test)]
#[path = "../tests/unit/build.rs"]
mod tests;
