# feat(folder-options): Show Folder Options as its own compact modal window, as Windows 11 does

Model: claude-opus-5-5

## Summary

Folder Options ("…" > Options) is now its own window, sized and laid out like Windows 11's.

**Windows of their own, modal to their window only**
- Folder Options is a top-level window transient for the Explorer window it was opened from. It has a title bar with its name and a close button, and can be moved anywhere, including off its parent. The close button and Esc cancel; OK and Apply save.
- Format, Map network drive and Add a network location get the same title bar (`Dialog::with_title_bar`). Windows also shows these as separate windows.
- Each Explorer window now has its own GTK window group, which its dialogs join. In GTK, a modal window blocks input to its whole group. All windows used to share the default group, so one window's dialog blocked every Explorer window. Now it blocks only its own window, as in Windows.
- The app adds no dimming of its own. KWin's "Dialog Parent" effect, which darkens the parent of any modal dialog, is left to the desktop.

**Compact layout at Windows' size**
- Built on the Properties window: about 380 by 483 pixels at 100 %.
- Classic General / View / Search tabs at the top left, with "Open OpenXplorer to:" and its list next to each other.
- Group boxes have their title on the top line and a 32-pixel picture at the left: Browse folders, Click items as follows, Privacy, Folder views.
- Option rows are 16 pixels apart and sub-choices are indented 16. Buttons are Windows push buttons.
- Each tab has Restore Defaults at its bottom right, then OK, Cancel and Apply. Apply is greyed until something changes.
- On the View tab, the Advanced settings list takes the rest of the tab. The tab pages keep one height, as Windows' property sheet pages do.
- Folder Options text is 11 pixels, so every option fits on one line with the app's fonts, which are wider than Segoe UI. A selector that takes priority over `window.ox` applies it.
- The Properties window can now take another width (`set_window_width`).

**Options left out**
- Only options the app can honor are shown. Windows' others are left out, for example "Open desktop folders and external folder links in new tab", Show files from Office.com, and the Search tab's index options. Single-click and its underline choices stay greyed with a tooltip, since the file list has no single-click mode.
- "More settings…" is removed: Windows' dialog has no such button, and Settings opens from the command bar's gear and Ctrl+,.

**The app's own name**
- "Open File Explorer to:" and "Clear File Explorer history" read "Open OpenXplorer to:" and "Clear OpenXplorer history", as Settings names the app.
- The manual, the web guides (regenerated from `docs.json`), the UI spec, the parity notes and the message template follow. The zh_CN catalog has no translation for the two renamed strings yet.

## Overlap

#253 (opening folders in their own window from Open) edits the same VIEW-021 note line in `native/parity/features.toml`, so whichever merges second has a one-line conflict there. Its change to `tests/folder_options.rs` merges cleanly and uses the same test helpers.

## Tests

- `cargo fmt --check` is clean. `cargo clippy --workspace --all-targets` shows the same 24 warnings as main, none in changed files.
- `native/parity/check.py`, `tools/i18n.py check` and `tools/sync-docs.py --check` pass.
- New: `folder_options_is_a_modal_window_of_its_own` (transient, modal, title bar and close button, own window group, size) and `folder_options_as_a_window_is_captured_light_and_dark`.
- Passing on Xvfb: folder_options (13), view_styles (17), file_name_extensions (5), dialog:: (11), modal:: (3), dialogs:: (64), theme:: (40), details_pane (32), properties (59 of 60).
- The failing properties test (mount assistant) fails the same way on main in the test container.
- Captures were taken at 2x under Openbox on Xvfb, with the window manager moving Folder Options partly off its parent. They were not taken on a Wayland compositor.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
