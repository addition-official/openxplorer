# Title

feat(settings): Export and import settings, and let mapped network folders drop their drive letters

# Body

Model: claude-opus-5-5

## Summary

Two additions to Settings.

**Back up settings.** Settings > General now ends with a "Back up settings" group holding Export settings… and Import settings…, so a whole OpenXplorer setup can move to a new computer in one file.

- Export writes one readable JSON file, chosen in the desktop's Save dialog (the portal's where there is one), suggested as `OpenXplorer settings.json` and written private (0600). It starts with `"kind": "openxplorer-settings"` and `"format": 1`, then the sections in the layout of `settings.json`: the Quick access pins, the standard folders unpinned from it, the Quick access order, the saved network folders, and every preference chosen in Settings (views and layout, the confirmations, the settings with a Windows behavior, the column header style, the icon file paths, the file icons, Favorites, Gallery's folders, the folder views and so on).
- Left on the computer: window and dialog sizes, the daily update check with its last run and dismissed notice, the star prompt's count, the installed service actions turned on (keyed by files of that computer, and opt-in on each one), and the recent files. Nothing kept outside `settings.json` is read at all: keyring entries and saved passwords, typed and recent locations, the thumbnail cache, the passwordless-update opt-in.
- Import reads the file, checks it, and asks "Replace your OpenXplorer settings with the ones in <file name>?" with Replace and Cancel. Replace runs the file through the same reader as `settings.json`, keeps this computer's own values listed above, saves, draws the theme, font and text size again, and tells every window that the places and folder styles changed, so open windows follow at once.
- Keys this version does not know are ignored. A file of a newer format imports what this version understands. Folders and files that are not on this computer (pins, Favorites, Gallery folders, the start folder, icon files, the symbol font) are kept as they are and named in the message after the import. A file that is not JSON, not an OpenXplorer settings file, or has a section of the wrong type is reported without asking, and nothing changes.

**Show letters on mapped network folders.** A new switch in Settings > Appearance > Layout, on by default (the current behavior). Off, a share Map network drive gave a letter is named without it, "Projects" rather than "Projects (Z:)", in the navigation pane, This PC (and so the Details pane, which takes This PC's name), and Send to. Map network drive then hides its Drive list and asks "Specify the folder that you want to connect to:". The saved label keeps its letter, so existing mappings, the letters already taken and Disconnect work as before, and turning the switch on again shows the letters. Tabs, the address bar and Properties name a share by its folder and never showed this label, so they need no change.

This switch is not registered with "Behave like Windows Explorer" (claude/windows-mode-switch). That branch's guard flags only setting titles containing "(like Windows)" or "(Windows)", and this title has neither, so no exception is needed there. The export includes the switch, and it will include `behaveLikeWindows` with its saved earlier values once that branch is merged, because the export takes every preference except the short list of this-computer keys in `COMPUTER_PREFERENCES`.

## Changes

- `ox-core`: `settings/backup.rs` (file format, export, import, missing paths, `Settings::import`), `Preferences::show_drive_letters` stored as `showDriveLetters` only when off, `location::without_drive_letter`.
- `ox-app`: `settings_page/backup.rs` (rows, file dialogs, question, messages), `AppContext::import_settings`, the new row in `appearance.rs`, `Places::shown_label` used by the sidebar, This PC and Send to, and `map_network_dialog_with` for Map network drive.
- Parity features SET-021 and SET-022, the interface guide (`apps/web/lib/docs.json`, regenerated with `tools/sync-docs.py`), and `native/po/openxplorer.pot` regenerated with `native/tools/i18n.py extract`. No `.po` file changed.

## How it was tested

- Round trip in `ox-core` (`export_change_everything_import_and_everything_is_back`): settings with every `Preferences` field at a non-default value, pins, a share and a Quick access order are exported; then everything is changed; the import brings every exported value back, and the window size, update check and recent files stay as they were just before it. The test counts the fields of `Preferences` and fails when a new one is not covered.
- Other new tests: what the file holds and leaves out, key order, a newer format, broken and foreign files, missing paths (`settings/backup.rs`); `a_label_loses_only_its_drive_letter`; `without_letters_a_mapped_share_is_named_by_its_folder` (sidebar); `without_letters_map_network_drive_asks_only_for_the_folder`; the import message.
- Run on a private Xvfb display and session bus with a throwaway HOME: the `ox-core` settings and location tests; the `ox-app` tests of `settings_page`, `dialogs::network_map`, `places`, `window::sidebar`, `window::landing`, and `window::tests` landing_pages, send_to, network_shares and the mapped drive letter test. All pass except four Python-app comparison tests that need `check.py`'s extracted v2.0.0 sources, which this checkout cannot fetch, and `create_shortcut_in_a_read_only_folder_offers_the_desktop`, which fails as root (mode 0555 does not stop root) and passes as an unprivileged user. `every_preference_switch_saves_persists_and_turns_back_exactly` timed out once in a long batch and passed in the next batch and three times alone. No full test run was done.
- rustfmt, Clippy (no warnings in the changed files), `native/tools/i18n.py check`, `native/parity/check.py` and `tools/sync-docs.py --check` pass.
- In the real app with real mouse and keyboard input, on a private display, as an unprivileged account with a throwaway HOME and test files only: letters on and off in the sidebar, This PC and the Details pane; Map network drive with letters off; Export through GTK's Save dialog; theme, two switches, letters and a pin changed by hand, then Import asked the question and Replace put all of them back in both open tabs at once; a broken and a foreign file reported without asking and `settings.json` left byte-identical; a newer-format file with unknown keys, a missing pin and a missing icon file imported, with both named.
- Not checked: the portal's own Save dialog (the private session has no portal, so GTK's dialog ran), and Properties of a connected share (no SMB server).

## Notes

- Saved network folders are exported with the pins, since they are listed in the navigation pane. They hold no credentials; passwords stay in the keyring.
- Typing a mapped letter such as `Z:` in the address bar and pressing Enter is refused on test-combined-54 with letters on or off: Enter goes through `resolve_relative`, which never looks up mapped letters (only `resolve_address` does, and its test calls it directly). This is not changed here.
- Merging with claude/windows-mode-switch conflicts only in `apps/web/public/assets/markdown.json` and `native/po/openxplorer.pot` (regenerate both) and in the `added_since` list of `settings_page/tests/redesign.rs` (keep both sides).

CLA: I agree
