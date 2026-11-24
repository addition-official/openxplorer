# Title

feat(settings): Add "Behave like Windows Explorer", one switch for every setting with a Windows behavior

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/30-settings-export-letters)

This is the last PR of the series; its tree equals the full series.

## What changes for the user
- **"Behave like Windows Explorer"**, a new switch in its own "Windows Explorer" group, the first group of Settings > General. Several settings keep OpenXplorer's own behavior by default and offer Windows 11 File Explorer's as an option; this switch sets all of them at once.
  - **Off by default.** The owner's defaults are untouched until it is turned on.
  - **On** saves every listed setting's Windows value and remembers the values from before, stored as `preferences.behaveLikeWindows` (an object of the earlier values) only while it is on. Turning it on again while it is on keeps what was first remembered.
  - **Off** puts the remembered values back and removes `behaveLikeWindows`.
  - **Custom.** Each setting keeps its own row and works on its own. If one is changed while the switch is on, the switch stays on, drawn only outlined in the accent (as Windows draws a partly ticked box), with "Custom" before it and a line under the title: "Custom: 1 setting was changed since this was turned on."
  - The switch changes no behavior itself; it only saves settings, through the same `PreferencesUpdate::from_json` path as the settings file, so every value is checked as one from the file is.
- **The settings it sets**, with their Windows values:

  | Setting | Windows value |
  |---|---|
  | Open Quick access on Home | on |
  | Keep Properties windows a fixed size (like Windows) | on |
  | Ask for a GitHub star | off |
  | Ctrl+plus and Ctrl+minus change text size | off |
  | Hide expand arrows in the sidebar | on |
  | Show more items in the navigation pane | off |
  | Show more items in the tab menu | off |
  | Column header style | Windows |
  | Show a line under column titles | off |
  | Name column starts at Windows' width (256 px) | on |
  | When replacing a file, move the old one to the Recycle Bin | off |
  | Delete files on removable drives permanently (like Windows) | on |
  | Paste pictures as new files | off |
  | Also copy paths as text | off |
  | F6 moves like Windows | on |
  | Run installed apps' launchers on double-click | on |
  | Restore over the original (Windows) | on |
  | Match words at the start of file extensions (like Windows) | on |
  | Show all search matches (like Windows) | on |
  | Neutral selection colors (like Windows) | on |
  | Show Recent files in the navigation pane | off |
  | Connect to shares opened by other apps right away (like Windows) | on |
  | Keep Open/Save dialogs modal on GNOME (like Windows) | on |
  | Show check boxes only while Ctrl is held | off |

  The column title line and Name width rows are pass 7's settings from PR 26; the F6, launcher, Restore and extension-search rows come from PRs 20, 18 and 27; the search matches, selection colors, Recent files, shares and modal dialog rows come from PR 27.
- **"Ctrl+plus and Ctrl+minus change text size"**, a new row in Settings > Appearance, right after Text size, on by default, as before. Off, Ctrl+plus and Ctrl+minus on the main keyboard do nothing, in the browser windows and in dialogs, as in Windows 11; Ctrl+plus on the number pad still sizes the Details columns to fit, and Ctrl+number pad minus, Ctrl+0 and the Text size control keep working. Stored as `ctrlPlusMinusTextSize`, only when off.
- **"Ask for a GitHub star"**, a new row in Settings > About, after the update rows, on by default (the owner's choice). Off, the star prompt never pops up, as Windows asks for nothing of the kind; Dismiss works as before. Stored as `askForStar`, only when off, and exported with the other preferences, unlike the prompt's count and dismissal. Whatever the setting says, another app's Open or Save dialog never shows the prompt and leaves it due for a browser window.
- **"Keep Properties windows a fixed size (like Windows)"**, a new row in Settings > General with the window settings, off by default. A Properties window opens at Windows' 380 x 520 and can be made larger, as before; with the setting on, every Properties window, for a file, a folder, a drive, a shortcut or several items, opens at that one size and cannot be resized, as in Windows. Stored as `fixedPropertiesSize`, only when on.
- The Interface guide describes the switch and the two new rows.
- For developers: `native/crates/ox-core/src/settings/windows_mode.rs` holds `WINDOWS_SETTINGS`, one `like_windows!` line per setting (the `settings.json` key, the `Preferences` field, the Windows value and the row's title). A guard test fails, naming the title and file, when a setting title names Windows (the whole word, in any letter case) and is neither in the registry nor in a listed exception with its reason (the switch itself, "Show letters on mapped network folders", the three Windows icon file and font rows, and four titles about the app's own windows). Parity SET-020; the settings backup round trip now also changes the Windows-mode and Name column preferences. New tests: `an_open_dialog_never_shows_the_star_prompt` (`window/picker.rs`) and Properties window size tests in `window/tests/properties_width.rs`.

## Why
- Settings with a Windows behavior were spread over several pages, so getting Windows 11 File Explorer's behavior meant finding and changing each one.
- In Windows 11 File Explorer, Ctrl+plus and Ctrl+minus on the main keyboard do nothing, and Ctrl+plus on the number pad sizes all columns to fit; the app uses the main keys for its text size, so this is now a setting the switch can turn off.
- Without a guard, a new setting with a Windows behavior could be left out of the switch; the first guard matched only titles with "(like Windows)" or "(Windows)", so titles such as "F6 moves like Windows" were covered only because they were registered by hand.
- Windows asks for no GitHub star, and Windows' property sheets have one fixed size; both now have a setting the switch can set.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Windows Explorer"
- "Behave like Windows Explorer"
- "Sets every setting that has a Windows 11 File Explorer behavior to it, all at once. Each setting still has its own switch. Turning this off puts back the settings you had before you turned it on."
- "Uses Windows 11 File Explorer's behavior wherever OpenXplorer offers a choice."
- "Custom"
- "Custom: {count} setting was changed since this was turned on." / "Custom: {count} settings were changed since this was turned on."
- "Ctrl+plus and Ctrl+minus change text size"
- "Ctrl+plus and Ctrl+minus on the main keyboard make text larger and smaller. Off: they do nothing, as in Windows; Ctrl+plus on the number pad still sizes the Details columns to fit, and Text size still changes it."
- "Ask for a GitHub star"
- "After a few starts, a card asks once …" (the GitHub star card's description; its full text is in `native/po/openxplorer.pot`)
- "Keep Properties windows a fixed size (like Windows)"
- "Every Properties window, for a file, a folder, a drive, a shortcut or several items, opens at the same size and cannot be resized, as in Windows. Off: it opens at that size and can be made larger."
- Search keywords for the new rows: "windows explorer mode like windows behave all settings at once master switch file explorer 11 defaults compatibility", "ctrl plus minus equal keyboard shortcut keys text size larger smaller number pad numpad keypad columns fit windows", "github star prompt card ask rate enjoying nag popup reminder dismiss", "properties window size fixed resize resizable larger same size dialog windows 380 520"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked` with Rust 1.92.0 as a separate tester user (no findings in the changed files); the ox-app `text_size` tests (28), including the check that the main keyboard's plus and minus are left out when the setting is off and the existing test that the text-size keys work inside dialogs; the ox-core `settings::` unit tests, including the registry check (`every_line_reads_and_writes_the_same_setting`), on and off restore, Custom and off restore, and the untrusted-value check; the ox-app `settings_page` tests under a private Xvfb display, message bus and throwaway HOME, including the guard test and the GTK test that turns the switch on, checks each row shows its Windows value, changes one row (Custom), turns it off and checks each row shows its earlier value and the settings file has no `behaveLikeWindows`; `python3 native/parity/check.py`, `python3 native/tools/i18n.py check` and `python3 tools/sync-docs.py --check`. The full `native/tools/check.py` run was not done.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
