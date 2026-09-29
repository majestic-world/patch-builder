//! Renders a comparison into the table and the Build Summary.

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::comparison::{ComparedFile, FileStatus, Summary};
use crate::format;
use crate::tree::{self, RowKind, SortOrder, Tree};
use crate::{AppWindow, NameSort, RowStatus, SummaryView, TreeRow};

/// Summary subtitle while nothing has been compared.
pub const SUMMARY_PROMPT: &str = "What will be published";

/// The table's rows and the folder tree they come from; lives on the UI thread.
pub struct Table {
    rows: Rc<VecModel<TreeRow>>,
    tree: RefCell<Option<Tree>>,
}

impl Table {
    pub fn install(ui: &AppWindow) -> Rc<Self> {
        let table = Rc::new(Self { rows: Rc::default(), tree: RefCell::default() });
        ui.set_rows(ModelRc::from(table.rows.clone()));
        ui.set_summary(empty_summary());
        ui.on_row_clicked({
            let (ui, table) = (ui.as_weak(), table.clone());
            move |node| {
                let Some(ui) = ui.upgrade() else { return };
                if let (Some(tree), Ok(node)) = (table.tree.borrow_mut().as_mut(), usize::try_from(node)) {
                    tree.activate(node);
                }
                table.refresh(&ui);
            }
        });
        ui.on_sort_clicked({
            let (ui, table) = (ui.as_weak(), table.clone());
            move || {
                let Some(ui) = ui.upgrade() else { return };
                if let Some(tree) = table.tree.borrow_mut().as_mut() {
                    tree.cycle_sort();
                }
                table.refresh(&ui);
            }
        });
        table
    }

    pub fn show(&self, ui: &AppWindow, files: Vec<ComparedFile>, note: &str) {
        ui.set_summary(summary_view(&Summary::of(&files), note));
        ui.set_has_comparison(true);
        *self.tree.borrow_mut() = Some(Tree::from_files(files));
        self.refresh(ui);
    }

    /// Highlights the file at a Source-relative path, if it is shown.
    pub fn select(&self, ui: &AppWindow, path: &str) {
        if let Some(tree) = self.tree.borrow_mut().as_mut()
            && let Some(node) = tree.find(path)
        {
            tree.select(node);
        }
        self.refresh(ui);
    }

    pub fn clear(&self, ui: &AppWindow) {
        *self.tree.borrow_mut() = None;
        ui.set_has_comparison(false);
        ui.set_summary(empty_summary());
        self.refresh(ui);
    }

    fn refresh(&self, ui: &AppWindow) {
        let tree = self.tree.borrow();
        let Some(tree) = tree.as_ref() else {
            self.rows.clear();
            return;
        };
        ui.set_sort(match tree.sort() {
            SortOrder::Source => NameSort::Source,
            SortOrder::Ascending => NameSort::Ascending,
            SortOrder::Descending => NameSort::Descending,
        });
        self.rows.set_vec(tree.rows().into_iter().map(tree_row).collect::<Vec<_>>());
    }
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
