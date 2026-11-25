// SPDX-License-Identifier: AGPL-3.0-only
//! The previous-versions service every window shares, and the write
//! protection it gives file operations, which also keeps Google Drive and
//! `OneDrive` read-only for now (DEV-013).
//!
//! The Python app kept one `PreviousVersions` service for the whole
//! application (`v2.0.0:desktop/winspace.py`): the snapshot collections a lookup
//! finds in one window are read-only in every window for the rest of the
//! session (PROP-024), and every write asks it first (XFER-020).

use std::sync::Arc;

use gtk::subclass::prelude::*;
use ox_core::ops::WriteProtection;
use ox_core::transfer::TransferError;
use ox_core::versions::PreviousVersions;

use super::AppContext;

impl AppContext {
    /// The previous-versions service.
    pub(crate) fn previous_versions(&self) -> &Arc<PreviousVersions> {
        self.imp()
            .previous_versions
            .get()
            .expect("AppContext::new creates the previous-versions service")
    }

    /// The protection every file operation runs with: previous versions
    /// (snapshots and backups) are never changed in place
    /// (`PreviousVersions.assert_writable` in `v2.0.0:desktop/previous_versions.py`).
    pub(crate) fn write_protection(&self) -> WriteProtection {
        WriteProtection::new(self.write_guard())
    }

    /// The write guard of every file and archive operation: previous
    /// versions, and Google Drive and `OneDrive` (DEV-013). `OneDrive` is
    /// read-only for now, and so is Google Drive unless Settings > Files &
    /// folders > "Allow changes to Google Drive" is on; its top folders
    /// (My Drive itself, Shared with me, the shared drives) never change.
    /// Copying items off a drive changes nothing there, so it is never
    /// asked about them; moving, renaming or deleting them, or writing
    /// into a drive, is refused while the drive is read-only.
    pub(crate) fn write_guard(&self) -> impl Fn(&str) -> Result<(), TransferError> + Send + Sync + 'static {
        let previous_versions = self.previous_versions().write_guard();
        move |uri: &str| {
            // The guard is asked about folders written into as well as
            // items changed, so it refuses only what no write may touch;
            // the GIO adapter itself refuses changing My Drive and the
            // items Shared with me lists.
            if let Some(refusal) = ox_core::location::cloud_writes::cloud_folder_write_refusal(uri) {
                return Err(TransferError::failed(refusal));
            }
            previous_versions(uri)
        }
    }
}
