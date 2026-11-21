# Title

fix(views): Head groups in the icon sizes and fix Home and reflow in grouped views

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Overnight pass 7 (views) checked every layout against Windows 11 with real xdotool input in a private Xvfb session, as an unprivileged user with a throwaway home and a fictional test tree (mp3, mp4, pdf, jpg, png, odt, zip, 7z, a 3 GB file, a 150 MB file, odd and long names, and a folder of 10,000 files). It fixes three bugs in grouped views. It also merges fork branch `review4-diffs` (Small icons as icon-left rows in columns about 310 px apart), because this pass changes the same files.

- **Group headings in the icon sizes.** With Sort > Group by on, Extra large, Large and Medium icons (and the Ctrl+wheel sizes between them) listed the items in group order with no headings. A folder grouped by Type therefore showed a 7-Zip archive before the folders, with nothing to say why. Windows 11 heads each group in these views. They now use the grouped grid that Tiles and Content already use. Each group starts a new row under a heading with the same chevron, hover, click-to-select-the-group, double-click, keyboard (arrows rest on headings, Left collapses, Right expands) and right-click menu. Grouped tiles draw their padding, hover, selection and focus on their own box (`icon-groups`), so a heading row is one line of title tall and not a tile. Small icons and List fill columns top to bottom, so they still keep the group order without headings. That difference from Windows is documented and listed for the VM check.
- **Home in grouped views.**
  - **Details:** GTK's Home moved the keyboard to the list's first row, which is the first group's heading. The heading then selected the whole first group, and the keyboard outline stayed on the heading. Home and End now go to the first and last item. Shift extends the selection and Ctrl moves only the keyboard. Home also scrolls to the top.
  - **Tiles, Content and the icon sizes:** Home selected the first item but scrolled it to the top edge, so the first heading stayed hidden above it. A key that lands on a group's first row now brings that group's heading into view too.
- **Grouped grid reflow.** The grouped grid keeps one object for each heading-line slot and each blank. When the column count changed (Ctrl+wheel, a narrower window), the change it reported to GTK treated any line slot as unchanged, even when its object had moved. GTK printed "Duplicate item detected in list. Picking one randomly." and could draw the wrong slot. This could already happen in Tiles when the window was resized; zooming grouped icons made it easy to hit. Line and blank slots now match only when they are the same group's same one.
- **User guide.** It said item check boxes are on by default, as in Windows 11. The owner decision (SEL-014) has them off, and with them on a box shows on hover only while Ctrl is held. The guide now says both.

No new dependency. `native/po/*.po` untouched; no new strings, so the `.pot` is unchanged.

## Before and after

| Case | Before (2.0.21) | After | Windows 11 |
|---|---|---|---|
| Large icons, Group by Type | Group order, no headings ("Backup set" 7z first, then a zip, then folders) | "7-zip Archive", "Compressed (zipped) Folder", "File folder"... headings, each group on its own rows | Headings in every icon size |
| Grouped Details, End then Home | Outline on the "7-zip Archive" heading, its whole group selected | "Backup set" selected with the outline, heading in view | First item focused and selected |
| Grouped Tiles, End then Home | "Backup set" at the top edge, its heading hidden | Scrolled to the top, heading shown | Scrolled to the top |
| Grouped Large icons, Ctrl+wheel through every size on 10,000 files | 2x Gtk-WARNING "Duplicate item detected in list" | None (same sequence plus four window resizes) | n/a |

## Tests

New, each failing before its fix and passing now:
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
- `python3 native/tools/check.py` as a whole stops at display_scale because the app refuses to run as root, as it does on unchanged main.

## CHANGED OWNER DECISIONS

- VIEW-022 / VIEW-071: "Icons and List keep the grouped order without headings" becomes "Small icons and List keep the grouped order without headings". Extra large, Large and Medium icons now head their groups, as Windows 11 does.

## Not tested

Wayland, 2x scale, Orca. Real Windows 11 behavior that no Microsoft source confirms is listed for a VM check in the status file.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
