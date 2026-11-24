# Title

fix(errors): Say every system error in Windows 11's words, with Windows' error codes

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/15-pass-14-tabs-windows)

## What changes for the user
- **System errors read as Windows says them.** A new `ox-core/src/system_error.rs` is the one place a GIO, GLib (file, resolver, spawn) or local I/O error becomes the user's sentence. It maps the error by the `errno` that ends GIO's message, else by its GIO code, to the Win32 error Windows reports for the same failure, and says it in the words of Microsoft's System Error Codes pages ("Access is denied.", "The media is write protected.", "The network path was not found."); anything else is `E_FAIL`, "Unspecified error". The item GIO named is kept as a path, never a `file://` URI ("/home/…/one.txt: Access is denied."). GIO's own message goes to the debug log (`G_MESSAGES_DEBUG`).
- Operation, entry, transfer and network errors, and the file errors of settings, the search cache, update, terminal, reveal, file dialogs, Brave, versions and the mount helper, say the system's error this way; so do app places that showed a raw GIO error (opening through the portal, links, web pages, help, tools, extraction, folder icon, Software search, the File Conflict listing, type associations, service actions). Closing the desktop's app chooser is no longer reported as a failure.
- **Interrupted Action shows Windows' error code line**, for example "Error 0x8007048F: The device is not connected.", instead of the system's text.
- **Location is not available is Windows' message box** (error icon, the message, OK), without the Details expander that repeated GIO's message.
- **A refused rename in place asks Windows' question** instead of showing a toast: File Access Denied, Item Not Found, File In Use, or Interrupted Action with the error code line, with Try Again and Cancel.
- **A share that cannot be reached shows Windows 11's "Network Error"** (checked on a Windows 11 VM) and keeps the tab where it was: the heading "OpenXplorer cannot access \\\\server\\share", "Check the spelling of the name. Otherwise, there might be a problem with your network.", See details / Hide details with "Error code: 0x80070043" and "The network name cannot be found.", and Diagnose / Cancel. Diagnose opens the desktop's network settings (GNOME's Network panel, KDE's Connections, or nm-connection-editor); when none is installed it is left out, together with the sentence that mentions it. A tab with nowhere to stay keeps the unavailable page, whose Details shows the same code. For shares, a server that refuses, cannot be reached or does not answer, and a missing share, read "The network name cannot be found." (0x80070043); a name that does not resolve, "The network path was not found." (0x80070035); a server that went away, "The specified network name is no longer available." (0x80070040).
- **A wrong password** makes the sign-in dialog say "The user name or password is incorrect."
- **A copy onto a write-protected drive asks as Windows 11 does** (checked on a VM): "1 Interrupted Action", "The disk is write-protected." / "Remove the write-protection or use another disk.", no error code line, no "Do this for all current items", the picture's Item type, Dimensions and Size, Try Again / Skip / Cancel, More details grayed.
- **New is grayed on a drive whose file system is read-only**, as Windows 11 grays it on a write-protected USB stick.
- **ZIP:** "The Compressed (zipped) Folder is invalid." for a damaged archive, and "This ZIP archive uses a feature that isn't supported. To open it, use a different archive app." for an unsupported one; zipfile's words go to the log.
- **Format:** "OpenXplorer was unable to complete the format."; a drive formatted but not mounted again no longer quotes UDisks.
- The `gio` workspace dependency gets the `v2_74` feature (the app already needs GLib 2.76) for `G_IO_ERROR_NO_SUCH_DEVICE`.
- The message template is regenerated and the parity notes are updated (DEV-015, VIEW-047, NET-003 and NET-007's sign-in text); no `.po` file is touched.

Unchanged: disk full, a typed address that is not found, and a missing app. New folder, New file and Paste refused by the system still say it in the message line (now in Windows' words), not in Windows' "Destination Folder Access Denied" dialog. A name too long on rename keeps "This name is longer than 255 bytes.".

## Why
Errors reached the interface as GIO, GLib, Samba, libmtp, UDisks or Python zipfile text, with paths and URIs inside, for example "Error moving file /home/…/keep.txt: Permission denied" in a toast after a rename, "Error creating directory /mnt/…/beach.jpg: Read-only file system" in Interrupted Action, "Failed to mount Windows share: Connection refused" under an unreachable share's Details, "File is not a zip file" for a damaged ZIP, and "Permission denied (os error 13): /home/…/settings.json". Windows 11 says the same failures in its own words and, where its dialogs have one, with the error code line. An unreachable share moved the tab to an unavailable page, where Windows 11 stays and asks.

## CHANGED OWNER DECISIONS
The parity specs in `native/parity/features.toml` record these decisions, and this PR changes them:
- DEV-015 (A drive mounted read-only does not explain itself). Before: "A write that fails there shows the system's own error, as the Python app's error_payload did. New and Paste stay on too, as they do in any folder the user may not write to (CMD-011): a New or Paste that fails there shows the system's error and makes nothing." After: a write that fails there shows the system's error in Windows' words ("The media is write protected.", in Interrupted Action "Error 0x80070013: The media is write protected."); Paste stays on as before, but New is grayed on a drive whose file system is read-only, as Windows 11 grays it on a write-protected USB stick; and a copy that meets the drive asks "1 Interrupted Action", "The disk is write-protected.", "Remove the write-protection or use another disk." with Try Again, Skip and Cancel, no error code and no "Do this for all current items", More details grayed. The rest of the entry (no notice, no explanation of why the drive is read-only, Paste into folder, Cut, Rename, Duplicate and Delete on) is unchanged.
- NET-007 (Network sign-in dialog layout). Before: a refused sign-in said "The previous sign-in was not accepted. Check your username and password."; after: "The user name or password is incorrect.", Windows' words.
- VIEW-047 (Empty, loading and unavailable states). Before: a folder that may not be read showed "Access is denied." with the system's error under Details; after: Windows' message box (error icon, OK, nothing under Details).
- NET-003 (SMB server listing shows shares as network folders). New note: a share that cannot be reached keeps the tab on the folder it was opened from and shows Windows 11's "Network Error" dialog, instead of moving the tab to an unavailable page.

## New and changed visible text
Changed:
- "Could not format the drive: {error}" → "OpenXplorer was unable to complete the format."
- "{drive} is formatted, but could not be mounted again: {error}" → "{drive} is formatted, but could not be mounted again."
- "{error} (It could not be mounted again: {reason})" → "{error} The drive could not be mounted again."
- "The previous sign-in was not accepted. Check your username and password." → "The user name or password is incorrect."

New, in dialogs:
- "Network Error"
- "OpenXplorer cannot access {location}"
- "Check the spelling of the name. Otherwise, there might be a problem with your network."
- "Check the spelling of the name. Otherwise, there might be a problem with your network. To try to identify and resolve network problems, click Diagnose."
- "Diagnose"
- "See details"
- "Hide details"
- "Error code: {code}" followed by a line break and "{message}"
- "Error {code}: {message}"
- "An unexpected error is keeping you from renaming the file. If you continue to receive this error, you can use the error code to search for help with this problem."
- "An unexpected error is keeping you from renaming the folder. If you continue to receive this error, you can use the error code to search for help with this problem."
- "The disk is write-protected."
- "Remove the write-protection or use another disk."
- "Dimensions: {dimensions}"
- "The Compressed (zipped) Folder is invalid."
- "This ZIP archive uses a feature that isn't supported. To open it, use a different archive app."

New, the system error sentences:
- "Access is denied."
- "An operation is not supported on a directory."
- "An unexpected network error occurred."
- "Cannot create a file when that file already exists."
- "Incorrect function."
- "No application is associated with the specified file for this operation."
- "Not enough memory resources are available to process this command."
- "The data is invalid."
- "The device is not connected."
- "The device is not ready."
- "The directory is not empty."
- "The directory name is invalid."
- "The file or directory is corrupted and unreadable."
- "The file or directory is not a reparse point."
- "The file size exceeds the limit allowed and cannot be saved."
- "The filename or extension is too long."
- "The filename, directory name, or volume label syntax is incorrect."
- "The handle is invalid."
- "The media is write protected."
- "The name of the file cannot be resolved by the system."
- "The network connection was aborted by the local system."
- "The network location cannot be reached."
- "The network name cannot be found."
- "The network path was not found."
- "The operation was canceled by the user."
- "The parameter is incorrect."
- "The process cannot access the file because it is being used by another process."
- "The remote computer refused the network connection."
- "The request could not be performed because of an I/O device error."
- "The request is not supported."
- "The requested file operation failed because the storage quota was exceeded. To free up disk space, move files to a different location or delete unnecessary files. For more information, contact your system administrator."
- "The semaphore timeout period has expired."
- "The specified network name is no longer available."
- "The system cannot find the file specified."
- "The system cannot find the path specified."
- "The system cannot move the file to a different disk drive."
- "The system cannot open the file."
- "The user name or password is incorrect."
- "There is not enough space on the disk."
- "This operation returned because the timeout period expired."
- "Unspecified error"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: new tests, each seen failing before its fix: `system_error` (11, the whole mapping), `ops::error`, `entry::error`, `transfer::error`, `network::error`, `failure_dialog::an_unexpected_error_shows_its_error_code` and `each_failure_takes_windows_wording`, `tests::listing::a_folder_that_may_not_be_read_keeps_the_tab_where_it_was`, `tests::renaming::a_refused_rename_in_place_asks_to_try_again_in_windows_words` (as an unprivileged user), `empty_page::an_unreachable_share_shows_the_windows_error_code_under_details`, `zip::error::the_user_reads_windows_words`, `archive_browse::a_file_that_is_not_a_zip_reports_a_bad_zip`, `tests::archives::a_corrupt_zip_says_why_it_cannot_be_browsed`, the `drive_format` and `disk_tools` messages, and `sign_in_queue::a_retry_says_why_…` (some of these were later removed as redundant and are not in this branch); real input with xdotool on a private Xvfb and bus with a throwaway HOME as an unprivileged user, before and after (a chmod 000 folder, rename in a chmod 555 folder, a damaged ZIP, paste onto a read-only tmpfs and onto a full 2 MB tmpfs, an unknown file type, and a local Samba server with a missing share, a server that is off, a wrong password and the server stopped while browsing); every test executable as check.py runs it, against unchanged main (5,148 passed / 29 failed against 4,951 / 30, every failing name also failing on main); `python3 native/parity/check.py`; `python3 native/tools/check.py`'s parity, i18n, tools, icon and rustfmt steps, stopping at Clippy on a Rust 1.97 lint in files this branch does not change, as on main.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
