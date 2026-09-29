//! Sample comparison shown with `--preview`, until the Build produces real ones.

use crate::comparison::{ComparedFile, FileStatus};

/// Path of the file selected when the preview opens.
pub const SELECTED_PATH: &str = "system/Fonts.utx";

pub const SOURCE_PATH: &str = "D:/Games/L2/Client";
pub const UPDATE_TREE_PATH: &str = "D:/Publish/l2-live";

pub fn files() -> Vec<ComparedFile> {
    let file = |path: &str, status, size, hash: &str| ComparedFile {
        path: path.to_owned(),
        status,
        size,
        hash: hash.to_owned(),
    };
    vec![
        file(
            SELECTED_PATH,
            FileStatus::Changed,
            29_779_558,
            "a3f1e7c9b2d4e6f10b8c7d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b9c8d7e6f5a",
        ),
        file(
            "system/L2.exe",
            FileStatus::New,
            13_212_058,
            "7b9d3a4c8e1f2d6a5c4b3a29180f7e6d5c4b3a2918f7e6d5c4b3a2918f7e6d5c",
        ),
        file(
            "textures/Loading.utx",
            FileStatus::Unchanged,
            6_501_171,
            "4e2c6f8a9b7d1c3e2f4a6b8c0d1e3f5a7b9c1d3e5f7a9b1c3d5e7f9a1b3c5d7e",
        ),
        file(
            "maps/23_21.unr",
            FileStatus::Changed,
            44_774_195,
            "9d7a3c1e6f4b8d2c1a3b5c7d9e0f2a4b6c8d0e1f3a5b7c9d1e3f5a7b9c0d2e4f",
        ),
    ]
}
