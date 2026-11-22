# Title

fix(parsers): Fuzz the readers of untrusted bytes and fix what it found

# Body

Model: claude-opus-5-5
CLA: I agree

## Problem and resulting behavior

The app reads many kinds of bytes it did not write: archives, pictures, media files, text and rich text, desktop entries, the answers of its own child processes, D-Bus requests, mount tables, catalogs, the root helpers' command lines and the update server's answers. None of these readers had been fuzzed with coverage guidance. This branch adds cargo-fuzz targets for them and fixes what they found.

Fixed:

- **Catalogs.** A `.mo` header that claimed more messages than the file can hold sized the message map from that count, so a few bytes could ask for gigabytes. Such a header is now refused.
- **BMP headers.** A BMP whose information header claims to be gigabytes long, or whose pixels start gigabytes in, passed the header check, and gdk-pixbuf's loader reserved that much before reading it. Both are now refused as damaged before any loader sees them.
- **Icons and cursors.** The same kind of reservation came from an icon directory putting an image gigabytes in, an image inside the directory, or a bitmap header millions of pixels wide. These are refused as damaged.
- **Vector pictures.** The drawing process always writes PNG; anything else in its answer is now refused instead of being handed to another loader.
- **Archive previews.** A ZIP or 7z member whose data ended before the size the archive records, with a checksum to match what was there, opened as a silently shorter copy, although Extract All refuses it. Opening it now fails the same way.
- **Search words.** Each `NOT` rebuilt its clauses by joining one clause at a time with `OR`, copying up to 262,144 terms to keep 64 clauses, so a long query of negated groups took a noticeable time on every keystroke. The negation now makes each clause once, in the same order. Each group of parentheses was also read by a call of its own, so pasting a long run of `(` into the search box ran out of stack; groups nested past 32 are now read as if their parentheses were not there.

The fuzz targets live in `native/fuzz` (see its README). That crate is not a workspace member, so builds, checks, packages and the Flatpak never compile it. The targets reach private parsers through new `fuzzing` features of `ox-core` and `ox-app`, which are off in every build of the app. Every fix has a regression test next to its parser, with the minimized input written in the test's code.

## Validation

Fuzzing (cargo-fuzz 0.13.2, libFuzzer with AddressSanitizer, nightly), from 22:45 to 06:27 with a 2 s timeout and a 1 GiB memory limit, seeds from `native/fuzz/seeds.py`:

```
Target                             Executions  Crashes  Fixed  Still open
app_protocols                         421,336        0      0           0
archive_extract                     4,401,998        0      0           0
archive_fixup                       1,505,649        0      0           0
archive_list-vp                     3,896,436        0      0           0
archive_list                       13,016,364        1      1           0
archive_zipgen                      2,632,926        0      0           0
core_formats                       11,795,811        2      2           0
core_text                          23,124,656        1      1           0
media_details                     103,013,196        0      0           0
media_streams                          11,193        1      0           1
picture_decode                     14,984,992        4      4           0
picture_headers                    20,745,849        1      1           0
text_preview                       15,869,547        0      0           0
pe_icons (fork branch, local)       3,581,300        0      0           0
truetype (fork branch, local)      19,293,686        0      0           0
Total                             238,294,939       10      9           1
```

"Crashes" counts real defects (crashes, hangs over 2 s, allocations over 1 GiB); one BMP header was reached by two targets. The open one is inside GStreamer 1.24.2 (Matroska through parsebin), seen only after hundreds of files in one long-lived fuzzing process and not in 300 fresh processes; the app reads each media file in a new child process. Every saved failing input reruns clean against the fixed code. The last two rows fuzzed the fork branch `user-icon-files` locally; nothing there is changed.

Checks, with Rust 1.92.0 as CI uses:

- `python3 native/parity/check.py`: passed.
- `python3 native/tools/check.py` as root: parity tests, i18n check, driver tests, drawn-icon guard, `cargo fmt --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed. Its test step stopped at `display_scale`, because the app refuses to run as root.
- The 87 test executables and the doctests were then run through `check.py`'s own `run_isolated` as an unprivileged user. All pass except `integration_file_chooser` (a GLib critical made fatal), `location_fixture_drift` and `network_python` (Python 3.13 here, against the system's Python 3.12 `gi`), and `settings_page::tests::every_preference_switch_saves_persists_and_turns_back_exactly` (2,642 of 2,643 ox-app library tests pass). Each fails the same way on upstream main 5d21fd3e built the same way.
- After the last fix: ox-core's 1,309 library tests, its search test executables and ox-app's 195 search and filter tests pass.
- `cargo clippy -p ox-core -p ox-app --lib --features ox-app/fuzzing -- -D warnings`: clean.

Debug info was off for the test build to fit the disk. Not run: a full `check.py` in one go as a regular user; no website, Flatpak or SMB check (nothing there changed).

## Compatibility and release impact

No persisted data, desktop integration, dependency, license notice or public documentation changes. The fuzz crate's own Cargo.lock adds libfuzzer-sys and arbitrary for that crate only. Pictures now refused as damaged are BMP files whose headers claim more than 124 bytes of information header or pixels past 1 MiB, and icons whose images end past 64 MiB, lie inside their directory or claim bitmaps wider than 1,024 pixels: no picture a program writes. All fuzz seeds are generated by `native/fuzz/seeds.py`; no user file, customer data or screenshot is used.
