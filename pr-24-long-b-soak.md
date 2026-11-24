# Title

fix(previews): Keep idle picture decoders alive and write "Today at" dates from the new day

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/23-long-d-doc-previews)

## What changes for the user
- Idle picture decoders stay alive. Decoders are now started from one thread that lasts as long as the app, so an idle decoder is no longer killed when a GIO worker thread retires, and no dead `bwrap` children are left waiting to be reaped. An idle decoder that ended anyway (killed, out of processor time) is dropped and reaped before the next picture instead of being handed it. The liveness check uses `waitid` with `WNOWAIT`, so the process group kill in `stop()` still targets a pid that belongs to the dead decoder.
- "Today at" dates move on after midnight. With relative dates on, the rows under the new Yesterday heading, and every row of a folder that is not grouped, now read "Yesterday at ..." once the day changes, without input. The Details view remembers the day its relative dates were written from and draws the date columns again when the window follows the day (the midnight timer, the window becoming active, F5), grouped or not. The cells count from the same clock as the date groups, so headings and rows agree; ox-core gains `column_date_text_at` for that.

## Why
An overnight soak of a release build on an isolated desktop, driven with real keyboard and mouse input and with a local Samba share stopped twice for 10 minutes, found two defects:
- Pictures are decoded in a pool of sandboxed child processes (`bwrap --die-with-parent`). `PR_SET_PDEATHSIG` fires when the thread that started a process ends, not the app, and the pool was filled from GIO's worker threads, which retire after a short idle spell. Every idle decoder was killed and stayed in the pool unreaped until the next picture found it dead; the soak showed up to four zombie `bwrap` children after every burst. PR 23 already made the pool drop such a decoder; this PR stops the decoders from dying in the first place.
- With relative dates on, the date groups moved from Today to Yesterday at midnight, but the rows under them, and every row of a folder that is not grouped, kept "Today at 10:45 PM". After a key press only the selected row read "Yesterday at", and F5 left the rest stale.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: as an unprivileged user with Rust 1.92 and Python 3.12, `python3 native/tools/check.py` (inventories, parity unit tests, i18n, tool tests, the drawn-icon guard, `cargo fmt --check` and `cargo clippy --workspace --all-targets --locked -D warnings` passed; the test executables passed except two sandboxed-thumbnailer tests and nine `admin_copy` tests that fail identically on upstream in that container, and two `ox_app` UI tests that timed out under load and passed on rerun; the executables after `ox_app` and the doctests were run with `check.py`'s own isolation and passed except the `admin_copy` ones). `python3 native/parity/check.py`. New guard tests, each failing before its fix: `picture_guard::decoder::tests::a_pooled_decoder_outlives_the_thread_that_started_it`, `picture_guard::decoder::tests::a_decoder_that_died_while_idle_is_replaced_not_refused`, `window::tests::group_by::relative_dates_move_on_when_the_day_changes`, and `window::tests::group_by::relative_dates_under_the_groups_move_on_when_the_day_changes`, which this PR does not carry; the decoder tests passed four runs in a row after the fix. The soak restarted on the fixed build: no zombie children, and the idle pool kept four live decoders (about 46 MiB PSS in all). The fixed release build taken across midnight again under libfaketime: headings and rows moved to Yesterday together without input.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
