// SPDX-License-Identifier: AGPL-3.0-only
//! The folder trees of the navigation pane's drives and shares, as
//! Windows 11 draws them: This PC's drives, Local Disk and the network
//! shares under Network each have a chevron that lists their folders
//! under them, and every folder that holds folders has one too.
//!
//! The rows are put in by [`BrowserWindow::add_folder_trees`] each time
//! the places are drawn; this module keeps the window's part: which
//! folders are expanded, what each one's listing gave, and the work that
//! reads them. A folder is read when it is expanded, off the main loop
//! (GIO's asynchronous listing, as the file list's), and again each time
//! it is expanded anew; a share that needs mounting is mounted first,
//! asking for a password only when it wants one. Whether a local folder
//! holds folders, which decides its chevron, is checked on a worker
//! thread, stopping at the first folder found; on a network share the
//! chevron shows until the folder is expanded, as in Windows. With
//! "Expand to current folder" on, the trees open down to the folder the
//! tab shows.

use std::cell::RefCell;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use ox_core::entry::{Entry, EntryError};
use ox_core::location::{is_server_location, is_virtual_location, same_location};

use super::sidebar::entries::{
    folder_entry, ExpanderKind, FolderNode, RowExpander, RowLevel, RowTarget, Section, SidebarEntry,
};
use super::BrowserWindow;
use crate::folder_view::loader;

/// The most folders listed under one folder; a row after them opens the
/// folder to show the rest.
const MAX_FOLDERS: usize = 1000;

/// The most entries read of one local folder to learn whether it holds a
/// folder; past them it is taken to hold one.
const MAX_PEEK: usize = 5000;

/// The window's state of the navigation pane's folder trees.
#[derive(Debug, Default)]
pub(super) struct PaneFolders {
    /// The locations expanded.
    expanded: RefCell<Vec<String>>,
    /// Each expanded location's folders, once listed, and whether more
    /// were left out.
    listed: RefCell<Vec<(String, Vec<FolderNode>, bool)>>,
    /// The listings that run, by location; dropping one cancels it.
    listings: RefCell<Vec<(String, loader::Listing)>>,
    /// The folder the trees open down to, with "Expand to current
    /// folder" on, until they reach it or the user toggles a folder.
    following: RefCell<Option<String>>,
}

/// Whether `entry`, a row inside This PC or Network, roots a folder tree:
/// a mounted drive, Local Disk, or a network location other than a
/// computer, which has its own chevron.
fn roots_a_tree(entry: &SidebarEntry) -> Option<&str> {
    let RowTarget::Location(uri) = &entry.target else {
        return None;
    };
    let inside = match entry.section {
        Section::ThisPc => entry.level == RowLevel::Child,
        Section::Network => matches!(entry.level, RowLevel::Child | RowLevel::Grandchild),
        _ => false,
    };
    let is_folder = !is_virtual_location(uri) && !uri.starts_with("network:");
    (inside && entry.expander.is_none() && is_folder).then_some(uri.as_str())
}

/// The subfolder a listed row is, `None` for a file or a hidden folder
/// while hidden items are not shown.
fn folder_node(entry: &Entry, show_hidden: bool) -> Option<FolderNode> {
    if !entry.is_dir || (entry.is_hidden && !show_hidden) {
        return None;
    }
    Some(FolderNode {
        name: entry.name.clone(),
        uri: entry.navigation_uri().to_owned(),
        has_subfolders: None,
    })
}

/// Whether the local folder `path` holds a folder (hidden ones only when
/// `show_hidden`), reading at most [`MAX_PEEK`] entries; `None` when it
/// cannot be read. Runs on a worker thread.
fn holds_folders(path: &std::path::Path, show_hidden: bool) -> Option<bool> {
    let entries = std::fs::read_dir(path).ok()?;
    for (read, entry) in entries.enumerate() {
        if read >= MAX_PEEK {
            return Some(true);
        }
        let Ok(entry) = entry else { continue };
        let hidden = entry.file_name().to_string_lossy().starts_with('.');
        if hidden && !show_hidden {
            continue;
        }
        // A link to a folder counts, as the file list opens it as one.
        if std::fs::metadata(entry.path()).is_ok_and(|meta| meta.is_dir()) {
            return Some(true);
        }
    }
    Some(false)
}

impl BrowserWindow {
    fn pane_folders(&self) -> &PaneFolders {
        &self.imp().pane_folders
    }

    /// `entries` with a chevron on every drive and share that roots a
    /// folder tree, and the folders of the expanded ones under them.
    pub(super) fn add_folder_trees(&self, entries: Vec<SidebarEntry>) -> Vec<SidebarEntry> {
        let folders = self.pane_folders();
        let expanded = folders.expanded.borrow();
        let listed = folders.listed.borrow();
        let locations = self.imp().locations.borrow();
        let mut rows = Vec::with_capacity(entries.len());
        for mut entry in entries {
            let Some(uri) = roots_a_tree(&entry).map(str::to_owned) else {
                rows.push(entry);
                continue;
            };
            let is_expanded = expanded.contains(&uri);
            entry.expander = Some(RowExpander {
                uri: uri.clone(),
                expanded: is_expanded,
                kind: ExpanderKind::Folder,
            });
            let (section, depth, label) = (entry.section, entry.level.depth(), entry.label.clone());
            rows.push(entry);
            if is_expanded {
                let tree = Tree {
                    expanded: &expanded,
                    listed: &listed,
                    locations: &locations,
                };
                push_subfolders(&mut rows, section, depth + 1, &uri, &label, &tree);
            }
        }
        rows
    }

    /// Expands the drive, share or folder at `uri` to its folders, or
    /// collapses it, as its chevron does.
    pub(super) fn toggle_folder(&self, uri: &str) {
        // The user takes over from "Expand to current folder".
        self.pane_folders().following.replace(None);
        let expand = !self
            .pane_folders()
            .expanded
            .borrow()
            .iter()
            .any(|known| known == uri);
        self.expand_folder(uri, expand);
    }

    /// Expands the folder at `uri` when `expand`, else collapses it;
    /// false when it already is.
    pub(super) fn expand_folder(&self, uri: &str, expand: bool) -> bool {
        let folders = self.pane_folders();
        {
            let mut expanded = folders.expanded.borrow_mut();
            if expanded.iter().any(|known| known == uri) == expand {
                return false;
            }
            expanded.retain(|known| known != uri);
            if expand {
                expanded.push(uri.to_owned());
            }
        }
        if expand {
            self.list_folder_for_pane(uri);
        } else {
            folders.listings.borrow_mut().retain(|(known, _)| known != uri);
        }
        self.render_places();
        true
    }

    /// Whether the folder at `uri` is expanded in the navigation pane.
    pub(super) fn is_folder_expanded(&self, uri: &str) -> bool {
        self.pane_folders()
            .expanded
            .borrow()
            .iter()
            .any(|known| known == uri)
    }

    /// Reads the folders of `uri` for the navigation pane, mounting its
    /// share first when it needs it, then whether each local one holds
    /// folders, and draws them.
    fn list_folder_for_pane(&self, uri: &str) {
        let window = self.downgrade();
        let uri = uri.to_owned();
        let task_uri = uri.clone();
        let show_hidden = self.context().settings_data().preferences.show_hidden;
        let listing = loader::Listing::spawn(async move {
            let found = read_folders(&window, &uri, show_hidden).await;
            let checks = match found {
                // The window closed, or the listing was dropped.
                None => return,
                Some(found) => window
                    .upgrade()
                    .and_then(|window| window.show_listed_folders(&uri, found)),
            };
            // Which local folders hold folders, on a worker thread.
            if let Some(paths) = checks {
                let checked = gio::spawn_blocking(move || {
                    paths
                        .iter()
                        .map(|path| path.as_deref().and_then(|path| holds_folders(path, show_hidden)))
                        .collect::<Vec<_>>()
                })
                .await;
                if let (Ok(checked), Some(window)) = (checked, window.upgrade()) {
                    window.mark_folders_with_subfolders(&uri, checked);
                }
            }
            // Last: dropping its own listing ends this task.
            if let Some(window) = window.upgrade() {
                let folders = window.pane_folders();
                let finished = folders
                    .listings
                    .borrow_mut()
                    .iter()
                    .position(|(known, _)| *known == uri);
                let listing = finished.map(|at| folders.listings.borrow_mut().remove(at));
                window.follow_current_folder();
                drop(listing);
            }
        });
        let mut listings = self.pane_folders().listings.borrow_mut();
        listings.retain(|(known, _)| *known != task_uri);
        listings.push((task_uri, listing));
    }

    /// Shows the folders `found` under `uri`, or collapses it again when
    /// it could not be read; the local paths whose subfolders are still
    /// to be checked, if any.
    fn show_listed_folders(
        &self,
        uri: &str,
        found: Option<(Vec<FolderNode>, bool)>,
    ) -> Option<Vec<Option<PathBuf>>> {
        let folders = self.pane_folders();
        let Some((mut nodes, more)) = found else {
            // An unreadable folder is collapsed again.
            folders.expanded.borrow_mut().retain(|known| known != uri);
            self.render_places();
            return None;
        };
        nodes.sort_by_cached_key(|node| node.name.to_lowercase());
        let paths: Vec<Option<PathBuf>> = nodes
            .iter()
            .map(|node| gio::File::for_uri(&node.uri).path())
            .collect();
        {
            let mut listed = folders.listed.borrow_mut();
            listed.retain(|(known, _, _)| known != uri);
            listed.push((uri.to_owned(), nodes, more));
        }
        self.render_places();
        // With "Expand to current folder" on, the next folder on the way
        // to the one shown opens now that it is listed.
        self.follow_current_folder();
        (!paths.iter().all(Option::is_none)).then_some(paths)
    }

    /// Marks which folders listed under `uri` hold folders, as `checked`
    /// says in their order, and draws their chevrons again.
    fn mark_folders_with_subfolders(&self, uri: &str, checked: Vec<Option<bool>>) {
        {
            let mut listed = self.pane_folders().listed.borrow_mut();
            let Some((_, nodes, _)) = listed.iter_mut().find(|(known, _, _)| known == uri) else {
                return;
            };
            for (node, holds) in nodes.iter_mut().zip(checked) {
                node.has_subfolders = holds;
            }
        }
        self.render_places();
    }

    /// With "Expand to current folder" on, opens the trees down to the
    /// folder at `uri`: each folder on the way that a shown tree holds is
    /// expanded, and the next ones once its folders are read.
    pub(super) fn expand_pane_to(&self, uri: &str) {
        let follows = self.folder_tree().options().expand_to_current && !is_virtual_location(uri);
        self.pane_folders()
            .following
            .replace(follows.then(|| uri.to_owned()));
        self.follow_current_folder();
    }

    /// Expands the next folder on the way to the one "Expand to current
    /// folder" follows, or stops following once there is none.
    fn follow_current_folder(&self) {
        let Some(uri) = self.pane_folders().following.borrow().clone() else {
            return;
        };
        let target = gio::File::for_uri(&uri);
        let rows: Vec<String> = self
            .sidebar()
            .tree_locations()
            .into_iter()
            .filter(|row| !is_server_location(row))
            .collect();
        for row in rows {
            let folder = gio::File::for_uri(&row);
            let on_the_way = target.has_prefix(&folder) && !same_location(&row, &uri);
            if on_the_way && !self.is_folder_expanded(&row) {
                self.expand_folder(&row, true);
                return;
            }
        }
        // Every folder on the way is open, or waits for its listing.
        let waiting = !self.pane_folders().listings.borrow().is_empty();
        if !waiting {
            self.pane_folders().following.replace(None);
        }
    }
}

/// What the folder rows are drawn from.
struct Tree<'a> {
    /// The locations expanded.
    expanded: &'a [String],
    /// What each expanded location's listing gave.
    listed: &'a [(String, Vec<FolderNode>, bool)],
    /// How locations are named.
    locations: &'a ox_core::location::LocationContext,
}

/// Pushes onto `rows` the folders listed for `uri`, the row named
/// `name`, `depth` levels inside `section`, each followed by its own
/// folders while it is expanded.
fn push_subfolders(
    rows: &mut Vec<SidebarEntry>,
    section: Section,
    depth: u8,
    uri: &str,
    name: &str,
    tree: &Tree<'_>,
) {
    let Some((_, nodes, more)) = tree.listed.iter().find(|(known, _, _)| known == uri) else {
        return;
    };
    for node in nodes.iter().take(MAX_FOLDERS) {
        let is_expanded = tree.expanded.contains(&node.uri);
        rows.push(folder_entry(section, depth, node, is_expanded, tree.locations));
        // Deep trees stop at the deepest level a row can draw.
        if is_expanded && depth < u8::MAX {
            push_subfolders(rows, section, depth + 1, &node.uri, &node.name, tree);
        }
    }
    if *more || nodes.len() > MAX_FOLDERS {
        rows.push(more_folders_entry(section, depth, uri, name, tree.locations));
    }
}

/// The row after the first [`MAX_FOLDERS`] folders of `uri`, the row
/// named `name`, which opens it to show them all.
fn more_folders_entry(
    section: Section,
    depth: u8,
    uri: &str,
    name: &str,
    locations: &ox_core::location::LocationContext,
) -> SidebarEntry {
    SidebarEntry {
        section,
        level: RowLevel::at_depth(depth),
        label: ox_core::i18n::format_message("More folders in {name}…", &[("name", name)]),
        icon: crate::icons::Art::Glyph(crate::icons::Icon::MoreHorizontal),
        target: RowTarget::Location(uri.to_owned()),
        tooltip: locations.display_location(uri),
        pinned: false,
        menu: None,
        eject: None,
        expander: None,
    }
}

/// The folders at `uri` and whether more were left out:
/// `Some(None)` when it could not be read, `None` when the window closed.
async fn read_folders(
    window: &glib::WeakRef<BrowserWindow>,
    uri: &str,
    show_hidden: bool,
) -> Option<Option<(Vec<FolderNode>, bool)>> {
    let mut mounted = false;
    loop {
        let rows = std::rc::Rc::new(RefCell::new(Vec::new()));
        let (sender, receiver) = async_channel::bounded(1);
        let _listing = {
            // Holds the window only while the listing starts.
            let _window = window.upgrade()?;
            let collected = std::rc::Rc::clone(&rows);
            let on_batch = move |entries: Vec<Entry>| {
                let mut rows = collected.borrow_mut();
                rows.extend(entries.iter().filter_map(|entry| folder_node(entry, show_hidden)));
            };
            let on_done = move |result: Result<(), EntryError>| {
                // The task awaiting it may have been dropped.
                let _ = sender.try_send(result);
            };
            loader::list_folder(uri, on_batch, on_done)
        };
        let result = receiver.recv().await.unwrap_or(Err(EntryError::Cancelled));
        match result {
            Ok(()) => {
                let mut nodes = rows.take();
                let more = nodes.len() > MAX_FOLDERS;
                nodes.truncate(MAX_FOLDERS);
                return Some(Some((nodes, more)));
            }
            Err(error) if error.needs_mount() && !mounted => {
                mounted = true;
                let mounting = window.upgrade()?.network().mount(uri);
                if mounting.await.is_err() {
                    return Some(None);
                }
            }
            Err(EntryError::Cancelled) => return None,
            Err(error) => {
                glib::g_debug!(
                    ox_core::LOG_DOMAIN,
                    "The navigation pane could not list {uri}: {error}"
                );
                return Some(None);
            }
        }
    }
}
