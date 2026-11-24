# Title

fix(parsers): Fuzz the readers of untrusted bytes and fix what the fuzzing found

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/21-sort-header-windows)

## What changes for the user
- A damaged `.mo` catalog whose header claims more messages than the file can hold is refused, instead of sizing the message map from that count (a few bytes could ask for gigabytes).
- A BMP whose information header claims to be gigabytes long, or whose pixels start gigabytes in, is refused as damaged before any loader sees it; gdk-pixbuf's loader used to reserve that much memory first.
- Icons and cursors whose directory puts an image gigabytes in, puts an image inside the directory, or claims a bitmap millions of pixels wide are refused as damaged.
- The vector drawing process's answer is accepted only as PNG; anything else is refused instead of being handed to another loader.
- A ZIP or 7z member whose data ends before the size the archive records no longer opens as a silently shorter copy in a preview; opening it fails, as Extract All already refused it.
- Search words: a long query of negated groups no longer stalls every keystroke (each `NOT` now makes each clause once, in the same order), and a long run of `(` pasted into the search box no longer runs out of stack (groups nested past 32 are read as if their parentheses were not there).
- No picture a program writes is affected: the newly refused files are BMPs whose headers claim more than 124 bytes of information header or pixels past 1 MiB, and icons whose images end past 64 MiB, lie inside their directory or claim bitmaps wider than 1,024 pixels.

For developers:
- New cargo-fuzz targets in `native/fuzz` (see its README) for archives, pictures, media, text and rich text previews, desktop entries, the child processes' answers, D-Bus requests, mount tables, catalogs, the root helpers' command lines, mount plans and the update server's answers, with generated seeds (`seeds.py`, `seed-maker`), `run.sh` and `stats.py`. With `OX_FUZZ_WORK` set, a long run keeps its corpus, artifacts and logs outside the checkout.
- The fuzz crate is not a workspace member, so builds, checks, packages and the Flatpak never compile it. The targets reach private parsers through new `fuzzing` features of `ox-core` and `ox-app`, off in every build of the app. Its own `Cargo.lock` adds libfuzzer-sys and arbitrary for that crate only.

## Why
The app reads many kinds of bytes it did not write, and none of these readers had been fuzzed with coverage guidance. The fuzzing found the memory reservations, the short archive members and the search stalls above. Every fix has a regression test next to its parser, with the minimized input written in the test's code.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: fuzzing with cargo-fuzz 0.13.2 (libFuzzer with AddressSanitizer, nightly) overnight with a 2 s timeout and a 1 GiB memory limit, about 238 million executions over the targets `app_protocols`, `archive_extract`, `archive_fixup`, `archive_list`, `archive_zipgen`, `core_formats`, `core_text`, `media_details`, `media_streams`, `picture_decode`, `picture_headers` and `text_preview` (plus two local targets for the user icon files work in PR 25): 10 defects found, 9 fixed, and one left open inside GStreamer 1.24.2 (Matroska through parsebin), seen only after hundreds of files in one long-lived fuzzing process and not in 300 fresh processes, while the app reads each media file in a new child process; every saved failing input reruns clean against the fixed code. `python3 native/parity/check.py`. `python3 native/tools/check.py` as root (Rust 1.92.0): parity tests, i18n check, driver tests, drawn-icon guard, `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed; its test step stopped because the app refuses to run as root. The 87 test executables and the doctests through `check.py`'s `run_isolated` as an unprivileged user: all passed except `integration_file_chooser`, `location_fixture_drift`, `network_python` and `settings_page::tests::every_preference_switch_saves_persists_and_turns_back_exactly`, which fail the same way on upstream main built the same way. After the last fix: ox-core's 1,309 library tests, its search test executables and ox-app's 195 search and filter tests. `cargo clippy -p ox-core -p ox-app --lib --features ox-app/fuzzing -- -D warnings`: clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
