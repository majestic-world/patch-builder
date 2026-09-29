use super::*;

fn file(path: &str) -> ComparedFile {
    ComparedFile { path: path.to_owned(), status: FileStatus::Unchanged, size: 1, hash: String::new() }
}

fn tree(paths: &[&str]) -> Tree {
    Tree::from_files(paths.iter().map(|path| file(path)).collect())
}

fn names(tree: &Tree) -> Vec<&str> {
    tree.rows().into_iter().map(|row| row.name).collect()
}

#[test]
fn keeps_source_order_and_groups_files_under_their_folders() {
    let tree = tree(&["system/Fonts.utx", "textures/Loading.utx", "system/L2.exe", "readme.txt"]);
    assert_eq!(names(&tree), ["system", "Fonts.utx", "L2.exe", "textures", "Loading.utx", "readme.txt"]);
}

#[test]
fn connectors_stop_at_the_last_child() {
    let tree = tree(&["system/Fonts.utx", "system/L2.exe"]);
    let rows = tree.rows();
    assert_eq!((rows[1].name, rows[1].depth, rows[1].is_last), ("Fonts.utx", 1, false));
    assert_eq!((rows[2].name, rows[2].depth, rows[2].is_last), ("L2.exe", 1, true));
}

#[test]
fn guides_continue_only_past_non_last_ancestors() {
    let tree = tree(&["a/b/one", "a/b/two", "a/c/three"]);
    let guides: Vec<(&str, Vec<bool>)> = tree.rows().into_iter().map(|row| (row.name, row.guides)).collect();
    assert_eq!(
        guides,
        [
            ("a", vec![]),
            ("b", vec![]),
            // `b` has a sibling below, so its level keeps drawing through its children.
            ("one", vec![true]),
            ("two", vec![true]),
            ("c", vec![]),
            // `c` is last, so nothing continues at its level.
            ("three", vec![false]),
        ]
    );
}

#[test]
fn collapsing_a_folder_hides_every_descendant() {
    let mut tree = tree(&["a/b/one", "a/two", "other"]);
    let a = tree.find("a").unwrap();
    tree.activate(a);
    assert_eq!(names(&tree), ["a", "other"]);
    tree.activate(a);
    assert_eq!(names(&tree), ["a", "b", "one", "two", "other"]);
}

#[test]
fn activating_a_file_selects_only_that_file() {
    let mut tree = tree(&["a/one", "a/two"]);
    tree.activate(tree.find("a/two").unwrap());
    let selected: Vec<&str> = tree.rows().into_iter().filter(|row| row.selected).map(|row| row.name).collect();
    assert_eq!(selected, ["two"]);
}

#[test]
fn sorting_puts_folders_first_and_ignores_case() {
    let mut tree = tree(&["b.txt", "Zeta/x", "A.txt", "alpha/y"]);
    tree.cycle_sort();
    assert_eq!(tree.sort(), SortOrder::Ascending);
    assert_eq!(names(&tree), ["alpha", "y", "Zeta", "x", "A.txt", "b.txt"]);

    tree.cycle_sort();
    assert_eq!(tree.sort(), SortOrder::Descending);
    assert_eq!(names(&tree), ["Zeta", "x", "alpha", "y", "b.txt", "A.txt"]);
}

#[test]
fn third_sort_click_restores_source_order() {
    let paths = ["system/L2.exe", "maps/23_21.unr", "system/Fonts.utx"];
    let mut tree = tree(&paths);
    let source_order = names(&tree).into_iter().map(str::to_owned).collect::<Vec<_>>();
    tree.cycle_sort();
    tree.cycle_sort();
    tree.cycle_sort();
    assert_eq!(tree.sort(), SortOrder::Source);
    assert_eq!(names(&tree), source_order);
}

#[test]
fn find_rejects_paths_through_a_file() {
    let tree = tree(&["system/Fonts.utx"]);
    assert!(tree.find("system/Fonts.utx").is_some());
    assert_eq!(tree.find("system/Fonts.utx/extra"), None);
    assert_eq!(tree.find("missing"), None);
}
