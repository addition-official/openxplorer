# fix(properties): Draw Properties text at Windows' 9-point dialog size

Follows up #271, which gave every item Properties one fixed 402 x 570 size.

## Problem

Every Properties window drew its text at 13 px rather than the 12 px that `theme/fonts.rs` intends for `.ox-properties`. The `window.ox` rule (13 px) outranks a bare class, so the 12 px rule never applied. Windows draws dialog and property sheet text in Segoe UI 9 pt (Microsoft, "Fonts", Windows UX guidelines, https://learn.microsoft.com/en-us/windows/win32/uxguide/vis-fonts: normal text is "9 pt. ... Segoe UI"; "For Segoe UI, use a 9 point font size or larger."). At 96 dpi, 9 pt is 12 px.

## Changes

- `theme/fonts.rs`: new rule `window.ox.ox-properties:not(.ox-folder-options) { font-size: 12px }`, scaled with the text size like every other font rule. File, several items, folder, drive and shortcut Properties and their dialogs (Change Icon, Security's Edit and Advanced, Advanced Sharing) now use 12 px. Folder Options is left out, so it keeps its own size (13 px on its own, or 11 px with #269). The old bare `.ox-properties` rule stays in place, so this merges cleanly with #269, whose rule sits next to it.
- With Noto Sans (what the app renders on Zorin), a drive's Customize and Previous Versions tabs were taller than the page under two rows of tabs, so they scrolled inside the fixed window. This also happens after #271 at 13 px. Two fixes:
  - Group boxes (Sharing, Customize, Advanced Sharing) draw their title on the box's top line, as Windows does. `properties::group()` turns off the frame's clipping, and `properties.css` lifts the title. Folder Options keeps its own group rules.
  - The Previous Versions list's minimum height is now 216 px, down from 230. The list still grows to fill the tab.

## Tests

- New: `theme::fonts::tests::properties_windows_draw_windows_dialog_text` and `window::tests::properties_width::every_kind_of_properties_draws_windows_dialog_text`. The second test covers every kind of Properties and every tab. It checks that every label is 12 px, that only a long value's one-line field ends in "…", and that no label is cut. One line of text is 17 px high on every window with Noto Sans. It also checks that Folder Options keeps its own size.
- Named tests (properties, tab_rows, item_dialogs, customize, details_tab, security, sharing, dialog_keys, late_replies, folder_options, view_styles, theme::) pass at 1x and 2x under Xvfb with Noto Sans, on current main: 185 of 186. The one failure, `mount_assistant::the_assistant_prepares_a_command_and_hands_over_the_linux_folder`, is reported failing on main without this change. `every_kind_of_properties_has_one_size` passes at both scales. #269 (Folder Options, not yet merged) has its own conflicts with current main, but none in the files this PR changes. Its 11 px rule sits beside the unchanged bare `.ox-properties` line, and an earlier trial merge showed Folder Options at 11 px.
- rustfmt is clean. Clippy shows no warnings in the changed files. `native/parity/check.py` passes.

## Checklist (real input with xdotool under Xvfb and Openbox, app run as an unprivileged user on fictional files)

For each of 1x light, 1x dark, 2x light and 2x dark (89 checks each, all pass):

- [x] File: Alt+Enter opens it; General, Security, Details and Previous Versions (Ctrl+Tab) each stay 402 x 570 and don't scroll under the mouse wheel; Esc closes it
- [x] Shortcut: the same four tabs
- [x] Several items (click, then Ctrl+click): General and Security
- [x] Folder: General, Sharing, Security, Previous Versions and Customize; Change Icon... opens its dialog and Esc closes it
- [x] Drive (Local Disk from the sidebar): General, Hardware, Sharing, Security and Previous Versions
- [x] Mount point by path (two rows of tabs): General, Hardware, Sharing, Security, Previous Versions and Customize
- [x] Measured title text ink height: 12 px on every window at 1x, and 12.5 px at 2x (25 device px). Before this change it was 13 px.
- [x] Folder Options ("..." > Options) is pixel-identical to its look before this change

Before this change, the same script finds the folder and mount-point General pages scrolling under the wheel.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
