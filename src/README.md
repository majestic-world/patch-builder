# Application Core

## Overview

The Rust binary scans a Source tree into an in-memory Plan, renders that comparison in the Slint UI, and applies the Plan to an Update tree. Domain terms such as Source, Archive, Manifest, Scan, Plan, and Build follow [`../GLOSSARY.md`](../GLOSSARY.md).

## Interface

- `scan::run` validates the selected folders, hashes every Source file, compares it with the current Manifest, and returns a path-sorted `Plan`. [`src/scan.rs:Plan`, `src/scan.rs:run`]
- `build::run` applies a `Plan` and returns a `Report` describing the published Manifest, re-zipped files, and removed Archives. [`src/build.rs:Report`, `src/build.rs:run`]
- `manifest::{Manifest, Entry, HashAlgorithm}` define the JSON contract; `manifest::{read, write}` load and atomically replace `manifest.json`. [`src/manifest.rs:Manifest`, `src/manifest.rs:Entry`, `src/manifest.rs:HashAlgorithm`, `src/manifest.rs:read`, `src/manifest.rs:write`]
- `comparison::{FileStatus, ComparedFile, Summary}` represent per-file differences and aggregate the work a Build will perform. [`src/comparison.rs:FileStatus`, `src/comparison.rs:ComparedFile`, `src/comparison.rs:Summary`]
- `Session::start` restores settings and wires folder selection, Scan, Build, hash selection, appearance, progress, and error handling to `AppWindow`. [`src/session.rs:Session::start`]
- `Tree` and `Table` provide the hierarchical comparison model and its Slint-facing rows and summary. [`src/tree.rs:Tree`, `src/view.rs:Table`]
- `settings::{load, save}` persist `Settings`; missing or unreadable settings load as defaults. [`src/settings.rs:Settings`, `src/settings.rs:load`, `src/settings.rs:save`]
- The `--preview` process argument bypasses `Session` and fills the UI with deterministic sample comparison data. [`src/main.rs:main`, `src/preview.rs:files`]

## Invariants

- Source and Update tree must be separate, non-nested paths; Source must be a directory containing regular, non-symbolic-link files with Unicode paths and no case-insensitive path collisions. [`src/scan.rs:check_folders`, `src/scan.rs:walk`, `src/scan.rs:relative_path`]
- Manifest and Archive paths preserve on-disk case, use `/` separators relative to Source, and map each Source file to `path + ".zip"`. [`src/scan.rs:relative_path`, `src/manifest.rs:archive_path`]
- `Plan.files` and `Manifest.files` are sorted by path. A new Manifest version starts at 1 and increases only when the file list or selected hash algorithm changes. [`src/scan.rs:run`, `src/scan.rs:Plan::next_version`, `src/manifest.rs:Manifest`]
- `HashAlgorithm::Both` requires every Manifest entry to contain `crc32`; single-algorithm Manifests must omit it. BLAKE3 is stored in `hash` for `Both`, while CRC32 is eight lowercase hexadecimal digits. [`src/manifest.rs:read`, `src/scan.rs:Hashes::selected`, `src/scan.rs:Hashers::finish`]
- A Build refuses a stale Plan if the Update tree Manifest changed after Scan. Every re-zipped Source file is hashed while being compressed and must still match the scanned size and selected hashes. [`src/build.rs:run`, `src/build.rs:write_archive`]
- Archives are installed from temporary files; the Manifest is written last through `manifest.json.tmp` and renamed only after all listed Archives exist. [`src/build.rs:write_archive`, `src/build.rs:run`, `src/manifest.rs:write`]
- At least one hash algorithm remains selected, Scan and Build never overlap, and displayed progress never moves backward. [`src/session.rs:Session::start`, `src/session.rs:busy`, `src/session.rs:progress_reporter`]
- Tree node IDs retain Source order; only folders containing New or Changed files start expanded, and name sorting always places folders before files. [`src/tree.rs:Tree::from_files`, `src/tree.rs:Tree::expand_changed_folders`, `src/tree.rs:compare`]

## Side Effects

- Scan recursively reads Source metadata and file bytes, reads the current Manifest, and may read/decompress existing Archives when a BLAKE3 check is required; it does not write the Update tree. [`src/scan.rs:run`, `src/scan.rs:walk`, `src/scan.rs:archive_matches_blake3`]
- Build creates Update-tree directories, writes and renames temporary Archives, removes Archives and now-empty directories for deleted Source files, and writes the Manifest last. [`src/build.rs:run`, `src/build.rs:write_archive`, `src/build.rs:remove_archive`]
- Settings are read from and written to the platform configuration directory, under `Patch Builder/settings.json`. [`src/settings.rs:file`, `src/settings.rs:load`, `src/settings.rs:save`]
- Session work runs on background threads, posts results and progress to the Slint event loop, opens native folder/error dialogs, and opens the Update tree in the platform file manager after a successful Build. [`src/session.rs:in_background`, `src/session.rs:progress_reporter`, `src/session.rs:pick_folder`, `src/session.rs:show_error`, `src/session.rs:open_folder`]
- `main` creates and runs the Slint window; Windows builds also query the work area and set the initial window position. [`src/main.rs:main`, `src/main.rs:center_on_work_area`]

## Verification

- Run: `cargo test`
