# Title

fix(views): Space icon views, Tiles and Content as Windows 11 measures, and add two Details column settings

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/25-user-icon-files)

This is the second half of overnight pass 7 (views). Its first commits are PR 08 (`pr/08-pass-7-views`); these later ones were written on top of the later passes, so they come here.

## What changes for the user
- **Spacing measured against Windows 11.** Reference captures of Windows 11 (1920x1080, 1x, dark, Compact view) were compared with the app by script, and gaps over 1 px fixed:

  | Measure | Windows 11 | After |
  |---|---|---|
  | Compact List row pitch | 21 | 21 |
  | Extra large / Large / Medium column pitch | 291 / 109 / 75 | 291 / 109 / 75 |
  | Extra large / Large / Medium selection width | 267 / 101 / 70 | 267 / 101 / 70 |
  | Tiles pitch / box width | 254 / 246 | 254 / 246 |

  The icon sizes and Tiles keep Windows' distance between columns and leave the spare width free at the right instead of spreading the columns. Details keeps its earlier paddings (12 px cell padding, 26 px start and end, 11 px between icon and name), which the owner prefers to Windows' tighter ones; a commit in this PR that applied Windows' measured Details paddings is reverted by a later one.
- **Larger text (Ctrl+plus).** Rows in Details, List and Small icons, Content's rows and the Tiles columns and rows now grow by the same factor as the text, as Windows' scaling grows its layout.
- **Content's columns.** A picture shows its "Type:" and "Dimensions:" in a middle column, as Windows 11 does. Each column starts where Windows starts it, measured in a 775 and a 1745 pixel row: Type 368 to 374 pixels from the row's left, Date modified 357 pixels plus 18% of the row. A name is cut about 16 pixels before the next column.
- **Column widths at larger text.** A saved column width is now a width at 100% text and grows with the text size, as Windows' scaling grows it, so a saved Date modified width no longer cuts every date at 150% and up.
- **"Show a line under column titles"**, a new setting in Appearance > Layout, on by default (the owner's look); off draws no line under the Details titles, as Windows 11. Stored as `columnTitleLine`, only when off.
- **Column title dividers.** Windows 11 draws a faint divider after each column title all the time, not only on hover (its reference capture: #636363 on #191919). The app now does too.
- **"Name column starts at Windows' width (256 px)"**, a new setting in Appearance > Layout, off by default (Name keeps filling the room, the owner's choice). On, Name starts 256 px wide (times the text size), as in Windows 11, so the rows, their selection and their hover end after the last column. Stored as `nameColumnWindowsWidth`, only when on.
- **Long names in Tiles** run onto a second line and are cut with "..." after it, as Windows 11's tiles do; the size line then gives way so the tile keeps three lines.
- **Home's Recent and Favorites** cut a long name, date or location with "..." at its column's edge, 12 px before the next column, and show the whole name on hover, instead of widening the column past its header.
- **Long names in the icon sizes.** In Extra large, Large and Medium icons an unselected long name shows up to four lines, cut with "..." after the fourth, as Windows 11 does (counted on its captures); the lines past the second run into the gap under the row, so the rows keep Windows' spacing. The name's lines are 15 px apart, its first right under the icon, and it may run to 5 px inside the item's box, so a one-line item's box is 116 px tall in Large icons, as Windows' is. Names no longer get hyphens inserted at a break ("2026.xl-sx"), and the focus ring is drawn once, on the item's box, not also on its taller cell.
- **Redundant tests removed.** The "Drop redundant tests" commit removes 15 tests that earlier PRs in this series added and that other tests already cover, and adds none: PR 23's document preview tests (`each_kind_of_document_converts_in_the_sandbox`, `a_changed_document_gets_a_new_cache_name`, `the_preview_stops_a_thumbnails_conversion`, `office_and_opendocument_files_are_documents`, `a_documents_thumbnail_is_its_first_page`, `a_document_keeps_its_type_icon_in_small_rows`, `the_checksum_is_zips`); PR 06's menu tests (`the_recycle_bin_menus_are_windows_11s`, `the_menus_inside_a_zip_are_windows_11s`, `the_blank_space_of_this_pc_home_and_network_has_windows_11s_menu`, `a_network_computer_in_the_navigation_pane_pins_to_quick_access`); PR 10's `the_progress_and_conflict_windows_follow_the_text_size`, `a_thumbnail_smaller_than_the_size_asked_for_is_missing` and `device_pixels_beyond_256_ask_for_the_larger_flavors`; and PR 12's `a_short_name_alone_stays_on_one_line`; with the helpers only they used. A follow-up commit drops the blank lines they left. Separately, the spacing commit replaces the older `large_icon_cells_are_the_web_grid_cells` with `icon_cells_are_spaced_as_windows_11_spaces_them`.

## Why
- The icon sizes and Tiles were spaced from the old web grid (for example Extra large / Large / Medium columns 335 / 186 / 139 px apart), not from Windows 11's measures.
- At larger text the rows grew by less than the text: at 200% Content rows were 82 px apart where 96 keep the 100% proportions, and Tiles 452 instead of 508.
- At 150% and up a saved Date modified width cut every date to "11/22/20...".
- Windows 11 draws no line under the column titles and starts Name at 256 px; the owner's look keeps the line and lets Name fill the room, so both are settings.
- The dividers between column titles showed only on hover, unlike Windows 11.
- A long Tiles name was cut on one line where Windows 11 uses two, and a long name in the icon sizes was cut after two lines where Windows 11 shows four.
- A long name in Home's Recent or Favorites widened its column and ran up to the next one.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Show a line under column titles"
- "In the details view, a thin line separates the column titles from the list. Windows 11 draws none."
- "Name column starts at Windows' width (256 px)"
- "In the details view, Name starts 256 pixels wide, so the rows and their selection end after the last column, as in Windows 11. Off: Name fills the room the other columns leave."
- "Type:"
- "Dimensions:"
- Search keywords for the new rows: "column header title line border separator underline divider details view windows", "name column width wide fill details view selection row windows 256"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: `python3 native/parity/check.py`, the parity and tools unit tests, `tools/i18n.py check` and the icon guard; `cargo +1.92 fmt --all --check` and `cargo +1.92 clippy --workspace --all-targets --locked -- -D warnings`; every test executable under check.py's isolation, continuing past failures (ox-app lib 2581 passed and 12 failed, the 12 also failing on unchanged main: read-only folders, no Trash, administrator prompts; ox-core lib 1292 passed and 1 failed, the `admin_copy` fstab test, as root); doctests; after merging the newer tips of passes 3 to 6, 520 selection, view, grouping, sidebar, menu, rename and address bar tests passed and 2 root-only tests failed, as on unchanged main. Not tested: Wayland, 2x scale, Orca.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
