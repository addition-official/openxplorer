# fix(properties): Draw Properties text at Windows' 9-point dialog size

Follows up #271, which gave every item Properties one fixed size (402 x 598 on main).

## Problem

Every Properties window drew its text at 13 px instead of the 12 px that `theme/fonts.rs` sets for `.ox-properties`. The reason: the `window.ox` rule (13 px) outranks a bare class, so the 12 px rule never applied.

Windows draws dialog and property sheet text in Segoe UI 9 pt. Source: Microsoft, "Fonts", Windows UX guidelines, https://learn.microsoft.com/en-us/windows/win32/uxguide/vis-fonts. It lists normal text as "9 pt. ... Segoe UI" and says "For Segoe UI, use a 9 point font size or larger." At 96 dpi, 9 pt is 12 px.

## Changes

- `theme/fonts.rs`: adds the rule `window.ox.ox-properties:not(.ox-folder-options) { font-size: 12px }`. It scales with the text size like every other font rule.
  - These windows now use 12 px: file, several items, folder, drive and shortcut Properties, and their dialogs (Change Icon, Security's Edit and Advanced, Advanced Sharing).
  - Folder Options is left out, so it keeps its own size. The old bare `.ox-properties` rule stays where it was, so #269's Folder Options rule next to it merges cleanly.
- Group boxes (Sharing, Customize, Advanced Sharing) draw their title on the box's top line, as Windows does.
  - `properties::group()` turns off the frame's clipping.
  - `properties.css` lifts the title by 0.75 em and gives the box 0.34 em above it. In pixels, that is 9 and 4 at 100 % text, and 18 and 8 at 200 %. The title stays centred on the line at every text size.
  - This also shortens each group, which gives the two-row tab windows room at 12 px in Noto Sans, the font the app renders on Zorin.
- The Previous Versions list's minimum height goes from 230 to 216 px. The list still grows to fill the tab.

## Tests

- New tests:
  - `theme::fonts::tests::properties_windows_draw_windows_dialog_text`.
  - `window::tests::properties_width::every_kind_of_properties_draws_windows_dialog_text`. On every kind of Properties and every tab, it checks three things:
    - Every label is 12 px.
    - Only a long value's one-line field or a path ends in "…".
    - No label is cut.

    It also checks that one line of text is the same height on every window (17 px with Noto Sans), and that Folder Options keeps its own size.
  - `window::tests::properties_width::group_titles_sit_on_the_box_line_at_every_text_size`. It checks every text size from 80 % to 200 %, and it fails with the earlier fixed 8-pixel lift.
- Named tests were run on main e4520a3 at 1x and 2x, under Xvfb with Noto Sans: properties, tab_rows, item_dialogs, customize, details_tab, security, sharing, dialog_keys, late_replies, folder_options, view_styles and theme::.
  - 191 of 192 pass, including `every_kind_of_properties_has_one_size`.
  - The one failure is `mount_assistant::the_assistant_prepares_a_command_and_hands_over_the_linux_folder`, which also fails without this change.
- rustfmt is clean, and clippy shows no warnings in the changed files.

## Checklist

All checks used real input with xdotool under Xvfb and Openbox, with the app run as an unprivileged user on fictional files. Each of the four variants (1x light, 1x dark, 2x light, 2x dark) ran 105 checks, and all passed.

- [x] File: Alt+Enter opens it. General, Security, Details and Previous Versions (reached with Ctrl+Tab) each stay 402 x 598, and the mouse wheel doesn't scroll them. Esc closes it.
- [x] Shortcut: the same four tabs.
- [x] Several items (click, then Ctrl+click): General and Security.
- [x] Folder: General, Sharing, Security, Previous Versions and Customize. Change Icon... opens its dialog, and Esc closes it.
- [x] Drive (Local Disk from the sidebar): General, Hardware, Sharing, Security and Previous Versions.
- [x] Mount point by path: opens as a folder, with five tabs.
- [x] The mount point's Target > Properties opens the other drive, with six tabs in two rows. All six fit.
- [x] Measured title text ink height: 12 px on every window at 1x, and 12.5 logical px (25 device px) at 2x. Before this change it was 13 px.
- [x] Folder Options ("..." > Options) opens and closes, and stays at 508 x 664 without #269.
- [x] Ctrl+minus, Ctrl+0 and Ctrl+plus in an open folder Properties (Customize and Sharing), each variant:
  - At 80 % and 100 %, the window stays 402 x 598.
  - At 200 %, the window grows to 546 x 618 (main bab3fb5 without this change: 589 x 618), as main already lets it. Its group titles sit on the line.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
