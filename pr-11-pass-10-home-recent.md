# Title

fix(home): Act on Recent and Favorites files, and drag Quick access tiles into a new order

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/10-pass-9-scaling)

## What changes for the user
- **Recent rows match Windows 11** (checked in a Windows 11 VM, Home > Recent). A Recent row's menu has Copy, and Share where the desktop can share, alone in the icon strip, then Open with (Enter), Share with, Open file location, **Remove from list**, Add to Favorites, Compress to, Copy as path, Properties, the text editor's Edit in, and Show more options. Share, Share with, Open with and Edit in act on the selected row.
- **Favorites rows** have Cut, Copy, Rename and Delete in the strip, by key and in the command bar, plus Open, Open file location, Remove from Favorites, Copy as path and Properties.
- **The command bar follows a selected row.** With a Recent row selected, Copy (and Share) are enabled and Cut, Paste, Rename and Delete stay grayed, as in Windows; Ctrl+C copies the file, and Delete, Shift+Delete, F2 and Ctrl+X do nothing. A click on blank space (either button) clears the selection and grays them again.
- **Remove from list** takes the file off the desktop's recently used list (`recently-used.xbel`, shared by every app) and its row off Recent; the file stays.
- **The Details pane counts Home.** With nothing selected it shows "Home (N items)" with Home's own count, as the status bar counts it.
- **A renamed file stays in Recent and Favorites.** Any rename in the app moves the file's entry in the recently used list and in Favorites (keeping its place there), also for files inside a renamed folder.
- **Gone files leave Recent.** A Recent file deleted elsewhere still says "Item Not Found" when opened, as Windows does, and its row now leaves Recent once the dialog is closed. A file deleted from a Favorites row leaves Recent at once.
- **Quick access tiles can be dragged into a new order.** A tile dropped on the left half of another goes before it, on the right half after it, and on blank space at the end. The order is the navigation pane's, so both show it, and it is saved.
- **No stuck pressed look on Wayland.** A tile or row that was right-clicked no longer stays drawn as pressed after its menu closes.
- Home's blank-space menu (from PR 06) now also clears the selection.
- Parity: HOME-036 (rows act on their files) and HOME-037 (tile reordering) are added as done; the HOME-014 and HOME-015 notes are updated. The message template is regenerated; no `.po` file changed.

Not done: no insertion mark is drawn while dragging a tile; a file deleted by another app leaves Recent only once Home is drawn again or the file is opened; selecting a Recent row leaves the Details pane on Home.

## Why
- A Recent row's menu had only Open, Open file location, Add to Favorites, Copy as path and Properties, and the whole command bar stayed grayed on Home.
- The Details pane said "Home (0 items)" on Home.
- A renamed file dropped out of Recent and its favorite pointed at nothing.
- A Recent file found gone kept its row until Home was drawn again.
- Quick access tiles could not be reordered by dragging.
- Under Wayland the menu's grab takes the button release, so a right-clicked tile or row stayed pressed.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Remove from list"
- "Could not remove from Recent: {error}"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix with a test that failed before it, with real clicks, keys and drags through the in-tree XTEST `Pointer`: `window::tests::home_recent` (`a_recent_rows_menu_is_windows_11s`, `remove_from_recent_drops_the_row_and_keeps_the_file`, `delete_f2_and_ctrl_x_do_nothing_on_a_recent_row`, `a_renamed_file_stays_in_recent_and_favorites`, `ctrl_c_on_a_recent_row_copies_the_file`, `a_recent_file_found_gone_leaves_recent`, `the_command_bar_offers_copy_alone_on_a_recent_row`, `the_details_pane_names_home_with_its_items`, `a_tile_dragged_onto_another_moves_there`, `a_right_click_on_blank_space_clears_the_selection`), the guards `clearing_history_empties_recent_and_keeps_favorites` and `a_right_clicked_row_is_not_left_drawn_as_pressed`, `ox_core::settings::tests::preferences::a_renamed_favorite_keeps_its_place_under_its_new_name`, and `theme::stylesheets::tests::focus_rings_sit_on_controls_and_items_not_around_the_file_list` (some of these were later removed as redundant and are not in this branch); the Wayland pressed look checked by hand with xdotool in Weston, before and after; `native/tools/check.py` with Rust 1.92 through the inventories, i18n, icon guard, rustfmt and Clippy, and the remaining executables and doctests in check.py's isolation, failing only where unchanged main fails; by hand: files opened from other apps appear live, 500 recently used files (Home lists the newest 50), search from Home, light and dark at 1x.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
