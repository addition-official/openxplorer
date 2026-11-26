// SPDX-License-Identifier: AGPL-3.0-only
//! Writing to Google Drive through `GVfs` (DEV-013).
//!
//! Every write to a `google-drive:` location first asks
//! [`crate::location::cloud_writes`]: with Settings' "Allow changes to
//! Google Drive" off, which is the default, nothing on the drive is
//! created, moved, renamed or deleted, whatever asked for it.
//!
//! With the switch on, the quirks of `GVfs`'s Google backend
//! (`gvfsbackendgoogle.c`) are handled here:
//!
//! - An address names each item by its Drive ID; a name in an address is
//!   looked up among the titles of the folder's items, and the item found
//!   that way is reported as a link to its ID ("volatile"). Drive has no
//!   real links, so queries follow them: a folder created by its title is
//!   a folder, not a link.
//! - Two items in one folder may share a title. A copy, a move or a new
//!   item never makes a second one: a taken title counts as a taken name,
//!   as on a local disk, and the usual Skip, Keep both and Replace
//!   question follows. Replace is never done by `GVfs`'s overwriting
//!   move, which deletes the old item permanently once the new one is
//!   there; the engine sets the old item aside under a private name and
//!   removes it only once the new one is in place, as it does on phones.
//!   A Replace onto a title two items share is refused, since `GVfs`
//!   would pick one of them.
//! - Copies are uploaded under a private title beside the final one and
//!   given their final title only once complete, by a rename within the
//!   folder, which every `GVfs` version can do. The upload's size is
//!   checked against the source's before that, so an interrupted upload
//!   is never shown as a complete file.
//! - Older `GVfs` versions cannot move an item to another folder; such a
//!   move is finished by copying, after the user agreed, as across drives.
//! - `GVfs` has no Trash for Google Drive (`access::can-trash` is false),
//!   and its delete removes the item for good, not into Drive's trash.
//!   Delete therefore asks Windows' "Are you sure you want to permanently
//!   delete…" first, as for any location without a Recycle Bin.
//! - Docs, Sheets and Slides files are web pages, not bytes: `GVfs` lists
//!   them as shortcuts. Copying one is refused with a message that says
//!   so; nothing, not even an empty file, is written.
//! - Titles may hold "/": a copy off the drive takes `GVfs`'s copy name,
//!   with "/" made "-".
//! - A full drive and a lost connection are reported in Windows' words.
//!   `GVfs` turns Google's "storage quota exceeded" into "Permission
//!   denied", so a refused upload is checked against the drive's free
//!   space.

use gio::prelude::*;

use super::{gio_cancellable, GioNode};
use crate::location::cloud_writes::{cloud_write_refusal, is_google_drive_location};
use crate::system_error::describe;
use crate::transfer::{check_cancelled, Cancellation, Node, TransferError};

impl GioNode {
    /// True for an item on Google Drive.
    pub(super) fn is_google_drive(&self) -> bool {
        self.file.has_uri_scheme("google-drive")
    }

    /// The flags of a query about this item: on Google Drive, the "link"
    /// `GVfs` reports for an item addressed by its title is followed to
    /// the item, since Drive has no real links.
    pub(super) fn query_flags(&self) -> gio::FileQueryInfoFlags {
        if self.is_google_drive() {
            gio::FileQueryInfoFlags::NONE
        } else {
            gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS
        }
    }

    /// Refuses to change this item, or to create it, while Google Drive
    /// is read-only (DEV-013). `Failed`, not a denied permission, so no
    /// administrator is offered.
    pub(super) fn refuse_cloud_change(&self) -> Result<(), TransferError> {
        match cloud_write_refusal(&self.uri()) {
            Some(message) => Err(TransferError::Failed(message)),
            None => Ok(()),
        }
    }

    /// Refuses a move or rename of this item to `target` while either
    /// side is a read-only drive (DEV-013).
    pub(super) fn refuse_cloud_move(&self, target: &dyn Node) -> Result<(), TransferError> {
        self.refuse_cloud_change()?;
        GioNode::new(&target.uri()).refuse_cloud_change()
    }

    /// Refuses a permanent delete of this item while its drive is
    /// read-only (DEV-013).
    pub(super) fn refuse_cloud_delete(&self) -> Result<(), TransferError> {
        self.refuse_cloud_change()
    }

    /// A failed write in this item's words: Google Drive's in Windows'
    /// words, the usual mapping elsewhere.
    pub(super) fn write_error(&self, error: glib::Error) -> TransferError {
        if self.is_google_drive() {
            drive_error(error)
        } else {
            error.into()
        }
    }

    /// Replace onto Google Drive is never `GVfs`'s overwriting move, which
    /// deletes the old item permanently: the engine keeps it under a
    /// private title until the new one is in place (XFER-010). A title two
    /// items share is refused, as `GVfs` would pick one of them.
    pub(super) fn refuse_drive_overwrite(
        target: &dyn Node,
        cancel: Option<&Cancellation>,
    ) -> Result<(), TransferError> {
        let destination = GioNode::new(&target.uri());
        if destination.is_google_drive() {
            if let Some(title) = destination.drive_title(cancel) {
                if destination.items_titled(&title, cancel)? > 1 {
                    return Err(TransferError::Failed(crate::i18n::format_message(
                        "More than one item in this Google Drive folder is named “{name}”, so it is not clear \
                         which one to replace. Rename them on Google Drive, then try again. Nothing was changed.",
                        &[("name", &title)],
                    )));
                }
            }
        }
        Err(TransferError::ReplaceUnsupported(crate::i18n::gettext(
            "The backend does not support direct overwrite.",
        )))
    }

    /// True for a Docs, Sheets or Slides file, which `GVfs` lists as a
    /// shortcut to its web page.
    pub(super) fn is_google_document(&self) -> bool {
        if !self.is_google_drive() {
            return false;
        }
        self.file
            .query_info(
                "standard::type",
                gio::FileQueryInfoFlags::NONE,
                gio::Cancellable::NONE,
            )
            .is_ok_and(|info| info.file_type() == gio::FileType::Shortcut)
    }

    /// The refusal of a copy of a Docs, Sheets or Slides file.
    pub(super) fn google_document_refusal(&self) -> TransferError {
        TransferError::Failed(crate::i18n::format_message(
            "“{name}” is a Google Docs, Sheets or Slides file. It exists only online, so it cannot be copied \
             as a file. Open it in the browser and use File > Download to save a copy.",
            &[("name", &self.display_name())],
        ))
    }

    /// The title of this item on Google Drive, as `GVfs` reports it.
    fn drive_title(&self, cancel: Option<&Cancellation>) -> Option<String> {
        self.file
            .query_info(
                "standard::display-name",
                gio::FileQueryInfoFlags::NONE,
                gio_cancellable(cancel),
            )
            .ok()
            .map(|info| info.display_name().to_string())
    }

    /// How many items of this item's folder have `title`.
    pub(super) fn items_titled(
        &self,
        title: &str,
        cancel: Option<&Cancellation>,
    ) -> Result<usize, TransferError> {
        let Some(folder) = self.file.parent() else {
            return Ok(0);
        };
        let listing = folder.enumerate_children(
            "standard::display-name",
            gio::FileQueryInfoFlags::NONE,
            gio_cancellable(cancel),
        )?;
        let mut count = 0;
        loop {
            check_cancelled(cancel)?;
            let Some(info) = listing.next_file(gio_cancellable(cancel))? else {
                break;
            };
            if info.display_name() == title {
                count += 1;
            }
        }
        let _ = listing.close(gio::Cancellable::NONE);
        Ok(count)
    }

    /// The new title this item's address asks for: the last part of the
    /// address, unless it names an item by ID.
    pub(super) fn requested_title(&self) -> Option<String> {
        let name = self.file.basename()?;
        Some(name.to_string_lossy().into_owned())
    }

    /// Publishes this completed upload on Google Drive under `target`'s
    /// title, by a rename within the folder: `GVfs` changes only the
    /// title, which every version can do, and never overwrites. A taken
    /// title is refused first, as Drive itself would keep both.
    pub(super) fn publish_on_drive(
        &self,
        target: &dyn Node,
        cancel: Option<&Cancellation>,
    ) -> Result<(), TransferError> {
        check_cancelled(cancel)?;
        let target_file = gio::File::for_uri(&target.uri());
        let same_folder = match (self.file.parent(), target_file.parent()) {
            (Some(first), Some(second)) => first.equal(&second),
            _ => false,
        };
        if !same_folder {
            return self.move_item(target, cancel, super::move_item::Overwrite::Never);
        }
        let Some(title) = GioNode::from_file(target_file).requested_title() else {
            return Err(TransferError::failed(crate::i18n::gettext("Enter a name.")));
        };
        self.retitle(&title, cancel)
    }

    /// Gives this item the title `title` within its folder, refusing a
    /// title another item already has (nothing is overwritten).
    pub(super) fn retitle(&self, title: &str, cancel: Option<&Cancellation>) -> Result<(), TransferError> {
        self.refuse_cloud_change()?;
        check_cancelled(cancel)?;
        if self.items_titled(title, cancel)? > 0 {
            return Err(TransferError::Exists(crate::i18n::format_message(
                "An item named “{name}” already exists. Nothing was overwritten.",
                &[("name", title)],
            )));
        }
        self.file
            .set_display_name(title, gio_cancellable(cancel))
            .map_err(drive_error)?;
        Ok(())
    }

    /// Checks that the upload at `target` holds every byte of this
    /// source, so an interrupted upload never becomes a complete-looking
    /// file. The staging is removed by the engine on the error.
    pub(super) fn verify_drive_upload(
        &self,
        target: &gio::File,
        cancel: &Cancellation,
    ) -> Result<(), TransferError> {
        let attributes = "standard::type,standard::size";
        let source = self
            .file
            .query_info(attributes, self.query_flags(), Some(cancel.cancellable()))?;
        if source.file_type() != gio::FileType::Regular {
            return Ok(());
        }
        let uploaded = target.query_info(
            attributes,
            gio::FileQueryInfoFlags::NONE,
            Some(cancel.cancellable()),
        )?;
        if uploaded.size() == source.size() {
            return Ok(());
        }
        Err(TransferError::Failed(crate::i18n::format_message(
            "The copy of “{name}” on Google Drive is incomplete, so it was not kept. Try again.",
            &[("name", &self.display_name())],
        )))
    }

    /// The free bytes Google Drive reports for the folder of `target`.
    fn drive_free_space(target: &gio::File, cancel: &Cancellation) -> Option<u64> {
        let folder = target.parent()?;
        let info = folder
            .query_filesystem_info("filesystem::free", Some(cancel.cancellable()))
            .ok()?;
        info.has_attribute("filesystem::free")
            .then(|| info.attribute_uint64("filesystem::free"))
    }

    /// A failed upload to `target` on Google Drive in Windows' words: a
    /// full drive, which `GVfs` reports as a denied permission, or a lost
    /// connection.
    pub(super) fn upload_error(
        &self,
        error: glib::Error,
        target: &gio::File,
        cancel: &Cancellation,
    ) -> TransferError {
        let denied = error.matches(gio::IOErrorEnum::PermissionDenied);
        if denied && !cancel.is_cancelled() {
            let size = self
                .file
                .query_info("standard::size", self.query_flags(), gio::Cancellable::NONE)
                .map_or(0, |info| u64::try_from(info.size()).unwrap_or(0));
            if Self::drive_free_space(target, cancel).is_some_and(|free| free < size) {
                return TransferError::NoSpace(drive_full_message());
            }
        }
        drive_error(error)
    }
}

/// Renames the item at `uri` on Google Drive to `title`, which is its
/// new title: `GVfs` changes the title and the item keeps its ID, so its
/// address stays. Returns the item's address afterwards. A title another
/// item of the folder has is refused (nothing is overwritten), as on a
/// local disk; Drive itself would keep both.
///
/// # Errors
///
/// A refusal while Google Drive is read-only, [`TransferError::Exists`]
/// for a taken title, cancellation, and the backend's error in Windows'
/// words where it has some.
pub fn rename_on_google_drive(
    uri: &str,
    title: &str,
    cancel: &Cancellation,
) -> Result<String, TransferError> {
    let item = GioNode::new(uri);
    item.refuse_cloud_change()?;
    cancel.check()?;
    if item.drive_title(Some(cancel)).as_deref() == Some(title) {
        return Ok(uri.to_owned());
    }
    if item.items_titled(title, Some(cancel))? > 0 {
        return Err(TransferError::Exists(crate::i18n::format_message(
            "An item named “{name}” already exists. Nothing was overwritten.",
            &[("name", title)],
        )));
    }
    let renamed = item
        .file
        .set_display_name(title, Some(cancel.cancellable()))
        .map_err(drive_error)?;
    Ok(renamed.uri().to_string())
}

/// The title of the item at `uri` on Google Drive, as `GVfs` reports it.
pub fn google_drive_title(uri: &str, cancel: &Cancellation) -> Option<String> {
    GioNode::new(uri).drive_title(Some(cancel))
}

/// The address by ID of the item at `uri` on Google Drive, which may name
/// it by its title, as an item just created by its title is; `uri` itself
/// when the backend does not say.
pub fn google_drive_id_uri(uri: &str) -> String {
    let file = gio::File::for_uri(uri);
    // GVfs reports an item reached by its title under that title as its
    // name; `id::file` is its ID either way.
    let (Some(folder), Ok(info)) = (
        file.parent(),
        file.query_info("id::file", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE),
    ) else {
        return uri.to_owned();
    };
    match info.attribute_string("id::file") {
        Some(id) if !id.is_empty() && !id.contains('/') => folder.child(id.as_str()).uri().to_string(),
        _ => uri.to_owned(),
    }
}

/// True when `uri` or `other` is on Google Drive.
pub(super) fn touches_google_drive(uri: &str, other: &str) -> bool {
    is_google_drive_location(uri) || is_google_drive_location(other)
}

/// Windows' words for a full drive, for Google Drive.
pub(crate) fn drive_full_message() -> String {
    crate::i18n::gettext(
        "There is not enough space on Google Drive. Delete or move files so you have enough space, then try again.",
    )
}

/// A Google Drive error in Windows' words where it has some: a full
/// drive, and a connection that is lost or never made, which `GVfs`
/// passes on as the network library's message. Others map as usual.
pub(crate) fn drive_error(error: glib::Error) -> TransferError {
    if is_quota_error(&error) {
        return TransferError::NoSpace(drive_full_message());
    }
    if is_offline_error(&error) {
        let unreachable = glib::Error::new(gio::IOErrorEnum::HostUnreachable, "");
        return TransferError::Failed(describe(&unreachable));
    }
    error.into()
}

/// True for a Google Drive error that Windows words its own way: a full
/// drive or a lost connection.
pub(super) fn has_windows_words(error: &glib::Error) -> bool {
    is_quota_error(error) || is_offline_error(error)
}

/// True for a full drive: `NO_SPACE`, or Google's own quota message.
fn is_quota_error(error: &glib::Error) -> bool {
    if error.matches(gio::IOErrorEnum::NoSpace) {
        return true;
    }
    let message = error.message().to_ascii_lowercase();
    message.contains("quota") && (message.contains("exceed") || message.contains("full"))
}

/// True for a connection to Google that is lost or never made.
fn is_offline_error(error: &glib::Error) -> bool {
    let offline_code = matches!(
        error.kind::<gio::IOErrorEnum>(),
        Some(
            gio::IOErrorEnum::HostUnreachable
                | gio::IOErrorEnum::NetworkUnreachable
                | gio::IOErrorEnum::ConnectionRefused
                | gio::IOErrorEnum::TimedOut
                | gio::IOErrorEnum::HostNotFound
                | gio::IOErrorEnum::BrokenPipe
        )
    );
    if offline_code {
        return true;
    }
    if error.kind::<gio::ResolverError>().is_some() {
        return true;
    }
    // GVfs passes libsoup's and the resolver's errors on as plain
    // failures, with their message.
    let message = error.message().to_ascii_lowercase();
    [
        "error resolving",
        "name resolution",
        "could not connect",
        "cannot connect",
        "network is unreachable",
        "no route to host",
        "connection refused",
        "connection timed out",
        "socket i/o timed out",
    ]
    .iter()
    .any(|text| message.contains(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A full drive and a lost connection read as Windows says them,
    /// whichever way `GVfs` passed them on.
    ///
    /// parity: DEV-013
    #[test]
    fn quota_and_offline_errors_take_windows_words() {
        let full = glib::Error::new(gio::IOErrorEnum::NoSpace, "No space left");
        assert_eq!(drive_error(full), TransferError::NoSpace(drive_full_message()));
        let quota = glib::Error::new(
            gio::IOErrorEnum::Failed,
            "The user's Drive storage quota has been exceeded.",
        );
        assert_eq!(drive_error(quota), TransferError::NoSpace(drive_full_message()));
        let unreachable = describe(&glib::Error::new(gio::IOErrorEnum::HostUnreachable, ""));
        assert_eq!(unreachable, "The network location cannot be reached.");
        for offline in [
            glib::Error::new(
                gio::IOErrorEnum::Failed,
                "Error resolving “www.googleapis.com”: Temporary failure in name resolution",
            ),
            glib::Error::new(gio::IOErrorEnum::NetworkUnreachable, "Network is unreachable"),
            glib::Error::new(gio::IOErrorEnum::TimedOut, "Socket I/O timed out"),
            glib::Error::new(gio::ResolverError::NotFound, "Name or service not known"),
        ] {
            assert_eq!(drive_error(offline), TransferError::Failed(unreachable.clone()));
        }
        let other = glib::Error::new(gio::IOErrorEnum::NotFound, "No such file or directory");
        assert!(drive_error(other).is_not_found());
    }
}
