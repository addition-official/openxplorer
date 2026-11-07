Model: claude-opus-5-5

Title: fix(navigation): Leave the keyboard shortcuts out of the Back, Forward, Up and Refresh tooltips, as Windows 11 does

## Summary

AKolenda/openxplorer#210 added keyboard shortcuts to the tooltips of the Back, Forward, Up and Refresh buttons, for example "Back to Downloads (Alt + Left Arrow)". Windows 11 File Explorer's tooltips name where each button leads but leave out the shortcut. This change removes the shortcut text and keeps the target names.

| Button  | Before                                  | After (as in Windows 11) |
|---------|-----------------------------------------|--------------------------|
| Back    | Back to Downloads (Alt + Left Arrow)    | Back to Downloads        |
| Forward | Forward to Documents (Alt + Right Arrow)| Forward to Documents     |
| Up      | Up to "Desktop" (Alt + Up Arrow)        | Up to "Desktop"          |
| Refresh | Refresh "Documents" (F5)                | Refresh "Documents"      |

With no target, the tooltips read "Back", "Forward", "Up" and "Refresh".

Source for Windows 11's wording: hovering over these buttons in Windows 11 File Explorer, with and without a target.

The keys are unchanged: Alt+Left, Alt+Right, Alt+Up, F5 and Ctrl+R work as before.

## Changes

- `native/crates/ox-app/src/window/navigation_buttons.rs`: the tooltips no longer include the keys. With no target, a button's tooltip is its accessible name, so the separate untargeted tooltip strings are removed.
- `native/parity/features.toml`: NAV-001, NAV-010, NAV-013 and CMD-016 quote the new tooltips.
- `native/po/openxplorer.pot`: regenerated with `tools/i18n.py extract`.
- Tests: `window/tests/history.rs` expects the new tooltips; two test names that mentioned the keys are renamed, and the parity notes point to the new names. `window/tests/slow_locations.rs` checks Refresh's tooltip without F5.

## Testing

- `cargo fmt --all --check`
- `cargo clippy -p ox-app --all-targets --locked`: no warnings in the changed files
- `window::tests::history` and `window::tests::slow_locations` under xvfb-run and dbus-run-session: 35 passed
- `python3 native/tools/i18n.py check` and `python3 native/parity/check.py`
- The Back tooltip, checked by eye in light and dark at 2x, in an isolated session with the fictional demo tree
- Tested in a combined Debian package build

The full `native/tools/check.py` was not run locally.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
