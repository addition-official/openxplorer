# Title

fix(details-pane): Play previews in a separate process so no media file can close the app

# Body

Model: claude-opus-5-5
CLA: I agree

## Problem and resulting behavior

On Kubuntu, OpenXplorer closed completely while a file was renamed, and again while the user was just browsing a folder of MP3 files. The journal shows `ERROR:../gst/playback/gstdecodebin3.c:3381:mq_slot_handle_stream_start: assertion failed: (collection)`, then `Bail out!` and SIGABRT of the app.

There are two problems behind this, and both happen in the app's own process:

* **Leaked players.** Every recording the Details or Preview pane showed left its GStreamer player alive (GTK's media backend leaks them), at about 10 open files and 8 threads each. Arrowing through a folder of recordings reaches the soft limit of 1024 open files within seconds; then GLib aborts ("Creating pipes for GWakeup: Too many open files") or GStreamer asserts.
* **Aborts inside GStreamer.** An abort inside GStreamer, like this decodebin3 assertion, takes the whole window down.

The Details and Preview panes played audio and video with `gtk::MediaFile` in the app's own process. GTK 4.22 (Ubuntu 26.04) plays through GStreamer's own `GstPlay`, which in GStreamer 1.28 always builds `playbin3`, and so `decodebin3`. The 1.28.2 source no longer reads `GST_PLAY_USE_PLAYBIN3`, so the app can't fall back to the classic playbin. The assertion is at line 3381 in GStreamer 1.28.1 to 1.28.4. A failed `g_assert` aborts whatever process it runs in, and it can't be caught. Windows never closes Explorer because of a media file.

Now:

* **The recording is decoded in a player process.** This is the app started again with `--ox-play-media`, as the Details tab's reader already is (`media_details.rs`). It plays with the classic `playbin` (`decodebin`, never `decodebin3`) and plays the sound itself. It sends scaled RGBA frames, its position and its state back on its standard output, after a hello line. A `gtk::MediaStream` subclass (`media_player/stream.rs`) shows them, so `gtk::Video` and `gtk::MediaControls` work as before.
* **A player starts only after the selection has stayed on a recording for 400 ms,** and at most one runs at a time, so arrowing through a folder starts none.
* **Whatever GStreamer leaks dies with the player process.**
* **A player that aborts, hangs up or sends something malformed** puts the stream in error. The pane shows the file's icon and keeps working, and the next file plays.
* **The player is limited.** It has a 2 GiB data limit, gets SIGKILL when the app dies, and quits when its input closes. It's told to quit and killed after one second if it doesn't.
* **The player stops before anything changes the file, and stays off it until the change is done.** It is paused, its pipeline set to NULL, then released, when the selection changes. Before the app renames, moves, deletes or replaces a file, the operation holds the file until its write ends, and from before any question it asks. The hold stops every player of the file and waits for it to exit. A player that doesn't quit within one second is killed and waited for. No new player of a held file starts, so one still waiting out its 400 ms doesn't start during a Replace question either.
  * This covers renaming in place, the Rename dialog, the Properties name box, batch rename, numbered renames of several items (F2), Undo and Redo of every kind, and every transfer job: its sources, unless copied, and the namesakes a Replace overwrites.
  * Renaming or moving a folder stops the players of the files inside it.
  * A share's player plays through the share's local path but is also known by the share's own URI (`smb://…`), which is what file operations use.
* **The app raises its soft limit on open files to the hard limit at startup,** as defense in depth.
* **A folder change never restarts a stream mid-playback.** After the app stops a player for an operation, the file's next player waits for Play, even with auto-play on. A monitor event for the same file still keeps the player as it was.

`details_pane/media.rs` only hands off to the new `details_pane/player.rs`. The branch has upstream `main` 5336b108 (2.0.21, with the file-icons change) merged in, with no conflicts.

## Validation

* **Tests that fail first:**
  * Review findings. Each test below failed on 445f1098 merged with 2.0.21 (or, for the share case, couldn't be written against it) and passes now:
    * `a_paste_over_a_recording_whose_player_waits_to_start_keeps_it_from_starting`: a player started while the Replace question was open.
    * `a_player_of_a_share_is_stopped_by_the_shares_own_uri`: this needs the new constructor argument, so it couldn't compile before. A player had no way to be found by its `smb://` URI.
    * `a_player_that_will_not_quit_is_gone_before_its_release_returns`: the killed player still existed when the release returned.
    * `undoing_a_copy_stops_the_copys_player_first`: never stopped.
    * `a_numbered_rename_stops_a_player_of_one_of_its_files_first`: never stopped.
    * Two more pass with the fix: `no_player_starts_for_a_held_file`, and `ox-core`'s `a_reversal_names_every_file_it_may_change`, which failed to compile first because `UndoRecord::locations` didn't exist.
  * `window/tests/preview_player.rs`, original four:
  * `arrowing_through_200_recordings_leaves_nothing_behind` walks 200 generated recordings with each pane, quickly and then slowly. On `main`, the test process's open files grew from 26 to 1,132 and its threads from 21 to 834, and the test failed. With the fix they come back to where they started.
  * On `main`, rename and delete each failed: the stream stopped 25 to 30 ms *after* the file changed, and the renamed file started playing again. A watcher thread timestamps the file change and the stream's own notify timestamps the stop, so the order is checked exactly.
  * The aborting-player test timed out, because the player can't abort without taking the app with it.
  * All four pass with the fix, along with the new protocol and entry tests and `details_preview.rs`.
* **Attempts at the decodebin3 assertion itself:** these ran in private Xvfb sessions as an unprivileged user, with a null PulseAudio sink and fictional files generated with gst-launch (MP3, Ogg, MP4/H.264+AAC, WebM). Input was real (xdotool).
  * Ubuntu 24.04 (GStreamer 1.24.2, GTK 4.14.5), with GTK forced to playbin3: about 240 random actions while playing, including F2 and right-click rename, delete, external rename, overwrite, truncate, delete and rapid selection. No crash.
  * Ubuntu 26.04 root filesystem (GStreamer 1.28.2, GTK 4.22.4, the same decodebin3 code as the report), unfixed app, both pane modes: about 1,700 random actions over 42 minutes, and 4 × 150 s of rapid selection in a Downloads-like folder. That folder had ID3v2.4 MP3s with APIC covers (JPEG and 1280×720 PNG), VBR Xing headers, lengths of 4 s to 6 min 40 s, and names with emoji and Tamil script. I also played those MP3s directly with playbin3 and playbin, and overwrote 64 playing files in place under playbin3. No crash. **The assertion was not reproduced.**
* **The leak crash, real input:** I held Down and then Up three times through 231 generated recordings (MP4, WebM, MKV, MP3 with a cover, Ogg and FLAC) at 25 key presses a second.
  * Unfixed app on Ubuntu 24.04: open files rose 37 → 1,024 and threads 33 → 893, then the app died with SIGTRAP and "Too many open files".
  * Fixed app, 24.04 and 26.04, both panes: open files stayed at 28-31 and threads at 25-31, with no player while arrowing and one once the selection rests. At 1.5 s per file with auto-play on, through 40 files, at most one player ran, and open files (31) and threads (25-29) stayed flat.
  * The app ran with an open-file limit of 20000 (the hard limit) where the user's shell soft limit was 1024.
* **Fixed app, real input:**
  * The main process maps no GStreamer playback or decoder plugins while previewing; the player maps them.
  * `kill -ABRT` of the player while playing leaves the app running with the file's icon in the pane, and the next file plays.
  * F2 rename while playing works (24.04).
  * On 26.04, right-click Rename and Delete while playing work in both pane modes. The renamed MP4 shows its first frame and Length, waiting for Play. 60 rapid clicks across eight files leave exactly one player process.
* **Checks** at d0c873dd:
  * `python3 native/parity/check.py` passed, as did the parity and tools unit tests check.py runs and `cargo fmt --check`. Clippy passed with the workspace lints on Rust 1.92.0, the toolchain CI pins.
  * `python3 native/tools/check.py` can't pass as one command in this container, because of the environment:
    * As root, the app refuses to start, so `display_scale` fails.
    * The container's Python 3.13 can't import the distro's PyGObject, so `location_fixture_drift` and `network_python` fail.
  * check.py stops at its first failing executable, so I ran every test executable and the doctests with check.py's own isolation (private Xvfb, D-Bus and home), without stopping, as an unprivileged user, with system Python 3.12 and node 22. 169 of 171 executables passed, with 4,910 tests passed and 10 failed:
    * `ox_app`: 1 failed, `slow_locations::working_on_it_carries_over_from_the_lookup_to_the_listing`, a timing test. It passed in the previous full run and 3 of 3 times alone.
    * `ox_core`: 9 `admin_copy` tests failed, because they need paths only root can change. Run as root, 12 of those 13 tests pass. `an_fstab_that_cannot_be_read_refuses` fails as root, because root can read the unreadable file.
  * Real input again with this build on the Ubuntu 26.04 root:
    * Right-click Rename and Delete while playing, in both panes.
    * Fast arrowing through 231 recordings: open files stayed at 28-31.
    * `kill -ABRT` of the player left the app running.

## Compatibility and release impact

* **New behavior:** the player is a new command-line mode of the same binary (`--ox-play-media`, next to `--ox-read-media-details`).
* **Dependencies:** none new. The `stdio` feature of the existing `rustix` dependency is enabled, which does not change `Cargo.lock`.
* **No changes to:** persisted data, desktop integration, translations or license notices.
* **Behavior changes:**
  * A file GStreamer can't play now shows its icon instead of an empty player.
  * After a rename, move or replace of the playing file, the file's next player doesn't auto-play.
  * A file replaced in place keeps its icon until it is selected again.
* **Fixtures:** all test and reproduction files are generated, fictional media.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01CQbN3PshAViEgjzDBGj4sP
