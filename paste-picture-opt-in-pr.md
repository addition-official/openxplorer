Model: claude-opus-5-5

# feat(clipboard): Make picture paste and paths-as-text copy opt-in, off by default

## Summary

This keeps two clipboard features merged upstream and makes each one opt-in, off by default, because Windows 11 File Explorer does neither:

- #240: Paste writes a picture on the clipboard (for example a screenshot) as a new image file. In Windows 11 File Explorer, Paste is unavailable when the clipboard holds only a picture.
- #239: Copy and Cut also put the items' paths on the clipboard as plain text. In Windows 11 File Explorer, Ctrl+C copies only the files, and paths come from Copy as path (Ctrl+Shift+C).

Both are now switches in Settings > Files & folders > Clipboard:

- **Paste a picture from the clipboard as a new file** (`pastePicturesAsFiles`). Off: a picture is not read, so Paste and Paste into folder stay unavailable and Ctrl+V does nothing, as in Windows. On: #240's behavior, unchanged. Paste updates as soon as the switch changes, without copying again.
- **Also copy file paths as text when copying files** (`copyPathsAsText`). Off: Copy and Cut publish only the file formats, as in Windows. On: #239's `text/plain` paths, unchanged.

Each setting is stored only when on.

## Sources

- Shell clipboard formats (Microsoft Learn): https://learn.microsoft.com/en-us/windows/win32/shell/clipboard. Explorer transfers files as CF_HDROP, CFSTR_SHELLIDLIST and related formats. The page lists no picture format and does not say a file copy includes text.
- PowerToys Advanced Paste (Microsoft Learn): https://learn.microsoft.com/en-us/windows/powertoys/advanced-paste. Microsoft provides "Paste as .png file" as a separate PowerToys action.

## Changes

- ox-core: two new preferences, read and saved like the others.
- ox-app: a picture is read only while picture paste is on, and the window reads the clipboard again when the setting changes. The `text/plain` format is published only while paths-as-text is on. A new Clipboard group in Settings > Files & folders.
- Tests: the existing #239 and #240 tests turn their setting on first. New tests cover the defaults, turning each feature on, and the Settings switches (`a_picture_is_no_data_until_settings_turn_picture_paste_on`, `a_copy_offers_no_text_until_settings_turn_paths_as_text_on`, `the_clipboard_switches_are_off_by_default_and_save_the_choice`, `clipboard_extras_are_off_by_default_and_stored_only_when_on`).
- Parity CLIP-004, CLIP-015 and CLIP-016, `docs.json` with the generated Markdown, and the message template are updated.

## Testing

On the branch merged with main at 8fea905:

- `cargo fmt --check` and clippy: no new warnings.
- ox-core `clipboard` and `settings` tests; ox-app `window::tests::clipboard`, `window::tests::clipboard_interop`, `settings_page` and `file_ops::availability`, under Xvfb with a private D-Bus session. Two unreadable-file copy tests in `window::tests::clipboard` need a non-root user and fail as root in the test container, as they do on unchanged main.
- `parity/check.py`, `tools/i18n.py check` and `tools/sync-docs.py --check` pass.
- The Settings rows were captured in light and dark themes at 2x, and the branch was tested by hand in a combined build.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
