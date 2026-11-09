# Title

feat(updates): Hide the command bar's Update button unless Settings > About asks for it

# Body

When a newer release is out, OpenXplorer showed two controls that open the same Software updates window: an accent Update button in the command bar (Restart after an install) and Update available in the status bar. This change hides the command bar's button by default, since Windows 11 File Explorer's command bar holds only file commands, and adds Settings > About > Updates > "Show the Update button in the command bar" (off by default). Turning it on brings the button back exactly as it is on main, with Update while a release waits and Restart once one is installed. The status bar's Update available, the top-right notice and the rest of Settings > About are unchanged, and nothing about checking for or installing updates changes.

Question for the maintainer: this PR hides the command bar's Update button by default, as Windows 11 File Explorer has no such button, and keeps it behind the new setting "Show the Update button in the command bar". Would you prefer it hidden by default, or shown by default? Either way it is a one-line change to the setting's default.

Changes:

- A new preference, `showUpdateButton`, stored only when on, and its switch on Settings > About under Updates.
- Each window shows the command bar's Update only while the setting is on, and follows changes to it.
- The manual, the installation guide and its generated copies, the parity entries, the translation template and comments describe the setting.

How tested:

- New tests, written to fail first: `the_command_bar_update_button_is_hidden_until_chosen` (ox-core settings) and `the_command_bar_shows_update_only_when_settings_ask` (ox-app). The second finds a simulated release, checks that the command bar has no Update, flips the setting's switch in Settings > About, returns to a folder tab and checks that Update appears, then turns it off again.
- The touched suites pass in the isolated test session: ox-app `update::` (23), `window::tests::command_bar::` (28) and `settings_page::` (103), and ox-core `settings::` (122). `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `native/parity/check.py`, `tools/sync-docs.py --check` and `native/tools/i18n.py check` pass.
- The installing test now turns the setting on and still sees Restart in the command bar after an install.
- The real app, run on Xvfb at 1x in dark mode with fictional files and real pointer input: Settings > About shows the new switch off; clicking it turns it on and saves `showUpdateButton: true`. With no release available in that offline session, the command bar shows no Update, as expected.
- The full `native/tools/check.py` run was not repeated for this change.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
