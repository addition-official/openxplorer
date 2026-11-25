// SPDX-License-Identifier: AGPL-3.0-only
//! Whether Google Drive may be changed (DEV-013).
//!
//! Google Drive and `OneDrive` are read-only by default: their files are
//! listed, opened and copied off. Settings > Files & folders > "Allow
//! changes to Google Drive" lets copies, moves, renames, deletes and new
//! items reach Google Drive through `GVfs`. The switch is one fact for the
//! whole process, because every window, the write guard of every file
//! operation and the GIO adapter itself must agree on it: with the switch
//! off, nothing writes to the drive even when a command slipped through.
//! `OneDrive` stays read-only either way.
//!
//! The parts of a Google Drive that `GVfs` itself never changes are
//! read-only whatever the switch says: the account's top folder, which
//! lists My Drive, Shared with me and Shared drives, the list of shared
//! drives, and Shared with me with what it lists directly.

use std::sync::atomic::{AtomicBool, Ordering};

use super::parts::split_location;

/// The switch, off until the app reads the user's preference.
static GOOGLE_DRIVE_WRITABLE: AtomicBool = AtomicBool::new(false);

/// The ID `GVfs` gives Shared with me (`SHARED_WITH_ME_ID` in
/// `gvfsbackendgoogle.c`).
const SHARED_WITH_ME_ID: &str = "GVfsSharedWithMe";

/// The ID `GVfs` gives the list of shared drives (`SHARED_DRIVES_ID`).
const SHARED_DRIVES_ID: &str = "GVfsSharedDrives";

/// Turns changes to Google Drive on or off for the whole process.
pub fn set_google_drive_writable(writable: bool) {
    GOOGLE_DRIVE_WRITABLE.store(writable, Ordering::SeqCst);
}

/// Whether changes to Google Drive are on.
pub fn google_drive_writable() -> bool {
    GOOGLE_DRIVE_WRITABLE.load(Ordering::SeqCst)
}

/// True for a `google-drive:` location.
pub fn is_google_drive_location(uri: &str) -> bool {
    split_location(uri).is_ok_and(|parts| parts.scheme == "google-drive")
}

/// True for the folders of a Google Drive that `GVfs` never writes into
/// and whose items it never moves, renames or deletes: the account's top
/// folder, the list of shared drives and Shared with me. Their items are
/// My Drive, each shared drive and the items shared with the account.
fn is_fixed_drive_folder(uri: &str) -> bool {
    let Ok(parts) = split_location(uri) else {
        return false;
    };
    let components: Vec<&str> = parts.path.split('/').filter(|part| !part.is_empty()).collect();
    match components.as_slice() {
        [] => true,
        [first] => *first == SHARED_DRIVES_ID || *first == SHARED_WITH_ME_ID,
        _ => false,
    }
}

/// True where New and Paste may create items on Google Drive: the switch
/// is on and `folder_uri` is an ordinary Drive folder.
pub fn can_write_into_google_drive_folder(folder_uri: &str) -> bool {
    google_drive_writable() && is_google_drive_location(folder_uri) && !is_fixed_drive_folder(folder_uri)
}

/// True when the item at `uri` on Google Drive may be moved, renamed or
/// deleted: the switch is on and the item is not My Drive, Shared with
/// me, a shared drive, or an item shared with the account.
pub fn can_change_google_drive_item(uri: &str) -> bool {
    if !google_drive_writable() || !is_google_drive_location(uri) {
        return false;
    }
    let Ok(parts) = split_location(uri) else {
        return false;
    };
    let components: Vec<&str> = parts.path.split('/').filter(|part| !part.is_empty()).collect();
    match components.as_slice() {
        [] | [_] => false,
        [first, _] => *first != SHARED_DRIVES_ID && *first != SHARED_WITH_ME_ID,
        _ => true,
    }
}

/// The message of a refused write to Google Drive or `OneDrive`, for the
/// write guard; `None` for a location that is neither, or a Google Drive
/// item that may be changed.
///
/// `uri` is an item to change or create. A folder only written into is
/// asked about by [`cloud_folder_write_refusal`].
pub fn cloud_write_refusal(uri: &str) -> Option<String> {
    if !super::is_cloud_location(uri) {
        return None;
    }
    if !is_google_drive_location(uri) {
        return Some(read_only_message());
    }
    if !google_drive_writable() {
        return Some(google_drive_off_message());
    }
    if can_change_google_drive_item(uri) {
        return None;
    }
    Some(fixed_folder_message())
}

/// The message of a refused write into the folder `folder_uri`, as
/// [`cloud_write_refusal`] for its items.
pub fn cloud_folder_write_refusal(folder_uri: &str) -> Option<String> {
    if !super::is_cloud_location(folder_uri) {
        return None;
    }
    if !is_google_drive_location(folder_uri) {
        return Some(read_only_message());
    }
    if !google_drive_writable() {
        return Some(google_drive_off_message());
    }
    if can_write_into_google_drive_folder(folder_uri) {
        return None;
    }
    Some(fixed_folder_message())
}

/// The owner's words for a drive that is read-only (DEV-013).
pub fn read_only_message() -> String {
    crate::i18n::gettext(
        "Google Drive and OneDrive are read-only for now. Copy items to this PC to change them.",
    )
}

/// Why Google Drive refuses a change while the switch is off.
pub fn google_drive_off_message() -> String {
    crate::i18n::gettext(
        "Google Drive is read-only. To change it, turn on Allow changes to Google Drive in Settings > Files & folders.",
    )
}

/// Why the top folders of a Google Drive refuse a change.
fn fixed_folder_message() -> String {
    crate::i18n::gettext(
        "Google Drive does not let this folder be changed here. Open My Drive or a folder in it.",
    )
}

/// Serialises the tests of every test binary that turn the switch, which
/// is one fact for the whole process.
#[doc(hidden)]
pub fn switch_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MY_DRIVE: &str = "google-drive://alice@example.com/0AHomeId";
    const FOLDER: &str = "google-drive://alice@example.com/0AHomeId/1FolderId";
    const FILE: &str = "google-drive://alice@example.com/0AHomeId/1FolderId/1FileId";

    /// Off by default: nothing on Google Drive may be changed, and the
    /// owner's message for `OneDrive` stays.
    ///
    /// parity: DEV-013
    #[test]
    fn google_drive_is_read_only_until_the_switch_is_on() {
        let _lock = switch_lock();
        set_google_drive_writable(false);
        assert!(!can_write_into_google_drive_folder(MY_DRIVE));
        assert!(!can_change_google_drive_item(FILE));
        assert_eq!(cloud_write_refusal(FILE), Some(google_drive_off_message()));
        assert_eq!(
            cloud_folder_write_refusal(FOLDER),
            Some(google_drive_off_message())
        );
        assert_eq!(
            cloud_write_refusal("onedrive://alice@example.com/Documents/a.txt"),
            Some(read_only_message())
        );
        assert_eq!(cloud_write_refusal("file:///tmp/a.txt"), None);
    }

    /// On, ordinary Drive folders take items and their items change;
    /// `OneDrive` stays read-only.
    ///
    /// parity: DEV-013
    #[test]
    fn the_switch_opens_ordinary_drive_folders_only() {
        let _lock = switch_lock();
        set_google_drive_writable(true);
        assert!(can_write_into_google_drive_folder(MY_DRIVE));
        assert!(can_write_into_google_drive_folder(FOLDER));
        assert!(can_change_google_drive_item(FILE));
        assert!(can_change_google_drive_item(FOLDER));
        assert_eq!(cloud_write_refusal(FILE), None);
        assert_eq!(cloud_folder_write_refusal(MY_DRIVE), None);
        assert_eq!(
            cloud_folder_write_refusal("onedrive://alice@example.com/Documents"),
            Some(read_only_message())
        );
        // The account's top folder, the list of shared drives and Shared
        // with me take nothing, and My Drive and a shared drive stay.
        let top = "google-drive://alice@example.com/";
        let shared_drives = "google-drive://alice@example.com/GVfsSharedDrives";
        let shared_with_me = "google-drive://alice@example.com/GVfsSharedWithMe";
        for folder in [top, shared_drives, shared_with_me] {
            assert!(!can_write_into_google_drive_folder(folder), "{folder}");
            assert!(cloud_folder_write_refusal(folder).is_some(), "{folder}");
        }
        assert!(can_write_into_google_drive_folder(
            "google-drive://alice@example.com/GVfsSharedDrives/0ADriveId"
        ));
        for item in [
            MY_DRIVE,
            "google-drive://alice@example.com/GVfsSharedDrives/0ADriveId",
            "google-drive://alice@example.com/GVfsSharedWithMe/1SharedId",
        ] {
            assert!(!can_change_google_drive_item(item), "{item}");
            assert!(cloud_write_refusal(item).is_some(), "{item}");
        }
        set_google_drive_writable(false);
    }
}
