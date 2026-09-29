//! Why a Scan or a Build stopped.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::manifest::{self, HASH_ALGORITHM};

#[derive(Debug)]
pub enum BuildError {
    SourceNotAFolder(PathBuf),
    OverlappingFolders,
    SymbolicLink(PathBuf),
    NonUnicodePath(PathBuf),
    /// Two Source files whose paths differ only in letter case; Windows installs cannot hold both.
    CaseCollision { first: String, second: String },
    Manifest { path: PathBuf, error: manifest::ReadError },
    /// The Update tree's Manifest is no longer the one the Scan compared against.
    UpdateTreeChanged,
    /// A Source file to re-zip no longer matches the hash the Scan recorded.
    SourceChanged(String),
    Io { path: PathBuf, error: io::Error },
}

const RESCAN: &str = "Rescan before building.";

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
                    "{} uses the {algorithm} hash; this app only reads {HASH_ALGORITHM} Manifests.",
                    path.display()
                ),
            },
            Self::UpdateTreeChanged => write!(f, "The Update tree changed since the last Scan. {RESCAN}"),
            Self::SourceChanged(path) => write!(f, "{path} changed since the last Scan. {RESCAN}"),
            Self::Io { path, error } => write!(f, "{}: {error}", path.display()),
        }
    }
}

impl std::error::Error for BuildError {}

pub fn io_at(path: &Path) -> impl FnOnce(io::Error) -> BuildError + '_ {
    move |error| BuildError::Io { path: path.to_owned(), error }
}
