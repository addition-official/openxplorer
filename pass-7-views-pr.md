# Title

fix(views): Head groups in every layout, match Windows 11's spacing, and fix Home and reflow in grouped views

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Overnight pass 7 (views) checked every layout against Windows 11 with real xdotool input in a private Xvfb session, as an unprivileged user with a throwaway home and a fictional test tree (mp3, mp4, pdf, jpg, png, odt, zip, 7z, a 3 GB file, a 150 MB file, odd and long names, and a folder of 10,000 files). It fixes three bugs in grouped views and, after a Windows 11 VM check (build 26200), changes Small icons and grouped List to match Windows. It also merges fork branch `review4-diffs` and the lower-numbered passes 1 to 6, because they change the same files.

- **Group headings in the icon sizes.** With Sort > Group by on, Extra large, Large and Medium icons (and the Ctrl+wheel sizes between them) listed the items in group order with no headings. A folder grouped by Type therefore showed a 7-Zip archive before the folders, with nothing to say why. Windows 11 heads each group in these views. They now use the grouped grid that Tiles and Content already use. Each group starts a new row under a heading with the same chevron, hover, click-to-select-the-group, double-click, keyboard (arrows rest on headings, Left collapses, Right expands) and right-click menu. Grouped tiles draw their padding, hover, selection and focus on their own box (`icon-groups`), so a heading row is one line of title tall and not a tile. 
- **Small icons (VM-checked).** Small icons now fill rows left to right and scroll down, as Windows 11 does, with the 16 px icon left of each name and the columns 310 px apart at 100%. The grid takes only the width its columns need, so the spacing stays at 310 px and does not spread across the window. This replaces the column-first, sideways-scrolling layout from `review4-diffs`. Grouped, Small icons head each group with a chevron, the title and a line under the heading's row.
- **Grouped List (VM-checked).** Each group now stands in a column of its own with its title at the top (no chevron), its items running down under it and on into the next column when they overflow. Before, List showed the group order with no headings. Up and Down step through the items across groups, Left and Right keep the row on screen, and List's groups never collapse, as their headings have no chevron.
- **A new view opens at the top.** Switching views with nothing selected could open the new view part way down. GTK scrolled it to whichever item last had the keyboard there, selected or not. Windows 11 keeps the scroll at the top and scrolls only to show a selected item. The new view now starts at the top, takes the keyboard on its first selected item (else its first item) without scrolling, and scrolls only as far as a selected item out of sight needs.
- **Home in grouped views.**
  - **Details:** GTK's Home moved the keyboard to the list's first row, which is the first group's heading. The heading then selected the whole first group, and the keyboard outline stayed on the heading. Home and End now go to the first and last item. Shift extends the selection and Ctrl moves only the keyboard. Home also scrolls to the top.
  - **Tiles, Content and the icon sizes:** Home selected the first item but scrolled it to the top edge, so the first heading stayed hidden above it. A key that lands on a group's first row now brings that group's heading into view too.
- **Grouped grid reflow.** The grouped grid keeps one object for each heading-line slot and each blank. When the column count changed (Ctrl+wheel, a narrower window), the change it reported to GTK treated any line slot as unchanged, even when its object had moved. GTK printed "Duplicate item detected in list. Picking one randomly." and could draw the wrong slot. This could already happen in Tiles when the window was resized; zooming grouped icons made it easy to hit. Line and blank slots now match only when they are the same group's same one.
- **Spacing measured against Windows 11.** Reference captures of Windows 11 (1920x1080, 1x, dark, Compact view) were compared with ours by script, and gaps over 1 px fixed:

  | Measure | Windows 11 | Before | After |
  |---|---|---|---|
  | Compact List row pitch | 21 | 26 | 21 |
  | Extra large / Large / Medium column pitch | 291 / 109 / 75 | 335 / 186 / 139 | 291 / 109 / 75 |
  | Extra large / Large / Medium selection width | 267 / 101 / 70 | not measured | 267 / 101 / 70 |
  | Tiles pitch / box | 254 / 246x52 | not measured | 254 / 246x52 |
  | Content icon x / name x | 36 / 74 | 18 / 59 | 36 / 74 |

  Details keeps its earlier paddings (icon 15 px and name 42 px into the row), which the owner prefers to Windows' tighter ones.

  The icon sizes now keep Windows' distance between columns and leave the spare width at the right instead of spreading the columns. Tiles show folders and zip files by name alone. Content shows nothing under a document's name, the date alone for folders and zip files, and keeps the name level with Date modified. Still open: Small icons rows are 28-29 px in the real app (21 in Windows), and Content's date column sits at the row's right where Windows puts it at 38.6% of the row (one width measured, so the rule is unknown).
- **Larger text (Ctrl+plus).** Rows in Details, List and Small icons, Content's rows and the Tiles columns and rows now grow by the same factor as the text, as Windows' scaling grows its layout. Before, they grew by less, so at 200% Content rows were 82 px apart where 96 keep the 100% proportions, and Tiles 452 instead of 508.
- **Content's columns.** A picture shows its type and dimensions in a middle column, as Windows 11 does. Each column starts where Windows starts it, measured in a 775 and a 1745 pixel row: Type 368 to 374 pixels from the row's left, Date modified 357 pixels plus 18% of the row. A name is cut 16 pixels before the next column.
- **Column widths at larger text.** A saved column width is now a width at 100% text and grows with Ctrl+plus, as Windows' scaling grows it. Before, at 150% and up a saved Date modified width cut every date to "11/22/20...".
- **"Show a line under column titles".** A new setting in Appearance > Layout, on by default (the owner's look); off draws no line, as Windows 11. For the "Behave like Windows Explorer" registry (`claude/windows-mode-switch`, not merged here), its line is: `like_windows!("columnTitleLine", column_title_line, false, "Show a line under column titles"),`
- **User guide.** It said item check boxes are on by default, as in Windows 11. The owner decision (SEL-014) has them off, and with them on a box shows on hover only while Ctrl is held. The guide now says both.

No new dependency. `native/po/*.po` untouched; no new strings, so the `.pot` is unchanged.

## Before and after

| Case | Before (2.0.21) | After | Windows 11 |
|---|---|---|---|
| Small icons | Columns filled top to bottom, sideways scroll (review4-diffs) | Rows filled left to right, 310 px columns, scrolls down | Rows, about 310 px apart (VM) |
| List, Group by Type | Group order, no headings | Groups side by side as columns, titles on top | Same (VM) |
| Large icons, Group by Type | Group order, no headings ("Backup set" 7z first, then a zip, then folders) | "7-zip Archive", "Compressed (zipped) Folder", "File folder"... headings, each group on its own rows | Headings in every icon size |
| Grouped Details, End then Home | Outline on the "7-zip Archive" heading, its whole group selected | "Backup set" selected with the outline, heading in view | First item focused and selected |
| Grouped Tiles, End then Home | "Backup set" at the top edge, its heading hidden | Scrolled to the top, heading shown | Scrolled to the top |
| Grouped Large icons, Ctrl+wheel through every size on 10,000 files | 2x Gtk-WARNING "Duplicate item detected in list" | None (same sequence plus four window resizes) | n/a |

## Tests

New, each failing before its fix and passing now:
- `window::tests::selection::switching_to_small_icons_keeps_the_list_at_the_top`
- `window::tests::view_styles::small_icons_put_the_icon_left_of_each_name_across_rows` and `window::tests::selection::arrows_move_across_rows_in_small_icons` (replacing review4-diffs' column-first tests)
- `folder_view::group_slots::tests::list_heads_each_group_at_the_top_of_its_column` and `window::tests::tiles_and_content::list_heads_each_group_at_the_top_of_its_column`
- `window::tests::tiles_and_content::icon_sizes_keep_the_folders_groups_under_headings` (Extra large, Large, Medium, 64 px and 28 px, with real-world file types: headings as Details shows them, each group on its own row, heading row as tall as Page Up/Down count it, tile row its tile's height; Small icons and List without headings)
- `window::tests::selection::arrows_cross_the_group_headings_in_tiles_and_content`, `a_group_collapses_under_its_heading_and_its_items_are_passed_by` and `a_click_on_a_heading_selects_its_group_and_the_chevron_folds_it`, extended with the icon sizes
- `theme::fonts::tests::the_default_text_size_draws_13_pixel_text_and_30_pixel_rows` (the grouped tile rule)
- `window::tests::selection::home_shows_the_first_groups_heading_in_a_grouped_grid`
- `window::tests::selection::home_and_end_in_grouped_details_go_to_the_first_and_last_item`
- `folder_view::group_slots::tests::a_change_of_columns_covers_every_rule_and_blank_that_moved`

Added as coverage (passed before and after): `window::tests::selection::changing_the_view_of_a_grouped_folder_keeps_the_focused_item_in_view`.

Checks run in this container (tests run as root here; Clippy and rustfmt with the pinned Rust 1.92):
- `python3 native/parity/check.py`, the parity and tools unit tests, `tools/i18n.py check` and the icon guard: passed.
- `cargo +1.92 fmt --all --check` and `cargo +1.92 clippy --workspace --all-targets --locked -- -D warnings`: passed.
- Every test executable under check.py's isolation, continuing past failures: ox-app lib 2581 passed and 12 failed. I reran the 12 failures by name on unchanged main 5336b108, and all 12 fail there too (read-only folders, no Trash, administrator prompts). ox-core lib 1292 passed and 1 failed (the admin_copy fstab test, root). The display_scale, integration_file_chooser, launch, location_fixture_drift, network_python and ops_delete_plan executables have root-dependent failures that review4-diffs also found on unchanged main; this change touches none of their code. Doctests pass.
- After merging the newer tips of passes 3 to 6: rustfmt, Clippy, parity and i18n checks pass; 520 selection, view, grouping, sidebar, menu, rename and address bar tests pass, and the 2 that fail are root-only tests that fail on unchanged main too.
- `python3 native/tools/check.py` as a whole stops at display_scale because the app refuses to run as root, as it does on unchanged main.

## CHANGED OWNER DECISIONS

- VIEW-022 / VIEW-071: "Icons and List keep the grouped order without headings" is gone: every icon size heads its groups, and List stands its groups side by side as columns, as the Windows 11 VM shows.
- VIEW-008 / VIEW-005: Small icons fill rows left to right and scroll down (VM-checked), replacing the column-first layout of 2026-11-18 (`review4-diffs`).

Round five ran only the touched modules' tests (205 passed, 1 root-only failure that also fails on main) and `cargo +1.92 fmt`.

## Not tested

Wayland, 2x scale, Orca. Real Windows 11 behavior that no Microsoft source confirms is listed for a VM check in the status file.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
