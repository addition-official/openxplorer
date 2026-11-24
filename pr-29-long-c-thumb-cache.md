# Title

feat(thumbnails): Show a revisited folder's thumbnails at once from the shared cache, and a folder's contents in its icon

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/28-review-6-fixes)

This PR comes after PR 28 because its last feature commit, the folder-contents preview, needs PR 28's signed file times.

## What changes for the user
- **A folder of photos opened again shows its thumbnails at once.** The app uses the desktop's shared thumbnail cache as the freedesktop Thumbnail Managing Standard lays it out (`~/.cache/thumbnails/{normal,large,x-large,xx-large}/<MD5 of the URI>.png`), which Files, Dolphin and the thumbnailer service also read and write, as Windows keeps `thumbcache_*.db`.
- **The cache is looked up first, by the app itself.** Each entry's `Thumb::URI`, `Thumb::MTime` and `Thumb::Size` are read from its PNG text chunks in safe Rust (also from compressed `zTXt` and international `iTXt` chunks, within 64 KB inflated), without decoding the picture. A larger size stands in for a missing smaller one, never the other way round.
- **Cached reads go through the sandboxed decoder pool in batches** of up to 12 per decoder request, each answer handed back as soon as it is decoded. A picture that crashes its decoder is refused alone and tried once more in another decoder before it is recorded as damaged.
- **Recently shown thumbnails stay in memory** (64 MB, each kept at the size it is shown, least recently shown first out), checked against the file's time and size. This one store replaces the earlier 512-preview store; the 30-second rule that lets previews and idle decoders go once nothing has asked for a preview stays, and now also stops the decoders started for batched reads.
- **No icon flash.** The cache's entry names are listed once in the background (kept as 128-bit numbers), so a tile whose thumbnail the cache holds waits for it without its type icon (250 ms at most). A thumbnail shows as soon as it is read, without waiting for the custom-icon query; a custom icon found later takes its place, and one that cannot be drawn no longer hides the thumbnail.
- **Visible tiles first, and new thumbnails off the main thread** one fewer at a time than the processors, between 2 and 3 (2 before). A tile off the screen waits while tiles on the screen wait, and the screen's tiles are told only once GTK has laid them out. A thumbnail the app makes is shown from the picture it decoded: one decode instead of three.
- **Writes follow the standard:** a new entry is written to a temporary file created 0600 (exclusive, no symlink following) in a 0700 folder, then renamed into place, and carries `Thumb::Size` and `Thumb::Mimetype` too.
- **Failures are recorded** in `thumbnails/fail/openxplorer/` and, with GNOME's `fail/gnome-thumbnail-factory/`, not tried again until the file changes. A missing thumbnailer, a timeout or a canceled request is not recorded.
- **Moves and renames keep thumbnails.** A file the app moves or renames, or every file in a folder it moves or renames (up to 5,000), takes its thumbnails with it, Undo and Redo included; a batch reads every entry before writing, and moves over network file systems are not followed. Thumbnails of deleted files stay in the cache, as Windows keeps them; cleanup is left to the desktop's cleaners. With "Always show icons, never thumbnails" on, nothing is read, listed, started or written.
- **A tile bound to another item** while its thumbnail was being made no longer shows the old item's picture.
- **Extra large icons at twice the pixels** use `x-large` (512 px), asked of the thumbnailer service only when its `GetFlavors` lists it.
- **Office documents** from PR 23 take the cache's path: a document in a small row keeps its type icon and nothing is looked up for it; its thumbnail is read from the cache first; one that could not be drawn otherwise goes to LibreOffice, one at a time, tiles on the screen first.
- **A folder shows its contents in its icon** at Medium icons and larger and in Tiles, as Windows 11 does: the thumbnails of its first two pictures or videos by name peek out between the folder's back and a lower front, the second behind the first and slightly turned; with no picture, the icon of its first file peeks out as a page; an empty folder, or one with only folders, stays plain, as does every folder in Details, List, Small icons and Content, with "Always show icons, never thumbnails" on, and on a network while network thumbnails are off. The folder is read off the main thread after its own icon, kept by its modification time, and placed by fractions measured on Windows reference screenshots (`native/tools/measure-folder-previews.py`). The app's own folder icon is drawn behind and, cut along the front's edge, in front of the contents; with Windows icon files on, imageres groups 5 and 6 are used as the back and front when the files hold them (a best match, not confirmed in Windows 11).
- The Interface guide now says how to clear thumbnails, as on Windows: GNOME removes thumbnails older than six months, and BleachBit, or deleting `~/.cache/thumbnails`, clears them at once. No button was added.
- For developers: new `ox_core::thumbnail_cache`, `thumbnails/{memory,reader,folder_preview,trace}.rs`, and `native/tools/thumbnail-bench.py`, which counts main-loop stalls and can scroll. The PR also carries a Clippy 1.92 fix in three files (`properties/general_tab.rs`, `properties/item_window.rs`, `entry/enumerate.rs`), its own "Drop redundant tests" pruning, and a rustfmt-only commit for the folder-contents preview code.

## Why
- A folder visited again redrew every thumbnail from a GIO lookup and a decode, and its tiles showed their type icons first.
- Before, the lookup went through GIO's `thumbnail::*` attributes and only looked at the size GIO picked; a stale entry or a recorded failure still cost a decode.
- New thumbnails were made in GTK's bind order (about 190 tiles bound for the 30 a window shows), and each new photo cost three decodes.
- A tile rebound while its thumbnail was being made could show the wrong picture, because glib detaches a task whose handle is dropped.
- Windows 11 shows a folder's contents in its icon in the larger views.

## CHANGED OWNER DECISIONS
None. (The parity note's open item "Missing, needs an owner decision because it changes the Windows 11 folder art: previews of a folder's contents inside its folder icon" is replaced by the description of the folder-contents previews this PR adds.)

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: new core tests in `ox-core/src/thumbnail_cache.rs` (hits, misses on changed time, size or URI, larger serving smaller, damaged, linked and oversized entries, failures held until the file changes, atomic 0600 writes, keys replaced with pixels kept, moves, listing), seen failing before the implementation; app tests for batched decoding with a crashing picture, the memory's eviction and version checks, a cached thumbnail replaced when its file changes, the fail records, a renamed folder keeping its thumbnails, a dropped network check, and on-screen tiles taking slots first; the folder preview tests in `thumbnails/folder_preview.rs`; every test binary through check.py's isolation as an unprivileged user (4,999 passed and 4 failed: the two sandboxed-thumbnailer tests, failing the same way on upstream because bubblewrap cannot run there, and the Python-app comparison tests `location_fixture_drift` and `network_python`); doctests; check.py up to the Rust tests (parity tests, `parity/check.py`, i18n, driver tests, drawn-icon guard, rustfmt and Clippy with `-D warnings` on Rust 1.92.0) and `tools/sync-docs.py --check`; `native/tools/thumbnail-bench.py --scroll` on a private Xvfb display with real input (2,000 photos at Large icons: visible tiles drawn with their thumbnail on their first frame on a revisit while the app runs went from 0 to 30 of 30); `gio info` on the app's entries; Tumbler 4.18.1 on a private message bus; real-input rename of a folder of 20 photos; the real app with "Always show icons, never thumbnails" on; the fail cache across two launches; `preview-smoke.py` (300 real and mangled files of 18 kinds) and the same files paged through in Large icons; `GDK_SCALE=2` by hand. Not confirmed: whether Windows 11 draws a cached folder's thumbnails on its first frame after a restart, and whether Dolphin finds the app's entries valid. Known limitation: with a cache of 200,000 other entries, the first folder after a restart showed its type icons briefly in 7 of 10 restarts.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
