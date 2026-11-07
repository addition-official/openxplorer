Model: claude-opus-5-5

# feat(transfers): Continue as administrator when a copy, move or delete is denied, asked in the progress window

## Summary

- **Destination Folder Access Denied** (copy and move) offers **Continue** with the administrator shield in place of Try Again, beside Skip and Cancel.
- Deleting an item from a folder only an administrator may change asks **File Access Denied** (or Folder Access Denied), "You'll need to provide administrator permission to delete this file" ("folder"), with Continue, Skip and Cancel.
- Every question about a failed item (access denied for copy, move and delete, File In Use, Item Not Found, Interrupted Action) is now asked inside the progress window in place of the job's progress, titled "1 Interrupted Action", the way Replace or Skip Files and "There is not enough space" already are. The window's own More details shows the item's source and destination, and the job shows its progress again once answered. The item's name is plain text and the keyboard starts on the first button. "Do this for all current items" is kept.
- Continue reuses the existing administrator paths and adds no new privilege mechanism. A copy or move runs again into the same folder under `admin://`, the GVfs access "Open as administrator" uses. A delete runs again under `admin://`: a folder goes through the existing `openxplorer-admin-delete` helper, anything else through GVfs.
- Before any password is asked for, the location is checked with the same refusals as the administrator delete work (#192, #194). A folder other accounts can change, or one on a drive users can mount, is refused, and the reason is shown in Interrupted Action. Into `admin://` the app reads the source itself, so GVfs only writes the destination. A move into `admin://` is finished by copying and then removing the original as the user. A Replace merges into an existing folder only if that folder passes the same check. The write guard treats `admin://` locations as their local paths.
- "Do this for all current items" with Continue in a delete run applies administrator rights only to later items the user may not delete; the user's own items are deleted normally.

## Windows 11's behavior, and one difference

- Checked on Windows 11 by pasting a file into `C:\Program Files`: the question reads "Destination Folder Access Denied", "You'll need to provide administrator permission to copy to this folder", shows the folder's icon, name and Date created, and offers Continue (with the shield), Skip, Cancel and More details. Continue asks for administrator permission and then copies.
- The shield on Continue follows Microsoft's UAC guidance to "Display controls with the UAC shield to indicate that the task requires immediate elevation" ([User Account Control, Design basics](https://learn.microsoft.com/en-us/windows/win32/uxguide/winenv-uac); written for Windows 7).
- **Difference:** Windows' administrator Continue still sends a deleted item to the Recycle Bin. GVfs has no administrator Recycle Bin, so in a Recycle Bin run Continue first asks "This file can't be moved to the Recycle Bin. Do you want to permanently delete it?" (Yes/No, asked whatever the confirm-delete setting says), and then **deletes the item permanently**.

## Testing

- rustfmt and Clippy (Rust 1.92.0, the CI toolchain) are clean; `parity/check.py`, the i18n template check, `sync-docs.py --check` and the public-data audit pass.
- Run as an unprivileged user (root ignores permissions): `transfer` (135, including the new `transfer_cases/elevation.rs`), `ops_transfer`, `ops_delete_plan`, `ops_recycle_bin`, `gio_node`, the ox-core `admin_copy`, `admin_delete`, `transfer`, `gio_node` and `ops` unit tests, and the ox-app `worker_questions`, `failure_dialog`, `context_menus`, `progress_window`, `transfer_panel`, `clipboard`, `delete` and `recycle_bin` tests.
- Real app, on a private X display as an unprivileged user with fictional files, in a container where a polkit rule stood in for the administrator password:
  - copy, move, and Continue with "Do this for all" into a root-owned folder;
  - deleting root-owned files, and a nested root-owned folder through the helper;
  - refusals for a group-writable folder (copy, delete and Replace merge), with no password request.
- The real password prompt and the Recycle Bin path were not exercised in the real app. The Recycle Bin path is covered by a GTK test.

## Overlap with open pull requests

- #252 (transfers) changes `transfer/engine.rs` and `transfer/item_failure.rs`, which this branch also changes. A trial merge resolves those files automatically, and conflicts only in the OPS-047 note of `native/parity/features.toml`.
- #258 and #259 conflict only in the generated `apps/web/public/assets/markdown.json`; `tools/sync-docs.py` regenerates it.
- #253-#257, #260 and #261 merge cleanly.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
