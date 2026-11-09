# Title

fix(recent): Open a Recent files row as the file it stands for

# Body

Model: claude-opus-5-5

## What was wrong

Double-clicking a file in the navigation pane's Recent files (or pressing Enter, or choosing Open) showed "Could not open the item - Only local paths, smb:// locations and connected devices are supported in this build." The same file opened fine from its own folder.

A Recent files row is GVfs's `recent:///<hash>` item. The app already kept the real file's address in `entry.meta.recent_target` (Open file location, Send to and Share use it), but opening the row read and launched the row's own `recent:` address, which the opener refuses. Copy, Copy as path, a drag, Properties and Open with had the same problem: they used the `recent:` address, so Copy as path copied "Recent / <hash>", Properties showed the row, Copy and the drag were refused.

## What changes

- `Entry::opened_uri()` and `Entry::item_uri()` (ox-core) give the file a Recent row stands for, and the row's own address for everything else. `navigation_uri()` is unchanged.
- Double-click, Enter, Open, Open with (menu apps and Choose another app), Copy, Copy as path, a drag and Properties on a Recent row now act on that file, exactly as from its folder: the same default app, the same mount-then-read rule for an smb:// file, the same drag rules (a drag onto a folder on the same drive moves the file, as it does from the file's folder).
- When the file is gone, opening the row or Open with on it shows the app's existing "Item Not Found" / "Could not find this item" / "This is no longer located in <folder>. Verify the item's location and try again." wording (the progress window's existing strings; no new text to translate).
- A Recent files row may be copied, not cut, renamed or deleted, as a file in Windows 11's Home Recent list (checked in a Windows 11 VM): Cut, Duplicate, Rename and Delete are greyed out in the command bar, left out of the right-click menu's strip (which keeps Copy and Share), and Ctrl+X, F2 and Delete do nothing, with no message. No other menu items were added or removed. A new disabled-reason tooltip, "Items in Recent files can be opened and copied, not changed.", is the one new string.
- A share unmounted since the listing is mounted first, as opening a file does, for Open with (menu app and Choose another app) when several Recent rows are opened together, and before several rows' Properties; several rows are each read again and opened as their files, one after another so tabs keep the view's order (a `.desktop` link is followed). Properties already mounted first.
- Several Recent rows' Properties describe the files (`Entry::as_item`): location, size on disk and Read-only/Hidden use the files' addresses.
- A Recent row that GVfs types as a shortcut is copied as its file, as a file-typed row is.
- Recent locations rows are the real folders already; they open in place, in a new tab and in a new window. Tests now cover that.
- `native/parity/features.toml` SIDE-026 note updated.

## How it was tested

Tests first: `native/crates/ox-app/src/window/tests/recent_rows.rs` (18 tests). Each fix had a test that failed first. On upstream main 016c335 the 9 original bug tests fail (open, Item Not Found, Copy as path, Copy, drag, Properties, Open with, Open with on a gone file, Cut/Rename/Delete greyed out) and the Recent locations test passes (that part already worked); with this branch all 18 pass. The 8 tests added after the two review rounds failed on the head before them, except the single-item Properties mount test, which already passed and is kept as a guard. They run under Xvfb with a private D-Bus session and GVfs's real Recent backend, with fictional files.

- `cargo fmt --check` and the Clippy command from CONTRIBUTING.md: clean.
- `python3 native/parity/check.py`, `python3 native/tools/i18n.py check`, `python3 tools/audit-public-data.py`: pass.
- `python3 native/tools/check.py` (run with Python 3.12 as `python3`, as on Ubuntu 24.04): every stage and test binary passed except 2 of 2,119 ox-app tests, `a_sandboxed_thumbnailer_runs_within_its_limits` and `a_sandboxed_thumbnailer_sees_only_its_item`. Both fail the same way on unmodified upstream main in this container; they only run where bubblewrap is installed, and this change does not touch thumbnails. With the container's Python 3.13 as `python3`, ox-core's `location_fixture_drift` stops the run first (Python 3.13's URL parser now accepts `smb://[V1.x]/`); that test is unrelated to this change.
- Manual run with real input (xdotool) under Xvfb with openbox, a fictional `recently-used.xbel` and mock viewer/editor apps: double-click, Enter, Open, Open with, Copy, Copy as path, Properties and a drag to a folder on a local Recent file, on a Recent file whose folder was moved away, and on a Recent locations folder; the folder also in place, new tab and new window; and on a Recent file, Cut/Rename/Delete greyed in the command bar, a strip with only Copy, and Ctrl+X, F2 and Delete doing nothing. All passed.
- Not tested: a Recent row for an smb:// file. GVfs 1.54's Recent backend in this container only lists `file:` entries, and there is no GVfs FUSE here, so such a row could not be made. The code path for it is the existing open-from-folder path.

Pictures (1x, light and dark, fictional demo tree, isolated the way `tools/native_capture.py` does): a Recent file opened in a mock viewer, and "Item Not Found" for a Recent file whose folder was moved.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
