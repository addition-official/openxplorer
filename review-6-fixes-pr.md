# Title

fix: Keep compressed folders complete and fix Open, Share, indexing, Hidden and error pages

# Body

Model: claude-opus-5-5
CLA: I agree

## Problem and resulting behavior

Data safety comes first: compressing a folder could make an incomplete archive without a word. The other changes fix everyday actions that did nothing or failed, and differences from Windows 11 File Explorer.

- **Compress to and Create archive left out links, FIFOs and sockets silently.** A symbolic link is now stored as what it points to, under the link's name, as Windows' Compressed (zipped) Folder stores it and as its Create archive dialog does with "Retain symbolic links" clear. A dangling link, a link to a folder that holds it (or to `/`), a link on a share and a special file cannot be stored; a message "Some items were not compressed" names each one with the reason. Special files are never opened, and a linked file is opened without following a link put in its place later.
- **A new, empty text document could not be opened.** GIO types every empty file as `application/x-zerosize`, which no app handles. An empty file now takes the type its name gives, so opening, Open with and Properties' "Opens with" work, as in Windows.
- **Share did nothing when no email app was set up.** The portal's answer arrives within a millisecond, and the subscription by the portal's well-known name could miss it. The answer is now matched by the request's own object path. Share says "Opening your email app…" once the files are open, and a failure shows a message that no email app is set up.
- **"Index pinned folders automatically" indexed nothing on a first run.** Desktop, Documents, Downloads, Pictures, Music and Videos look pinned in Quick access but were not pins in the settings file. They now count as pins, and an existing cache indexes them once. The empty state no longer asks to pin folders that already look pinned.
- **Properties' Hidden renamed the item to a dot-name**, breaking links and other apps' lists. In Windows, Hidden is an attribute. The name is now listed in the folder's `.hidden` file (which GIO, Nautilus and Dolphin honor), and clearing Hidden takes it off; other lines are kept, and the list is replaced through a new file. A dot-name still reads as hidden. Only an item off this computer, where no list can be written, still gets a leading dot.
- **A local folder that could not be listed** showed the network icon and GIO's raw text, which for a path past 4,096 bytes was the whole path, drawn selected. It now shows a folder icon and a short sentence ("The file name or path is too long.", "This folder can't be found…", "Access is denied.") with the system's text under Details, unselected.
- **Dates at and before 1 January 1970 showed the unknown dash** in Date modified, Properties and the Recycle Bin, and sorted as unknown. Item times are now signed, as GIO keeps them, so these dates show and sort as in Windows. The search cache, the settings file and the thumbnail cache keep their unsigned times, where 0 still means unknown.
- **Details pane on a picture:** the preview now runs edge to edge with square corners, as tall as the picture's shape needs; the name is larger beside a 32px icon centered on it, and the Details rows are 22px apart with their values at the pane's right edge, as measured on Windows 11.
- **Set as background, Rotate left and Rotate right** now show in the command bar whenever pictures are selected, not only in Gallery. Set as background hands the picture to the desktop's Wallpaper portal, which shows its own preview and asks first; it is grayed where the desktop has no Wallpaper backend. Rotation stays lossless (EXIF orientation for JPEG).
- **Share and Set as background are offered where Windows offers them**, also where the desktop has no Email or Wallpaper portal; a click then explains why nothing opens.
- **Details pane, measured against Windows 11:** the preview sits in a box across the top of the pane with Windows' bands around a picture, and every Details value stays on one line, cut with "…".
- **Open and Save dialogs for other apps:** a dialog is never titled with a filter's name. Chrome's Save page as was titled "All Files"; a title that is empty, names one of the request's filters or is Chromium's "All Files" now gives way to Windows' own "Save As", "Open" or "Select Folder".
- **Picture commands with the Windows symbol font:** with Windows icon files and a symbol font on, Set as background, Rotate left and Rotate right now take their glyphs from the font like the rest of the command bar (Picture, RotateMapLeft, RotateMapRight), instead of the rotate buttons borrowing Refresh's and Restore's. Without the font they look as before.
- **Menus measured against Windows 11:** context menu rows are 32px apart and 41px across a separator (were 33 and 36); New's menu opens 1px left of the button; the View menu's dot sits 19px in, with 21px to the icon.
- **Smaller fixes:** a local folder with one unreadable subfolder reads "Indexed, some folders could not be read" instead of "Incomplete / offline"; Previous Versions of a local file says only "There are no previous versions available."; Alt+D, Ctrl+L, F4 and Ctrl+F no longer reach the window behind an in-window dialog; the Gallery year scale marks every year that has photos; emptying the Recycle Bin no longer warns that the search cache could not follow.

| Case | Before | After |
|---|---|---|
| Compress a folder with links, a FIFO and a socket | Only regular files; no message | Link targets stored; the rest named in a message |
| Open an empty `.txt` | "No application is installed for this file type" | Opens in the default text app |
| Share with no email app | Nothing | "No email app is set up…" |
| First run, Settings > Search | "No folders are indexed yet" | "Instant search is on for 6 folders" |
| Properties > Hidden on `Meeting notes.txt` | Renamed to `.Meeting notes.txt` | Name kept, listed in `.hidden` |
| Folder path over 4,096 bytes | Network icon, 4 KB path selected | Folder icon, "The file name or path is too long." |
| File dated 1960 or 1970-01-01 | Unknown dash | 03/01/1960 12:00, 01/01/1970 00:00 |
| Gallery, photos from 2019 to 2025 | Scale showed 2025, 2023, 2020 | 2025 down to 2019 |

Not changed here:
- Right-to-left names in the Name column and two-line names in Tiles need changes in `folder_view/cells.rs`, which another open change edits.
- The long-path "Interrupted Action" window belongs to the copy and move code.
- Several differences need a check on Windows first: copying file links, a deleted folder's tab, time-zone changes, Print, File contents in Search options, Japanese dates and folder link types.

## Validation

Run with Rust 1.92.0. App tests ran as an unprivileged user with a throwaway HOME on a private Xvfb display and D-Bus session.

- `archive_compress_links` (new): fails on the previous code, passes now. Also passing: `archive_compress_sources`, `archive_progress`, `archive_tar_links` and the `archive::create` unit tests.
- ox-core unit tests: entry, email portal, search pins and places pass. In the whole binary, 28 tests fail in this container, all in `admin_copy`, `admin_delete` and one clipboard test, which need a trusted path and the Python app sources. None of them is in a changed module.
- ox-app tests of the changed modules, all passing: properties and item dialogs (153), search and indexing (71), Share, item dialogs, disconnected tabs, empty pages, address bar, input and search box (340), Gallery and scrubber (32), and empty and unavailable pages (49). After the date change: grouping, sorting, model, Gallery, details pane, Recycle Bin, thumbnails and Details columns, 428 passed and 2 failed (the two bubblewrap thumbnailer sandbox tests, whose code is unchanged; not re-run on the base).
- Real input with xdotool in the full app, with generated files: long download names with spaces and dashes, mp3, mp4, jpg, pdf, docx, and accented and Japanese names. Checked on screen: dates of 1960 and 1970 in Details, Properties and sorting, compressing, opening an empty file, Share, first-run indexing, Hidden, the error page, Alt+D behind a dialog and the Gallery scale. The indexed-folder wording, Previous Versions and the Recycle Bin warning were checked only by tests and the build. Share was checked with `xdg-desktop-portal` and its GTK backend and no email app set up.
- Long names, compared with Windows 11 captures by a script that measures lines, widths and spill into the next item: Tiles, Large icons, Content, the Details pane, tab titles and the address bar. Nothing overlaps its neighbor. The Tiles, Large icons and Content layouts still differ from Windows, as listed above.
- `native/tools/check.py` steps: the inventory and driver tests, the drawn-icon guard and rustfmt pass. `cargo clippy -D warnings` reports three warnings that the unchanged base reports too, and none from this change.
- `native/parity/check.py`, `tools/i18n.py check`, `tools/sync-docs.py --check` and `tools/audit-public-data.py` pass.
- Second round, measured by script against Windows 11 captures (1920x1080, dark): details pane, command bar, context menu, New and View menus. Tests of those modules: 259 of 259 and 285 of 287 passed; the two failures need the Python app sources or fail the same way without these changes (a right-click on the Network page).
- Not run: the whole ox-app test binary in one process, Wayland, HiDPI, the packaged .deb, a real email app.

## Compatibility and release impact

- Persisted data: Hidden now writes the folder's `.hidden` file instead of renaming the item. The search cache stores two new one-time markers in its existing table. The status word stored for an indexed folder is unchanged.
- Archives now hold the contents of linked files and folders inside the selection.
- `native/po/openxplorer.pot` was regenerated for the new messages. No translation file was changed.
- `native/parity/features.toml` notes for ARC-023, PROP-002 and PROP-005 were updated.
- No new dependency, license change or public documentation change. All examples use fictional files and paths.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
