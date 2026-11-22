# Title

fix(clipboard): Keep copied files after the last window and paste shares and odd names into other apps

# Body

Model: claude-opus-5-5
CLA: I agree

## Problem

Copy, cut and paste between OpenXplorer and other apps were tested with real keys (xdotool) on X11 (Xvfb) and on Wayland (Weston and KWin nested), against Chrome, Dolphin, Nautilus, Thunar, Konsole, xterm, Kate and Mousepad. Four things broke:

1. **Copied files were lost when the last window closed.** An X11 or Wayland clipboard empties when its owner's process ends. GNOME Shell keeps only text and pictures, and Weston or a plain X11 session keep nothing. Windows Explorer's process outlives its windows, so on Windows the files still paste.
2. **KDE's Klipper brought a pasted cut back.** After a paste moved a cut, the app emptied the clipboard. Klipper ("Prevent empty clipboard", on by default) then put the cut back. Paste stayed enabled, and a second Ctrl+V in the same folder asked whether to replace the moved file with itself, then said "Item Not Found".
3. **Files from a network share reached other apps as `smb://` addresses.** A browser upload, an email attachment or a terminal got nothing usable. A drag already gave them the share's local files.
4. **One name with a line break refused the whole Copy**, with "Encoded control characters are not allowed.". A drag carries such names, and Nautilus copies them.

## What changes

1. While the app owns the clipboard, closing the last window leaves it running without a window. It keeps serving what Copy, Cut or Copy as path put there. It ends as soon as another app copies something, or a paste empties a cut. Quit and `--quit` still end it at once, and a launch meanwhile opens its window in that process. (CLIP-010)
2. A cut that another app offers, naming only items a paste of this app moved, is no data. Paste stays disabled and nothing is dimmed, as Windows leaves it after a cut is pasted. (CLIP-008)
3. `text/uri-list` gives a mounted share's items (CIFS mount or GVfs FUSE export, found without mounting) as their local `file://` paths, as a drag does. `application/x-kde4-urilist`, which KDE apps prefer, keeps the `smb://` addresses. So do this app's format and GNOME's. (CLIP-004)
4. Names with a line break or a tab go on the clipboard escaped (`%0A`), one line per item, with no text format. Another app's list holding one pastes. NUL stays refused. (CLIP-005)

After a check on Windows 11 (2026-11-22):

5. A cut item fades only its icon. Its name and its Date modified, Type and Size columns stay at full strength, in Details and in icon views. (CLIP-002)

"Paste pictures as new files" stays on by default, as the owner chose. Turned off in Settings, a picture leaves Paste grayed, as in Windows 11.

Terminals that ask for text still get the copied files' paths while "Also copy paths as text" is on (the default), as GNOME Files gives them. Windows' classic console pastes nothing; Windows Terminal was not checked.

The parity notes, the user guide (`docs.json` and the generated Markdown) and `openxplorer.pot` are updated. No `.po` file is changed.

## Tests

- New: `window/tests/clipboard_lifetime.rs` (3 tests), `clipboard_shares.rs` (1) and `cut_look.rs` (2, on drawn pixels). In ox-core `clipboard/tests.rs`: two share tests, and the line-break test rewritten. Each failed before its fix and passes after.
- Real input, private Xvfb, unprivileged user, throwaway home: Ctrl+C, close the window, paste in Thunar, then copy text elsewhere and the process exits. The same on KWin Wayland with wl-paste and wl-copy. A cut pasted with Klipper running leaves Paste disabled, and a second Ctrl+V does nothing. Eight odd names (line break, unicode, quotes, `$HOME`, backticks) reach Chrome and another folder. Files on a local Samba share mounted through GVfs reach Chrome (193 and 10,867 bytes), Konsole (quoted local paths) and Dolphin. The FUSE export is stood in for by a folder link, because this container refuses FUSE mounts.
- rustfmt, Clippy with `-D warnings` (Rust 1.92.0), `parity/check.py`, `i18n.py check` and `sync-docs.py --check` pass. Every test executable ran through `check.py`'s isolation as an unprivileged user: 5,158 passed and 31 failed, exactly the 31 that fail on unchanged main in this container (its Python, sandboxing and mounts).

## Limitations

- The FUSE export itself and a kernel CIFS mount were not available here.
- Mutter nested and gnome-terminal did not run in this container.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
