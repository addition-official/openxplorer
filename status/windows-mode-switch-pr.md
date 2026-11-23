# Add "Behave like Windows Explorer", one switch for every setting with a Windows behavior

Model: claude-opus-5-5
CLA: I agree

## Summary

Several settings keep OpenXplorer's own behavior by default and offer Windows 11 File Explorer's as an option. This adds one switch, **Behave like Windows Explorer**, as the first row of Settings > General, that sets all of them at once.

- **Off by default.** The owner's defaults are untouched until the switch is turned on.
- **On** saves every setting's Windows value and remembers the values from before, stored as `preferences.behaveLikeWindows` (an object of the earlier values) only while it is on. Turning it on again while it is on keeps what was first remembered.
- **Off** puts the remembered values back and removes `behaveLikeWindows`.
- **Custom.** Each setting keeps its own row and works on its own. If one is changed while the switch is on, the switch stays on, drawn only outlined in the accent (as Windows draws a partly ticked box), with a "Custom" label before it and a line under the title: "Custom: 1 setting was changed since this was turned on."
- The switch changes no behavior itself; it only saves settings, through the same `PreferencesUpdate::from_json` path as the settings file, so every value is checked as one from the file is.

## The registry

`native/crates/ox-core/src/settings/windows_mode.rs` has `WINDOWS_SETTINGS`, one line per setting:

```rust
like_windows!("pastePicturesAsFiles", paste_pictures_as_files, false, "Paste pictures as new files"),
```

(the `settings.json` key, the `Preferences` field, the Windows value, the row's title). It lists:

| Setting | Windows value |
|---|---|
| Open Quick access on Home | on |
| Hide expand arrows in the sidebar | on |
| Show more items in the navigation pane | off |
| Show more items in the tab menu | off |
| Column header style | Windows |
| When replacing a file, move the old one to the Recycle Bin | off |
| Delete files on removable drives permanently (like Windows) | on |
| Paste pictures as new files | off |
| Also copy paths as text | off |

A unit test checks that each line's key reads into the field it names and that the default is not already the Windows value.

## Guard test

`settings_page::windows_mode::tests::every_setting_titled_like_windows_is_in_the_registry` scans the app's sources for string literals containing "(like Windows)" or "(Windows)" and fails, naming the title and file, when one is not a registry title.

## Still to hook in

These settings are not on this branch yet. Each needs one line in `WINDOWS_SETTINGS` once its branch merges; the guard test fails until it is added (for the titled ones).

- "Restore over the original (Windows)", key `restoreOverOriginal`, Windows value `true`:
  `like_windows!("restoreOverOriginal", restore_over_original, true, "Restore over the original (Windows)"),`
- "Match words at the start of file extensions (like Windows)", key `extensionWordStarts`, Windows value `true`:
  `like_windows!("extensionWordStarts", extension_word_starts, true, "Match words at the start of file extensions (like Windows)"),`
- Running installed launchers like Windows (shortcuts work): `pass-17-shortcuts` as merged here has no setting for it yet.
- F6 like Windows: `pass-19-a11y` as merged here has the F6 regions but no setting for them yet.

A setting stored inside `viewOptions` would need a small extension of the macro; none of the current entries does.

## Other changes

- Settings row in `native/crates/ox-app/src/settings_page/windows_mode.rs`, first group of the General page; the `mixed` switch style in `resources/skin/settings.css`.
- Docs: a paragraph in `apps/web/lib/docs.json` (Interface guide), regenerated with `tools/sync-docs.py`.
- Parity: `SET-020` in `native/parity/features.toml`.
- `native/po/openxplorer.pot` regenerated with `native/tools/i18n.py extract`; no `.po` file changed.
- The branch also carries `sort-header-windows` (Column header style), merged here so that setting is covered.

## Tests run

- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked` with Rust 1.92.0 as a separate tester user: no findings in the changed files. Clippy with `-D warnings` still stops on three findings in files this change does not touch (`ox-core/src/entry/enumerate.rs:386`, `ox-app/src/properties/general_tab.rs:87`, `ox-app/src/properties/item_window.rs:113`), which come from the merged branches.
- ox-core `settings::` unit tests, including the registry check, on → off restore, Custom → off restore and the untrusted-value check.
- ox-app `settings_page` tests under a private Xvfb display, D-Bus session and throwaway HOME (check.py's isolation), including the guard test and the GTK test that turns the switch on, checks each registry row shows its Windows value, changes one row (Custom), turns it off and checks each row shows its earlier value and the settings file has no `behaveLikeWindows`.
- `python3 native/parity/check.py`, `python3 native/tools/i18n.py check` and `python3 tools/sync-docs.py --check`.
- The full `native/tools/check.py` run was not done.
