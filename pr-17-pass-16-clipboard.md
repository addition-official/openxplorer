# Title

fix(clipboard): Keep copied files after the last window and paste shares and odd names into other apps

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/16-pass-15-errors)

## What changes for the user
- Copied or cut files still paste after the last OpenXplorer window is closed. While the app owns the clipboard, closing its last window leaves it running without a window, serving what Copy, Cut or Copy as path put there. It ends as soon as another app copies something or a paste empties a cut. Quit and `--quit` still end it at once, and a launch meanwhile opens its window in that process. (CLIP-010)
- After a cut is pasted, a cut that another app (such as KDE's Klipper) puts back, naming only items this app already moved, counts as no data. Paste stays disabled and nothing is dimmed, as Windows leaves it after a cut is pasted. (CLIP-008)
- Files from a mounted network share (CIFS mount or GVfs FUSE export, found without mounting) reach other apps through `text/uri-list` as their local `file://` paths, as a drag already gives them, so a browser upload, an email attachment or a terminal gets usable files. `application/x-kde4-urilist`, this app's own format and GNOME's keep the `smb://` addresses. (CLIP-004)
- Names with a line break or a tab no longer refuse the whole Copy. They go on the clipboard escaped (`%0A`), one line per item, with no text format, and another app's list holding one pastes. NUL stays refused. (CLIP-005)
- A cut item fades only its icon (or custom icon, preview or folder picture). Its name and its Date modified, Type and Size columns stay at full strength, in Details and in icon views, as checked on Windows 11. A cut hidden item's icon fades further. (CLIP-002)
- "Paste pictures as new files" stays on by default. Turned off in Settings, a picture leaves Paste grayed, as in Windows 11.
- The interface guide (`docs/interface.md`, `docs.json` and the generated Markdown), `ui-spec.md` and the parity notes describe the cut look, the clipboard outliving the last window and share files pasting as local files.

## Why
Copy, cut and paste between OpenXplorer and other apps were tested with real keys on X11 and Wayland against browsers, other file managers, terminals and editors. An X11 or Wayland clipboard empties when its owner's process ends, so copied files were lost when the last window closed, while Windows Explorer's process outlives its windows. Klipper brought a pasted cut back, so a second Ctrl+V asked whether to replace the moved file with itself and then said "Item Not Found". Share files reached other apps as `smb://` addresses they could not use. One name with a line break refused the whole Copy with "Encoded control characters are not allowed.". Cut items faded their whole row, where Windows 11 fades only the icon.

## CHANGED OWNER DECISIONS
None. One commit in this PR made Paste ignore a picture by default; the next commit reverts it, so the owner's choice ("Paste pictures as new files" on by default) stands.

## New and changed visible text
None. (The message "Encoded control characters are not allowed." only moves in `openxplorer.pot`: the clipboard no longer uses it, the address bar still does.)

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: new `window/tests/clipboard_lifetime.rs` (3 tests), `clipboard_shares.rs` (1) and `cut_look.rs` (2, on drawn pixels); in ox-core `clipboard/tests.rs`, two share tests and the rewritten line-break test, each seen failing before its fix; real input on a private Xvfb as an unprivileged user with a throwaway home (Ctrl+C, close the window, paste in Thunar, then copy text elsewhere and the process exits; the same on KWin Wayland with wl-paste and wl-copy; a cut pasted with Klipper running leaves Paste disabled; eight odd names reaching Chrome and another folder; files on a local Samba share mounted through GVfs reaching Chrome, Konsole and Dolphin, with a folder link standing in for the FUSE export); rustfmt, Clippy with `-D warnings` (Rust 1.92.0), `parity/check.py`, `i18n.py check` and `sync-docs.py --check`; every test executable through `check.py`'s isolation as an unprivileged user, where the only failures were the 31 that also fail on unchanged main in that container.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
