# Title

fix(home): Act on Recent and Favorites files, and drag Quick access tiles into a new order

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 10 tested Home (with Settings > General > Start up > "Open Quick access on Home" on) against Windows 11 with real pointer and keyboard input (XTEST/xdotool) on a private Xvfb display, and Weston nested on it for the Wayland checks, as an unprivileged user with a throwaway home and real files (mp3, mp4, pdf, jpg, docx, odt, zip, 7z, odd and Unicode names, broken and 2 GB files). No crash, hang, CRITICAL or "Bail out" was seen. These are the fixes:

- **Recent and Favorites rows act on their files.** A row's menu is now Windows 11's menu of a file on Home: Cut, Copy, Rename and Delete in the icon strip, then Open, Open file location, Add to or Remove from Favorites, **Remove from Recent** (Recent rows only), Copy as path, Properties and Show more options, which opens the classic menu with the same commands. Before, a row had only Open, Open file location, Add to Favorites, Copy as path and Properties, with no Show more options.
- **Keys on a row.** Ctrl+C and Ctrl+X put the file on the clipboard, F2 opens the Rename dialog, Delete moves the file to the Recycle Bin and Shift+Delete deletes it after its question. Before, F2 and Delete did nothing on Home and Ctrl+C worked on tiles only.
- **The command bar follows a selected row.** Cut, Copy, Rename and Delete stay grayed out on Home until a Recent or Favorites row is selected, then act on its file, as in Windows. A click on blank space (either button) clears the selection.
- **Remove from Recent** takes the file off the desktop's recently used list (`recently-used.xbel`, shared by every app) and its row off Recent; the file stays.
- **A renamed file stays in Recent and Favorites.** Any rename in the app now moves the file's entry in the recently used list and in Favorites (keeping its place there), also for files inside a renamed folder. Before, a renamed file dropped out of Recent and its favorite pointed at nothing.
- **Gone files leave Recent.** A Recent file that was deleted elsewhere still said "Item Not Found" when opened, as Windows does, but its row stayed until Home was drawn again. It now leaves Recent once the dialog is closed. A file deleted from Home leaves at once.
- **Quick access tiles can be dragged into a new order.** A tile dropped on the left half of another goes before it, on the right half after it, and on blank space at the end. The order is the navigation pane's, so both show it, and it is saved.
- **No stuck pressed look on Wayland.** A tile or row that was right-clicked stayed drawn as pressed after its menu closed under Wayland, because the menu's grab takes the button release. The place menus now clear it when they close.

Home's blank-space menu came from pass-6-context-menu, merged here; this branch only adds the selection clearing to it.

## Merges

Merged network-like-windows, pass-3-rename, pass-4-nav-pane, pass-6-context-menu and pass-7-views (which together hold every lower pass). pass-4-nav-pane's 5cb3a27c and network-like-windows each list Network's computers in the navigation pane, in two incompatible ways (two actions, two row models). This branch keeps network-like-windows' version (SIDE-035) and reverts 5cb3a27c in a separate, labeled commit before merging pass-6 and pass-7, which carried it. That choice is the owner's to confirm; see the status file's QUESTION.

## Tests

Each fix has a test that failed before it and passes after it; clicks, keys and drags are real (the in-tree XTEST `Pointer`):

- `window::tests::home_recent` (new): `a_recent_rows_menu_has_the_file_commands_and_remove_from_recent`, `remove_from_recent_drops_the_row_and_keeps_the_file`, `delete_on_a_recent_row_recycles_the_file_and_drops_the_row`, `f2_on_a_recent_row_renames_the_file_and_recent_follows_it`, `ctrl_c_on_a_recent_row_copies_the_file`, `a_recent_file_found_gone_leaves_recent`, `the_command_bar_acts_on_the_selected_row`, `a_tile_dragged_onto_another_moves_there`, `a_right_click_on_blank_space_clears_the_selection`.
- Guards that passed before any change: `clearing_history_empties_recent_and_keeps_favorites` and `a_right_clicked_row_is_not_left_drawn_as_pressed`. Xvfb cannot show the Wayland pressed look, so that fix was proven by hand with xdotool in Weston, before and after.
- `ox_core::settings::tests::preferences::a_renamed_favorite_keeps_its_place_under_its_new_name`.
- `theme::stylesheets::tests::focus_rings_sit_on_controls_and_items_not_around_the_file_list` now expects pass-3-rename's `:not(.renaming)` focus rules; it failed on pass-3's own tip.

Full run, every test executable isolated as an unprivileged user: 5,091 passed, 30 failed before the stylesheet test update; 29 of those (27 root-only admin tests and 2 Python-fixture tests) fail the same on unchanged main. Clippy is clean apart from three Rust 1.97 lints that fail in untouched files on main too.

Also checked by hand: files opened from other apps (written to `recently-used.xbel`) appear live; opening a file from Home moves it to the top; with 500 recently used files Home lists the newest 50, GVfs reads the whole list in about 50 ms and 30 quick changes cost the app about 0.15 s of CPU; search from Home; light and dark at 1x.

Parity: HOME-036 (rows act on their files) and HOME-037 (tile reordering) added as done; HOME-014 and HOME-015 notes updated. The message template is regenerated; no `.po` file changed.

## Not done

- Rename on Home opens the Rename dialog; Windows renames in place.
- No insertion mark is drawn while dragging a tile.
- A file deleted by another app leaves Recent only once Home is drawn again or the file is opened.
- The classic menu opened low in a short window runs off the screen's bottom instead of opening upward (shared menu code, tiles too).
- Share is not in a row's strip.
