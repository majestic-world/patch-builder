use super::*;
use crate::manifest::HashAlgorithm;

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
    scan::run(&folders.source, &folders.update_tree, HashAlgorithm::Blake3, &|_, _| {}).unwrap()
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
    assert_eq!((manifest.version, manifest.hash_algorithm), (1, HashAlgorithm::Blake3));
    assert_eq!(
        manifest.files,
        [
            manifest::Entry { path: "L2.exe".into(), size: 6, hash: blake3::hash(b"client").to_hex().to_string(), crc32: None },
            manifest::Entry {
                path: "system/Fonts.utx".into(),
                size: 5,
                hash: blake3::hash(b"fonts").to_hex().to_string(),
                crc32: None,
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

#[test]
fn progress_advances_while_a_file_is_being_zipped() {
    let folders = folders();
    let size = 4 << 20;
    put(&folders.source, "big", &vec![7; size]);
    let partial = AtomicU64::new(0);
    run(&scan(&folders), &|done, all| {
        if 0 < done && done < all {
            partial.fetch_add(1, Ordering::Relaxed);
        }
    })
    .unwrap();
    assert!(partial.into_inner() > 1, "a lone file must report progress before it finishes");
}

fn scan_with(folders: &Folders, algorithm: HashAlgorithm) -> Plan {
    scan::run(&folders.source, &folders.update_tree, algorithm, &|_, _| {}).unwrap()
}

#[test]
fn manifest_json_uses_the_selected_hashes() {
    for algorithm in [HashAlgorithm::Blake3, HashAlgorithm::Crc32, HashAlgorithm::Both] {
        let folders = folders();
        put(&folders.source, "data", b"123456789");
        build(&scan_with(&folders, algorithm)).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&fs::read(manifest::path_in(&folders.update_tree)).unwrap()).unwrap();
        let blake3 = blake3::hash(b"123456789").to_hex().to_string();
        let expected = match algorithm {
            HashAlgorithm::Blake3 => serde_json::json!({"version": 1, "hash_algorithm": "blake3", "files": [
                {"path": "data", "size": 9, "hash": blake3}
            ]}),
            HashAlgorithm::Crc32 => serde_json::json!({"version": 1, "hash_algorithm": "crc32", "files": [
                {"path": "data", "size": 9, "hash": "cbf43926"}
            ]}),
            HashAlgorithm::Both => serde_json::json!({"version": 1, "hash_algorithm": "blake3+crc32", "files": [
                {"path": "data", "size": 9, "hash": blake3, "crc32": "cbf43926"}
            ]}),
        };
        assert_eq!(json, expected);
        assert_eq!(unzip(&folders.update_tree, "data").1, b"123456789");
        assert!(!scan_with(&folders, algorithm).has_work());
    }
}

#[test]
fn algorithm_transitions_reuse_only_matching_archives() {
    let algorithms = [HashAlgorithm::Blake3, HashAlgorithm::Crc32, HashAlgorithm::Both];
    for previous in algorithms {
        for selected in algorithms {
            if previous == selected {
                continue;
            }
            let folders = folders();
            put(&folders.source, "same", b"123456789");
            put(&folders.source, "changed", b"old");
            build(&scan_with(&folders, previous)).unwrap();
            let original_archive = fs::read(manifest::archive_path(&folders.update_tree, "same")).unwrap();
            put(&folders.source, "changed", b"new");
            let plan = scan_with(&folders, selected);
            assert!(plan.has_work());
            assert_eq!(plan.files.iter().find(|file| file.path == "same").unwrap().status, FileStatus::Unchanged);
            assert_eq!(plan.files.iter().find(|file| file.path == "changed").unwrap().status, FileStatus::Changed);
            let report = build(&plan).unwrap();
            assert_eq!((report.rezipped, report.manifest.version, report.manifest.hash_algorithm), (1, 2, selected));
            assert_eq!(fs::read(manifest::archive_path(&folders.update_tree, "same")).unwrap(), original_archive);
            assert_eq!(unzip(&folders.update_tree, "changed").1, b"new");
            assert!(!scan_with(&folders, selected).has_work());
        }
    }
}

#[test]
fn selection_alone_publishes_without_rezipping_including_empty_source() {
    for with_file in [false, true] {
        let folders = folders();
        if with_file {
            put(&folders.source, "data", b"123456789");
        }
        build(&scan_with(&folders, HashAlgorithm::Blake3)).unwrap();
        for (version, algorithm) in [(2, HashAlgorithm::Crc32), (3, HashAlgorithm::Both), (4, HashAlgorithm::Blake3)] {
            let plan = scan_with(&folders, algorithm);
            assert!(plan.has_work());
            let report = build(&plan).unwrap();
            assert_eq!((report.rezipped, report.manifest_written, report.manifest.version), (0, true, version));
            assert!(!scan_with(&folders, algorithm).has_work());
        }
    }
}

#[test]
fn all_selected_hashes_are_checked_against_archived_bytes() {
    for algorithm in [HashAlgorithm::Blake3, HashAlgorithm::Crc32, HashAlgorithm::Both] {
        let folders = folders();
        put(&folders.source, "data", b"123456789");
        let plan = scan_with(&folders, algorithm);
        put(&folders.source, "data", b"987654321");
        assert!(matches!(build(&plan), Err(BuildError::SourceChanged(_))));
        assert!(!manifest::path_in(&folders.update_tree).exists());
    }

    let folders = folders();
    put(&folders.source, "data", b"123456789");
    let mut plan = scan_with(&folders, HashAlgorithm::Both);
    plan.files[0].crc32 = Some("00000000".into());
    assert!(matches!(build(&plan), Err(BuildError::SourceChanged(_))));
    assert!(!manifest::path_in(&folders.update_tree).exists());
}

#[test]
fn previous_dual_hashes_are_both_checked() {
    let folders = folders();
    put(&folders.source, "data", b"123456789");
    build(&scan_with(&folders, HashAlgorithm::Both)).unwrap();
    let mut previous = manifest_of(&folders);
    previous.files[0].crc32 = Some("00000000".into());
    manifest::write(&folders.update_tree, &previous).unwrap();
    let plan = scan_with(&folders, HashAlgorithm::Blake3);
    assert_eq!(plan.files[0].status, FileStatus::Changed);
}

#[test]
fn malformed_or_unsupported_manifest_algorithms_are_rejected() {
    let folders = folders();
    fs::create_dir(&folders.update_tree).unwrap();
    for json in [
        r#"{"version":1,"hash_algorithm":"sha256","files":[]}"#,
        r#"{"version":1,"hash_algorithm":"blake3+crc32","files":[{"path":"data","size":0,"hash":"abc"}]}"#,
    ] {
        fs::write(manifest::path_in(&folders.update_tree), json).unwrap();
        assert!(matches!(manifest::read(&folders.update_tree), Err(manifest::ReadError::Invalid(_))));
    }
}

#[test]
fn legacy_blake3_manifest_is_read_and_can_transition() {
    let folders = folders();
    put(&folders.source, "data", b"123456789");
    build(&scan_with(&folders, HashAlgorithm::Blake3)).unwrap();
    let hash = blake3::hash(b"123456789").to_hex().to_string();
    let legacy = format!(r#"{{"version":7,"hash_algorithm":"blake3","files":[{{"path":"data","size":9,"hash":"{hash}"}}]}}"#);
    fs::write(manifest::path_in(&folders.update_tree), legacy).unwrap();
    let plan = scan_with(&folders, HashAlgorithm::Both);
    assert_eq!(plan.files[0].status, FileStatus::Unchanged);
    let report = build(&plan).unwrap();
    assert_eq!((report.rezipped, report.manifest.version), (0, 8));
    assert_eq!(report.manifest.files[0].crc32.as_deref(), Some("cbf43926"));
}

#[test]
fn crc32_collision_requires_rezipping_when_adding_blake3() {
    for selected in [HashAlgorithm::Blake3, HashAlgorithm::Both] {
        let folders = folders();
        put(&folders.source, "data", b"plumless");
        build(&scan_with(&folders, HashAlgorithm::Crc32)).unwrap();
        assert_eq!(manifest_of(&folders).files[0].hash, "4ddb0c25");
        put(&folders.source, "data", b"buckeroo");
        let plan = scan_with(&folders, selected);
        assert_eq!(plan.files[0].status, FileStatus::Changed);
        let report = build(&plan).unwrap();
        assert_eq!(report.rezipped, 1);
        assert_eq!(unzip(&folders.update_tree, "data").1, b"buckeroo");
        assert!(!scan_with(&folders, selected).has_work());
    }
}

#[test]
fn crc32_transition_rechecks_reused_archive_at_build_time() {
    let folders = folders();
    put(&folders.source, "data", b"plumless");
    build(&scan_with(&folders, HashAlgorithm::Crc32)).unwrap();
    let plan = scan_with(&folders, HashAlgorithm::Both);
    assert_eq!(plan.files[0].status, FileStatus::Unchanged);
    fs::write(manifest::archive_path(&folders.update_tree, "data"), b"corrupt").unwrap();
    let report = build(&plan).unwrap();
    assert_eq!(report.rezipped, 1);
    assert_eq!(unzip(&folders.update_tree, "data").1, b"plumless");
}
