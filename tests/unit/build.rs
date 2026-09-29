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

fn scan(folders: &Folders) -> Plan {
    scan::run(&folders.source, &folders.update_tree, &|_, _| {}).unwrap()
}

fn build(plan: &Plan) -> Result<Report, BuildError> {
    run(plan, &|_, _| {})
}

fn scan_and_build(folders: &Folders) -> Report {
    build(&scan(folders)).unwrap()
}

/// Name and contents of the single entry inside an Archive.
fn unzip(update_tree: &Path, path: &str) -> (String, Vec<u8>) {
    let mut archive = zip::ZipArchive::new(File::open(manifest::archive_path(update_tree, path)).unwrap()).unwrap();
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

    let report = scan_and_build(&folders);

    assert_eq!((report.rezipped, report.manifest_written), (2, true));
    assert_eq!(unzip(&folders.update_tree, "system/Fonts.utx"), ("Fonts.utx".to_owned(), b"fonts".to_vec()));
    let manifest = manifest_of(&folders);
    assert_eq!(manifest, report.manifest);
    assert_eq!((manifest.version, manifest.hash_algorithm.as_str()), (1, "blake3"));
    assert_eq!(
        manifest.files,
        [
            manifest::Entry { path: "L2.exe".into(), size: 6, hash: blake3::hash(b"client").to_hex().to_string() },
            manifest::Entry {
                path: "system/Fonts.utx".into(),
                size: 5,
                hash: blake3::hash(b"fonts").to_hex().to_string(),
            },
        ]
    );
}

#[test]
fn unchanged_files_keep_their_archive_and_the_manifest() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    scan_and_build(&folders);
    // A reused Archive is never rewritten, so this marker survives the next Build.
    fs::write(manifest::archive_path(&folders.update_tree, "system/Fonts.utx"), b"marker").unwrap();

    let report = scan_and_build(&folders);

    assert_eq!((report.rezipped, report.manifest_written, report.manifest.version), (0, false, 1));
    assert_eq!(fs::read(manifest::archive_path(&folders.update_tree, "system/Fonts.utx")).unwrap(), b"marker");
}

#[test]
fn changed_file_is_rezipped_and_bumps_the_version() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    put(&folders.source, "L2.exe", b"client");
    scan_and_build(&folders);
    put(&folders.source, "system/Fonts.utx", b"fonts v2");

    let report = scan_and_build(&folders);

    assert_eq!(report.rezipped, 1);
    assert_eq!(unzip(&folders.update_tree, "system/Fonts.utx").1, b"fonts v2");
    assert_eq!(manifest_of(&folders).version, 2);
}

#[test]
fn removed_file_loses_its_archive_and_emptied_folders() {
    let folders = folders();
    put(&folders.source, "maps/old/23_21.unr", b"map");
    put(&folders.source, "L2.exe", b"client");
    scan_and_build(&folders);
    fs::remove_dir_all(folders.source.join("maps")).unwrap();

    let report = scan_and_build(&folders);

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
    scan_and_build(&folders);
    fs::remove_file(manifest::archive_path(&folders.update_tree, "L2.exe")).unwrap();

    let report = scan_and_build(&folders);

    assert_eq!((report.rezipped, report.manifest_written), (1, false));
    assert_eq!(unzip(&folders.update_tree, "L2.exe").1, b"client");
    assert_eq!(manifest_of(&folders).version, 1);
}

#[test]
fn file_edited_after_the_scan_stops_the_build_before_the_manifest() {
    let folders = folders();
    put(&folders.source, "L2.exe", b"client");
    let plan = scan(&folders);
    put(&folders.source, "L2.exe", b"client, edited later");

    let error = build(&plan).err().unwrap();

    assert!(matches!(error, BuildError::SourceChanged(path) if path == "L2.exe"));
    assert!(!manifest::archive_path(&folders.update_tree, "L2.exe").exists());
    assert!(!manifest::path_in(&folders.update_tree).exists());
}

#[test]
fn plan_scanned_before_another_build_is_refused() {
    let folders = folders();
    put(&folders.source, "L2.exe", b"client");
    let stale = scan(&folders);
    scan_and_build(&folders);

    let error = build(&stale).err().unwrap();

    assert!(matches!(error, BuildError::UpdateTreeChanged));
}

#[test]
fn progress_ends_at_the_size_of_the_files_to_rezip() {
    let folders = folders();
    put(&folders.source, "same", b"1234");
    scan_and_build(&folders);
    put(&folders.source, "new", b"567");
    let (furthest, total) = (AtomicU64::new(0), AtomicU64::new(0));
    run(&scan(&folders), &|done, all| {
        furthest.fetch_max(done, Ordering::Relaxed);
        total.store(all, Ordering::Relaxed);
    })
    .unwrap();
    assert_eq!((furthest.into_inner(), total.into_inner()), (3, 3));
}
