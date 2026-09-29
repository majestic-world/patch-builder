use super::*;
use crate::build;

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
    run(&folders.source, &folders.update_tree, &|_, _| {}).unwrap()
}

fn publish(folders: &Folders) -> Plan {
    let plan = scan(folders);
    let report = build::run(&plan, &|_, _| {}).unwrap();
    plan.into_published(report.manifest)
}

fn statuses(plan: &Plan) -> Vec<(&str, FileStatus)> {
    plan.files.iter().map(|file| (file.path.as_str(), file.status)).collect()
}

#[test]
fn first_scan_plans_every_file_as_new_without_touching_the_update_tree() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    put(&folders.source, "L2.exe", b"client");

    let plan = scan(&folders);

    assert_eq!(statuses(&plan), [("L2.exe", FileStatus::New), ("system/Fonts.utx", FileStatus::New)]);
    assert_eq!(plan.files[1].hash, blake3::hash(b"fonts").to_hex().as_str());
    assert!(plan.has_work());
    assert_eq!(plan.next_version(), 1);
    assert!(!folders.update_tree.exists());
}

#[test]
fn scan_after_a_build_sees_changes_removals_and_missing_archives() {
    let folders = folders();
    put(&folders.source, "same.u", b"same");
    put(&folders.source, "edited.u", b"v1");
    put(&folders.source, "gone.u", b"gone");
    put(&folders.source, "lost.u", b"lost");
    publish(&folders);
    put(&folders.source, "edited.u", b"v2");
    put(&folders.source, "added.u", b"new");
    fs::remove_file(folders.source.join("gone.u")).unwrap();
    fs::remove_file(manifest::archive_path(&folders.update_tree, "lost.u")).unwrap();

    let plan = scan(&folders);

    assert_eq!(
        statuses(&plan),
        [
            ("added.u", FileStatus::New),
            ("edited.u", FileStatus::Changed),
            ("lost.u", FileStatus::Changed),
            ("same.u", FileStatus::Unchanged),
        ]
    );
    assert_eq!(plan.removed, ["gone.u"]);
    assert_eq!(plan.next_version(), 2);
}

#[test]
fn a_missing_archive_alone_keeps_the_manifest_version() {
    let folders = folders();
    put(&folders.source, "L2.exe", b"client");
    publish(&folders);
    fs::remove_file(manifest::archive_path(&folders.update_tree, "L2.exe")).unwrap();

    let plan = scan(&folders);

    assert!(plan.has_work());
    assert!(!plan.changes_manifest());
    assert_eq!(plan.next_version(), 1);
}

#[test]
fn published_plan_matches_a_fresh_scan() {
    let folders = folders();
    put(&folders.source, "system/Fonts.utx", b"fonts");
    put(&folders.source, "old.u", b"old");
    publish(&folders);
    put(&folders.source, "system/Fonts.utx", b"fonts v2");
    fs::remove_file(folders.source.join("old.u")).unwrap();

    let published = publish(&folders);
    let fresh = scan(&folders);

    assert_eq!(published.files, fresh.files);
    assert_eq!(published.removed, fresh.removed);
    assert_eq!(published.previous, fresh.previous);
    assert!(!published.has_work());
}

#[test]
fn update_tree_inside_the_source_is_rejected_even_before_it_exists() {
    let folders = folders();
    let error = run(&folders.source, &folders.source.join("dist"), &|_, _| {}).err().unwrap();
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
