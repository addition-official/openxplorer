# Title

fix(views): Head groups in the icon sizes and List, fill Small icons by rows and fix Home and reflow in grouped views

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/07-network-like-windows)

This PR holds the early commits of pass 7 (views). Pass 7's later commits (measured Details, icon, Tiles and Content spacing, the column title line setting, "Name column starts at Windows' width (256 px)", two-line names in Tiles and test pruning) are PR 26 of this series, because they were written on top of the later passes.

## What changes for the user
- **Group headings in the icon sizes.** With Group by on, Extra large, Large and Medium icons (and the Ctrl+wheel sizes between them) head each group, as Windows 11 does. They use the grouped grid that Tiles and Content already use: each group starts a new row under a heading with the same chevron, hover, click-to-select-the-group, double-click, keyboard (arrows rest on headings, Left collapses, Right expands) and right-click menu. Grouped tiles draw their padding, hover, selection and focus on their own box (`icon-groups`), so a heading row is one line of title tall.
- **Small icons fill rows.** Small icons now fill rows left to right and scroll down, as Windows 11 does (checked in the VM, build 26200), with the 16 px icon left of each name and the columns 310 px apart at 100%. The grid takes only the width its columns need, so the columns do not spread across the window. This replaces the column-first, sideways-scrolling layout from PR 06. Grouped, Small icons head each group with a chevron, the title and a line under the heading's row.
- **Grouped List.** Each group stands in a column of its own with its title at the top (no chevron), its items running down under it and on into the next column when they overflow. Up and Down step through the items across groups, Left and Right go to the same row of the next column, and List's groups never collapse.
- **A new view opens at the top.** Switching views takes the keyboard on the first selected item (else the first item) without scrolling, starts at the top, and scrolls only as far as a selected item out of sight needs.
- **Home in grouped Details.** Home and End go to the first and last item, Shift extends the selection and Ctrl moves only the keyboard. Home also scrolls to the top so the first heading shows.
- **Home in grouped Tiles, Content and the icon sizes.** A key that lands on a group's first row brings that group's heading into view too, so Home goes back to the top of the list.
- **Grouped grid reflow.** When the column count changes (Ctrl+wheel, a narrower window), heading-line and blank slots now match only when they are the same group's same one, so GTK no longer prints "Duplicate item detected in list. Picking one randomly." or draws the wrong slot.
- The user guide (`docs/interface.md` and the website's copies) says that every icon size heads its groups and describes grouped List.

## Why
- The icon sizes listed items in group order with no headings, so a folder grouped by Type showed a 7-Zip archive before the folders with nothing to say why.
- Grouped List showed the group order with no headings, where Windows 11 stands the groups side by side.
- Switching views with nothing selected could open the new view part way down: GTK scrolled it to whichever item last had the keyboard there, selected or not. Windows 11 keeps the scroll at the top.
- In grouped Details, GTK's Home moved the keyboard to the first group's heading, which then selected the whole first group, and the keyboard outline stayed on the heading. In grouped grids, Home scrolled the first item to the top edge and left its heading hidden.
- The grouped grid's change report to GTK treated any line slot as unchanged, even when its object had moved. This could already happen in Tiles on a window resize; zooming grouped icons made it easy to hit.

## CHANGED OWNER DECISIONS
- VIEW-022 / VIEW-071: old: "Icons and List keep the grouped order without headings". New: every icon size heads its groups, and List stands its groups side by side as columns, as the Windows 11 VM shows.
- VIEW-008 / VIEW-005: old: Small icons filled columns top to bottom and scrolled sideways (the layout of 2026-11-18 from `review4-diffs`, in PR 06). New: Small icons fill rows left to right and scroll down (VM-checked).

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: for this PR's fixes, each failing before its fix, `window::tests::selection::switching_to_small_icons_keeps_the_list_at_the_top`, `window::tests::view_styles::small_icons_put_the_icon_left_of_each_name_across_rows`, `window::tests::selection::arrows_move_across_rows_in_small_icons`, `folder_view::group_slots::tests::list_heads_each_group_at_the_top_of_its_column`, `window::tests::tiles_and_content::list_heads_each_group_at_the_top_of_its_column`, `window::tests::tiles_and_content::icon_sizes_keep_the_folders_groups_under_headings`, `window::tests::selection::arrows_cross_the_group_headings_in_tiles_and_content`, `a_group_collapses_under_its_heading_and_its_items_are_passed_by` and `a_click_on_a_heading_selects_its_group_and_the_chevron_folds_it` (extended with the icon sizes), `theme::fonts::tests::the_default_text_size_draws_13_pixel_text_and_30_pixel_rows`, `window::tests::selection::home_shows_the_first_groups_heading_in_a_grouped_grid`, `window::tests::selection::home_and_end_in_grouped_details_go_to_the_first_and_last_item` and `folder_view::group_slots::tests::a_change_of_columns_covers_every_rule_and_blank_that_moved`, plus `window::tests::selection::changing_the_view_of_a_grouped_folder_keeps_the_focused_item_in_view` as added coverage; `python3 native/parity/check.py`, the parity and tools unit tests, `tools/i18n.py check` and the icon guard; `cargo +1.92 fmt --all --check` and `cargo +1.92 clippy --workspace --all-targets --locked -- -D warnings`; every test executable under `check.py`'s isolation, whose failures were root-only and also fail on unchanged main; real input with xdotool on a private Xvfb as an unprivileged user with a fictional tree including a folder of 10,000 files (grouped Large icons through every Ctrl+wheel size and four window resizes with no "Duplicate item detected in list" warning). Those runs were made on the whole pass 7 branch, which also held the commits that are PR 26 here. Two of the tests named above (`small_icons_put_the_icon_left_of_each_name_across_rows` and the `group_slots` List test) are not in this branch.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
