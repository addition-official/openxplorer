# Title

feat(settings): Export and import settings, and let mapped network folders drop their drive letters

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/29-long-c-thumb-cache)

## What changes for the user
- **Back up settings.** Settings > General ends with a "Back up settings" group holding "Export settings…" and "Import settings…", so a whole OpenXplorer setup can move to a new computer in one file.
- **Export** writes one readable JSON file, chosen in the desktop's Save dialog (the portal's where there is one), suggested as `OpenXplorer settings.json` and written private. It starts with `"kind": "openxplorer-settings"` and `"format": 1`, then the sections in the layout of `settings.json`: the Quick access pins, the standard folders unpinned from it, the Quick access order, the saved network folders (which hold no credentials), and every preference chosen in Settings.
- **Left on the computer:** window and dialog sizes, the daily update check with its last run and dismissed notice, the star prompt's count, the installed service actions turned on, and the recent files. Nothing kept outside `settings.json` is read: keyring entries and saved passwords, typed and recent locations, the thumbnail cache, the passwordless-update opt-in.
- **Import** checks the file and asks "Replace your OpenXplorer settings with the ones in {name}?" with Replace and Cancel. Replace runs the file through the same reader as `settings.json`, keeps this computer's own values listed above, saves, draws the theme, font and text size again, and tells every window, so open windows follow at once.
- Keys this version does not know are ignored, and a file of a newer format imports what this version understands. Folders and files not on this computer (pins, Favorites, Gallery folders, the start folder, icon files, the symbol font) are kept as they are and named in the message after the import. A file that is too large, not JSON, not an OpenXplorer settings file, or has a section of the wrong type is reported without asking, and nothing changes.
- **"Show letters on mapped network folders"**, a new switch in Settings > Appearance > Layout, on by default (the current behavior). Off, a share that Map network drive gave a letter is named without it, "Projects" rather than "Projects (Z:)", in the navigation pane, This PC (and so the Details pane) and Send to, and Map network drive hides its Drive list and asks "Specify the folder that you want to connect to:". The saved label keeps its letter, so existing mappings, the letters already taken and Disconnect work as before. Stored as `showDriveLetters`, only when off.
- For developers: `ox-core` `settings/backup.rs` (file format, export, import, missing paths, `Settings::import`, `COMPUTER_PREFERENCES`), `location::without_drive_letter`; `ox-app` `settings_page/backup.rs`, `AppContext::import_settings`, `Places::shown_label` and `map_network_dialog_with`. Parity SET-021 and SET-022; the Interface guide is updated.

## Why
- There was no way to take a setup to another computer short of copying `settings.json` by hand, which would also carry this computer's window sizes, update opt-in and service actions.
- A mapped network folder was always named with its drive letter, with no way to name it by its folder alone.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Back up settings"
- "Export settings…"
- "Export settings"
- "Saves your OpenXplorer settings in one file you can take to another computer: views, layout, confirmations, icons, Quick access pins and network folders. Passwords, history and window sizes stay here."
- "Import settings…"
- "Import settings"
- "Replaces your OpenXplorer settings with the ones in a file saved with Export settings."
- "Settings files (JSON)"
- "Settings exported to {name}."
- "Could not export settings to {name}: {error}"
- "Replace your OpenXplorer settings with the ones in {name}?"
- "Settings imported from {name}."
- "Settings imported from {name}. Not on this computer, kept as they are: {paths}."
- "{count} more"
- "Could not import settings from {name}: {error}"
- "This file is too large to be a settings file."
- "This is not an OpenXplorer settings file. Nothing was changed."
- "This settings file is damaged. Nothing was changed."
- "Show letters on mapped network folders"
- "Names a network folder mapped to a drive letter with its letter, as in Projects (Z:). Off, it is just Projects everywhere, and Map network drive asks for no letter. Typing a letter such as Z: in the address bar still opens its folder."
- "Specify the folder that you want to connect to:"
- Search keywords for the new rows: "export save backup back up transfer settings preferences json new computer migrate", "import load backup back up transfer settings preferences json new computer migrate", "drive letter letters mapped network drive folder share smb z: navigation pane sidebar map network drive label"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: the `ox-core` round trip `export_change_everything_import_and_everything_is_back` (every `Preferences` field at a non-default value, failing when a new field is not covered) and the other `settings/backup.rs` tests (contents and exclusions, key order, a newer format, broken and foreign files, missing paths); `a_label_loses_only_its_drive_letter`; `without_letters_a_mapped_share_is_named_by_its_folder`; `without_letters_map_network_drive_asks_only_for_the_folder`; the import message; on a private Xvfb display and message bus with a throwaway HOME, the `ox-core` settings and location tests and the `ox-app` tests of `settings_page`, `dialogs::network_map`, `places`, `window::sidebar`, `window::landing`, and `window::tests` landing_pages, send_to, network_shares and the mapped drive letter test (all passed except four Python-app comparison tests that need check.py's extracted v2.0.0 sources, and `create_shortcut_in_a_read_only_folder_offers_the_desktop`, which fails as root and passes as an unprivileged user; `every_preference_switch_saves_persists_and_turns_back_exactly` timed out once in a long batch and passed on rerun); rustfmt, Clippy, `native/tools/i18n.py check`, `native/parity/check.py` and `tools/sync-docs.py --check`; the real app with real mouse and keyboard input on a private display, as an unprivileged account with test files only: letters on and off, Map network drive with letters off, Export through GTK's Save dialog, Import with Replace restoring the theme, two switches, letters and a pin in both open tabs at once, a broken and a foreign file reported with `settings.json` left byte-identical, and a newer-format file with unknown keys and missing paths. Not checked: the portal's own Save dialog, and Properties of a connected share. No full test run was done.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
