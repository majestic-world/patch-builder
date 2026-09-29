//! Folder tree over a comparison, flattened into the rows the comparison table shows.

use std::cmp::Ordering;
use std::collections::HashMap;

use crate::comparison::{ComparedFile, FileStatus};

pub type NodeId = usize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortOrder {
    /// Order in which the Source listed the files.
    #[default]
    Source,
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn next(self) -> Self {
        match self {
            Self::Source => Self::Ascending,
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Source,
        }
    }
}

enum NodeKind {
    Folder { children: Vec<NodeId>, expanded: bool },
    File { status: FileStatus, size: u64, hash: String },
}

struct Node {
    name: String,
    kind: NodeKind,
}

pub struct Tree {
    /// Indexed by `NodeId`; ids grow in Source order, so sorting siblings by id restores it.
    nodes: Vec<Node>,
    roots: Vec<NodeId>,
    sort: SortOrder,
    selected: Option<NodeId>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RowKind<'a> {
    Folder { expanded: bool },
    File { status: FileStatus, size: u64, hash: &'a str },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Row<'a> {
    pub node: NodeId,
    pub name: &'a str,
    pub kind: RowKind<'a>,
    pub depth: usize,
    /// Last child of its parent: its connector stops at the row's middle.
    pub is_last: bool,
    /// One entry per ancestor level `1..depth`: whether that level's guide line runs through this row.
    pub guides: Vec<bool>,
    pub selected: bool,
}

impl Tree {
    /// Builds the tree in Source order; only folders holding a New or Changed file start expanded.
    pub fn from_files(files: Vec<ComparedFile>) -> Self {
        let mut tree = Self { nodes: Vec::new(), roots: Vec::new(), sort: SortOrder::Source, selected: None };
        let mut folders: HashMap<Option<NodeId>, HashMap<String, NodeId>> = HashMap::new();

        for file in files {
            let mut parent = None;
            let mut components = file.path.split('/').filter(|part| !part.is_empty()).peekable();
            while let Some(name) = components.next() {
                if components.peek().is_none() {
                    let kind = NodeKind::File { status: file.status, size: file.size, hash: file.hash };
                    tree.push_node(parent, name.to_owned(), kind);
                    break;
                }
                let siblings = folders.entry(parent).or_default();
                parent = Some(match siblings.get(name) {
                    Some(&id) => id,
                    None => {
                        let kind = NodeKind::Folder { children: Vec::new(), expanded: false };
                        let id = tree.push_node(parent, name.to_owned(), kind);
                        siblings.insert(name.to_owned(), id);
                        id
                    }
                });
            }
        }
        tree.expand_changed_folders();
        tree
    }

    /// Expands every folder with a New or Changed file somewhere below it.
    fn expand_changed_folders(&mut self) {
        let mut changed = vec![false; self.nodes.len()];
        // Children always get larger ids than their parent, so walking ids backwards settles
        // every child before the folder that holds it.
        for id in (0..self.nodes.len()).rev() {
            changed[id] = match &self.nodes[id].kind {
                NodeKind::File { status, .. } => *status != FileStatus::Unchanged,
                NodeKind::Folder { children, .. } => children.iter().any(|&child| changed[child]),
            };
            if let NodeKind::Folder { expanded, .. } = &mut self.nodes[id].kind {
                *expanded = changed[id];
            }
        }
    }

    fn push_node(&mut self, parent: Option<NodeId>, name: String, kind: NodeKind) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(Node { name, kind });
        match parent {
            None => self.roots.push(id),
            Some(parent) => match &mut self.nodes[parent].kind {
                NodeKind::Folder { children, .. } => children.push(id),
                NodeKind::File { .. } => unreachable!("files never get children"),
            },
        }
        id
    }

    pub fn sort(&self) -> SortOrder {
        self.sort
    }

    /// Clicking a row: folders expand or collapse, files become the selection.
    pub fn activate(&mut self, id: NodeId) {
        match &mut self.nodes[id].kind {
            NodeKind::Folder { expanded, .. } => *expanded = !*expanded,
            NodeKind::File { .. } => self.selected = Some(id),
        }
    }

    pub fn select(&mut self, id: NodeId) {
        self.selected = Some(id);
    }

    /// Node at a `/`-separated path relative to the Source root.
    pub fn find(&self, path: &str) -> Option<NodeId> {
        let mut level: &[NodeId] = &self.roots;
        let mut found = None;
        for name in path.split('/').filter(|part| !part.is_empty()) {
            let id = *level.iter().find(|&&id| self.nodes[id].name == name)?;
            found = Some(id);
            level = match &self.nodes[id].kind {
                NodeKind::Folder { children, .. } => children,
                NodeKind::File { .. } => &[],
            };
        }
        found
    }

    pub fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
        let sort = self.sort;
        let mut roots = std::mem::take(&mut self.roots);
        roots.sort_by(|a, b| compare(&self.nodes, sort, *a, *b));
        self.roots = roots;
        for id in 0..self.nodes.len() {
            let NodeKind::Folder { children, .. } = &mut self.nodes[id].kind else { continue };
            // A folder's own children list is never read while ordering its children.
            let mut sorted = std::mem::take(children);
            sorted.sort_by(|a, b| compare(&self.nodes, sort, *a, *b));
            if let NodeKind::Folder { children, .. } = &mut self.nodes[id].kind {
                *children = sorted;
            }
        }
    }

    /// Visible rows, depth-first, skipping the contents of collapsed folders.
    pub fn rows(&self) -> Vec<Row<'_>> {
        let mut rows = Vec::new();
        let mut guides = Vec::new();
        self.push_rows(&self.roots, 0, &mut guides, &mut rows);
        rows
    }

    fn push_rows<'a>(&'a self, ids: &[NodeId], depth: usize, guides: &mut Vec<bool>, rows: &mut Vec<Row<'a>>) {
        for (index, &id) in ids.iter().enumerate() {
            let node = &self.nodes[id];
            let is_last = index + 1 == ids.len();
            let (kind, open_children) = match &node.kind {
                NodeKind::Folder { children, expanded } => {
                    (RowKind::Folder { expanded: *expanded }, expanded.then_some(children))
                }
                NodeKind::File { status, size, hash } => {
                    (RowKind::File { status: *status, size: *size, hash }, None)
                }
            };
            rows.push(Row {
                node: id,
                name: &node.name,
                kind,
                depth,
                is_last,
                guides: guides.clone(),
                selected: self.selected == Some(id),
            });
            if let Some(children) = open_children {
                // Root rows draw no connector, so they add no guide level for their descendants.
                if depth > 0 {
                    guides.push(!is_last);
                }
                self.push_rows(children, depth + 1, guides, rows);
                if depth > 0 {
                    guides.pop();
                }
            }
        }
    }
}

/// Folders before files; names compared case-insensitively in the requested direction.
fn compare(nodes: &[Node], sort: SortOrder, a: NodeId, b: NodeId) -> Ordering {
    let is_file = |id: NodeId| matches!(nodes[id].kind, NodeKind::File { .. });
    let by_name = || {
        let lower = |id: NodeId| nodes[id].name.bytes().map(|byte| byte.to_ascii_lowercase());
        lower(a).cmp(lower(b))
    };
    match sort {
        SortOrder::Source => a.cmp(&b),
        SortOrder::Ascending => is_file(a).cmp(&is_file(b)).then_with(by_name),
        SortOrder::Descending => is_file(a).cmp(&is_file(b)).then_with(|| by_name().reverse()),
    }
}

#[cfg(test)]
#[path = "../tests/unit/tree.rs"]
mod tests;
