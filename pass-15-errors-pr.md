# Title

fix(errors): Say every system error in Windows 11's words, with Windows' error codes

# Body

Model: claude-opus-5-5

CLA: I agree

## Problem

Errors reached the interface as GIO, GLib, Samba, libmtp, UDisks or Python zipfile text, with paths and URIs inside: "Error moving file /home/…/keep.txt: Permission denied" in a toast after a rename, "Error creating directory /mnt/…/beach.jpg: Read-only file system" in Interrupted Action, "Failed to mount Windows share: Connection refused" under an unreachable share's Details, "File is not a zip file" for a damaged ZIP, "libmtp error: could not rename", "Could not format the drive: <UDisks text>", "Permission denied (os error 13): /home/…/settings.json". Windows 11 says the same failures in its own words ("Access is denied.", "The media is write protected.", "The network path was not found.") and, where its dialogs have one, with the error code line ("Error 0x80070013: The media is write protected.").

## Changes

- `ox-core/src/system_error.rs` (new): the one place a GIO, GLib (file, resolver, spawn) or local I/O error becomes the user's sentence. It sorts the error by the `errno` whose words end GIO's message (also in brackets, as GLib's spawn errors put them), else by its GIO code, into the Win32 error Windows reports for the same failure, and says it in Microsoft's words from learn.microsoft.com's System Error Codes pages; anything else is `E_FAIL` "Unspecified error". `describe` keeps the item GIO named, as a path, never a `file://` URI ("/home/…/one.txt: Access is denied."). `code_line` gives "Error 0x8007048F: The device is not connected." (`HRESULT_FROM_WIN32`). `on_network` gives Windows' words for shares: a server that refuses, cannot be reached or does not answer, and a missing share, are "The network name cannot be found." (0x80070043, as the Windows 11 VM showed for a dead address), a name that does not resolve "The network path was not found." (0x80070035), a server that went away "The specified network name is no longer available." (0x80070040). GIO's own message goes to the debug log (`G_MESSAGES_DEBUG`). Tests for every mapping.
- `OpsError`, `EntryError`, `TransferError`, `NetworkError` and every `Io { path, error }` error (settings, search cache, update, terminal, reveal, file dialogs, Brave, versions, mount helper) say the system's error through it; app sites that showed a raw GIO error (open with the portal, links, web pages, help, tools, extraction, folder icon, Software search, File Conflict listing, type associations, service actions) use it too. Closing the desktop's app chooser is no longer reported as a failure.
- Interrupted Action shows Windows' error code line instead of the system's text.
- Location is not available is Windows' message box (error icon, the message, OK), without the Details expander that repeated GIO's message.
- Renaming in place that the system refuses asks Windows' question instead of a toast: File Access Denied, Item Not Found, File In Use, or Interrupted Action with the error code line, with Try Again and Cancel.
- A share that cannot be reached, gone to from a folder, keeps the tab there and asks Windows 11's "Network Error" (checked on a Windows 11 VM): the blue heading "OpenXplorer cannot access \\\\server\\share", "Check the spelling of the name. Otherwise, there might be a problem with your network.", See details / Hide details with "Error code: 0x80070043" and "The network name cannot be found.", and Diagnose / Cancel. **Diagnose** opens the desktop's network settings (GNOME's Network panel, KDE's Connections, or nm-connection-editor); when none is installed it is left out, with the sentence "To try to identify and resolve network problems, click Diagnose.". A tab with nowhere to stay keeps the unavailable page, whose Details shows the same code. The sign-in dialog says "The user name or password is incorrect." after a refused sign-in.
- A copy onto a write-protected drive asks as Windows 11 does (checked on a VM): "1 Interrupted Action", "The disk is write-protected." / "Remove the write-protection or use another disk.", no error code line, no "Do this for all current items", the picture's Item type, Dimensions and Size, Try Again / Skip / Cancel, More details greyed. New is greyed on a drive whose file system is read-only, as Windows greys it there; this revises DEV-015, whose note is updated.
- ZIP: "The Compressed (zipped) Folder is invalid." for a damaged archive, "This ZIP archive uses a feature that isn't supported. To open it, use a different archive app." for an unsupported one; zipfile's words go to the log.
- Format: "OpenXplorer was unable to complete the format."; a drive formatted but not mounted again no longer quotes UDisks.
- `gio` workspace dependency gets `v2_74` (the app already needs GLib 2.76) for `G_IO_ERROR_NO_SUCH_DEVICE`.
- Merged, as the job rules ask: network-like-windows (with passes 1, 2, 5, 8, 9), pass-7-views (with passes 3 to 6) and passes 10 to 14 (one conflict, the New rule in availability.rs, both kept). pass-4's 5cb3a27c, a second listing of Network's computers in the navigation pane that network-like-windows left out on purpose, stays out: its conflicts take network-like-windows' side and its other hunks are dropped.
- `.pot` regenerated, parity notes updated (DEV-015, VIEW-047, OPS-047, sign-in text); no `.po` touched.

## Error table

| Where | How to trigger it | Before | Windows 11 | Now |
|---|---|---|---|---|
| Interrupted Action (copy to a read-only drive) | paste into a drive mounted read-only | Error creating directory /mnt/rofs/beach.jpg: Read-only file system | The disk is write-protected. / Remove the write-protection or use another disk. (Windows 11 VM) | same, no code line |
| Command bar on a read-only drive | open a write-protected stick | New on | New greyed (Windows 11 VM) | New greyed |
| Interrupted Action (drive pulled) | FUSE drive gone mid-copy (unit test) | Error splicing file: Transport endpoint is not connected | Error 0x8007048F: The device is not connected. (Microsoft) | same |
| Interrupted Action (anything unknown) | e.g. a phone's refusal | libmtp error: could not rename | Error 0x80004005: Unspecified error (Microsoft) | same |
| Rename in place | rename in a folder you may not write | toast: Error moving file /…/keep.txt: Permission denied | File Access Denied, You need permission to perform this action, Try Again / Cancel (needs VM) | that dialog |
| Rename in place, item gone / other | rename an item deleted meanwhile; other refusals | toast with GIO text | Item Not Found / Interrupted Action with Try Again / Cancel (needs VM) | those dialogs |
| Location is not available | open a folder you may not read | message + Details "Error opening directory …: Permission denied" | Location is not available, error icon, "… is not accessible. Access is denied.", OK (needs VM) | message box, no Details |
| Share that cannot be reached | type \\192.0.2.250\Projects from a folder | the tab moves to an unavailable page, Details: Failed to mount Windows share: Connection refused | Network Error, "Windows cannot access \\…", See details: Error code: 0x80070043 The network name cannot be found., Diagnose / Cancel, the tab stays (Windows 11 VM) | the same, "OpenXplorer cannot access"; Diagnose only with network settings installed |
| Unavailable share, Details | a share the server does not have | Failed to mount Windows share: No such file or directory | 0x80070043 The network name cannot be found. (Microsoft) | same |
| Unavailable share, Details | server stopped while browsing, F5 | Software caused connection abort | 0x80070040 The specified network name is no longer available. (Microsoft) | same |
| Network sign-in | wrong password | The previous sign-in was not accepted. Check your username and password. | The user name or password is incorrect. (Microsoft) | same |
| ZIP browser | open a damaged ZIP | File is not a zip file / Bad CRC-32 for file 'x' / Overlapped entries … | Compressed (zipped) Folders Error: Windows cannot open the folder. The Compressed (zipped) Folder '…' is invalid. (needs VM) | The Compressed (zipped) Folder is invalid. |
| ZIP browser | ZIP with an unsupported method or version | zip file version 6.3 / strong encryption (flag bit 6) | needs VM | This ZIP archive uses a feature that isn't supported. To open it, use a different archive app. |
| Toasts of New, Paste, Delete, pin, Duplicate, Open… | e.g. New folder on a read-only drive | Error creating directory /…: Read-only file system | The media is write protected. (Microsoft) | /…/New folder: The media is write protected. |
| Format | UDisks refuses | Could not format the drive: <UDisks text> | Windows was unable to complete the format. (needs VM) | OpenXplorer was unable to complete the format. |
| Format | formatted, not mounted again | … could not be mounted again: <UDisks text> | needs VM | {drive} is formatted, but could not be mounted again. |
| Disk tools | the tool cannot start | Could not start /usr/bin/x: No such file or directory (os error 2) | The system cannot find the file specified. (Microsoft) | Could not start x: The system cannot find the file specified. |
| Settings, caches, update, terminal | a file the app cannot write | Permission denied (os error 13): /home/…/settings.json | Access is denied. (Microsoft) | …could not be saved: Access is denied. |
| Programs and links | a program, link or web page that cannot open | GLib/GTK/portal text | the matching Win32 sentence | same |
| Disk full, typed address not found, missing app | — | unchanged (pass 1, #358, network-like-windows' Open with) | — | unchanged |

## Tests run

- New tests, each seen failing before its fix: `system_error` (11, the whole mapping), `ops::error`, `entry::error`, `transfer::error`, `network::error` (GIO messages read as the Windows error; read-only file system), `failure_dialog::an_unexpected_error_shows_its_error_code` and `each_failure_takes_windows_wording`, `tests::listing::a_folder_that_may_not_be_read_keeps_the_tab_where_it_was` (no Details), `tests::renaming::a_refused_rename_in_place_asks_to_try_again_in_windows_words` (as an unprivileged user), `empty_page::an_unreachable_share_shows_the_windows_error_code_under_details`, `zip::error::the_user_reads_windows_words`, `archive_browse::a_file_that_is_not_a_zip_reports_a_bad_zip`, `tests::archives::a_corrupt_zip_says_why_it_cannot_be_browsed`, `drive_format` and `disk_tools` messages, `sign_in_queue::a_retry_says_why_…`.
- Real input (xdotool) on a private Xvfb, private bus, throwaway HOME, as an unprivileged user, before (unchanged main 5d21fd3e) and after: a chmod 000 folder, rename in a chmod 555 folder, a damaged ZIP, paste onto a read-only tmpfs, an unreadable file pasted onto a 2 MB tmpfs, a 64 MB file onto it, an unknown file type, and a local Samba server: a missing share, a server that is off, a wrong password, the server stopped while browsing. No "Bail out", CRITICAL, panic or signal in any run's stderr.
- Every test executable run as check.py runs it (run_isolated, private Xvfb and bus, throwaway HOME, unprivileged user, Python app from v2.0.0), final tree against unchanged main 5d21fd3e: 5,148 passed / 29 failed against 4,951 / 30; every failing name also fails on main.
- `python3 native/parity/check.py` passes; `python3 native/tools/check.py` passes its parity, i18n, tools, icon and rustfmt steps, then stops at Clippy on Rust 1.97's `duration_suboptimal_units` in files this branch does not change, as on main.

## Known limitations

- Windows' exact dialog layouts (title bar, item details, More details) are on the VM CHECK list; only the Win32 sentences and E_FAIL are from Microsoft's pages.
- New folder, New file and Paste refused by the system still say it in the message line (now in Windows' words), not in Windows' "Destination Folder Access Denied" dialog.
- Name too long on rename keeps the app's "This name is longer than 255 bytes."; invalid characters on FAT and SMB, a share timing out after connecting, and a file locked by another program could not be triggered here (no vfat, Linux does not lock files).
