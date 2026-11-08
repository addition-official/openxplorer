# Overnight review: OpenXplorer main 016c335

Reviewed upstream main `016c33598d3292a52c1b8879c38ec52fc12564e5` (after #299). Report only: no code changes, no PR, no deb.

**Rank** is how soon you'd notice it in daily use: 1 = right away, 5 = rarely.

**Windows sources.** Windows behavior is taken only from Microsoft pages:
- KB = the keyboard shortcuts page, https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec (Windows 11 tab, File Explorer section).
- FE = the File Explorer overview page, https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows.

Anything Microsoft doesn't state is in "Not sure".

## How it was tested

- **Builds.** A debug build and a release build of 016c335, run as `tester` on private Xvfb displays with a private D-Bus and a throwaway HOME, using fictional files (Documents/Budget 2026.txt, Notes 1-3.md, Pictures/sunset.png). Driven with xdotool. Timings are from the release build.
- **SMB.** A local smbd on 127.0.0.1 with fictional guest and password shares, opened through GVfs.
- **Edge cases.** Read-only and full drives were small tmpfs mounts. Also tested: 10,000 and 50,000 files, a 110-level tree, sparse 3-6 GB files, chmod 000, and symlink loops.
- **Test suite.** Every test binary, run in isolation: 4,211 passed, 3 failed (see Regressions).
- **Installed for this run:** xdotool, samba, smbclient and the en_US.UTF-8 locale. They were missing.
- **Not run:**
  - Clippy: the container has Rust 1.97, whose newer lints don't apply to 1.92.
  - Doctests.
  - `native/parity/check.py`.
- **Environment limits.**
  - Xvfb has no window manager, so dialogs open at the top-left and menus have black corners. Those are not app bugs.
  - The Open/Save dialog only answers the desktop portal, so it was checked by its tests and code, not by hand.

## A. Clearly wrong (a Microsoft source says so)

| Rank | What's wrong | Steps | Windows (source) | Suggested fix |
|---|---|---|---|---|
| 1 | Ctrl+Shift+1 to 8 don't change the view. | Focus the file list and press Ctrl+Shift+2 or Ctrl+Shift+7. It stays in Details. Ctrl+Shift+N works. Seen in two separate sessions. Please confirm once on a real keyboard. | "Ctrl+Shift+Number (1-9): Change the view style. For example, 2 is Large icons and 6 is Details" (KB) | Match these keys by hardware keycode, since Shift+2 arrives as "@". The view keys are set in `window/folder_pane/view.rs:157`. Add a test that presses the keys, not one that only checks they're registered. |
| 2 | Ctrl+1 to 9 don't switch tabs. Only Alt+1 to 9 do. | Open 2 tabs and press Ctrl+1. Nothing happens. | "Ctrl+Number (1-9): Move to that tab number" (KB) | Add `<Primary>1` to `<Primary>9` beside the Alt keys in `window/window_keys.rs:79`. |
| 3 | Ctrl+Shift+E does nothing. | In ~/Documents/Projects, press Ctrl+Shift+E. The navigation pane doesn't expand to the folder. | "Ctrl+Shift+E: Expands all folders from the tree in the navigation pane" (KB) | Bind it to expand the pane's tree down to the current folder. |
| 4 | Compact view is in View > Show, not in the View menu itself. | Open View. Compact view is only under Show. | The View menu "includes Compact view". View > Show offers Item check boxes, File name extensions, Hidden items and Navigation pane (FE) | Move Compact view to the View menu, after Content. |
| 4 | A name ending in a space or a period is saved as typed. | Rename a folder to `Archive. ` (dot, then space). The folder on disk is `Archive. `. | "Do not end a file or directory name with a space or a period... the Windows shell and user interface does not [support it]" (https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file) | Trim trailing spaces and periods on rename and New folder, as the Windows shell does. |
| 5 | The "..." button is labelled "More options". | Hover "..." in the command bar. | Microsoft calls it "See more", which leads to "Options" (FE) | Rename the tooltip and accessible name to "See more". |

## B. Not sure (needs a Windows check)

| Rank | What OpenXplorer does | What to check on Windows 11 |
|---|---|---|
| 2 | Home lists your home folder like any folder. | Windows' Home page shows Quick access, Favorites and Recent sections. Microsoft mentions a "modernized Home" (https://blogs.windows.com/windows-insider/2023/07/21/announcing-windows-11-insider-preview-build-22621-2050-and-22631-2050/) but doesn't describe its layout. |
| 2 | The Preview pane (Alt+P) shows only a large icon for a .txt file. | Does Windows preview text contents? The preview-pane-contents job on the status branch is already working on this. |
| 3 | The Sort menu has Name, Date modified, Type and More, but no Size, even when the Size column is shown. | Which columns the Windows 11 Sort menu lists. |
| 3 | Shift+F10 opens the classic "Show more options" menu. | Microsoft says Shift+F10 shows "the context menu" and Shift+right-click shows Show more options (KB). Which menu does Shift+F10 open? |
| 3 | The tab right-click menu has Move tab to new window, Move tab to window, Duplicate tab, Open windows, Close tab and Close other tabs. | Does Windows have "Close tabs to the right"? Microsoft documents only Duplicate tab (https://blogs.windows.com/windows-insider/2024/07/26/announcing-windows-11-insider-preview-build-26120-1330-dev-channel/). |
| 3 | Changing an extension asks "Change the extension?" with Cancel and Change extension buttons. | The Windows dialog's title, buttons and default button. Microsoft has no official page for its text. |
| 4 | The navigation pane has a usage bar under Local Disk, plus Recent files, Recent locations and Recycle Bin rows. | What a default Windows 11 navigation pane shows. |
| 4 | A gear button sits in the command bar. | Windows 11 reaches Options only through See more (FE). Probably a deliberate choice (#198). |
| 4 | Properties shows dates as "Today, November 8, 2026, 3:56:41 AM". | Windows' date format in Properties (weekday name, or "Today"?). |
| 4 | Items sort "Budget 2026 (2)" before "Budget 2026". | Windows' natural sort order for these names. |
| 5 | The This PC, Recycle Bin and Recent tabs show a yellow folder icon. The address bar also shows one for Recycle Bin. | Windows shows each location's own icon there (not confirmed). |

## C. Bugs and edge cases (tested for real)

| Rank | What's wrong | Steps | What happened | Suggested fix |
|---|---|---|---|---|
| 1 | Ctrl+A while renaming selects every item in the folder. | Select one file, press F2, then Ctrl+A. | The status bar says all items are selected. If you then press Esc and Delete, everything goes to the Recycle Bin. Reproduced 3 times. | Let the rename box keep Ctrl+A. The window's capture-phase Ctrl+A (`window/file_ops/shortcuts.rs:50`, `:66`, guard at `:104-117`) doesn't see the rename entry as a text field. |
| 1 | Tooltips stay drawn over a menu that has just opened. | Hover a file, then right-click it. The file's tooltip covers the Cut/Copy strip. The same happens with a tab's right-click menu and the "..." menu. | The tooltip stays until the mouse moves. | Hide tooltips while any menu is open. `quiet_tooltips.rs` only does this for drags. |
| 2 | One file shows three different sizes. | Look at sunset.png in Details, then its tooltip and Details pane, then Tiles and the status bar. | "5 KB", "4.4 KB" and "4.38 KB". | Use one formatter, or document why Details rounds up. |
| 2 | Copying a folder that has one unreadable subfolder copies nothing. | Copy a folder with `ok.txt` and a `chmod 000` subfolder, then paste. | "Operation stopped: ... Permission denied" with only OK, and nothing is copied. A single unreadable file correctly asks Try Again / Skip. | Let the pre-copy walk (`ox-core/src/transfer/guard.rs:111`) skip what it can't read, and ask about that folder in the progress window. |
| 2 | The Size column cuts off big files at its default width. | Show a 6 GB file in Details. | "6,291,45…" | Make the column's minimum width fit 10-digit KB values. |
| 3 | Files whose names contain a newline, or bytes that aren't valid UTF-8, can't be deleted or renamed. Neither can a folder that holds one. | Create such a file and press Delete. | "Operation stopped: Encoded control characters are not allowed" or "...must be valid UTF-8". Rename silently does nothing. | Check only typed addresses with `unquote_without_controls` (`ox-core/src/location/text.rs:117-123`, `normalise.rs:594`), never URIs of items already listed. |
| 3 | A read-only drive gives raw developer errors, and New, Paste and Delete stay enabled. | Paste into a read-only mount, then try Delete and Ctrl+Shift+N there. | "Error creating directory .../p1.bin: Read-only file system" (it is a file), "os error 30", and a raw toast. | Turn EROFS/EACCES into plain wording that names the item, and gray out commands that need write access. |
| 3 | The Recycle Bin opened empty after 50,000 files were recycled. | Press Ctrl+A, then Delete, in a folder of 50,000 files, then open Recycle Bin. | "This folder is empty" for more than 60 s, and Empty and Restore stay gray. F5 then showed all 50,002 items. At 10,000 files it worked. | List the bin again when it comes back empty but the Trash folder has entries. |
| 4 | Emptying or restoring a big Recycle Bin is slow. | Empty a bin holding 50,000 items, or Restore all for 10,000. | 201 s to empty. Restore runs at about 200 items/s. | For the local Trash, remove the files/ and info/ entries directly instead of making one GIO call per item. |
| 4 | Search results: one row's Size sits out of line with the others. | Search "Notes" in Documents when one hit is in a subfolder with a longer path. | That row's "1 KB" is about 14 px left of the others. | Give search-result columns fixed widths so long Folder path cells don't shift Size. |
| 4 | The not-enough-space message names the wrong drive and uses the wrong number. | Copy 8 MB into a full 20 MB mount inside home. | "There is not enough space on Local Disk", and "...to copy this item" for 2 items. Properties also names a tmpfs mount "Local Disk". | Use the mount's own name (`window/file_ops/failure_dialog.rs:287-293`) and plural wording. |
| 4 | Arabic and Hebrew names are pushed to the far right of the Name column. | Open a folder holding a file with an Arabic name. | The icon stays left and the text sits about 600 px away at the right edge. | Align the label by the view's direction, not the text's. |
| 5 | A very deep tree breaks the error dialog. | Copy a 110-level tree whose path is longer than 4,096 bytes. | The dialog is 2,006 px tall and OK is off-screen. Nothing is copied. | Wrap or ellipsize the path, cap the dialog's height, and make the deep item a skippable failure. |
| 5 | Smaller items. | Various. | Each listed in the notes below. | Fix each on its own. |

Notes on the "Smaller items" row:
- A name with a trailing space shows its type as "TXT  File", with two spaces.
- The column-header tooltip covers the first row.
- After the SMB server stopped, the navigation-pane tooltip still said "Connected".
- A second share on the same server asks for credentials again after you connected to the first as guest.
- After a wrong password, the user name resets.
- The progress header shows a raw "smb://" address.
- Sparse files are written out in full.
- One Gtk-CRITICAL `gtk_widget_is_ancestor` after going to a share whose server had stopped. It didn't happen again.

**Worked well:**
- Listing 50,000 local files took 2.4 s; 20,000 over SMB took 2.2 s.
- Ctrl+A on 50,000 files took 1.4 s, and sorting about 0.4 s.
- Memory stayed steady at about 690 MB.
- Symlinks are copied as links, including loops, and recycling a link removes only the link.
- 255-byte, emoji and leading-space names display fine.
- A full drive gets a proper question, and Cancel leaves nothing behind.
- When a share vanishes mid-copy you get Try Again / Skip / Cancel, and Cancel removes the partial copy.
- An unreachable server doesn't freeze the UI.
- No crashes.
- Delete to the Recycle Bin, Ctrl+Z/Ctrl+Y on delete, "- Copy" / "- Copy (2)" names, the Replace or Skip window, Ctrl+W on the last tab, Alt+P, Alt+Shift+P, Ctrl+E, Alt+D and Ctrl+L, and Ctrl+wheel all behave as Microsoft describes.

**Not tested:**
- Throttled or slow SMB (no `tc` here).
- Real USB, FAT, exFAT or NTFS drives, and loop devices.
- Continue as administrator (no pkexec).
- Wayland.

## D. Regressions in our merged PRs (#205-#297)

**None found.** Each PR's behavior was checked on 016c335:
- **Run in the app:** #205, #206, #241, #242, #246, #248, #249, #250, #262, #263, #264, #265, #266, #267, #268, #269, #271, #280, #281, #282, #283, #284, #286, #287, #288, #289, #290, #293, #297.
- **By their tests and code:** #243, #247, #270, #272, #285, #296, #299.
- **Skipped:** #292 (docs only).

**Three test failures, none of them regressions:**
- Two Python-fixture tests (`location_fixture_drift`, `network_python`) fail because the container's Python 3.13 now accepts `smb://[V1.x]/`. That comes from the environment.
- #272's restore-conflict test failed once with "The connection is closed", then passed 4 runs in a row. It's flaky.

**Not checked by hand:**
- The Open/Save dialog (portal-only).
- Recent rows (the test HOME had no recent files).


## E. Second pass

**What it covered.**
- The Open/Save dialog, driven for real: a script took the portal's bus name on a private session bus and called OpenFile and SaveFile on the app.
- Views, panes, Properties, Folder Options, archives, the New menu and Undo/Redo, keyboard focus.
- A slow and stalling SMB share behind a throttling proxy (about 30-50 KB/s).
- Drag and drop, live changes made by another program, concurrent operations, the Recycle Bin and search.

All of it ran on the release build with fictional files.

### Clearly wrong (a Microsoft source says so)

| Rank | What's wrong | Steps | Windows (source) | Suggested fix |
|---|---|---|---|---|
| 2 | Alt+drag copies the file instead of creating a shortcut. | Hold Alt and drag a file onto a folder. A plain copy appears there. | "Alt+mouse drag: create a shortcut to the original file in that location" (KB) | In `window/file_drop/action.rs` `with_keys`, map Alt to Link. On X11 GTK never offers the ASK action the code waits for. |
| 2 | A .tar.gz opens in a separate "Compressed folder" box instead of browsing like a ZIP. The box also says "Read-only ZIP" and "1 unsafe names or links are hidden". | Double-click a .tar.gz. | File Explorer has "native support for reading" .tar, .tar.gz, .tar.bz2, .tar.zst, .tar.xz, .tgz, .rar and .7z (https://blogs.windows.com/windows-insider/2023/08/18/announcing-windows-11-insider-preview-build-22621-2199-and-22631-2199-beta-channel/) | Browse tar archives in the window with the ZIP browsing code, and fix the wording. 7z and RAR stay a BACKLOG gap. |
| 3 | Backspace in the Open/Save dialog does not go up a folder. | Open the dialog in a folder with no Back history and press Backspace. Nothing happens. | "Backspace: Open a folder one level up if a folder is selected in the Save As or Open dialog box" (KB, dialog box section) | In the picker, make Backspace go Up when Back has nowhere to go. |
| 4 | Compress to offers only ZIP File and Additional options, with no TAR or 7z entry. | Right-click a file, then Compress to. | Windows 11 adds "support for creating 7-zip and TAR archives in addition to ZIP" (https://blogs.windows.com/windows-insider/2023/11/08/announcing-windows-11-insider-preview-build-25992-canary-channel/) | Add "TAR File" to the submenu (`window/context_menu/entries.rs:545`); the writer already exists in `ox-core/src/archive/create.rs`. Add 7z once there is an encoder. |
| 4 | Pressing Alt alone does nothing. | Press and release Alt with the file list focused. | "Alt moves focus to the app bar" (https://support.microsoft.com/en-us/windows/use-a-screen-reader-to-explore-and-navigate-file-explorer-in-windows-e7d3a548-87dd-459f-a991-9fde3f7ce927) | On a lone Alt press and release, focus the command bar's first button. |

### Bugs

| Rank | What's wrong | Steps | What happened | Suggested fix |
|---|---|---|---|---|
| 1 | Pictures, web pages, JSON and Python files always show their extension, while other files hide theirs. | Default settings, a folder holding sunset.png, plan.pdf and Budget 2026.txt. | You see "sunset.png", "plan" and "Budget 2026". GIO marks png, html, json and py as "uncertain" (checked). | In `ox-core/src/entry/extension.rs:42`, drop the `!uncertain` check, as `window/picker.rs:404` already does. |
| 2 | A drop onto a tab leaves a green outline on that tab. | Drag a file from one tab onto another tab's header. | The move works, but the green outline stays through later navigation and drags. Reproduced twice. | Clear the tab's drop-highlight when the drop finishes (`window/file_drop/targets.rs:66-93`). |
| 2 | One missing item stops the whole paste. | Cut two files, delete one of them from outside the app, then paste. | "Operation stopped: ... No such file or directory" with only OK, and the other file isn't moved either. | Same cause as the unreadable-subfolder bug (`ox-core/src/transfer/guard.rs:111`): skip the missing item and ask about it in the progress window. |
| 3 | Any tar containing a symbolic link is refused completely. | Extract all on a tar.gz holding a relative link such as `lib/libx.so -> libx.so.1`. | "...contains a symbolic link or special file. Nothing was extracted." | Allow relative links that stay inside the destination; refuse only absolute or escaping links. |
| 3 | Extracting onto existing files shows an internal staging folder in the conflict window. | Extract all on the same ZIP twice. | "Moving 1 item from .openxplorer-extract-9d0288b2... to normal", then "...has 2 files with the same names". | Name the archive and say "Extracting" (`ox-core/src/archive/extract/staging.rs:31`). |
| 3 | The Save dialog keeps typed double quotes and still adds the extension. | Save as a Text file, type `"Holiday.txt"` with the quotes, press Enter. | The saved name is `"Holiday.txt".txt`, quotes included. | In `saved_name()` (`window/picker.rs:388`), strip one pair of surrounding quotes and then don't append the extension. |
| 3 | Cancelling a copy that hasn't started writing pops a modal "Operation was cancelled" with OK. | Run two copies and cancel the one still preparing. | The modal appears. A copy cancelled after writing began closes quietly. | Don't report a cancel the user chose (`window/file_ops/running.rs:356-370`). |
| 3 | Moving a folder into its own subfolder shows a generic error with a useless Try Again. | Cut Projects, open Projects/Sub, paste. | "Interrupted Action: An unexpected error... Cannot place a folder inside itself", with Try Again / Skip / Cancel. | Refuse it before starting, with a plain message and no Try Again. |
| 4 | Renaming an item to an empty name in the Open/Save dialog shows the wrong message. | Rename an item in the dialog, erase the name, press Enter. | "Use a name without slashes or control characters." | Cancel the rename and keep the old name, or say the name can't be empty. |
| 4 | The search bar contradicts itself on a share. | Search inside a folder on an SMB share. | The bar says "Current folder only" while the scope dropdown says "This folder + subfolders". | Show the scope actually used, or disable the dropdown there. |
| 4 | More tooltips cover open lists and menus. | Open the dialog's file type list, or the search "Search in" menu. | The tooltip sits on the first row. | Same fix as the first-pass tooltip bug. |
| 5 | Archive safety limits refuse real archives with no way to override. | Extract a ZIP with a 1 GB file of zeros. | Refused: the limits are 8 GB per file, 20 GB in total and a 1000:1 ratio (`ox-core/src/archive/extract/limits.rs:32-34`). | Ask the user when a limit is hit, and keep the disk-space check. |
| 5 | Smaller items. | Various. | Each listed in the notes below. | Fix each on its own. |

Notes on the "Smaller items" row:
- Tar files get a plain document icon.
- The zip-slip refusal says "unsafe for local/SMB extraction" and doesn't name the path.
- The credentials dialog drops the port ("127.0.0.1" instead of "127.0.0.1:4455").
- Location shows `\\127.0.0.1:4455\public`, which isn't valid UNC notation.
- "Working on it..." gets a focus ring while a folder loads.
- The Delete Folder question wraps "Date created:" onto two lines.

### Not sure (needs a Windows check)

| Rank | What OpenXplorer does | What to check on Windows 11 |
|---|---|---|
| 1 | A single big file's copy stays at "0% complete" until the file is done. This is on purpose (OPS-020, `transfer_panel.rs:14-30`). | Do Windows' percentage and bar move during one large file? |
| 2 | Ctrl+T, Ctrl+N and Ctrl+W do nothing while the address bar or search box has focus. The code (`window/window_keys.rs`) keeps them for the text field, but BACKLOG CMD-017 says they work from text fields. | Ctrl+T in Windows' address bar and search box. Then fix whichever of the code and BACKLOG is wrong. |
| 3 | The Save dialog's name box shows "Budget 2026.txt" even with extensions hidden. | Does Windows show the extension there? |
| 3 | Closing the window during a copy asks Keep open or Cancel and close. Ctrl+Q instead refuses with a toast. | Does a Windows copy keep running after its window closes? |
| 3 | Type-ahead searches forward from the focused item, so "extra_3" can select extra_30. | What Windows selects in the same case. |
| 4 | The column titles' menu has no "More..." (Windows' Choose Details). | Not in Microsoft docs. |
| 4 | Save over an existing file asks "Confirm Save As" with Replace and Cancel. Microsoft confirms only that a prompt appears. | The Windows buttons. |
| 4 | Esc doesn't dismiss the Replace or Skip question. Esc during a drag didn't cancel the drag either, which may be an Xvfb artifact. | Windows' Esc behavior in both cases. |
| 4 | `?` isn't a search wildcard. Microsoft documents only `*`. | Does Windows accept `?` in search? |
| 4 | When another program renames the open folder, the app drops to the parent folder. | Does Windows follow the rename? |
| 5 | Undo of a new file undoes its name first, then the creation. Typing the Recycle Bin's address is refused. | Windows' order for Undo, and whether "Recycle Bin" typed in the address bar opens it. |

### Second pass: worked well

- **Open/Save dialog:**
  - multiple selection
  - changing the type updates the extension
  - the overwrite prompt appears
  - it reopens in the last folder saved to
  - Esc cancels
- **Views and panes:** every view, rubber-band selection, Folder Options Apply and Restore defaults, and folder Properties counting 30,000 files in 0.3 s.
- **Archives:**
  - ZIP browsing, Extract all, and Skip on conflicts
  - zip-slip, absolute paths, tar `../`, escaping links and zip bombs are all refused before anything is written
- **Multi-rename and Undo:** renaming four items gives "Trip (1)" to "Trip (4)", and one Ctrl+Z restores them all.
- **Focus:** F6 cycles every region, and Delete moves focus to the next item.
- **Slow share:** the UI stayed responsive on a stalling share (a new tab opened in under 0.1 s), and Cancel cleaned up partial files.
- **Drag and drop:** plain, Ctrl, Ctrl+Shift and right-button drags, and drops onto a crumb, a navigation pane item or a tab.
- **Live changes:** files changed by other programs show within about 2 s.
- **Recycle Bin:** restore when the original folder is gone, and same-name restores going back to the right folders.
- **Names:** case-only rename shows at once, and a taken name offers "(2)".
- **Search:** `*` wildcards, contents search, and search on an SMB share.

### Second pass: not tested

- 7z, RAR and .tar.zst archives (no tools here).
- The Open with dialog and Send to targets.
- Focus coming back after a dialog closes (no window manager on Xvfb).
- Network thumbnails (off by default, and bubblewrap isn't installed).
- A dropped SMB connection, as opposed to a stalled one.
- Recycle Bin on a real FAT USB drive.

## Totals (both passes)

| Section | Count |
|---|---|
| Clearly wrong | 11 |
| Not sure | 22 |
| Bugs | 27 |
| Regressions | 0 |

Security notes: see chat (count in overnight-review.txt).
