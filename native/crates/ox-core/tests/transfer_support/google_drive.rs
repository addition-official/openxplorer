// SPDX-License-Identifier: AGPL-3.0-only
//! A simulated Google Drive behind real `google-drive://` URIs, answering
//! as `GVfs`'s Google backend (`gvfsbackendgoogle.c`, `GVfs` 1.54) does,
//! with no Google account and no network.
//!
//! GIO lets a process add its own handler for a URI scheme. Every
//! `google-drive://fake-drive-N@example.com/...` URI of a [`FakeDrive`]
//! resolves to a `gio::File` answered by that drive (see `file.rs`); every
//! other `google-drive://` URI still reaches the real backend. The
//! production `GioNode` therefore runs unchanged.
//!
//! What it imitates, from `gvfsbackendgoogle.c`:
//!
//! - Every item is named by its Drive ID: `standard::name` is the ID and
//!   the title is `standard::display-name`. A name in an address is the
//!   ID of an item of that folder or, failing that, its title; among items
//!   that share a title, the one with the smallest ID. An item found by
//!   its title is "volatile": without following links it is reported as a
//!   link to its ID path.
//! - `standard::copy-name` is the title with "/" made "-", and with the
//!   ID added before the extension when the title names another item.
//! - Making a folder, creating a file, uploading, copying or moving onto a
//!   title an item of the folder has fails with `EXISTS`; Drive itself
//!   would keep both. A move with `OVERWRITE` deletes the existing item
//!   for good after the move, which the app must never ask for.
//! - A copy within the drive is Google's own copy and refuses folders
//!   (`WOULD_RECURSE`); copies off the drive have no backend support
//!   (`NOT_SUPPORTED`), so GIO reads the file.
//! - An upload creates the file at once and fills it while it runs, as
//!   `GVfs`'s write fallback does, so an interrupted upload leaves a
//!   partial file behind under the name it was given.
//! - There is no Trash (`access::can-trash` is false), and delete removes
//!   an item for good; a folder with items is `NOT_EMPTY`.
//! - Docs, Sheets and Slides files are shortcuts to their web page and
//!   cannot be read (`NOT_REGULAR_FILE`).
//! - The account's top folder, Shared with me and the list of shared
//!   drives take nothing (`NOT_SUPPORTED`).
//! - A full drive refuses an upload with "Permission denied", as `GVfs`
//!   reports Google's 403 `storageQuotaExceeded`; free space comes from
//!   the drive's quota.
//! - Offline, every call fails with the resolver's message, as `GVfs`
//!   passes libsoup's error on.
//! - Older `GVfs` versions refuse moves to another folder
//!   (`NOT_SUPPORTED`), which [`FakeDrive::refuse_moves_across_folders`]
//!   turns on.

mod file;

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use gio::prelude::*;
use ox_core::gio_node::GioNode;

use file::DriveFile;

/// The ID of the account's top folder.
pub const ROOT_ID: &str = "GVfsRoot";
/// The ID of My Drive.
pub const MY_DRIVE_ID: &str = "0AMyDrive";
/// The ID of Shared with me.
pub const SHARED_WITH_ME_ID: &str = "GVfsSharedWithMe";
/// The ID of the list of shared drives.
pub const SHARED_DRIVES_ID: &str = "GVfsSharedDrives";

/// What an item is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemKind {
    /// A folder.
    Folder,
    /// A file whose bytes are kept in the drive's blob folder.
    File,
    /// A Docs, Sheets or Slides file: a shortcut to its web page.
    Document,
}

/// One item on the drive.
#[derive(Debug, Clone)]
pub struct Item {
    /// The Drive ID.
    pub id: String,
    /// The title people see.
    pub title: String,
    /// The ID of the folder that holds it.
    pub parent: String,
    /// What it is.
    pub kind: ItemKind,
}

/// One call that changed the drive, in the order it was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveCall {
    /// A folder was made.
    MakeDirectory(String),
    /// A file was created (New, or GIO's write fallback).
    Create(String),
    /// A file was uploaded.
    Upload(String),
    /// Google's own copy within the drive.
    ServerCopy(String),
    /// A move; `overwrite` when `G_FILE_COPY_OVERWRITE` was given.
    Move {
        /// The title moved.
        title: String,
        /// Whether the move asked to overwrite.
        overwrite: bool,
    },
    /// A title changed.
    Rename {
        /// The old title.
        from: String,
        /// The new title.
        to: String,
    },
    /// An item was deleted for good.
    Delete(String),
}

/// How an upload goes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum UploadFault {
    /// Every byte arrives.
    #[default]
    None,
    /// The connection drops after this many bytes.
    DropAfter(u64),
    /// The backend reports success though only this many bytes arrived.
    Truncate(u64),
}

/// The mutable state of a drive.
#[derive(Debug, Default)]
struct State {
    items: BTreeMap<String, Item>,
    next_id: usize,
    quota: Option<u64>,
    offline: bool,
    moves_across_folders_refused: bool,
    upload_fault: UploadFault,
    upload_chunk: usize,
    calls: Vec<DriveCall>,
}

/// One simulated Google Drive account.
#[derive(Debug)]
pub struct FakeDrive {
    account: String,
    blobs: tempfile::TempDir,
    state: Mutex<State>,
}

impl FakeDrive {
    /// A new, empty drive: My Drive, Shared with me and Shared drives.
    pub fn new() -> Arc<Self> {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        register_scheme_handler();
        let number = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let account = format!("fake-drive-{number}@example.com");
        let drive = Arc::new(Self {
            account: account.clone(),
            blobs: tempfile::tempdir().expect("blob folder"),
            state: Mutex::new(State {
                upload_chunk: 64 * 1024,
                next_id: 1,
                ..State::default()
            }),
        });
        {
            let mut state = drive.state.lock().expect("drive");
            for (id, title) in [
                (MY_DRIVE_ID, "My Drive"),
                (SHARED_WITH_ME_ID, "Shared with me"),
                (SHARED_DRIVES_ID, "Shared drives"),
            ] {
                state.items.insert(
                    id.to_owned(),
                    Item {
                        id: id.to_owned(),
                        title: title.to_owned(),
                        parent: ROOT_ID.to_owned(),
                        kind: ItemKind::Folder,
                    },
                );
            }
        }
        drives()
            .lock()
            .expect("drive registry")
            .insert(account, Arc::clone(&drive));
        drive
    }

    /// The URI of My Drive.
    pub fn my_drive(&self) -> String {
        format!("google-drive://{}/{MY_DRIVE_ID}", self.account)
    }

    /// The URI of the account's top folder.
    pub fn top(&self) -> String {
        format!("google-drive://{}/", self.account)
    }

    /// The ID path URI of the item `id`.
    pub fn uri_of(&self, id: &str) -> String {
        let state = self.state.lock().expect("drive");
        let mut segments = Vec::new();
        let mut current = id.to_owned();
        while current != ROOT_ID {
            segments.push(current.clone());
            current = state.items.get(&current).expect("known item").parent.clone();
        }
        segments.reverse();
        format!("google-drive://{}/{}", self.account, segments.join("/"))
    }

    /// The production adapter for the item `id`.
    pub fn node(&self, id: &str) -> GioNode {
        GioNode::new(&self.uri_of(id))
    }

    /// Adds a folder titled `title` to the folder `parent`; its ID.
    pub fn add_folder(&self, parent: &str, title: &str) -> String {
        self.add(parent, title, ItemKind::Folder, None)
    }

    /// Adds a file titled `title` holding `bytes` to `parent`; its ID.
    pub fn add_file(&self, parent: &str, title: &str, bytes: &[u8]) -> String {
        self.add(parent, title, ItemKind::File, Some(bytes))
    }

    /// Adds a Docs file titled `title` to `parent`; its ID.
    pub fn add_document(&self, parent: &str, title: &str) -> String {
        self.add(parent, title, ItemKind::Document, None)
    }

    /// Sets the drive's quota; free space is what the files leave of it.
    pub fn set_quota(&self, bytes: u64) {
        self.state.lock().expect("drive").quota = Some(bytes);
    }

    /// Takes the drive offline, or back online.
    pub fn set_offline(&self, offline: bool) {
        self.state.lock().expect("drive").offline = offline;
    }

    /// Answers moves to another folder as older `GVfs` versions do.
    pub fn refuse_moves_across_folders(&self) {
        self.state.lock().expect("drive").moves_across_folders_refused = true;
    }

    /// Sets how later uploads go.
    pub fn set_upload_fault(&self, fault: UploadFault) {
        self.state.lock().expect("drive").upload_fault = fault;
    }

    /// Sets how many bytes an upload sends between progress reports.
    pub fn set_upload_chunk(&self, bytes: usize) {
        self.state.lock().expect("drive").upload_chunk = bytes.max(1);
    }

    /// The calls that changed the drive, in order.
    pub fn calls(&self) -> Vec<DriveCall> {
        self.state.lock().expect("drive").calls.clone()
    }

    /// Every item of the folder `parent`, sorted by title then ID.
    pub fn children(&self, parent: &str) -> Vec<Item> {
        let state = self.state.lock().expect("drive");
        let mut items: Vec<Item> = state
            .items
            .values()
            .filter(|item| item.parent == parent)
            .cloned()
            .collect();
        items.sort_by(|a, b| (&a.title, &a.id).cmp(&(&b.title, &b.id)));
        items
    }

    /// The titles of the folder `parent`, sorted.
    pub fn titles(&self, parent: &str) -> Vec<String> {
        self.children(parent).into_iter().map(|item| item.title).collect()
    }

    /// The bytes of the file `id`.
    pub fn bytes(&self, id: &str) -> Vec<u8> {
        std::fs::read(self.blob(id)).expect("file bytes")
    }

    /// The ID of the only item titled `title` in `parent`.
    pub fn id_of(&self, parent: &str, title: &str) -> String {
        let matching: Vec<Item> = self
            .children(parent)
            .into_iter()
            .filter(|item| item.title == title)
            .collect();
        assert_eq!(matching.len(), 1, "one item titled {title}: {matching:?}");
        matching[0].id.clone()
    }

    /// Every item on the drive.
    pub fn all_items(&self) -> Vec<Item> {
        self.state
            .lock()
            .expect("drive")
            .items
            .values()
            .cloned()
            .collect()
    }

    fn add(&self, parent: &str, title: &str, kind: ItemKind, bytes: Option<&[u8]>) -> String {
        let id = {
            let mut state = self.state.lock().expect("drive");
            assert!(state.items.contains_key(parent), "parent {parent}");
            new_item(&mut state, parent, title, kind)
        };
        if let Some(bytes) = bytes {
            std::fs::write(self.blob(&id), bytes).expect("write blob");
        }
        id
    }

    /// Where the bytes of the file `id` are kept.
    fn blob(&self, id: &str) -> PathBuf {
        self.blobs.path().join(id)
    }
}

/// Adds an item and returns its new ID. IDs grow, so a later item sorts
/// after an earlier one, as the backend's tie-break expects.
fn new_item(state: &mut State, parent: &str, title: &str, kind: ItemKind) -> String {
    let id = format!("1Id{:05}", state.next_id);
    state.next_id += 1;
    state.items.insert(
        id.clone(),
        Item {
            id: id.clone(),
            title: title.to_owned(),
            parent: parent.to_owned(),
            kind,
        },
    );
    id
}

/// The drives by account, for the scheme handler.
fn drives() -> &'static Mutex<HashMap<String, Arc<FakeDrive>>> {
    static DRIVES: OnceLock<Mutex<HashMap<String, Arc<FakeDrive>>>> = OnceLock::new();
    DRIVES.get_or_init(Mutex::default)
}

/// Routes the `google-drive://` URIs of fake drives to [`DriveFile`],
/// once per test process.
fn register_scheme_handler() {
    static REGISTERED: OnceLock<()> = OnceLock::new();
    REGISTERED.get_or_init(|| {
        let registered = gio::Vfs::default().register_uri_scheme(
            "google-drive",
            Some(Box::new(|_vfs, uri| drive_file_for(uri))),
            Some(Box::new(|_vfs, parse_name| drive_file_for(parse_name))),
        );
        assert!(
            registered,
            "the google-drive scheme handler could not be registered"
        );
    });
}

/// The fake file for `uri`, or `None` to let the real backend handle it.
fn drive_file_for(uri: &str) -> Option<gio::File> {
    let rest = uri.strip_prefix("google-drive://")?;
    let (account, path) = rest.split_once('/').unwrap_or((rest, ""));
    let registry = drives().lock().expect("drive registry");
    let drive = registry.get(account)?;
    let segments: Vec<String> = path
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect();
    Some(DriveFile::new(Arc::clone(drive), &segments).upcast())
}
