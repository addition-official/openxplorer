# Title

feat(thumbnails): Show a revisited folder's thumbnails at once, from the shared cache

# Body

Model: claude-opus-5-5

CLA: I agree

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
- **Extra large icons at twice the pixels** use `x-large` (512 px). The thumbnailer service is asked for that size only when its `GetFlavors` lists it.

### Clearing the cache, as on Windows

Windows File Explorer has no "Clear thumbnail cache" command. The cache is cleared by Disk Cleanup or Settings → System → Storage → Temporary files ("Thumbnails"), and Windows rebuilds it while browsing. The Interface guide now says the same for OpenXplorer: GNOME removes thumbnails older than six months, and BleachBit, or deleting `~/.cache/thumbnails`, clears it at once. No button was added.

## Before and after

Run by `native/tools/thumbnail-bench.py` (added here) on a private Xvfb display, as an unprivileged user with a throwaway HOME (and so an empty cache), with real keyboard input through xdotool. Window 1400 × 950, Large icons, 30 tiles on screen (GTK binds about 190). Both builds use Rust 1.92.0 and carry the same trace instrumentation, which is the only change to upstream 5d21fd3e in the "before" build. Each figure is the median of 3 runs. "Photos" is 2,000 JPEGs of 2400 × 1600. "Mixed" is 120 files, by turns photos, MP4 videos (ffmpegthumbnailer) and PDFs (evince-thumbnailer), both thumbnailers in bubblewrap. "Restart" quits and starts the app again. "Cache drop" also drops the pages of the folder and the cache from memory (`posix_fadvise(DONTNEED)`), as a reboot would. "After the first tile" excludes the app's own start-up, which this change does not touch.

| Folder | Visit | All visible, from launch (ms) | All visible, after the first tile (ms) | Tile bound → its thumbnail, median (ms) | Visible tiles drawn with their thumbnail on their first frame | Visible tiles that never showed their type icon first | Pictures decoded | Thumbnailers started |
|---|---|---|---|---|---|---|---|---|
| Photos | Cold | 2,535 → 2,134 | 1,065 → 584 | 710 → 397 | 0 → 0 of 30 | 0 → 0 of 30 | 579 → 193 | 0 → 0 |
| Photos | Same-session revisit | 141 → 77 | 82 → 3 | 81 → 0 | 0 → 30 of 30 | 0 → 30 of 30 | 0 → 0 | 0 → 0 |
| Photos | Restart | 698 → 500 | 273 → 70 | 212 → 43 | 0 → 0 of 30 | 0 → 30 of 30 | 193 → 195 (18 batches) | 0 → 0 |
| Photos | Cache drop | 914 → 493 | 284 → 64 | 205 → 40 | 0 → 0 of 30 | 0 → 30 of 30 | 193 → 195 (18 batches) | 0 → 0 |
| Mixed | Cold | 3,303 → 1,210 | 2,923 → 755 | 1,565 → 426 | 0 → 0 of 30 | 0 → 0 of 30 | 360 → 120 | 80 → 80 |
| Mixed | Same-session revisit | 117 → 72 | 62 → 4 | 58 → 0 | 0 → 30 of 30 | 0 → 30 of 30 | 0 → 0 | 0 → 0 |
| Mixed | Restart | 729 → 496 | 258 → 61 | 186 → 21 | 0 → 1 of 30 | 0 → 30 of 30 | 120 → 144 (in batches) | 0 → 0 |
| Mixed | Cache drop | 773 → 530 | 268 → 62 | 201 → 23 | 0 → 0 of 30 | 0 → 30 of 30 | 120 → 144 (in batches) | 0 → 0 |

The two builds were run by turns, 3 times each, in the same hour. Earlier sets taken hours before gave the same picture with lower times all round; the virtual machine's speed drifted over the night. The extra decodes after a restart of the mixed folder are tiles GTK bound twice while laying out, whose reads had already been sent.

- **Restarts and cache drops:** the cache is now read in batches of up to 12 per decoder request. Before, each tile cost one GIO query and one decoder request.
- **Cold visits:**
  - Each new photo costs one decode instead of three. Before, there was a look for an earlier thumbnail that decoded a missing file, then the decode, then a read-back of the written PNG.
  - Visible tiles are made first.
  - Thumbnails are made one fewer at a time than the processors, between 2 and 3: 3 on this 4-processor machine. In an earlier set, 2 slots gave 512 ms (photos) and 796 ms (mixed) after the first tile, against 464 ms and 555 ms with 3. A 2-processor computer keeps 2.
- **Paging down through a new folder** of 2,000 photos for 4.7 s while its thumbnails are made, 3 interleaved runs each: the main loop missed its 5 ms timer for 34 ms or more 2, 6 and 2 times before (longest 40 ms), against 1, 1 and 0 after (longest 35 ms). About 570 thumbnails were made meanwhile in both. Other interleaved scroll-only runs gave 4, 11 and 6 stalls before against 4, 7 and 1 after: no regression, and the stalls come mostly from the machine.
- **Memory:** resident memory after scrolling through the cold 2,000-photo folder and back is 341 MB before and 357 MB after. After a restart, with the cache warm, it is 267 MB before and 286 MB after. These are single runs measured from `/proc/<pid>/status`; the picture decoders are separate processes and are not counted.
- **First frame after a restart:** after a restart, a tile still cannot show its thumbnail on its very first frame. The read takes 35 ms or so, and holding the window's first frame back for it would be slower overall. The tile shows nothing in the icon's place until then, so there is no icon flash.

## NOT CONFIRMED

- Whether Windows 11 File Explorer draws a cached folder's thumbnails on its first frame after a restart, or shows blank tiles or icons first.
- The default of GNOME's thumbnail housekeeping (`org.gnome.desktop.thumbnail-cache` `maximum-age`, 180 days) on Zorin OS 17/18.
- That Dolphin finds the app's entries valid. GIO, and so Files, does: `gio info` reports `thumbnail::is-valid: TRUE` for the app's entries, and FALSE once the file changes.

## Question

Thumbnails of files that were deleted or sent to the Recycle Bin stay in the cache. The standard says to remove them, GNOME and KDE mostly don't, and Windows keeps them in `thumbcache_*.db` (a known privacy complaint). This change removes only the entries of files the app moves. Should it also remove the entries of files it deletes or moves to the Recycle Bin?

## How it was tested

- **Tests first:**
  - New core tests in `ox-core/src/thumbnail_cache.rs`: a current entry of another program is a hit; changed time, size or URI misses; a larger size serves a smaller one; damaged, linked and oversized entries miss without panicking; failures hold until the file changes, including GNOME's; writes are atomic and 0600; keys are replaced with the pixels kept; a moved file keeps its thumbnails; the cache's names are listed. All were seen to fail before the implementation, as was the test of the cache names kept as numbers. The app tests below were written together with their code.
  - Also in ox-core: x-large and xx-large flavors, and the moved items of rename, move and Replace records.
  - App tests: a batch decoded by one decoder with a crashing picture refused alone; the memory's eviction and version checks; a cached thumbnail shown until its file changes, then made again and its entry replaced; a damaged picture recorded in `fail/` and a recorded failure not tried again until the file changes; a renamed folder keeping its files' thumbnails; a dropped network check not leaving other tiles waiting; tiles on the screen taking slots first.
  - The existing test that "Always show icons" turns thumbnails off everywhere still passes.
- **Every test binary** was run through `check.py`'s own isolation (`run_isolated`) as an unprivileged user, because the app refuses to run as root and `check.py` stops there in this container. At 77ae92d6, the last code change, 4,996 tests passed and 4 failed. The commit after it only fixes spellings in tests and the bench, and its tests and Clippy pass.
  - The two sandboxed-thumbnailer tests fail identically on unchanged upstream as the same user, because bubblewrap cannot run in this container.
  - The two Python-app comparison tests, `location_fixture_drift` and `network_python`, touch nothing changed here; the first is already noted as failing on main in this environment.
  - One earlier run of the ox-app suite also failed `home_has_no_band_under_the_titles_and_its_tiles_do_not_touch` once. It is a timing flake: it passes alone 3 of 3 times and in every other full run.
  - The doctests pass.
- **Other checks:** `check.py` up to the Rust tests (the parity tests and `parity/check.py`, i18n, the driver tests, the drawn-icon guard, rustfmt and Clippy with `-D warnings` on Rust 1.92.0) passes, and `parity/check.py` passes alone. `tools/sync-docs.py --check` passes after regenerating the guides from `docs.json`.
- **Benchmark:** the table above (`thumbnail-bench.py --scroll`).
- **Interop:** the release build was run by hand on a private Xvfb display as an unprivileged user with a throwaway HOME, then `gio info` was checked on its entries (0600 files in 0700 folders, with all four keys).
- **Tumbler 4.18.1**, D-Bus activated on a private session bus with the throwaway HOME. `GetFlavors` lists all four sizes. At 1x Large icons it makes `normal` thumbnails, and at 2x Extra large icons `x-large` ones (512 px), of 6 photos and an MP4. The app shows them all, and on the next launch reads all 7 from the cache without asking tumbler again. Tumbler writes `Thumb::MTime` with microseconds (`1795342906.979486`), which the app reads to the second.
- **Real input, rename** (Xvfb, xdotool): a folder of 20 photos was opened, then Alt+Up, F2, "Trip 2026" and Enter, then opened again. All 20 entries moved with it, all 20 thumbnails were read from the cache, none was made again, and the cache holds 20 entries in all.
- **The real app with "Always show icons, never thumbnails" on:** no thumbnail was requested, no decoder was started, and no thumbnails folder was created.
- **The real app's fail cache:** a PNG cut short is tried once and recorded in `fail/openxplorer` on the first launch. On the second launch it is answered from that entry, with no decode and nothing made, and the good PNG beside it is read from the cache.
- **2x scale by hand** (`GDK_SCALE=2`): Large icons write `large` entries, and Extra large icons write 512 × 341 `x-large` entries.
- The website's pictures of the app were not changed, and no website build was run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
