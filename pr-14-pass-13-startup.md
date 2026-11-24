# Title

perf(startup): Open later windows without reading the whole executable, and let go of picture decoders and previews

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/13-pass-12-details-status)

## What changes for the user
- **Another window opens faster while the app runs.** Super+E, `--new-window` or a folder opened from another app no longer reads and hashes the whole executable before handing its command line to the running instance. The running instance now names the executable file it runs (device, inode, size and times) in its `runtime-info` report; a launch of that very file is the same build, since a running executable cannot be rewritten and an upgrade puts a new file in its place, so it hands over at once. Another file, an instance that names none (an older release) and `--restart` still compare digests. The running instance also reads its own digest once instead of twice. Measured with a release build on Cairo and X11, the later launch exits after 105 ms instead of 246 ms and the new window shows its files after 161 ms instead of 304 ms (Wayland: 180 ms instead of 312 ms).
- **Picture decoders and previews are let go once unused.** Once no picture has been asked for 30 seconds, the waiting decoder processes are stopped and the kept previews dropped (the rows on screen keep their own pictures), and the kept previews never hold more than 64 MB. After browsing photos and closing every tab, the helper processes went from 81 MB and 4 processes to none.
- **`OX_STARTUP_TRACE=1`** makes the app write the steps of its start (`main`, `startup`, `state`, `built`, `presented`, `painted`, `listed`, `list-painted`, `idle`, `window-*` for later windows, and `executable-digest`) with the monotonic clock. The helper processes are left out of the trace. `native/README.md` describes it, and the parity notes record the faster launch and released memory.
- The first window's start time does not change: it is GTK's work (opening the display and GL, parsing the CSS, the window template).

## Why
- A later launch read and hashed its own executable (36 MB in a release build, 500 MB in a test build) on the main thread before handing over, which was 90 % of the launch.
- Browsing photos started up to four decoder processes (about 20 MB each) and kept up to 512 previews and 48 Gallery tile pictures for as long as the app ran; 512 previews of Extra large icons on a 200 % screen could pass the new 64 MB limit eightfold.

Not covered: the main process does not shrink after closing, since freed memory stays in glibc's heap; returning it would need `unsafe`, which the project does not allow, so that is left as a question. GL numbers are from Mesa's llvmpipe, not a real GPU.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix with a test seen failing before it: `tests/startup.rs` `windows_open_within_their_budgets` (the real executable in check.py's isolation: the first window lists its folder within 5 s, a later `--new-window` hands over within 1 s without tracing `executable-digest`, and its window lists within 4 s), `update::launch_guard::tests::a_launch_of_the_file_the_instance_runs_reads_no_digest`, `a_launch_of_another_file_still_compares_builds`, `the_runtime_info_action_names_the_running_file_at_once`, `update::service::tests::the_running_executable_is_read_once`, ox-core `update::identity` `an_action_state_can_name_the_running_file` and `a_replaced_or_rewritten_executable_is_another_file`, `window::tests::memory::previews_and_picture_decoders_are_let_go_once_unused`, `thumbnails::tests::the_kept_previews_stay_within_their_budget` and `picture_guard::decoder::tests::an_unused_decoder_is_stopped_when_its_time_is_up` (some of these were later removed as redundant and are not in this branch); startup and memory measurements on a release build under a private Xvfb with Openbox and on headless Weston, as an unprivileged user with a throwaway home, with real input through Pictures (2,000 photos), 10,000 files, Gallery, a local SMB share, a search and 20 tabs; every test executable in check.py's isolation as an unprivileged user, with every failure also failing by name on unchanged main except one test that timed out in a full run and passed alone; Clippy `-D warnings` on Rust 1.92, rustfmt, `native/parity/check.py`, check.py's inventory, driver and icon stages, `tools/sync-docs.py --check` and `tools/audit-public-data.py`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
