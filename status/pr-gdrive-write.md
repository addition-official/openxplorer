## Google Drive: allow changes as an opt-in setting

Google Drive stays read-only by default, as it is today. A new switch, Settings > Files & folders > Google Drive > "Allow changes to Google Drive", lets copies, moves, renames, deletes, New, Paste and drag and drop reach Google Drive through GVfs. OneDrive stays read-only either way. The switch is not part of "Behave like Windows Explorer", since Windows has no Google Drive.

### Setting
- New preference `allowGoogleDriveChanges`, off by default and stored only when on.
- Description in Settings: "Copies, moves, renames and deletes in Google Drive then change the files on your Drive, and New and Paste add items there. Deleting there removes items for good: they do not go to Drive's trash. Docs, Sheets and Slides files cannot be copied as files. Off: Google Drive is read-only, and its files can be opened and copied to this PC."
- The switch is one process-wide fact (`ox_core::location::cloud_writes`) read by the command rules, drop targets, the shared write guard and the GIO adapter. With it off, the adapter itself refuses every write to the drive, not only the greyed-out commands.
- The drive's top folders never take changes: the account's top folder, Shared with me, the list of shared drives, and My Drive itself as an item.

### GVfs google backend quirks handled (`gio_node/google_drive.rs`)
- IDs and titles: GVfs names items by Drive ID and resolves a title in an address, reporting such an item as a link. Queries on Google Drive follow these links, since Drive has no real links. New items, renamed items and landed copies are recorded by their ID address.
- Same titles: a title another item already has counts as a taken name, so the usual Skip / Keep both / Replace question applies and the app never creates a second item with the same title. Replace onto a title that two items share is refused, since GVfs would pick one of them.
- Replace never uses GVfs's overwriting move, which deletes the old item permanently first. The old item is set aside under a private title and removed only after the new one is in place.
- Uploads are staged under a private title beside the final one and published by a rename within the folder, which every GVfs version supports. The uploaded size is checked first, so an interrupted or short upload is never shown as a complete file, and the partial staging is removed.
- Moves between folders that older GVfs versions cannot do are finished by copying, only after the user agrees, as across drives.
- Trash: GVfs has no Trash for Google Drive (`access::can-trash` is false), and its delete is permanent; it does not use Drive's trash. Delete therefore asks Windows' "Are you sure you want to permanently delete this file?" first, as for any location without a Recycle Bin, and Trash never falls back to a silent delete.
- Docs, Sheets and Slides are refused with: "“{name}” is a Google Docs, Sheets or Slides file. It exists only online, so it cannot be copied as a file. Open it in the browser and use File > Download to save a copy." Nothing is written for them, not even an empty file. Moves within the drive still work.
- Titles with "/" are copied off the drive under GVfs's copy name, with "/" made "-".
- A full drive reads "There is not enough space on Google Drive. Delete or move files so you have enough space, then try again." (GVfs reports Google's quota refusal as "Permission denied", so the free space is checked). A lost connection reads Windows' "The network location cannot be reached."
- Big copies report progress and can be canceled; nothing is left behind.

### Tests
- A simulated GVfs Google backend (`tests/transfer_support/google_drive.rs`) answers behind real `google-drive://` URIs, modeled on `gvfsbackendgoogle.c` of GVfs 1.54, so the production GIO adapter runs unchanged with no account and no network.
- 19 new engine and ops tests against it (copy, folder copy, copy off with same titles and slashes, conflicts, Replace, shared-title Replace, moves within, older GVfs moves, moves in and out, Trash and Delete, Docs, cancel mid-upload, short upload, full drive, offline, switch off, top folders, rename with Undo, New items).
- Unit tests for the switch, the error wording, the preference, the command rules and the write guard.
- Only the touched modules were run; results per test are in the status file.

Model: claude-opus-5-5
CLA: I agree
