# Title

feat(thumbnails): Show a revisited folder's thumbnails at once, from the shared cache

# Body

Model: claude-opus-5-5

CLA: I agree

## Merge with the passes (efb363f1, a365af0f)

origin/pass-19-a11y (passes 1–19 and network-like-windows) is merged, with one memory design:

- **One store of previews:** this branch's LRU (64 MB, each preview at the size it is shown) replaces the passes' 512-preview, 64 MB store.
- **The passes' 30 s rule stays:** it empties that store and the Gallery's sharp pictures once no view has asked for a preview for 30 seconds, and it now also stops the decoders started for batched cache reads and the warm-up ones.
- **Revisits after 30 s idle:** a folder revisited after that reads its thumbnails back from the shared cache, in about 150 ms with no icon flash, instead of drawing them on the first frame. Keeping the 30 s release was decided over keeping previews longer, so memory comes back down.
- **Flavors:** the passes' `XXLarge` flavor and `makes` check are kept.
- **Tests:** the passes' budget and larger-view tests are ported to the cache.

## Summary

Windows keeps its thumbnails in `thumbcache_*.db`, so a folder of photos opened again shows them at once. OpenXplorer now does the same with the desktop's shared thumbnail cache, as the freedesktop Thumbnail Managing Standard lays it out (`~/.cache/thumbnails/{normal,large,x-large,xx-large}/<MD5 of the URI>.png`). Files, Dolphin and the thumbnailer service read and write that same cache.

- **The cache is looked up first, by the app itself.** Each entry's `Thumb::URI`, `Thumb::MTime` and (when present) `Thumb::Size` are read from its PNG text chunks in safe Rust, without decoding the picture. A stale entry or a recorded failure therefore costs no decode. A larger size stands in for a missing smaller one; a smaller one never stands in for a larger one, which would blur. Before, the lookup went through GIO's `thumbnail::*` attributes and only looked at the size GIO picked.
- **Cached reads still go through the sandboxed decoder pool, in batches.** The reads of one layout are gathered until the main loop is idle and sent 12 at a time to one decoder process in one request. Each answer is handed back as soon as it is decoded, the first tiles bound first. A picture that crashes its decoder is refused alone, and the rest of its batch goes to a new decoder.
- **Recently shown thumbnails stay in memory** (64 MB, each kept at the size it is shown, least recently shown first out; about 2,700 photos at Large icons), checked against the file's time and size. A folder visited again in the same run, or a row scrolled back, draws its thumbnails on the frame it binds them. With network thumbnails off, as by default, this works however long ago its folder was found to be on this computer.
- **No icon flash.** The names of the cache's entries are listed once in the background, as soon as a view with thumbnails is set up. They are kept as 128-bit numbers: 5.6 MB for a cache of 200,000 entries. A tile whose thumbnail the cache holds then waits for it without its type icon (250 ms at most), instead of showing the icon and replacing it a moment later. Decoders are started in the background at the same time.
- **A thumbnail shows as soon as it is read.** It no longer waits for the custom-icon metadata query, which runs at low priority behind the layout. A custom icon found later takes the thumbnail's place.
- **New thumbnails never block the UI, and the visible ones come first.** They are made off the main thread, one fewer at a time than the processors (2 or 3; 2 before). A request for a tile off the screen waits outside the queue while tiles on the screen wait. GTK binds about 190 tiles of Large icons for the 30 a window shows, and before, it made them in bind order. A thumbnail the app makes is shown from the picture it decoded, instead of being read back from disk. It costs one decode instead of three.
- **Writes follow the standard.** A new entry is encoded in the app, written to a temporary file created 0600 (exclusive, no symlink following) in a 0700 folder, then renamed into place. Entries now carry `Thumb::Size` and `Thumb::Mimetype` too.
- **Failures are recorded.** Damaged pictures, and files a desktop thumbnailer ran on and gave up on, are recorded in `thumbnails/fail/openxplorer/`. Together with GNOME's `fail/gnome-thumbnail-factory/`, they are not tried again until the file changes. A missing thumbnailer, a timeout or a cancelled request is not recorded.
- **Cache hygiene.**
  - **Changed files:** a file that changes (new time or size) misses its entry, gets a new one, and the new entry replaces the old one.
  - **Moves and renames:** a file the app moves or renames, or every file in a folder it moves or renames (up to 5,000), takes its thumbnails with it. This applies to Undo and Redo too. The entry is rewritten with the new URI and its old copy removed. The pixels are kept and nothing is decoded. An entry from another version, as after a copy that changed the time, is left alone.
  - **"Always show icons, never thumbnails":** with it on, nothing is read, listed, started or written.
  - **Cleanup:** removing old entries is left to the desktop's cleaners, as the standard intends.
- **Fixes from an independent review of the diff.**
  - A tile GTK bound to another item while its thumbnail was being made could show the old item's picture, because glib detaches a task whose handle is dropped. The reading now stops with its lookup and only paints a cell still bound to its item. A test reproduces the bug with the old code.
  - A custom icon that cannot be drawn no longer hides the thumbnail.
  - Keys are also read from compressed (`zTXt`) and international (`iTXt`) chunks, within 64 KB inflated. Qt, which writes Dolphin's thumbnails, compresses values of 40 characters or more.
  - A batch of moves reads every entry before writing, so files that swap names keep their own.
  - Moves are not followed over network file systems.
  - A picture a decoder refuses is tried once more, in another decoder, before it is recorded as damaged. A decoder that used up its processor time refuses the same way.
- **Extra large icons at twice the pixels** use `x-large` (512 px). The thumbnailer service is asked for that size only when its `GetFlavors` lists it.

### Clearing the cache, as on Windows

Windows File Explorer has no "Clear thumbnail cache" command. The cache is cleared by Disk Cleanup or Settings → System → Storage → Temporary files ("Thumbnails"), and Windows rebuilds it while browsing. The Interface guide now says the same for OpenXplorer: GNOME removes thumbnails older than six months, and BleachBit, or deleting `~/.cache/thumbnails`, clears it at once. No button was added.

## Before and after

Run by `native/tools/thumbnail-bench.py` (added here) on a private Xvfb display, as an unprivileged user with a throwaway HOME (and so an empty cache), with real keyboard input through xdotool. Window 1400 × 950, Large icons, 30 tiles on screen (GTK binds about 190). Both builds use Rust 1.92.0 and carry the same trace instrumentation, which is the only change to upstream 5d21fd3e in the "before" build. Each figure is the median of 3 runs. "Photos" is 2,000 JPEGs of 2400 × 1600. "Mixed" is 120 files, by turns photos, MP4 videos (ffmpegthumbnailer) and PDFs (evince-thumbnailer), both thumbnailers in bubblewrap. "Restart" quits and starts the app again. "Cache drop" also drops the pages of the folder and the cache from memory (`posix_fadvise(DONTNEED)`), as a reboot would. "After the first tile" excludes the app's own start-up, which this change does not touch.

| Folder | Visit | All visible, from launch (ms) | All visible, after the first tile (ms) | Tile bound → its thumbnail, median (ms) | Visible tiles drawn with their thumbnail on their first frame | Visible tiles that never showed their type icon first | Pictures decoded | Thumbnailers started |
|---|---|---|---|---|---|---|---|---|
| Photos | Cold | 2,338 → 1,742 | 1,114 → 511 | 744 → 356 | 0 → 0 of 30 | 0 → 0 of 30 | 579 → 193 | 0 → 0 |
| Photos | Same-session revisit | 161 → 73 | 89 → 3 | 88 → 0 | 0 → 30 of 30 | 0 → 30 of 30 | 0 → 0 | 0 → 0 |
| Photos | Restart | 673 → 500 | 263 → 67 | 201 → 37 | 0 → 0 of 30 | 0 → 30 of 30 | 193 → 195 (18 batches) | 0 → 0 |
| Photos | Cache drop | 780 → 621 | 282 → 67 | 212 → 38 | 0 → 0 of 30 | 0 → 30 of 30 | 193 → 195 (18 batches) | 0 → 0 |
| Mixed | Cold | 3,196 → 1,081 | 2,799 → 672 | 1,581 → 399 | 0 → 0 of 30 | 0 → 0 of 30 | 360 → 120 | 80 → 80 |
| Mixed | Same-session revisit | 124 → 64 | 78 → 4 | 61 → 0 | 0 → 30 of 30 | 0 → 30 of 30 | 0 → 0 | 0 → 0 |
| Mixed | Restart | 647 → 493 | 236 → 73 | 183 → 27 | 0 → 0 of 30 | 0 → 30 of 30 | 120 → 144 (in batches) | 0 → 0 |
| Mixed | Cache drop | 695 → 473 | 302 → 60 | 237 → 25 | 0 → 0 of 30 | 0 → 30 of 30 | 120 → 144 (in batches) | 0 → 0 |

Medians of 6 runs each (12 runs give the same picture), measured at 00b00987, the head of the branch, with the two builds run by turns. Sets taken earlier in the night at 9f5546de gave the same picture; the virtual machine's speed drifted over the night. The extra decodes after a restart of the mixed folder are tiles GTK bound twice while laying out, whose reads had already been sent.

Measured at 00b00987, the head of the branch, with the two builds run by turns, 3 times each, in the same hour. Sets taken earlier in the night at 9f5546de gave the same picture; the virtual machine's speed drifted over the night. The extra decodes after a restart of the mixed folder are tiles GTK bound twice while laying out, whose reads had already been sent.

The two builds were run by turns, 3 times each, in the same hour. Earlier sets taken hours before gave the same picture with lower times all round; the virtual machine's speed drifted over the night. The extra decodes after a restart of the mixed folder are tiles GTK bound twice while laying out, whose reads had already been sent.

- **Restarts and cache drops:** the cache is now read in batches of up to 12 per decoder request. Before, each tile cost one GIO query and one decoder request.
- **Cold visits:**
  - Each new photo costs one decode instead of three. Before, there was a look for an earlier thumbnail that decoded a missing file, then the decode, then a read-back of the written PNG.
  - Visible tiles are made first.
  - Thumbnails are made one fewer at a time than the processors, between 2 and 3: 3 on this 4-processor machine. In an earlier set, 2 slots gave 512 ms (photos) and 796 ms (mixed) after the first tile, against 464 ms and 555 ms with 3. A 2-processor computer keeps 2.
- **Paging down through a new folder** of 2,000 photos for 4.7 s while its thumbnails are made, 16 interleaved pairs at the head of the branch (12 within the full runs, 4 alone). The main loop missed its 5 ms timer for 34 ms or more 57 times in all before (longest 49 ms), against 29 after (longest 53 ms, in one run). About 577 thumbnails were made meanwhile in both.
- **Revisit after 40 s** (past the 30 s the network check is trusted for), 300 photos, real input: visible tiles drawn with their thumbnail on their first frame go from 0 to 30 of 30, and all visible arrive 110 → 3 ms after the first tile.
- **Memory:** resident memory after scrolling through the cold 2,000-photo folder and back is 341 MB before and 357 MB after. After a restart, with the cache warm, it is 267 MB before and 286 MB after. These are single runs measured from `/proc/<pid>/status`; the picture decoders are separate processes and are not counted.
- **First frame after a restart:** after a restart, a tile still cannot show its thumbnail on its very first frame. The read takes 35 ms or so, and holding the window's first frame back for it would be slower overall. The tile shows nothing in the icon's place until then, so there is no icon flash.

## NOT CONFIRMED

- Whether Windows 11 File Explorer draws a cached folder's thumbnails on its first frame after a restart, or shows blank tiles or icons first.
- The default of GNOME's thumbnail housekeeping (`org.gnome.desktop.thumbnail-cache` `maximum-age`, 180 days) on Zorin OS 17/18.
- That Qt compresses text values of 40 characters or more into `zTXt`, as I recall from its PNG writer. Either way, both kinds of chunk are read now.
- That Dolphin finds the app's entries valid. GIO, and so Files, does: `gio info` reports `thumbnail::is-valid: TRUE` for the app's entries, and FALSE once the file changes.

## Known limitation

With a cache of 200,000 other entries, the thumbnails still arrive 46 to 69 ms after the first tile after a restart, against 261 to 306 ms on upstream with the same cache. But in 7 of 10 restarts the first folder showed its type icons for that moment: none of its 30 tiles avoided the flash. The other 3 restarts, and every run with a small cache, had no flash. The background listing of the cache's names, which tells a tile to wait without its icon, races the first layout. Starting that listing at application start-up did not help (measured and reverted). A temporary trace point, not committed, shows why: the 200,193 names are listed 264 ms after the warm-up starts, and the first tiles are bound at 218 ms. A follow-up could hold the type icons for at most 250 ms while that listing is still running.

## Decision on deleted files

Thumbnails of files that were deleted or sent to the Recycle Bin stay in the cache, as Windows keeps them in `thumbcache_*.db`. Only moves and renames update entries. This was decided over the standard's advice to remove them.

## How it was tested

- **Tests first:**
  - New core tests in `ox-core/src/thumbnail_cache.rs`: a current entry of another program is a hit; changed time, size or URI misses; a larger size serves a smaller one; damaged, linked and oversized entries miss without panicking; failures hold until the file changes, including GNOME's; writes are atomic and 0600; keys are replaced with the pixels kept; a moved file keeps its thumbnails; the cache's names are listed. All were seen to fail before the implementation, as was the test of the cache names kept as numbers. The app tests below were written together with their code.
  - Also in ox-core: x-large and xx-large flavors, and the moved items of rename, move and Replace records.
  - App tests: a batch decoded by one decoder with a crashing picture refused alone; the memory's eviction and version checks; a cached thumbnail shown until its file changes, then made again and its entry replaced; a damaged picture recorded in `fail/` and a recorded failure not tried again until the file changes; a renamed folder keeping its files' thumbnails; a dropped network check not leaving other tiles waiting; tiles on the screen taking slots first.
  - The existing test that "Always show icons" turns thumbnails off everywhere still passes.
- **Every test binary** was run through `check.py`'s own isolation (`run_isolated`) as an unprivileged user, because the app refuses to run as root and `check.py` stops there in this container. At 00b00987, the head of the branch, 4,999 tests passed and 4 failed.
  - The two sandboxed-thumbnailer tests fail identically on unchanged upstream as the same user, because bubblewrap cannot run in this container.
  - The two Python-app comparison tests, `location_fixture_drift` and `network_python`, touch nothing changed here; the first is already noted as failing on main in this environment.
  - One earlier run of the ox-app suite also failed `home_has_no_band_under_the_titles_and_its_tiles_do_not_touch` once. It is a timing flake: it passes alone 3 of 3 times and in every other full run.
  - The doctests pass.
- **Other checks:** `check.py` up to the Rust tests (the parity tests and `parity/check.py`, i18n, the driver tests, the drawn-icon guard, rustfmt and Clippy with `-D warnings` on Rust 1.92.0) passes, and `parity/check.py` passes alone. `tools/sync-docs.py --check` passes after regenerating the guides from `docs.json`.
- **Benchmark:** the table above (`thumbnail-bench.py --scroll`), at the head of the branch.
- **Interop:** the release build was run by hand on a private Xvfb display as an unprivileged user with a throwaway HOME, then `gio info` was checked on its entries (0600 files in 0700 folders, with all four keys).
- **Tumbler 4.18.1**, D-Bus activated on a private session bus with the throwaway HOME. `GetFlavors` lists all four sizes. At 1x Large icons it makes `normal` thumbnails, and at 2x Extra large icons `x-large` ones (512 px), of 6 photos and an MP4. The app shows them all, and on the next launch reads all 7 from the cache without asking tumbler again. Tumbler writes `Thumb::MTime` with microseconds (`1795342906.979486`), which the app reads to the second.
- **Real input, rename** (Xvfb, xdotool): a folder of 20 photos was opened, then Alt+Up, F2, "Trip 2026" and Enter, then opened again. All 20 entries moved with it, all 20 thumbnails were read from the cache, none was made again, and the cache holds 20 entries in all.
- **The real app with "Always show icons, never thumbnails" on:** no thumbnail was requested, no decoder was started, and no thumbnails folder was created.
- **The real app's fail cache:** a PNG cut short is tried once and recorded in `fail/openxplorer` on the first launch. On the second launch it is answered from that entry, with no decode and nothing made, and the good PNG beside it is read from the cache.
- **`preview-smoke.py`** (300 real and mangled files of 18 kinds, through the Preview and Details panes): the app keeps running, with no bad output.
- **The same mangled files in Large icons**, paged through twice: the app keeps running, with no bad output. On the first visit 44 broken files are recorded in `fail/`. On the second, no thumbnailer is started for them and all 44 are answered from `fail/`, where upstream started 31 sandboxed thumbnailers again.
- **2x scale by hand** (`GDK_SCALE=2`): Large icons write `large` entries, and Extra large icons write 512 × 341 `x-large` entries.
- The website's pictures of the app were not changed, and no website build was run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
