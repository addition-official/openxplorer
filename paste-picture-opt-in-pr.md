Model: claude-opus-5-5

# feat(clipboard): Let users turn off paths-as-text copy and picture paste (both stay on by default)

## Summary

Nothing changes by default. This adds two switches, both on, that let users turn off two clipboard behaviors:

- Copy and Cut also put the items' paths on the clipboard as plain text (#239, CLIP-004).
- Paste writes a picture on the clipboard, such as a screenshot, as a new image file (#240, CLIP-015).

The switches are in Settings > Files & folders > Clipboard, a page whose heading reads "Your OpenXplorer, your way.":

- **Paste a picture from the clipboard as a new file** (`pastePicturesAsFiles`). On: unchanged. Off: a picture is not read, so Paste and Paste into folder are unavailable and Ctrl+V does nothing, as in Windows 11 File Explorer. Paste follows the switch at once, without copying again.
- **Also copy file paths as text when copying files** (`copyPathsAsText`). On: unchanged. Off: Copy and Cut publish only the file formats, as Windows 11 File Explorer does, where paths come from Copy as path (Ctrl+Shift+C).

Each setting is stored only when off, so fresh installs and existing settings files behave exactly as main does.

## Windows 11 behavior, for the off setting

- Shell clipboard formats (Microsoft Learn): https://learn.microsoft.com/en-us/windows/win32/shell/clipboard. Explorer transfers files as CF_HDROP, CFSTR_SHELLIDLIST and related formats. The page lists no picture format and does not say a file copy includes text.
- PowerToys Advanced Paste (Microsoft Learn): https://learn.microsoft.com/en-us/windows/powertoys/advanced-paste. Microsoft provides "Paste as .png file" as a separate PowerToys action.

## Changes

- ox-core: two preferences, on by default, stored only when off.
- ox-app: the picture read and the `text/plain` format follow their switches; the window reads the clipboard again when picture paste changes. A Clipboard group in Settings > Files & folders.
- Tests: the existing #239 and #240 tests are unchanged and pass at the defaults. New tests cover each switch off (`window/tests/clipboard_switches.rs`), the Settings switches (`the_clipboard_switches_are_on_by_default_and_save_the_choice`) and how the preferences are stored (`clipboard_extras_are_on_by_default_and_stored_only_when_off`).
- Parity CLIP-004, CLIP-015 and CLIP-016, `docs.json` with the generated Markdown, and the message template are updated.

## Testing

On the branch merged with main at bab3fb5:

- `cargo fmt --check` and clippy: no new warnings.
- ox-core `clipboard` and `settings` tests; ox-app `window::tests::clipboard`, `window::tests::clipboard_interop`, `window::tests::clipboard_switches`, `settings_page` and `file_ops::availability`, under Xvfb with a private D-Bus session. Two unreadable-file copy tests in `window::tests::clipboard` need a non-root user and fail as root in the test container, as they do on unchanged main.
- `parity/check.py`, `tools/i18n.py check` and `tools/sync-docs.py --check` pass.
- The release build was run under Xvfb as an ordinary user, in a temporary home with sample files, using real keyboard input (xdotool) and another application owning the clipboard (xclip):
  - Defaults: Ctrl+C offered the path as text, and Ctrl+V with a PNG on the clipboard wrote one .png file.
  - Picture paste off: Ctrl+V wrote nothing, and Paste was greyed out in the folder's menu; Ctrl+C still offered the path as text.
  - Paths as text off: Ctrl+C offered only the file formats and a text reader got nothing; Ctrl+V still wrote the picture.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
