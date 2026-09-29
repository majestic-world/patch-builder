//! Result of comparing the Source against the previous Build.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileStatus {
    /// Absent from the previous Manifest.
    New,
    /// Present in the previous Manifest with a different size or hash, or with its Archive missing.
    Changed,
    /// Identical to the previous Manifest entry; its Archive is reused.
    Unchanged,
}

#[derive(Clone, Debug)]
pub struct ComparedFile {
    /// Path relative to the Source root, `/`-separated.
    pub path: String,
    pub status: FileStatus,
    /// Size of the uncompressed Source file, in bytes.
    pub size: u64,
    /// BLAKE3 hash of the uncompressed Source file, lowercase hex.
    pub hash: String,
}

/// What a Build publishes, aggregated from a comparison.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub manifest_files: u64,
    pub manifest_bytes: u64,
    pub rezip_files: u64,
    pub rezip_bytes: u64,
    pub reused_archives: u64,
}

impl Summary {
    pub fn of(files: &[ComparedFile]) -> Self {
        files.iter().fold(Self::default(), |mut summary, file| {
            summary.manifest_files += 1;
            summary.manifest_bytes += file.size;
            match file.status {
                FileStatus::New | FileStatus::Changed => {
                    summary.rezip_files += 1;
                    summary.rezip_bytes += file.size;
                }
                FileStatus::Unchanged => summary.reused_archives += 1,
            }
            summary
        })
    }
}
