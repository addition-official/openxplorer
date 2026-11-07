Title: fix(dialogs): Make windows opened inside a file dialog its children, as in Windows 11

## The bug

OpenXplorer serves Open and Save dialogs for other applications through the desktop portal, and for its own Browse dialogs (for example Properties > Customize > Choose File...). On Wayland such a dialog is a modal child of the window that opened it.

A Properties window opened from inside one of these dialogs (from the Details pane's Properties button or the context menu) had no parent window. On KDE Plasma 6 (Wayland), KWin then showed the dialog as modal and the new Properties window as `transientFor=none`. That Properties window could be neither clicked nor closed while the dialog was open, so the dialog had to be cancelled to get rid of it. The progress window of a copy, move or delete started inside a dialog was parentless in the same way.

## The fix

- `BrowserWindow::own_in_dialog` makes a window opened from inside a file dialog the dialog's child (`set_transient_for`). It stays modeless, so the dialog stays usable beside it.
- Properties windows opened while the window is a file dialog are owned by it. They already close with the window that opened them (PROP-029), so they close with the dialog.
- The progress window takes the dialog as its parent when an action starts in a dialog. A browsing window's progress window still has no parent (OPS-019).
- `has_open_dialog` no longer counts these modeless Properties and progress windows as dialogs over the window, so they do not hold back drags and drops in the dialog.

Other windows a dialog can open (Open with, confirmations, sign-in) already have the dialog as their parent. New browser windows and Settings are disabled in dialogs.

## How Windows behaves

In Windows 11, Properties opened from inside an Open or Save dialog opens above that dialog and works normally, and it closes with the dialog. This follows the Win32 rules for owned windows: "An owned window is always above its owner in the z-order. The system automatically destroys an owned window when its owner is destroyed." ([Window Features: Owned Windows](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#owned-windows))

## Testing

- New tests in `native/crates/ox-app/src/window/picker.rs`, run against a real portal backend on a private D-Bus bus under Xvfb:
  - `properties_opened_from_a_dialog_belong_to_the_dialog`: Properties opened from a modal Open dialog is the dialog's modeless child and is not counted as a dialog over it. Its Cancel closes it and leaves the dialog open, and another Properties window closes with the dialog.
  - `the_progress_of_a_copy_in_a_dialog_belongs_to_the_dialog`: a browsing window's progress window has no parent, while a dialog's has the dialog as its parent, stays modeless and is not counted as a dialog over it.
  - `properties_from_a_dialog_are_captured_light_and_dark`: captures for review.
- `cargo fmt --check`, Clippy with `-D warnings` and `native/parity/check.py` pass. `native/tools/check.py` passes (75 test executables and the doctests). Its ox-app executable now takes about 11 minutes here, so it needed `--test-timeout` above the default 600 seconds.
- Xvfb has no compositor, so the KWin behavior itself was not reproduced there. GTK 4 leaves modal blocking to the compositor, and a click on the parentless Properties window already worked under Xvfb before the fix. On KDE Plasma 6 (Wayland), Properties opened from Chrome's Open dialog (Ctrl+O) no longer gets stuck.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
