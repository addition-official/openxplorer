// SPDX-License-Identifier: AGPL-3.0-only
//! The `gio::File` and `gio::FileEnumerator` of a [`FakeDrive`]: thin
//! `GObject` types that answer GIO's calls from the drive's model, the
//! way `gvfsbackendgoogle.c` answers them.

use std::ffi::OsStr;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use gio::prelude::*;
use gio::subclass::prelude::*;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, CONTROLS};

use super::{
    new_item, DriveCall, FakeDrive, Item, ItemKind, State, UploadFault, MY_DRIVE_ID, ROOT_ID,
    SHARED_DRIVES_ID, SHARED_WITH_ME_ID,
};

/// Characters escaped in one path segment of a URI, as GIO escapes them.
const SEGMENT_ESCAPES: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

/// The message `GVfs` passes on for a drive that cannot be reached.
const OFFLINE_MESSAGE: &str = "Error resolving “www.googleapis.com”: Temporary failure in name resolution";

glib::wrapper! {
    /// One address on a [`FakeDrive`].
    pub struct DriveFile(ObjectSubclass<imp::DriveFile>) @implements gio::File;
}

impl DriveFile {
    /// The address made of `segments` (decoded) on `drive`.
    pub(super) fn new(drive: Arc<FakeDrive>, segments: &[String]) -> Self {
        let file: Self = glib::Object::new();
        let decoded = segments
            .iter()
            .map(|segment| percent_decode_str(segment).decode_utf8_lossy().into_owned())
            .collect();
        file.imp()
            .segments
            .set(decoded)
            .expect("a new file has no address yet");
        file.imp().drive.set(drive).expect("a new file has no drive yet");
        file
    }
}

glib::wrapper! {
    /// The listing of one folder on a [`FakeDrive`].
    pub struct DriveListing(ObjectSubclass<imp::DriveListing>) @extends gio::FileEnumerator;
}

impl DriveListing {
    /// A listing of `folder` that hands out `infos` in order.
    fn new(folder: &DriveFile, mut infos: Vec<gio::FileInfo>) -> Self {
        infos.reverse();
        let listing: Self = glib::Object::builder()
            .property("container", folder.upcast_ref::<gio::File>())
            .build();
        *listing.imp().remaining.lock().expect("listing") = infos;
        listing
    }
}

/// A GIO error as the backend reports it.
fn error(code: gio::IOErrorEnum, message: &str) -> glib::Error {
    glib::Error::new(code, message)
}

/// The error of an address that names nothing.
fn not_found() -> glib::Error {
    error(gio::IOErrorEnum::NotFound, "No such file or directory")
}

/// The error of an operation the backend refuses here.
fn not_supported() -> glib::Error {
    error(gio::IOErrorEnum::NotSupported, "Operation not supported")
}

/// The error of a title that is taken.
fn exists() -> glib::Error {
    error(gio::IOErrorEnum::Exists, "Target file already exists")
}

/// The error of a drive that cannot be reached.
fn offline() -> glib::Error {
    error(gio::IOErrorEnum::Failed, OFFLINE_MESSAGE)
}

/// What an address resolves to.
#[derive(Debug, Clone)]
struct Resolved {
    /// The item's ID.
    id: String,
    /// Whether a part of the address named an item by its title.
    volatile: bool,
    /// The item's path of IDs.
    id_path: Vec<String>,
}

impl State {
    /// The item of `folder` that `name` names: by ID, else by title, the
    /// smallest ID among items sharing it (`insert_entry_full`).
    fn child_named(&self, folder: &str, name: &str) -> Option<(String, bool)> {
        if let Some(item) = self.items.get(name) {
            if item.parent == folder {
                return Some((item.id.clone(), false));
            }
        }
        self.items
            .values()
            .filter(|item| item.parent == folder && item.title == name)
            .map(|item| item.id.clone())
            .min()
            .map(|id| (id, true))
    }

    /// Resolves `segments` from the top folder (`resolve`).
    fn resolve(&self, segments: &[String]) -> Option<Resolved> {
        let mut current = ROOT_ID.to_owned();
        let mut volatile = false;
        let mut id_path = Vec::new();
        for segment in segments {
            let is_folder = current == ROOT_ID
                || self
                    .items
                    .get(&current)
                    .is_some_and(|item| item.kind == ItemKind::Folder);
            if !is_folder {
                return None;
            }
            let (id, by_title) = self.child_named(&current, segment)?;
            volatile |= by_title;
            id_path.push(id.clone());
            current = id;
        }
        Some(Resolved {
            id: current,
            volatile,
            id_path,
        })
    }

    /// How many items of `folder` have `title`.
    fn titled(&self, folder: &str, title: &str) -> usize {
        self.items
            .values()
            .filter(|item| item.parent == folder && item.title == title)
            .count()
    }

    /// The bytes all files use, for the quota.
    fn used(&self, drive: &FakeDrive) -> u64 {
        self.items
            .values()
            .filter(|item| item.kind == ItemKind::File)
            .map(|item| std::fs::metadata(drive.blob(&item.id)).map_or(0, |meta| meta.len()))
            .sum()
    }

    /// The copy name of `item` (`generate_copy_name`, then "/" made "-").
    fn copy_name(&self, item: &Item) -> String {
        let first = self.child_named(&item.parent, &item.title).map(|(id, _)| id);
        let name = if first.as_deref() == Some(item.id.as_str()) {
            item.title.clone()
        } else {
            match item.title.rfind('.') {
                Some(dot) if dot > 0 => format!("{} - {}{}", &item.title[..dot], item.id, &item.title[dot..]),
                _ => format!("{} - {}", item.title, item.id),
            }
        };
        name.replace('/', "-")
    }
}

/// True for the folders that take no items: the top folder, Shared with
/// me and the list of shared drives.
fn takes_no_items(folder: &str) -> bool {
    folder == ROOT_ID || folder == SHARED_WITH_ME_ID || folder == SHARED_DRIVES_ID
}

/// True for the items that cannot be changed: the top folders and what
/// Shared with me lists directly.
fn is_fixed(state: &State, id: &str) -> bool {
    if id == ROOT_ID || id == MY_DRIVE_ID || id == SHARED_WITH_ME_ID || id == SHARED_DRIVES_ID {
        return true;
    }
    state
        .items
        .get(id)
        .is_some_and(|item| takes_no_items(&item.parent))
}

mod imp {
    use super::*;

    /// The state behind a [`super::DriveFile`].
    #[derive(Debug, Default)]
    pub struct DriveFile {
        pub(super) segments: OnceLock<Vec<String>>,
        pub(super) drive: OnceLock<Arc<FakeDrive>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DriveFile {
        const NAME: &'static str = "OxTestDriveFile";
        type Type = super::DriveFile;
        type Interfaces = (gio::File,);
    }

    impl ObjectImpl for DriveFile {}

    impl DriveFile {
        fn segments(&self) -> &[String] {
            self.segments.get().expect("set at construction")
        }

        pub(super) fn drive(&self) -> &Arc<FakeDrive> {
            self.drive.get().expect("set at construction")
        }

        fn file_at(&self, segments: &[String]) -> gio::File {
            let escaped: Vec<String> = segments.iter().map(|segment| escape(segment)).collect();
            super::DriveFile::new(Arc::clone(self.drive()), &escaped).upcast()
        }

        fn location(&self) -> String {
            let escaped: Vec<String> = self.segments().iter().map(|segment| escape(segment)).collect();
            format!("google-drive://{}/{}", self.drive().account, escaped.join("/"))
        }

        /// The address of the folder that holds this one, and this
        /// address's last part; `None` at the top folder.
        fn split(&self) -> Option<(Vec<String>, String)> {
            let (last, parent) = self.segments().split_last()?;
            Some((parent.to_vec(), last.clone()))
        }

        /// Resolves this address, failing offline or when it names nothing.
        fn resolve(&self, state: &State) -> Result<Resolved, glib::Error> {
            if state.offline {
                return Err(offline());
            }
            state.resolve(self.segments()).ok_or_else(not_found)
        }

        /// Resolves the folder this address would be created in, and the
        /// title it asks for.
        fn resolve_parent(&self, state: &State) -> Result<(String, String), glib::Error> {
            if state.offline {
                return Err(offline());
            }
            let (parent, title) = self.split().ok_or_else(not_supported)?;
            let folder = state.resolve(&parent).ok_or_else(not_found)?;
            let is_folder = folder.id == ROOT_ID
                || state
                    .items
                    .get(&folder.id)
                    .is_some_and(|item| item.kind == ItemKind::Folder);
            if !is_folder {
                return Err(error(
                    gio::IOErrorEnum::NotDirectory,
                    "The file is not a directory",
                ));
            }
            Ok((folder.id, title))
        }

        /// `build_file_info` for `id`, reached as `resolved`.
        pub(super) fn info_of(
            &self,
            state: &State,
            id: &str,
            volatile: bool,
            flags: gio::FileQueryInfoFlags,
        ) -> gio::FileInfo {
            let info = gio::FileInfo::new();
            if id == ROOT_ID {
                info.set_name("/");
                info.set_display_name(&self.drive().account);
                info.set_file_type(gio::FileType::Directory);
                info.set_size(0);
                info.set_attribute_boolean("access::can-trash", false);
                return info;
            }
            let item = state.items.get(id).expect("resolved item");
            let link = volatile && flags.contains(gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS);
            let name = if volatile {
                self.segments().last().cloned().unwrap_or_default()
            } else {
                item.id.clone()
            };
            info.set_name(&name);
            info.set_display_name(&item.title);
            info.set_edit_name(&item.title);
            info.set_is_symlink(volatile);
            let file_type = match (&item.kind, link) {
                (_, true) => gio::FileType::SymbolicLink,
                (ItemKind::Folder, false) => gio::FileType::Directory,
                (ItemKind::File, false) => gio::FileType::Regular,
                (ItemKind::Document, false) => gio::FileType::Shortcut,
            };
            info.set_file_type(file_type);
            // GIO's getters need a size; GVfs gives folders and Docs none
            // that means anything.
            info.set_size(0);
            if item.kind == ItemKind::File {
                let size = std::fs::metadata(self.drive().blob(&item.id)).map_or(0, |meta| meta.len());
                info.set_size(i64::try_from(size).unwrap_or(i64::MAX));
            }
            if item.kind == ItemKind::Document {
                info.set_attribute_string(
                    "standard::target-uri",
                    &format!("https://docs.google.com/document/d/{}/edit", item.id),
                );
            }
            let fixed = is_fixed(state, id);
            info.set_attribute_boolean("access::can-trash", false);
            info.set_attribute_boolean("access::can-delete", !fixed);
            info.set_attribute_boolean("access::can-rename", !fixed);
            info.set_attribute_boolean("access::can-write", true);
            info.set_attribute_boolean("access::can-read", true);
            info.set_attribute_string("id::file", &item.id);
            info.set_attribute_uint64("time::modified", 1_700_000_000);
            if !fixed || item.parent != ROOT_ID {
                info.set_attribute_string("standard::copy-name", &state.copy_name(item));
            }
            info
        }

        /// Moves `id` into `folder` with `title`, overwriting only when
        /// `overwrite` (`g_vfs_backend_google_move`).
        pub(super) fn move_to(
            &self,
            id: &str,
            folder: &str,
            title: &str,
            overwrite: bool,
        ) -> Result<(), glib::Error> {
            let drive = self.drive();
            let mut state = drive.state.lock().expect("drive");
            if is_fixed(&state, id) || takes_no_items(folder) {
                return Err(not_supported());
            }
            let source_parent = state.items[id].parent.clone();
            if state.moves_across_folders_refused && source_parent != folder {
                return Err(not_supported());
            }
            let new_title = if title == id {
                state.items[id].title.clone()
            } else {
                title.to_owned()
            };
            let existing: Vec<String> = state
                .items
                .values()
                .filter(|item| item.parent == folder && item.title == new_title && item.id != id)
                .map(|item| item.id.clone())
                .collect();
            if !existing.is_empty() && !overwrite {
                return Err(exists());
            }
            state.calls.push(DriveCall::Move {
                title: new_title.clone(),
                overwrite,
            });
            let item = state.items.get_mut(id).expect("moved item");
            folder.clone_into(&mut item.parent);
            item.title = new_title;
            if overwrite {
                // GVfs deletes the item it overwrote, for good.
                for gone in existing {
                    state.items.remove(&gone);
                    state.calls.push(DriveCall::Delete(gone));
                }
            }
            Ok(())
        }
    }

    impl FileImpl for DriveFile {
        fn dup(&self) -> gio::File {
            self.file_at(self.segments())
        }

        fn hash(&self) -> u32 {
            self.location().bytes().fold(5381_u32, |hash, byte| {
                hash.wrapping_mul(33).wrapping_add(u32::from(byte))
            })
        }

        fn equal(&self, other: &gio::File) -> bool {
            other.uri() == self.location()
        }

        fn is_native(&self) -> bool {
            false
        }

        fn has_uri_scheme(&self, scheme: &str) -> bool {
            scheme.eq_ignore_ascii_case("google-drive")
        }

        fn uri_scheme(&self) -> Option<String> {
            Some("google-drive".to_owned())
        }

        fn basename(&self) -> Option<PathBuf> {
            match self.segments().last() {
                Some(last) => Some(PathBuf::from(OsStr::from_bytes(last.as_bytes()))),
                None => Some(PathBuf::from("/")),
            }
        }

        fn path(&self) -> Option<PathBuf> {
            None
        }

        fn uri(&self) -> String {
            self.location()
        }

        fn parse_name(&self) -> String {
            self.location()
        }

        fn parent(&self) -> Option<gio::File> {
            let (parent, _) = self.split()?;
            Some(self.file_at(&parent))
        }

        fn has_prefix(&self, prefix: &gio::File) -> bool {
            let prefix = format!("{}/", prefix.uri().trim_end_matches('/'));
            self.location().starts_with(&prefix)
        }

        fn resolve_relative_path(&self, relative_path: impl AsRef<Path>) -> gio::File {
            let mut segments = self.segments().to_vec();
            segments.push(relative_path.as_ref().to_string_lossy().into_owned());
            self.file_at(&segments)
        }

        fn query_info(
            &self,
            _attributes: &str,
            flags: gio::FileQueryInfoFlags,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<gio::FileInfo, glib::Error> {
            let state = self.drive().state.lock().expect("drive");
            let resolved = self.resolve(&state)?;
            Ok(self.info_of(&state, &resolved.id, resolved.volatile, flags))
        }

        fn query_filesystem_info(
            &self,
            _attributes: &str,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<gio::FileInfo, glib::Error> {
            let drive = self.drive();
            let state = drive.state.lock().expect("drive");
            if state.offline {
                return Err(offline());
            }
            let info = gio::FileInfo::new();
            info.set_attribute_boolean("filesystem::readonly", false);
            info.set_attribute_boolean("filesystem::remote", true);
            info.set_attribute_string("filesystem::type", "google-drive");
            if let Some(quota) = state.quota {
                info.set_attribute_uint64("filesystem::size", quota);
                info.set_attribute_uint64("filesystem::free", quota.saturating_sub(state.used(drive)));
            }
            Ok(info)
        }

        fn enumerate_children(
            &self,
            _attributes: &str,
            flags: gio::FileQueryInfoFlags,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<gio::FileEnumerator, glib::Error> {
            let state = self.drive().state.lock().expect("drive");
            let resolved = self.resolve(&state)?;
            let is_folder = resolved.id == ROOT_ID
                || state
                    .items
                    .get(&resolved.id)
                    .is_some_and(|item| item.kind == ItemKind::Folder);
            if !is_folder {
                return Err(error(
                    gio::IOErrorEnum::NotDirectory,
                    "The file is not a directory",
                ));
            }
            let mut children: Vec<&Item> = state
                .items
                .values()
                .filter(|item| item.parent == resolved.id)
                .collect();
            children.sort_by(|a, b| a.id.cmp(&b.id));
            let infos = children
                .iter()
                .map(|item| self.info_of(&state, &item.id, false, flags))
                .collect();
            Ok(super::DriveListing::new(&self.obj(), infos).upcast())
        }

        fn set_display_name(
            &self,
            name: &str,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<gio::File, glib::Error> {
            let drive = self.drive();
            let mut state = drive.state.lock().expect("drive");
            let resolved = self.resolve(&state)?;
            if is_fixed(&state, &resolved.id) {
                return Err(not_supported());
            }
            // GVfs sets the title as asked: Drive keeps both items when
            // another one has it.
            let item = state.items.get_mut(&resolved.id).expect("resolved item");
            let from = std::mem::replace(&mut item.title, name.to_owned());
            state.calls.push(DriveCall::Rename {
                from,
                to: name.to_owned(),
            });
            let mut id_path = resolved.id_path;
            drop(state);
            if let Some(last) = id_path.last_mut() {
                *last = resolved.id;
            }
            Ok(self.file_at(&id_path))
        }

        fn delete(&self, _cancellable: Option<&gio::Cancellable>) -> Result<(), glib::Error> {
            let drive = self.drive();
            let mut state = drive.state.lock().expect("drive");
            let resolved = self.resolve(&state)?;
            if state.items.values().any(|item| item.parent == resolved.id) {
                return Err(error(gio::IOErrorEnum::NotEmpty, "Directory not empty"));
            }
            if is_fixed(&state, &resolved.id) {
                return Err(not_supported());
            }
            state.items.remove(&resolved.id);
            state.calls.push(DriveCall::Delete(resolved.id.clone()));
            drop(state);
            let _ = std::fs::remove_file(drive.blob(&resolved.id));
            Ok(())
        }

        fn make_directory(&self, _cancellable: Option<&gio::Cancellable>) -> Result<(), glib::Error> {
            let drive = self.drive();
            let mut state = drive.state.lock().expect("drive");
            let (folder, title) = self.resolve_parent(&state)?;
            if takes_no_items(&folder) {
                return Err(not_supported());
            }
            if state.child_named(&folder, &title).is_some() {
                return Err(exists());
            }
            new_item(&mut state, &folder, &title, ItemKind::Folder);
            state.calls.push(DriveCall::MakeDirectory(title));
            Ok(())
        }

        fn read_fn(
            &self,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<gio::FileInputStream, glib::Error> {
            let drive = self.drive();
            let state = drive.state.lock().expect("drive");
            let resolved = self.resolve(&state)?;
            match state.items.get(&resolved.id).map(|item| &item.kind) {
                Some(ItemKind::File) => {}
                Some(ItemKind::Folder) | None => {
                    return Err(error(gio::IOErrorEnum::IsDirectory, "Can’t open directory"))
                }
                Some(ItemKind::Document) => {
                    return Err(error(
                        gio::IOErrorEnum::NotRegularFile,
                        "File is not a regular file",
                    ))
                }
            }
            gio::File::for_path(drive.blob(&resolved.id)).read(gio::Cancellable::NONE)
        }

        fn create(
            &self,
            _flags: gio::FileCreateFlags,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<gio::FileOutputStream, glib::Error> {
            let drive = self.drive();
            let mut state = drive.state.lock().expect("drive");
            let (folder, title) = self.resolve_parent(&state)?;
            if takes_no_items(&folder) {
                return Err(not_supported());
            }
            if state.child_named(&folder, &title).is_some() {
                return Err(exists());
            }
            // GVfs adds an empty document first and fills it as it is
            // written.
            let id = new_item(&mut state, &folder, &title, ItemKind::File);
            state.calls.push(DriveCall::Create(title));
            drop(state);
            gio::File::for_path(drive.blob(&id)).replace(
                None,
                false,
                gio::FileCreateFlags::NONE,
                gio::Cancellable::NONE,
            )
        }

        fn copy(
            source: &gio::File,
            destination: &gio::File,
            flags: gio::FileCopyFlags,
            cancellable: Option<&gio::Cancellable>,
            progress_callback: Option<&mut dyn FnMut(i64, i64)>,
        ) -> Result<(), glib::Error> {
            let Some(target) = destination.downcast_ref::<super::DriveFile>() else {
                // Copies off the drive: GVfs has no pull, so GIO reads the
                // file itself.
                return Err(not_supported());
            };
            let target = target.imp();
            if let Some(from) = source.downcast_ref::<super::DriveFile>() {
                if Arc::ptr_eq(from.imp().drive(), target.drive()) {
                    return target.server_copy(from.imp(), flags);
                }
            }
            target.upload(source, flags, cancellable, progress_callback)
        }

        fn move_(
            source: &gio::File,
            destination: &gio::File,
            flags: gio::FileCopyFlags,
            _cancellable: Option<&gio::Cancellable>,
            _progress_callback: Option<&mut dyn FnMut(i64, i64)>,
        ) -> Result<(), glib::Error> {
            let (Some(from), Some(to)) = (
                source.downcast_ref::<super::DriveFile>(),
                destination.downcast_ref::<super::DriveFile>(),
            ) else {
                return Err(not_supported());
            };
            if !Arc::ptr_eq(from.imp().drive(), to.imp().drive()) {
                return Err(not_supported());
            }
            let (id, folder, title) = {
                let state = from.imp().drive().state.lock().expect("drive");
                let resolved = from.imp().resolve(&state)?;
                let (folder, title) = to.imp().resolve_parent(&state)?;
                (resolved.id, folder, title)
            };
            to.imp().move_to(
                &id,
                &folder,
                &title,
                flags.contains(gio::FileCopyFlags::OVERWRITE),
            )
        }
    }

    impl DriveFile {
        /// Google's own copy within the drive (`g_vfs_backend_google_copy`).
        fn server_copy(&self, source: &DriveFile, flags: gio::FileCopyFlags) -> Result<(), glib::Error> {
            let drive = self.drive();
            let mut state = drive.state.lock().expect("drive");
            let from = source.resolve(&state)?;
            let (folder, title) = self.resolve_parent(&state)?;
            if takes_no_items(&folder) {
                return Err(not_supported());
            }
            if state.titled(&folder, &title) > 0 {
                return Err(if flags.contains(gio::FileCopyFlags::OVERWRITE) {
                    error(gio::IOErrorEnum::Failed, "Operation not supported")
                } else {
                    exists()
                });
            }
            let kind = state.items[&from.id].kind.clone();
            if kind == ItemKind::Folder {
                return Err(error(
                    gio::IOErrorEnum::WouldRecurse,
                    "Can’t recursively copy directory",
                ));
            }
            let id = new_item(&mut state, &folder, &title, kind.clone());
            state.calls.push(DriveCall::ServerCopy(title));
            drop(state);
            if kind == ItemKind::File {
                std::fs::copy(drive.blob(&from.id), drive.blob(&id)).map_err(|_| not_found())?;
            }
            Ok(())
        }

        /// An upload into the drive (`g_vfs_backend_google_push`, or GIO's
        /// write fallback): the file appears at once and fills as the
        /// bytes arrive.
        fn upload(
            &self,
            source: &gio::File,
            flags: gio::FileCopyFlags,
            cancellable: Option<&gio::Cancellable>,
            mut progress: Option<&mut dyn FnMut(i64, i64)>,
        ) -> Result<(), glib::Error> {
            let drive = self.drive();
            let source_info = source.query_info(
                "standard::type,standard::size",
                gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS,
                cancellable,
            )?;
            match source_info.file_type() {
                gio::FileType::Directory => {
                    return Err(error(
                        gio::IOErrorEnum::WouldRecurse,
                        "Can’t recursively copy directory",
                    ))
                }
                gio::FileType::Regular => {}
                _ => return Err(error(gio::IOErrorEnum::NotRegularFile, "Not a regular file")),
            }
            let total = u64::try_from(source_info.size()).unwrap_or(0);
            let (id, fault, chunk) = {
                let mut state = drive.state.lock().expect("drive");
                let (folder, title) = self.resolve_parent(&state)?;
                if takes_no_items(&folder) {
                    return Err(not_supported());
                }
                if state.titled(&folder, &title) > 0 {
                    return Err(if flags.contains(gio::FileCopyFlags::OVERWRITE) {
                        error(gio::IOErrorEnum::Failed, "Operation not supported")
                    } else {
                        exists()
                    });
                }
                // Google refuses an upload that would pass the quota with
                // 403, which GVfs reports as a denied permission.
                if let Some(quota) = state.quota {
                    if state.used(drive) + total > quota {
                        return Err(error(gio::IOErrorEnum::PermissionDenied, "Permission denied"));
                    }
                }
                let id = new_item(&mut state, &folder, &title, ItemKind::File);
                state.calls.push(DriveCall::Upload(title));
                (id, state.upload_fault, state.upload_chunk)
            };
            let mut input =
                std::fs::File::open(source.path().ok_or_else(not_supported)?).map_err(|_| not_found())?;
            let mut output = std::fs::File::create(drive.blob(&id)).map_err(|_| not_found())?;
            let mut written: u64 = 0;
            let mut buffer = vec![0_u8; chunk];
            loop {
                if cancellable.is_some_and(gio::Cancellable::is_cancelled) {
                    // The partial file stays, under the title it was given.
                    return Err(error(gio::IOErrorEnum::Cancelled, "Operation was cancelled"));
                }
                if drive.state.lock().expect("drive").offline {
                    return Err(offline());
                }
                if let UploadFault::DropAfter(limit) = fault {
                    if written >= limit {
                        return Err(offline());
                    }
                }
                let read = input.read(&mut buffer).map_err(|_| not_found())?;
                if read == 0 {
                    break;
                }
                let keep = match fault {
                    UploadFault::Truncate(limit) => usize::try_from(limit.saturating_sub(written))
                        .unwrap_or(usize::MAX)
                        .min(read),
                    _ => read,
                };
                output.write_all(&buffer[..keep]).map_err(|_| not_found())?;
                written += read as u64;
                if let Some(progress) = progress.as_deref_mut() {
                    progress(
                        i64::try_from(written).unwrap_or(i64::MAX),
                        i64::try_from(total).unwrap_or(i64::MAX),
                    );
                }
            }
            Ok(())
        }
    }

    /// The state behind a [`super::DriveListing`].
    #[derive(Debug, Default)]
    pub struct DriveListing {
        pub(super) remaining: Mutex<Vec<gio::FileInfo>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DriveListing {
        const NAME: &'static str = "OxTestDriveListing";
        type Type = super::DriveListing;
        type ParentType = gio::FileEnumerator;
    }

    impl ObjectImpl for DriveListing {}

    impl FileEnumeratorImpl for DriveListing {
        fn next_file(
            &self,
            _cancellable: Option<&gio::Cancellable>,
        ) -> Result<Option<gio::FileInfo>, glib::Error> {
            Ok(self.remaining.lock().expect("listing").pop())
        }

        fn close(&self, _cancellable: Option<&gio::Cancellable>) -> (bool, Option<glib::Error>) {
            (true, None)
        }
    }
}

/// `name` escaped for use as one path segment of a URI.
fn escape(name: &str) -> String {
    utf8_percent_encode(name, SEGMENT_ESCAPES).to_string()
}
