//! A Build: applies the last Scan's Plan to the Update tree, re-zipping only New and Changed files.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rayon::prelude::*;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::comparison::{ComparedFile, FileStatus};
use crate::error::{BuildError, io_at};
use crate::manifest::{self, HASH_ALGORITHM, Manifest};
use crate::scan::{self, Plan};

pub struct Report {
    /// The Manifest the Update tree holds after the Build.
    pub manifest: Manifest,
    pub manifest_written: bool,
    pub rezipped: u64,
    pub removed_archives: u64,
}

/// Brings the Update tree in line with `plan`.
///
/// Refuses to run when the Update tree's Manifest moved since the Scan, and stops when a file to
/// re-zip no longer matches its scanned hash: the Manifest must describe exactly what the Archives hold.
/// `on_progress(done, total)` reports bytes of re-zipped files; it is called from worker threads.
pub fn run(plan: &Plan, on_progress: &(dyn Fn(u64, u64) + Sync)) -> Result<Report, BuildError> {
    if scan::read_manifest(&plan.update_tree)? != plan.previous {
        return Err(BuildError::UpdateTreeChanged);
    }
    fs::create_dir_all(&plan.update_tree).map_err(io_at(&plan.update_tree))?;

    // Largest files first, so a big file never starts last and leaves the other cores idle.
    let mut pending: Vec<&ComparedFile> =
        plan.files.iter().filter(|file| file.status != FileStatus::Unchanged).collect();
    pending.sort_unstable_by_key(|file| std::cmp::Reverse(file.size));
    let total = pending.iter().map(|file| file.size).sum();
    let done = AtomicU64::new(0);
    on_progress(0, total);
    pending.par_iter().try_for_each(|file| {
        write_archive(plan, file)?;
        on_progress(done.fetch_add(file.size, Ordering::Relaxed) + file.size, total);
        Ok::<_, BuildError>(())
    })?;

    let mut removed_archives = 0;
    for path in &plan.removed {
        if remove_archive(&plan.update_tree, path)? {
            removed_archives += 1;
        }
    }

    let manifest_written = plan.changes_manifest();
    let manifest = if manifest_written {
        let manifest =
            Manifest { version: plan.next_version(), hash_algorithm: HASH_ALGORITHM.to_owned(), files: plan.entries() };
        // Written last: Launchers only see the new Manifest once every Archive it lists exists.
        manifest::write(&plan.update_tree, &manifest).map_err(io_at(&manifest::path_in(&plan.update_tree)))?;
        manifest
    } else {
        plan.previous.clone().expect("an unchanged Manifest exists")
    };

    Ok(Report { manifest, manifest_written, rezipped: pending.len() as u64, removed_archives })
}

/// Hashes everything it reads, to prove the zipped bytes are the scanned ones.
struct HashingReader<R> {
    inner: R,
    hasher: blake3::Hasher,
}

impl<R: Read> Read for HashingReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.hasher.update(&buffer[..read]);
        Ok(read)
    }
}

/// Zips one Source file under its own name, then moves the result over the old Archive.
fn write_archive(plan: &Plan, file: &ComparedFile) -> Result<(), BuildError> {
    let archive = manifest::archive_path(&plan.update_tree, &file.path);
    let folder = archive.parent().expect("Archives live inside the Update tree");
    fs::create_dir_all(folder).map_err(io_at(folder))?;
    let mut staging = archive.as_os_str().to_owned();
    staging.push(".tmp");
    let staging = PathBuf::from(staging);
    let name = file.path.rsplit('/').next().unwrap_or(&file.path);
    let source: PathBuf = plan.source.join(&file.path);

    let written = (|| -> io::Result<blake3::Hash> {
        let mut zip = ZipWriter::new(BufWriter::new(File::create(&staging)?));
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .large_file(file.size >= u64::from(u32::MAX));
        zip.start_file(name, options)?;
        let mut reader =
            HashingReader { inner: BufReader::with_capacity(1 << 20, File::open(&source)?), hasher: blake3::Hasher::new() };
        io::copy(&mut reader, &mut zip)?;
        zip.finish()?.flush()?;
        Ok(reader.hasher.finalize())
    })();
    let outcome = match written {
        Ok(hash) if hash.to_hex().as_str() == file.hash => {
            fs::rename(&staging, &archive).map_err(|error| BuildError::Io { path: archive.clone(), error })
        }
        Ok(_) => Err(BuildError::SourceChanged(file.path.clone())),
        Err(error) => Err(BuildError::Io { path: archive.clone(), error }),
    };
    if outcome.is_err() {
        // The staging file is incomplete or stale; losing it is harmless and the error says why.
        let _ = fs::remove_file(&staging);
    }
    outcome
}

/// Deletes the Archive of a file that left the Source, plus any folders it leaves empty.
fn remove_archive(update_tree: &Path, path: &str) -> Result<bool, BuildError> {
    let archive = manifest::archive_path(update_tree, path);
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
