# Title

fix(native): Keep idle picture decoders alive and write "Today at" dates from the new day

# Body

Model: claude-opus-5-5
CLA: I agree

## Problem and resulting behavior

An overnight soak (release build, 23:00 to 06:30 America/Chicago, an isolated
desktop session driven with real keyboard and mouse input, a local Samba
share stopped twice for 10 minutes) found two defects. Both are fixed here,
with a guard test for each written first and seen failing.

**Idle picture decoders died with their worker thread and lingered as
zombies.** Pictures are decoded in a pool of sandboxed child processes
(`bwrap --die-with-parent`). `PR_SET_PDEATHSIG` fires when the *thread* that
started a process ends, not the app, and the pool was filled from GIO's
worker threads, which retire after a short idle spell. Every idle decoder was
killed and stayed in the pool unreaped until the next picture found it dead,
failed on it and started a new sandbox. The soak showed up to four zombie
`bwrap` children after every burst; an strace showed the kernel killing the
sandbox the instant its starting thread exited. Decoders are now started from
one thread that lasts as long as the app, and an idle decoder that ended
anyway (killed, out of processor time) is dropped and reaped before the next
picture instead of being handed it. The liveness check uses `waitid` with
`WNOWAIT`, so the process group kill in `stop()` still targets a pid that
belongs to the dead decoder (this container's pid_max is 32768; an earlier
draft that reaped first made one test flaky).

**"Today at" dates stayed after midnight.** With relative dates on, the date
groups moved from Today to Yesterday on their own at midnight, but the rows
under the new Yesterday heading, and every row of a folder that is not
grouped, kept "Today at 10:45 PM". After a key press only the selected row
read "Yesterday at"; F5 left the rest stale. The details view now remembers
the day its relative dates were written from and draws the date columns
again when the window follows the day (the midnight timer, the window
becoming active, F5), grouped or not. The cells count from the same clock as
the date groups, so headings and rows always agree; ox-core gains
`column_date_text_at` for that.

## Validation

Commands run, as an unprivileged user (the app refuses root), Rust 1.92 and
Python 3.12 as CI uses:

- `python3 native/tools/check.py`: inventories, parity unit tests, i18n,
  tool tests, the drawn-icon guard, `cargo fmt --check` and
  `cargo clippy --workspace --all-targets --locked -D warnings` pass. Test
  executables: everything passes except tests that fail identically on
  upstream 5d21fd3e in this container: two sandboxed-thumbnailer tests and
  nine `admin_copy` tests (both depend on the host's /tmp and bubblewrap
  setup). Two `ox_app` UI tests timed out under load during the 32-minute
  run (`every_preference_switch_saves_persists_and_turns_back_exactly`,
  `a_tab_on_results_keeps_its_selection_and_scroll_position`) and pass on
  rerun. check.py stops at the first failing executable, so the executables
  after `ox_app` and the doctests were run with check.py's own isolation:
  all pass except the `admin_copy` ones above.
- `python3 native/parity/check.py`: passes.
- New guard tests, each failing before its fix:
  `picture_guard::decoder::tests::a_pooled_decoder_outlives_the_thread_that_started_it`,
  `picture_guard::decoder::tests::a_decoder_that_died_while_idle_is_replaced_not_refused`,
  `window::tests::group_by::relative_dates_move_on_when_the_day_changes`,
  `window::tests::group_by::relative_dates_under_the_groups_move_on_when_the_day_changes`.
  The decoder tests passed four runs in a row after the fix.
- The soak itself, restarted on the fixed build at 00:44: no zombie
  children; the idle pool keeps four live decoders (about 46 MiB PSS in
  all). The fixed release build was taken across midnight again under
  libfaketime: headings and rows moved to Yesterday together without input.
- The Samba checks used a real local smbd through GVfs; they are not a
  Windows server.

## Compatibility and release impact

No persisted data, settings, desktop integration, dependency or license
change. No documentation change. Test data and captures are generated and
fictional (paths under a throwaway home, a share on 127.0.0.1).
