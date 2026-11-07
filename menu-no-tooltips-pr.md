# Title

fix(menus): Show no tooltip on menu rows, as Windows 11's menus show none

# Body

Model: claude-opus-5-5

## Summary

Windows 11 File Explorer shows no tooltip when the pointer rests on a menu row: the row's text is the whole label. OpenXplorer showed one on two kinds of menu rows:

| Menu row | Before | After |
|---|---|---|
| A row whose label the menu cuts short | Tooltip with the whole label | No tooltip |
| A disabled row, such as Paste with nothing to paste | Tooltip with the label and the reason, for example "Paste / Nothing to paste here." | No tooltip; the reason stays in the row's accessible description for screen readers |

This applies to every menu built on the app's menu popover: the compact and classic right-click menus of files, folders and the folder background, New, Sort, View, "…", the navigation pane's menus, This PC's menus, and their submenus (such as Open with, Send to, Compress to, More and Group by).

Unchanged, as Windows has tooltips there:

- The command bar's icon-only buttons (Cut, Copy, Paste, Rename, Share, Delete) and the other toolbar buttons.
- The compact right-click menu's top strip of icon-only buttons, which still name their command and say why one is disabled.

Accessible names and descriptions are unchanged: every row keeps its label as its accessible name, and a disabled row still gives the reason as its accessible description.

Sources for Windows' behavior:

- Microsoft's design guidance for the command bar flyout that Windows 11's right-click menu follows: commands in the menu show their label and icon, while icon-only primary commands get a tooltip ([Command bar flyout, Microsoft Learn](https://learn.microsoft.com/windows/apps/design/controls/command-bar-flyout)).
- The Windows 11 screenshots of the Sort menu and the right-click menus already referenced in `native/parity/features.toml` (CMD-005) show rows with whole labels and no tooltip. No new Windows hover screenshot was taken for this change.

## Changes

- `native/crates/ox-app/src/window/menu_popover.rs`: menu rows no longer set a tooltip or answer tooltip queries; a disabled row gets only the accessible description (`describe_unavailable_row`). The strip buttons' code is unchanged.
- `native/crates/ox-app/src/window/menu_popover/inspection.rs`: `row_tooltip` is replaced by `disabled_reason_of`, which checks that a disabled row has an accessible description and returns the reason the menu gave it, since GTK cannot read an accessible description back.
- Tests: `window/tests/chrome.rs` has `no_menu_row_shows_a_tooltip_even_cut_short_or_disabled` in place of `a_menu_row_cut_short_shows_its_whole_label_as_a_tooltip`; `window/tests/command_bar.rs` and `window/tests/context_menus.rs` read the disabled rows' reasons with `disabled_reason_of` and check that the rows have no tooltip.
- `native/parity/features.toml`: the CMD-005 and CMD-031 notes describe the new behavior and test name.
- Doc comments in `disabled_reasons.rs`, `file_ops/availability.rs` and `menu_popover/items.rs` say where the reasons are shown now.

## Testing

- `cargo fmt --all --check`
- `cargo clippy -p ox-app --all-targets --locked`: no warnings in the changed files
- `python3 native/parity/check.py` and `python3 native/tools/i18n.py check`
- Menu, context menu, chrome and command bar tests by name, under `xvfb-run` and `dbus-run-session` with a private home: all pass except four that fail the same way on the unmodified base (they need a non-root user, the Python 1.x checkout, or the isolated XTEST marker)
- Real pointer input under Xvfb with `xdotool`, the app running as an unprivileged user on a fictional folder tree, at 1x (light) and 2x (dark): the pointer rested 2 s on every row of each menu and submenu listed above, and no tooltip appeared; the command bar buttons and the compact menu's strip buttons still showed theirs. The same check on the unmodified base shows the disabled-row tooltips, so the check detects them.
- The same real-input check on a local merge of this branch with the pending right-click menu rework (which adds the Open with and Compress to submenus and This PC's group heading menu), with two fictional applications installed: no tooltip on any of those rows, at 1x light and 2x dark.
- Rerun after merging the latest upstream main: the same results, and the real-input check of the file, background, Sort and "…" menus, their submenus, the strip and the command bar at 1x light and 2x dark
- Tested in a combined Debian package build

The full `native/tools/check.py` was not run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
