# Title

perf(startup): Open later windows without reading the whole executable, and let go of picture decoders and previews

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 13 measured how quickly the app starts and how much memory it holds, on a release build under a private Xvfb with Openbox, as an unprivileged user with a throwaway home: cold starts (page cache dropped, new home) and warm starts, to the first painted window, to the file list and to an idle main loop; another window while the app runs; and resident memory, threads, open files and child processes at idle and after Pictures with 2,000 photos in Large icons, a 10,000-file folder, Gallery, an SMB share, a search and 20 tabs, then again after closing them. It fixes what was slow or held:

- **A later launch read its whole executable first.** Super+E, `--new-window` or a folder opened from another app while OpenXplorer ran first read and hashed its own executable (36 MB in a release build, 500 MB in a test build) on the main thread, to tell whether the running instance is the installed build, before it handed its command line over. That was 90 % of the launch. The running instance now names the executable file it runs (device, inode, size and times) in its `runtime-info` report; a normal launch of that very file is the same build, since a running executable cannot be rewritten and an upgrade puts a new file in its place, so it hands over at once. Another file, an instance that names none (an older release) and `--restart` still compare digests. The running instance also reads its own digest once instead of twice.
- **Picture decoders and previews were held for as long as the app ran.** Browsing photos started up to four decoder processes (about 20 MB each) and kept up to 512 previews and 48 Gallery tile pictures for rows scrolled back into view. After the photos were left and every tab closed, the decoders still held 81 MB. Once no picture has been asked for 30 seconds, the waiting decoders are stopped and the kept previews dropped (the rows on screen keep their own pictures), and the kept previews never hold more than 64 MB, which 512 previews of Extra large icons on a 200 % screen could pass eightfold.
- **`OX_STARTUP_TRACE=1`** makes the app write the steps of its start (`main`, `startup`, `state`, `built`, `presented`, `painted`, `listed`, `list-painted`, `idle`, `window-*` for later windows, and `executable-digest`) with the monotonic clock, which the new startup test and these measurements read. `native/README.md` describes it.

## Measurements

Release build, private Xvfb 1920 x 1080 with Openbox (Wayland: Weston headless), 4 cores, unprivileged user, throwaway home. Times in ms from the moment the process is started, medians; "cold" drops the page cache and uses a new home. Before = upstream main 5d21fd3e with only the startup trace added; after = this branch with every merge. Rows marked A/B ran interleaved (before, after, after, before, ...).

| Start | Renderer | Step | Before | After |
| --- | --- | --- | ---: | ---: |
| Warm, A/B, n = 8 | Cairo | first window painted | 388 | 372 |
| | | file list shown | 422 | 404 |
| | | main loop idle | 442 | 420 |
| Cold, A/B, n = 6 | Cairo | first window painted | 567 | 558 |
| | | file list shown | 615 | 591 |
| | | main loop idle | 634 | 613 |
| Warm, same home, n = 5 (after: before the merges, run after the before batch) | GL (llvmpipe) | file list shown | 580 | 582 |
| First start in a new home, A/B, n = 6 | GL (llvmpipe) | file list shown | 1,862 | 1,890 |
| Cold, A/B, n = 4 | GL (llvmpipe) | file list shown | 2,139 | 2,150 |
| Another window while the app runs (`--new-window`), A/B, 12 launches | Cairo, X11 | later launch exits | 246 | 105 |
| | | new window painted | 295 | 153 |
| | | new window shows its files | 304 | 161 |
| Another window, 6 launches | Wayland | later launch exits | 221 | 82 |
| | | new window shows its files | 312 | 180 |

The first window does not change: it is GTK's work here (opening the display and GL 35-70 ms, parsing the 171 KB of CSS 34 ms, the window template 75 ms). With Mesa's software GL renderer, the first start in a new home compiles shaders (about 1.3 s) and a cold start loads LLVM; a real GPU driver replaces both.

Memory: main process RSS / anonymous memory, plus the helper processes' RSS, in MB; threads; open files; helper processes. One session each, Cairo renderer (GL below).

| Step | Before | After |
| --- | --- | --- |
| idle after start | 156 / 43 + 0, 23 threads, 22 fds, 0 helpers | 157 / 43 + 0, 23, 22, 0 |
| Pictures, 2,000 photos, Large icons | 187 / 71 + 77, 27, 30, 4 | 188 / 72 + 75, 27, 30, 4 |
| scrolled to the end | 196 / 80 + 77, 27, 30, 4 | 197 / 80 + 77, 27, 30, 4 |
| 10,000 files, Details | 211 / 98 + 77, 27, 30, 4 | 208 / 96 + 57, 26, 28, 3 |
| Gallery scrolled | 269 / 149 + 81, 27, 30, 4 | 273 / 156 + 82, 27, 30, 4 |
| SMB share (Samba on localhost) | 268 / 149 + 81, 29, 32, 4 | 276 / 156 + 82, 29, 32, 4 |
| search | 261 / 149 + 81, 30, 33, 4 | 269 / 156 + 82, 30, 33, 4 |
| 20 tabs | 266 / 150 + 81, 49, 52, 4 | 273 / 157 + 41, 47, 48, 2 |
| all closed, 5 s | 269 / 149 + 81, 30, 33, 4 | 277 / 156 + 41, 28, 29, 2 |
| all closed, 30 s | 269 / 149 + **81**, 30, 33, **4** | 277 / 156 + **0**, 26, 25, **0** |
| the same again, closed, 30 s | 278 / 161 + 81, 30, 33, 4 | 286 / 167 + 0, 26, 25, 0 |

With the GL renderer (llvmpipe), all closed after 30 s: main 499 + helpers 80 = 579 MB before, 460 + 0 = 460 MB after; idle after start 287 and 288 MB. The context menu, See more and the address suggestions were each opened ten times: memory grows only on the first opening of each.

The main process does not shrink after closing, in either build: what it frees stays in glibc's heap (reused by the app, not returned to the system). A `malloc_trim(0)` through gdb took anonymous memory from 173 to 129 MB (Cairo, before the fix) and from 367 to 252 MB (GL); the project allows no `unsafe`, which calling it needs, so that is left as a question.

## Tests

Each fix has a test that failed on the code before it (run and seen failing):

- `tests/startup.rs` `windows_open_within_their_budgets`: the real executable in check.py's isolation; the first window lists its folder within 5 s, a later `--new-window` hands over within 1 s without tracing `executable-digest`, and its window lists within 4 s (failed on the old code: the later launch traced `main`, `executable-digest`; with debug information it took 2.06 s)
- `update::launch_guard::tests::a_launch_of_the_file_the_instance_runs_reads_no_digest`, `a_launch_of_another_file_still_compares_builds` (guard), `the_runtime_info_action_names_the_running_file_at_once`; `update::service::tests::the_running_executable_is_read_once`; ox-core `update::identity` `an_action_state_can_name_the_running_file`, `a_replaced_or_rewritten_executable_is_another_file`
- `window::tests::memory::previews_and_picture_decoders_are_let_go_once_unused` (real JPEGs in Large icons, leave the folder: the kept previews and the decoder processes are gone, their PIDs no longer exist), `thumbnails::tests::the_kept_previews_stay_within_their_budget`, `picture_guard::decoder::tests::an_unused_decoder_is_stopped_when_its_time_is_up`

Real input with xdotool under the private Xvfb with Openbox, as an unprivileged user with a throwaway home: Pictures (2,000 photos) in Large icons scrolled to the end, 10,000 files in Details, Gallery scrolled, an SMB share signed in as guest, a search, 20 tabs and closing them, Extra large icons at GDK_SCALE=2, Documents (mp3, mp4, pdf, docx, odt, zip, 7z, broken files, odd and Unicode names) with the preview pane over every file, and the menus ten times each; Weston headless for Wayland. 249 measured sessions, no "Bail out", CRITICAL, panic or signal from the app.

Every test executable was run in `native/tools/check.py`'s isolation (`run_isolated`) as an unprivileged user, with the Python app of v2.0.0 provided, before and after merging the lower passes: 88 executables each time. The failures `ox_core` `admin_delete::tests::parent_path::*` (18, root-owned checkout), `location_fixture_drift` (2) and `network_python` (1) fail by the same names on unchanged main. One `ox_app` test timed out in each full run and passed alone and in its group: `search_history::a_tab_on_results_keeps_its_selection_and_scroll_position` before the merges (main passed it in its run) and `context_menus::a_pinned_folder_menu_copies_compresses_and_creates_in_that_folder` after them (a 7z compression, from the merged passes). Clippy `-D warnings` with the pinned Rust 1.92.0, rustfmt, `native/parity/check.py`, check.py's inventory, driver and icon stages, `tools/sync-docs.py --check` and `tools/audit-public-data.py` pass. check.py was not run as one command: the container runs as root, which the app refuses.

## Not covered

- No real GPU: GL numbers are Mesa's llvmpipe on Xvfb, whose shader compiler and per-context memory dominate them.
- No spinning disk: cold starts dropped the page cache on the VM's disk.
- Freed heap memory is not returned to the system (see Measurements); that needs a decision on `unsafe`.
- Not fixed: every start rebuilds each window's navigation pane once more after an empty settings change (about 10 ms), and each popover that has been shown keeps its own renderer.
- How quickly Windows 11 opens a window and returns memory is not from a Microsoft source; the status file lists steps to compare in a Windows 11 VM.
