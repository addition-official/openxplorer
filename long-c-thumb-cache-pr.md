# Title

feat(thumbnails): Show a revisited folder's thumbnails at once, from the shared cache

# Body

Model: claude-opus-5-5

CLA: I agree

## Summary

Windows keeps its thumbnails in `thumbcache_*.db`, so a folder of photos opened again shows them at once. OpenXplorer now does the same with the desktop's shared thumbnail cache, as the freedesktop Thumbnail Managing Standard lays it out (`~/.cache/thumbnails/{normal,large,x-large,xx-large}/<MD5 of the URI>.png`). Files, Dolphin and the thumbnailer service read and write that same cache.

- **The cache is looked up first, by the app itself.** Each entry's `Thumb::URI`, `Thumb::MTime` and (when present) `Thumb::Size` are read from its PNG text chunks in safe Rust, without decoding the picture. A stale entry or a recorded failure therefore costs no decode. A larger size stands in for a missing smaller one; a smaller one never stands in for a larger one, which would blur. Before, the lookup went through GIO's `thumbnail::*` attributes and only looked at the size GIO picked.
- **Cached reads still go through the sandboxed decoder pool, in batches.** The reads of one layout are gathered until the main loop is idle and sent 12 at a time to one decoder process in one request. Each answer is handed back as soon as it is decoded, the first tiles bound first. A picture that crashes its decoder is refused alone, and the rest of its batch goes to a new decoder.
- **Recently shown thumbnails stay in memory** (96 MB, least recently shown first out), checked against the file's time and size. A folder visited again in the same run, or a row scrolled back, draws its thumbnails on the frame it binds them. With network thumbnails off, as by default, this works however long ago its folder was found to be on this computer.
- **No icon flash.** The names of the cache's entries are listed once in the background, as soon as a view with thumbnails is set up. A tile whose thumbnail the cache holds then waits for it without its type icon (250 ms at most), instead of showing the icon and replacing it a moment later. Decoders are started in the background at the same time.
- **A thumbnail shows as soon as it is read.** It no longer waits for the custom-icon metadata query, which runs at low priority behind the layout. A custom icon found later takes the thumbnail's place.
- **New thumbnails never block the UI, and the visible ones come first.** They are made off the main thread on the same few slots as before. A request for a tile off the screen waits outside the queue while tiles on the screen wait. GTK binds about 190 tiles of Large icons for the 30 a window shows, and before, it made them in bind order. A thumbnail the app makes is shown from the picture it decoded, instead of being read back from disk. It costs one decode instead of three.
- **Writes follow the standard.** A new entry is encoded in the app, written to a temporary file created 0600 (exclusive, no symlink following) in a 0700 folder, then renamed into place. Entries now carry `Thumb::Size` and `Thumb::Mimetype` too.
- **Failures are recorded.** Damaged pictures, and files a desktop thumbnailer ran on and gave up on, are recorded in `thumbnails/fail/openxplorer/`. Together with GNOME's `fail/gnome-thumbnail-factory/`, they are not tried again until the file changes. A missing thumbnailer, a timeout or a cancelled request is not recorded.
- **Cache hygiene.**
  - **Changed files:** a file that changes (new time or size) misses its entry, gets a new one, and the new entry replaces the old one.
  - **Moves and renames:** a file the app moves or renames, or every file in a folder it moves or renames (up to 5,000), takes its thumbnails with it. This applies to Undo and Redo too. The entry is rewritten with the new URI and its old copy removed. The pixels are kept and nothing is decoded. An entry from another version, as after a copy that changed the time, is left alone.
  - **"Always show icons, never thumbnails":** with it on, nothing is read, listed, started or written.
  - **Cleanup:** removing old entries is left to the desktop's cleaners, as the standard intends.
- **Extra large icons at twice the pixels** use `x-large` (512 px). The thumbnailer service is asked for that size only when its `GetFlavors` lists it.

### Clearing the cache, as on Windows

Windows File Explorer has no "Clear thumbnail cache" command. The cache is cleared by Disk Cleanup or Settings → System → Storage → Temporary files ("Thumbnails"), and Windows rebuilds it while browsing. The Interface guide now says the same for OpenXplorer: GNOME removes thumbnails older than six months, and BleachBit, or deleting `~/.cache/thumbnails`, clears it at once. No button was added.

## Before and after

Run by `native/tools/thumbnail-bench.py` (added here) on a private Xvfb display, as an unprivileged user with a throwaway HOME (and so an empty cache), with real keyboard input through xdotool. Window 1400 × 950, Large icons, 30 tiles on screen (GTK binds about 190). Both builds use Rust 1.92.0 and carry the same trace instrumentation, which is the only change to upstream 5d21fd3e in the "before" build. Each figure is the median of 3 runs. "Photos" is 2,000 JPEGs of 2400 × 1600. "Mixed" is 120 files, by turns photos, MP4 videos (ffmpegthumbnailer) and PDFs (evince-thumbnailer), both thumbnailers in bubblewrap. "Restart" quits and starts the app again. "Cache drop" also drops the pages of the folder and the cache from memory (`posix_fadvise(DONTNEED)`), as a reboot would. "After the first tile" excludes the app's own start-up, which this change does not touch.

| Folder | Visit | All visible, from launch (ms) | All visible, after the first tile (ms) | Tile bound → its thumbnail, median (ms) | Visible tiles drawn with their thumbnail on their first frame | Visible tiles that never showed their type icon first | Pictures decoded | Thumbnailers started |
|---|---|---|---|---|---|---|---|---|
| Photos | Cold | 2,294 → 1,681 | 1,084 → 512 | 747 → 386 | 0 → 0 of 30 | 0 → 0 of 30 | 579 → 193 | 0 → 0 |
| Photos | Same-session revisit | 163 → 74 | 89 → 3 | 88 → 0 | 0 → 30 of 30 | 0 → 30 of 30 | 0 → 0 | 0 → 0 |
| Photos | Restart | 688 → 432 | 278 → 58 | 204 → 35 | 0 → 0 of 30 | 0 → 30 of 30 | 193 → 195 (18 batches) | 0 → 0 |
| Photos | Cache drop | 747 → 446 | 263 → 60 | 206 → 35 | 0 → 0 of 30 | 0 → 30 of 30 | 193 → 195 (18 batches) | 0 → 0 |
| Mixed | Cold | 2,928 → 1,162 | 2,510 → 796 | 1,444 → 483 | 0 → 0 of 30 | 0 → 0 of 30 | 360 → 120 | 80 → 80 |
| Mixed | Same-session revisit | 113 → 52 | 63 → 2 | 62 → 0 | 0 → 30 of 30 | 0 → 30 of 30 | 0 → 0 | 0 → 0 |
| Mixed | Restart | 632 → 397 | 218 → 39 | 168 → 34 | 0 → 0 of 30 | 0 → 30 of 30 | 120 → 120 (11 batches) | 0 → 0 |
| Mixed | Cache drop | 625 → 394 | 283 → 37 | 209 → 33 | 0 → 0 of 30 | 0 → 30 of 30 | 120 → 120 (11 batches) | 0 → 0 |

- **Restarts and cache drops:** the cache is now read in batches of up to 12 per decoder request. Before, each tile cost one GIO query and one decoder request.
- **Cold visits:**
  - Each new photo costs one decode instead of three. Before, there was a look for an earlier thumbnail that decoded a missing file, then the decode, then a read-back of the written PNG.
  - Visible tiles are made first. Over all 193 bound tiles, the cold photo folder takes as long as before (about 2.2 s).
- **Paging down through a new folder** of 2,000 photos for 4.7 s while its thumbnails are made: the main loop missed its 5 ms timer for 34 ms or more 0, 1 and 3 times (longest 37 ms), against 1, 0 and 0 (35 ms) before. About 577 thumbnails were made meanwhile in both.
- **First frame after a restart:** after a restart, a tile still cannot show its thumbnail on its very first frame. The read takes 35 ms or so, and holding the window's first frame back for it would be slower overall. The tile shows nothing in the icon's place until then, so there is no icon flash.

## NOT CONFIRMED

- Whether Windows 11 File Explorer draws a cached folder's thumbnails on its first frame after a restart, or shows blank tiles or icons first.
- The default of GNOME's thumbnail housekeeping (`org.gnome.desktop.thumbnail-cache` `maximum-age`, 180 days) on Zorin OS 17/18.
- Whether tumbler on XFCE lists `x-large` in `GetFlavors`. Without it, the app makes x-large thumbnails itself or with the desktop's thumbnailers.
- That Dolphin finds the app's entries valid. GIO, and so Files, does: `gio info` reports `thumbnail::is-valid: TRUE` for the app's entries, and FALSE once the file changes.

## Question

Thumbnails of files that were deleted or sent to the Recycle Bin stay in the cache. The standard says to remove them, GNOME and KDE mostly don't, and Windows keeps them in `thumbcache_*.db` (a known privacy complaint). This change removes only the entries of files the app moves. Should it also remove the entries of files it deletes or moves to the Recycle Bin?

## How it was tested

- **Tests first:**
  - New core tests in `ox-core/src/thumbnail_cache.rs`: a current entry of another program is a hit; changed time, size or URI misses; a larger size serves a smaller one; damaged, linked and oversized entries miss without panicking; failures hold until the file changes, including GNOME's; writes are atomic and 0600; keys are replaced with the pixels kept; a moved file keeps its thumbnails; the cache's names are listed. All were seen to fail before the implementation.
  - Also in ox-core: x-large and xx-large flavors, and the moved items of rename, move and Replace records.
  - App tests: a batch decoded by one decoder with a crashing picture refused alone; the memory's eviction and version checks; a cached thumbnail shown until its file changes, then made again and its entry replaced; a damaged picture recorded in `fail/` and a recorded failure not tried again until the file changes; a renamed folder keeping its files' thumbnails; a dropped network check not leaving other tiles waiting; tiles on the screen taking slots first.
  - The existing test that "Always show icons" turns thumbnails off everywhere still passes.
- **Every test binary** was run through `check.py`'s own isolation (`run_isolated`) as an unprivileged user, because the app refuses to run as root and `check.py` stops there in this container. 4,995 tests passed and 4 failed.
  - The two sandboxed-thumbnailer tests fail identically on unchanged upstream as the same user, because bubblewrap cannot run in this container.
  - The two Python-app comparison tests, `location_fixture_drift` and `network_python`, touch nothing changed here; the first is already noted as failing on main in this environment.
  - These are the results at 9e992f6a. The ox-app suite was run again at the last commit, which changed only ox-app: 2,647 passed, 2 failed (the same sandbox tests).
  - The doctests pass.
- **Other checks:** `check.py` up to the Rust tests (the parity tests and `parity/check.py`, i18n, the driver tests, the drawn-icon guard, rustfmt and Clippy with `-D warnings` on Rust 1.92.0) passes, and `parity/check.py` passes alone. `tools/sync-docs.py --check` passes after regenerating the guides from `docs.json`.
- **Benchmark:** the table above (`thumbnail-bench.py --scroll`).
- **Interop:** the release build was run by hand on a private Xvfb display as an unprivileged user with a throwaway HOME, then `gio info` was checked on its entries (0600 files in 0700 folders, with all four keys).
- The website's pictures of the app were not changed, and no website build was run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
