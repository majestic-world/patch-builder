use std::io::Read;

use super::*;

struct Folders {
    _root: tempfile::TempDir,
    source: PathBuf,
    update_tree: PathBuf,
}

fn folders() -> Folders {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("Source");
    let update_tree = root.path().join("Update tree");
    fs::create_dir(&source).unwrap();
    Folders { source, update_tree, _root: root }
}

fn put(root: &Path, path: &str, contents: &[u8]) {
    let file = root.join(path);
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, contents).unwrap();
}

fn build(folders: &Folders) -> Report {
    run(&folders.source, &folders.update_tree, &|_, _| {}).unwrap()
}

fn statuses(report: &Report) -> Vec<(&str, FileStatus)> {
    report.files.iter().map(|file| (file.path.as_str(), file.status)).collect()
}

/// Name and contents of the single entry inside an Archive.
fn unzip(update_tree: &Path, path: &str) -> (String, Vec<u8>) {
    let mut archive = zip::ZipArchive::new(File::open(archive_path(update_tree, path)).unwrap()).unwrap();
    assert_eq!(archive.len(), 1);
    let mut entry = archive.by_index(0).unwrap();
    let mut contents = Vec::new();
    entry.read_to_end(&mut contents).unwrap();
    (entry.name().to_owned(), contents)
}

fn manifest_of(folders: &Folders) -> Manifest {
    manifest::read(&folders.update_tree).unwrap().unwrap()
}

#[test]
fn first_build_zips_every_file_and_lists_it_in_the_manifest() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    put(&folders.source, "L2.exe", b"client");

    let report = build(&folders);

    assert_eq!(statuses(&report), [("L2.exe", FileStatus::New), ("system/Fonts.utx", FileStatus::New)]);
    assert_eq!(unzip(&folders.update_tree, "system/Fonts.utx"), ("Fonts.utx".to_owned(), b"fonts".to_vec()));
    let manifest = manifest_of(&folders);
    assert_eq!((manifest.version, manifest.hash_algorithm.as_str()), (1, "blake3"));
    assert_eq!(
        manifest.files,
        [
            Entry { path: "L2.exe".into(), size: 6, hash: blake3::hash(b"client").to_hex().to_string() },
            Entry { path: "system/Fonts.utx".into(), size: 5, hash: blake3::hash(b"fonts").to_hex().to_string() },
        ]
    );
}

#[test]
fn unchanged_files_keep_their_archive_and_the_manifest_version() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    build(&folders);
    // A reused Archive is never rewritten, so this marker survives the next Build.
    fs::write(archive_path(&folders.update_tree, "system/Fonts.utx"), b"marker").unwrap();

    let report = build(&folders);

    assert_eq!(statuses(&report), [("system/Fonts.utx", FileStatus::Unchanged)]);
    assert!(!report.manifest_written);
    assert_eq!(report.manifest_version, 1);
    assert_eq!(fs::read(archive_path(&folders.update_tree, "system/Fonts.utx")).unwrap(), b"marker");
}

#[test]
fn changed_file_is_rezipped_and_bumps_the_version() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    put(&folders.source, "L2.exe", b"client");
    build(&folders);
    put(&folders.source, "system/Fonts.utx", b"fonts v2");

    let report = build(&folders);

    assert_eq!(statuses(&report), [("L2.exe", FileStatus::Unchanged), ("system/Fonts.utx", FileStatus::Changed)]);
    assert_eq!(unzip(&folders.update_tree, "system/Fonts.utx").1, b"fonts v2");
    assert_eq!(manifest_of(&folders).version, 2);
}

#[test]
fn removed_file_loses_its_archive_and_emptied_folders() {
    let folders = folders();
    put(&folders.source, "maps/old/23_21.unr", b"map");
    put(&folders.source, "L2.exe", b"client");
    build(&folders);
    fs::remove_dir_all(folders.source.join("maps")).unwrap();

    let report = build(&folders);

    assert_eq!(report.removed_archives, 1);
    assert!(!folders.update_tree.join("maps").exists());
    let manifest = manifest_of(&folders);
    assert_eq!(manifest.version, 2);
    assert_eq!(manifest.files.iter().map(|entry| entry.path.as_str()).collect::<Vec<_>>(), ["L2.exe"]);
}

#[test]
fn missing_archive_is_restored_without_a_new_version() {
    let folders = folders();
    put(&folders.source, "L2.exe", b"client");
    build(&folders);
    fs::remove_file(archive_path(&folders.update_tree, "L2.exe")).unwrap();

    let report = build(&folders);

    assert_eq!(statuses(&report), [("L2.exe", FileStatus::Changed)]);
    assert_eq!(unzip(&folders.update_tree, "L2.exe").1, b"client");
    assert!(!report.manifest_written);
    assert_eq!(manifest_of(&folders).version, 1);
}

#[test]
fn update_tree_inside_the_source_is_rejected() {
    let folders = folders();
    let inside = folders.source.join("dist");
    let error = run(&folders.source, &inside, &|_, _| {}).err().unwrap();
    assert!(matches!(error, BuildError::OverlappingFolders));
}

#[test]
fn progress_ends_at_the_total_source_size() {
    let folders = folders();
    put(&folders.source, "a", b"12345");
    put(&folders.source, "b/c", b"678");
    // Workers report concurrently, so keep the furthest point reached rather than the last call.
    let (furthest, total) = (AtomicU64::new(0), AtomicU64::new(0));
    run(&folders.source, &folders.update_tree, &|done, all| {
        furthest.fetch_max(done, Ordering::Relaxed);
        total.store(all, Ordering::Relaxed);
    })
    .unwrap();
    assert_eq!((furthest.into_inner(), total.into_inner()), (8, 8));
}

// Case-insensitive file systems (Windows, default macOS) cannot hold such a pair.
#[cfg(target_os = "linux")]
#[test]
fn paths_differing_only_in_case_are_rejected() {
    let folders = folders();
    put(&folders.source, "Maps/a.unr", b"1");
    put(&folders.source, "maps/a.unr", b"2");
    let error = run(&folders.source, &folders.update_tree, &|_, _| {}).err().unwrap();
    assert!(matches!(error, BuildError::CaseCollision { .. }));
}
