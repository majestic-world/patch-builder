#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod build;
mod comparison;
mod format;
mod manifest;
mod preview;
mod tree;

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use slint::winit_030::WinitWindowAccessor;
use slint::winit_030::winit::window::{ResizeDirection, WindowAttributes};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use comparison::{ComparedFile, FileStatus, Summary};
use tree::{RowKind, SortOrder, Tree};

slint::include_modules!();

/// Comparison currently on screen, shared by the table callbacks.
type SharedTree = Rc<RefCell<Option<Tree>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .with_winit_window_attributes_hook(native_frame)
        .select()?;

    let ui = AppWindow::new()?;
    let rows = Rc::new(VecModel::<TreeRow>::default());
    ui.set_rows(ModelRc::from(rows.clone()));
    let tree: SharedTree = Rc::default();

    if std::env::args().skip(1).any(|arg| arg == "--preview") {
        ui.set_source_path(preview::SOURCE_PATH.into());
        ui.set_update_tree_path(preview::UPDATE_TREE_PATH.into());
        let mut preview_tree = show_comparison(&ui, preview::files(), SUMMARY_PROMPT);
        if let Some(id) = preview_tree.find(preview::SELECTED_PATH) {
            preview_tree.select(id);
        }
        *tree.borrow_mut() = Some(preview_tree);
    } else {
        ui.set_summary(empty_summary());
    }
    refresh_rows(&ui, &rows, &tree);

    install_table(&ui, &rows, &tree);
    install_folder_pickers(&ui, &rows, &tree);
    install_build(&ui, &rows, &tree);
    install_window_chrome(&ui);

    ui.run()?;
    Ok(())
}

/// Borderless window that keeps the system shadow and Windows 11 rounded corners.
fn native_frame(attributes: WindowAttributes) -> WindowAttributes {
    #[cfg(windows)]
    {
        use slint::winit_030::winit::platform::windows::{Color, CornerPreference, WindowAttributesExtWindows};
        attributes
            .with_undecorated_shadow(true)
            .with_corner_preference(CornerPreference::Round)
            .with_border_color(Some(Color::from_rgb(0xD9, 0xDE, 0xE8)))
    }
    #[cfg(not(windows))]
    attributes
}

/// Subtitle of the Build Summary before a Build has run.
const SUMMARY_PROMPT: &str = "What will be published";

fn show_comparison(ui: &AppWindow, files: Vec<ComparedFile>, note: &str) -> Tree {
    ui.set_summary(summary_view(&Summary::of(&files), note));
    ui.set_has_comparison(true);
    Tree::from_files(files)
}

/// Drops a comparison that no longer matches the chosen folders.
fn clear_comparison(ui: &AppWindow, rows: &VecModel<TreeRow>, tree: &SharedTree) {
    *tree.borrow_mut() = None;
    ui.set_has_comparison(false);
    ui.set_summary(empty_summary());
    refresh_rows(ui, rows, tree);
}

fn install_table(ui: &AppWindow, rows: &Rc<VecModel<TreeRow>>, tree: &SharedTree) {
    ui.on_row_clicked({
        let (ui, rows, tree) = (ui.as_weak(), rows.clone(), tree.clone());
        move |node| {
            let Some(ui) = ui.upgrade() else { return };
            if let (Some(tree), Ok(node)) = (tree.borrow_mut().as_mut(), usize::try_from(node)) {
                tree.activate(node);
            }
            refresh_rows(&ui, &rows, &tree);
        }
    });
    ui.on_sort_clicked({
        let (ui, rows, tree) = (ui.as_weak(), rows.clone(), tree.clone());
        move || {
            let Some(ui) = ui.upgrade() else { return };
            if let Some(tree) = tree.borrow_mut().as_mut() {
                tree.cycle_sort();
            }
            refresh_rows(&ui, &rows, &tree);
        }
    });
}

fn refresh_rows(ui: &AppWindow, rows: &VecModel<TreeRow>, tree: &SharedTree) {
    let tree = tree.borrow();
    let Some(tree) = tree.as_ref() else {
        rows.clear();
        return;
    };
    ui.set_sort(match tree.sort() {
        SortOrder::Source => NameSort::Source,
        SortOrder::Ascending => NameSort::Ascending,
        SortOrder::Descending => NameSort::Descending,
    });
    rows.set_vec(tree.rows().into_iter().map(tree_row).collect::<Vec<_>>());
}

fn tree_row(row: tree::Row<'_>) -> TreeRow {
    let (is_folder, expanded, status, size, hash) = match row.kind {
        RowKind::Folder { expanded } => (true, expanded, RowStatus::Unchanged, SharedString::new(), SharedString::new()),
        RowKind::File { status, size, hash } => (
            false,
            false,
            match status {
                FileStatus::New => RowStatus::New,
                FileStatus::Changed => RowStatus::Changed,
                FileStatus::Unchanged => RowStatus::Unchanged,
            },
            format::bytes(size).into(),
            format::short_hash(hash).into(),
        ),
    };
    TreeRow {
        node: i32::try_from(row.node).unwrap_or(i32::MAX),
        name: row.name.into(),
        depth: i32::try_from(row.depth).unwrap_or(i32::MAX),
        is_folder,
        expanded,
        status,
        size,
        hash,
        is_last: row.is_last,
        guides: ModelRc::new(VecModel::from(row.guides)),
        selected: row.selected,
    }
}

fn summary_view(summary: &Summary, note: &str) -> SummaryView {
    SummaryView {
        note: note.into(),
        manifest_count: format::count(summary.manifest_files).into(),
        manifest_unit: format::noun(summary.manifest_files, "file", "files").into(),
        manifest_size: format::bytes(summary.manifest_bytes).into(),
        rezip_count: format::count(summary.rezip_files).into(),
        rezip_unit: format::noun(summary.rezip_files, "file", "files").into(),
        rezip_size: format::bytes(summary.rezip_bytes).into(),
        reused_count: format::count(summary.reused_archives).into(),
        reused_unit: format::noun(summary.reused_archives, "Archive", "Archives").into(),
    }
}

/// Placeholder figures while no comparison exists.
fn empty_summary() -> SummaryView {
    SummaryView {
        note: SUMMARY_PROMPT.into(),
        manifest_count: "—".into(),
        rezip_count: "—".into(),
        reused_count: "—".into(),
        ..Default::default()
    }
}

/// Subtitle of the Build Summary after a Build: which Manifest Launchers now read.
fn build_note(report: &build::Report) -> String {
    let mut note = if report.manifest_written {
        format!("Published as Manifest v{}", report.manifest_version)
    } else {
        format!("Already up to date · Manifest v{}", report.manifest_version)
    };
    if report.removed_archives > 0 {
        let removed = report.removed_archives;
        note.push_str(&format!(" · {} {} removed", format::count(removed), format::noun(removed, "Archive", "Archives")));
    }
    note
}

fn install_folder_pickers(ui: &AppWindow, rows: &Rc<VecModel<TreeRow>>, tree: &SharedTree) {
    ui.on_pick_source({
        let (ui, rows, tree) = (ui.as_weak(), rows.clone(), tree.clone());
        move || {
            let Some(ui) = ui.upgrade() else { return };
            if let Some(path) = pick_folder(&ui, "Choose the Source folder") {
                ui.set_source_path(path);
                clear_comparison(&ui, &rows, &tree);
            }
        }
    });
    ui.on_pick_update_tree({
        let (ui, rows, tree) = (ui.as_weak(), rows.clone(), tree.clone());
        move || {
            let Some(ui) = ui.upgrade() else { return };
            if let Some(path) = pick_folder(&ui, "Choose the Update tree folder") {
                ui.set_update_tree_path(path);
                clear_comparison(&ui, &rows, &tree);
            }
        }
    });
}

/// Runs the Build on a worker thread and shows its comparison once it finishes.
fn install_build(ui: &AppWindow, rows: &Rc<VecModel<TreeRow>>, tree: &SharedTree) {
    ui.on_build({
        let (weak, rows, tree) = (ui.as_weak(), rows.clone(), tree.clone());
        move || {
            let Some(ui) = weak.upgrade() else { return };
            let source = PathBuf::from(ui.get_source_path().as_str());
            let update_tree = PathBuf::from(ui.get_update_tree_path().as_str());
            ui.set_build_progress(0.0);
            ui.set_building(true);

            let (sender, receiver) = futures_channel::oneshot::channel();
            let progress_ui = weak.clone();
            std::thread::spawn(move || {
                let reported = AtomicU32::new(0);
                let result = build::run(&source, &update_tree, &|done, total| {
                    // One UI update per whole percent keeps the event loop free on huge Sources.
                    let percent = (done * 100).checked_div(total).map_or(100, |percent| percent as u32);
                    if reported.fetch_max(percent, Ordering::Relaxed) < percent {
                        let _ = progress_ui.upgrade_in_event_loop(move |ui| {
                            // Workers may post out of order; progress only moves forward.
                            ui.set_build_progress(ui.get_build_progress().max(percent as f32 / 100.0));
                        });
                    }
                });
                // The receiver only disappears when the app is closing.
                let _ = sender.send(result);
            });

            let (weak, rows, tree) = (weak.clone(), rows.clone(), tree.clone());
            slint::spawn_local(async move {
                let result = receiver.await;
                let Some(ui) = weak.upgrade() else { return };
                ui.set_building(false);
                match result {
                    Ok(Ok(report)) => {
                        let note = build_note(&report);
                        *tree.borrow_mut() = Some(show_comparison(&ui, report.files, &note));
                        refresh_rows(&ui, &rows, &tree);
                    }
                    Ok(Err(error)) => show_error(&ui, &error.to_string()),
                    Err(_) => show_error(&ui, "The Build stopped unexpectedly."),
                }
            })
            .expect("callbacks run inside the Slint event loop");
        }
    });
}

fn show_error(ui: &AppWindow, message: &str) {
    let dialog = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Build failed")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok);
    ui.window().with_winit_window(|window| dialog.set_parent(window).show());
}

/// Native folder dialog, modal to the app window; `None` when cancelled.
fn pick_folder(ui: &AppWindow, title: &str) -> Option<SharedString> {
    let folder = ui
        .window()
        .with_winit_window(|window| rfd::FileDialog::new().set_title(title).set_parent(window).pick_folder())
        .flatten()?;
    Some(display_path(&folder).into())
}

/// Paths read with `/` separators, like Manifest paths.
fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Moving and resizing a borderless window; minimize, maximize and close are Slint builtins.
fn install_window_chrome(ui: &AppWindow) {
    // A failed drag or resize request leaves the window where it is; nothing to recover.
    ui.on_start_drag({
        let ui = ui.as_weak();
        move || {
            let Some(ui) = ui.upgrade() else { return };
            ui.window().with_winit_window(|window| {
                let _ = window.drag_window();
            });
        }
    });
    ui.on_start_resize({
        let ui = ui.as_weak();
        move |edge| {
            let Some(ui) = ui.upgrade() else { return };
            ui.window().with_winit_window(|window| {
                let _ = window.drag_resize_window(resize_direction(edge));
            });
        }
    });
}

fn resize_direction(edge: ResizeEdge) -> ResizeDirection {
    match edge {
        ResizeEdge::North => ResizeDirection::North,
        ResizeEdge::South => ResizeDirection::South,
        ResizeEdge::East => ResizeDirection::East,
        ResizeEdge::West => ResizeDirection::West,
        ResizeEdge::NorthEast => ResizeDirection::NorthEast,
        ResizeEdge::NorthWest => ResizeDirection::NorthWest,
        ResizeEdge::SouthEast => ResizeDirection::SouthEast,
        ResizeEdge::SouthWest => ResizeDirection::SouthWest,
    }
}
