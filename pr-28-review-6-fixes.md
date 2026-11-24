# Title

fix(native): Keep compressed folders complete and fix Open, Share, indexing, Hidden and error pages

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/27-owner-audit-fixes)

Data safety comes first: compressing a folder could make an incomplete archive without a word. The other changes fix everyday actions that did nothing or failed, and differences from Windows 11 File Explorer.

## What changes for the user
- **Compress to and Create archive keep what links point to.** A symbolic link is stored as what it points to, under the link's name, as Windows' Compressed (zipped) Folder stores it. A dangling link, a link to a folder that holds it (or to `/`), a link on a share, a name that is not valid text and a special file cannot be stored; the message "Some items were not compressed" names each one with its reason. Special files are never opened, and a linked file is opened without following a link put in its place later.
- **An empty file opens with the app its name's type has.** A new, empty text document now opens in the default text app, and Open with and Properties' "Opens with" work for it.
- **Share hears the portal's answer.** The answer is matched by the request's own object path, so Share no longer does nothing. It says "Opening your email app…" once the files are open, and a failure says that no email app is set up.
- **"Index pinned folders automatically" indexes the standard folders.** Desktop, Documents, Downloads, Pictures, Music and Videos, which look pinned in Quick access, now count as pins, and an existing cache indexes them once. The empty state no longer asks to pin folders that already look pinned. Recycle Bin addresses are no longer handed to the search cache, so emptying the Recycle Bin no longer warns that the cache could not follow.
- **Properties' Hidden keeps the name.** The item is listed in its folder's `.hidden` file (which GIO, Nautilus and Dolphin honor), and clearing Hidden takes it off; other lines are kept, and the list is replaced through a new file. A dot-name still reads as hidden. Only an item off this computer, where no list can be written, still gets a leading dot.
- **A local folder that cannot be listed** shows a folder icon and a short sentence ("The file name or path is too long.", "This folder can't be found. It may have been moved or deleted.", "This folder can't be opened."), unselected, instead of the network icon and GIO's raw text. A share mounted at a local path still says it was disconnected.
- **Dates at and before 1 January 1970** show and sort in Date modified, Properties and the Recycle Bin, as in Windows. Item times are now signed, as GIO keeps them; the search cache, the settings file and the thumbnail cache keep their unsigned times, where 0 still means unknown.
- **Details pane on a picture:** the preview runs edge to edge with square corners, as tall as the picture's shape needs; the name is 15 px semibold beside a 32 px icon centered on it, and the Details rows are 22 px apart with their values at the pane's right edge, as measured on Windows 11.
- **Set as background, Rotate left and Rotate right** show in the command bar whenever pictures are selected, not only in Gallery. Set as background hands one selected picture to the desktop's Wallpaper portal, which shows its own preview and asks first; it is grayed where the desktop has no Wallpaper portal. With the Windows symbol font chosen, these commands draw their glyphs from it.
- **Menus measured against Windows 11:** context menu rows are 32 px apart and 41 px across a separator; New's menu opens 1 px left of the button; a chosen choice's dot sits where Windows 11 draws it.
- **Open and Save dialogs for other apps** are never titled with a filter's name: a title that is empty, names one of the request's filters or is Chromium's "All Files" gives way to Windows' own "Save As", "Open" or "Select Folder".
- **Smaller fixes:** a local folder with an unreadable subfolder reads "Indexed, some folders could not be read" instead of offline; Previous Versions of a local file shows only Windows' heading and says there are no previous versions in Windows' words; Alt+D, Ctrl+L, F4 and Ctrl+F no longer reach the window behind an in-window dialog; the Gallery year scale marks every year that has photos.
- For developers: Hidden now writes the folder's `.hidden` file instead of renaming the item; the search cache stores two new one-time markers in its existing table. Parity notes for ARC-023, PROP-002 and PROP-005 are updated. Two style commits satisfy Clippy.

## Why
- Compressing a folder left out links, FIFOs and sockets silently.
- GIO types every empty file as `application/x-zerosize`, which no app handles, so a new empty text document could not be opened.
- The portal's answer arrives within a millisecond, and the subscription by the portal's well-known name could miss it.
- The standard folders looked pinned but were not pins in the settings file, so a first run indexed nothing.
- Hidden renamed the item to a dot-name, breaking links and other apps' lists; in Windows, Hidden is an attribute.
- An unlistable local folder showed the network icon and GIO's raw text, which for a path past 4,096 bytes was the whole path, drawn selected.
- Dates at and before 1970 showed the unknown dash and sorted as unknown.
- The Details pane, menus and command bar differed from Windows 11's measures, Set as background was not available, and the rotate buttons borrowed Refresh's and Restore's glyphs.
- Chrome's Save page as was titled "All Files".

## CHANGED OWNER DECISIONS
- Open and Save dialog titles for other apps (parity note, owner decision changed 2026-11-23). Old: only a Save title naming an All Files filter gave way to Windows' title, so Chrome's Save page as was titled "All Files". New: a title that is empty, names one of the request's filters or is Chromium's "All Files" gives way to Windows' own "Save As", "Open" or "Select Folder".

## New and changed visible text
- "Some items were not compressed"
- "{name} was created without these items:\n\n{items}"
- "{name} ({reason})"
- "…and {count} more."
- "a link whose target cannot be found"
- "a link to a folder that holds it"
- "a link on a server or device"
- "a name that is not valid text"
- "a special file, which holds no data"
- "Opening your email app…"
- "No email app is set up, so the email could not be written. Choose an email app in your desktop's settings, then share again."
- "No folders are indexed yet. Add one below, or pin a folder to Quick access." → "No folders are indexed. Add one below. Folders pinned to Quick access are indexed while “Index pinned folders automatically” is on."
- "Indexed, some folders could not be read"
- "The item could not be hidden or shown. {error}"
- "The file name or path is too long."
- "This folder can't be found. It may have been moved or deleted."
- "This folder can't be opened."
- "The desktop background could not be set."
- "The picture could not be opened."
- Removed: "Compressed {item_count} items into {name}.", "{compressed} {skipped_count} links or special files were left out." and the tooltip "Setting the desktop background is not available yet."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: with Rust 1.92.0, as an unprivileged user with a throwaway HOME on a private Xvfb display and message bus: `archive_compress_links` (new; fails on the previous code), `archive_compress_sources`, `archive_progress`, `archive_tar_links` and the `archive::create` unit tests; the ox-core entry, email portal, search pins and places unit tests (28 tests in `admin_copy`, `admin_delete` and one clipboard test failed there, none in a changed module); ox-app tests of the changed modules: properties and item dialogs (153), search and indexing (71), Share, item dialogs, disconnected tabs, empty pages, address bar, input and search box (340), Gallery and scrubber (32), empty and unavailable pages (49), and after the date change grouping, sorting, model, Gallery, details pane, Recycle Bin, thumbnails and Details columns (428 passed, 2 bubblewrap thumbnailer sandbox tests failed); the details pane, command bar, context menu, New and View menu tests (259 of 259 and 285 of 287, the two failures needing the Python app sources or failing the same way without these changes); real input with xdotool in the full app with generated files (dates of 1960 and 1970, compressing, opening an empty file, Share with `xdg-desktop-portal` and its GTK backend and no email app, first-run indexing, Hidden, the error page, Alt+D behind a dialog, the Gallery scale); long names compared by script with Windows 11 captures; the `native/tools/check.py` inventory and driver tests, the drawn-icon guard and rustfmt; `native/parity/check.py`, `tools/i18n.py check`, `tools/sync-docs.py --check` and `tools/audit-public-data.py`. Not run: the whole ox-app test binary in one process, Wayland, HiDPI, the packaged .deb, a real email app.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
