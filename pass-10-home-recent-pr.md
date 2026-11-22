# Title

fix(home): Act on Recent and Favorites files, and drag Quick access tiles into a new order

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 10 tested Home (with Settings > General > Start up > "Open Quick access on Home" on) against Windows 11 with real pointer and keyboard input (XTEST/xdotool) on a private Xvfb display, and Weston nested on it for the Wayland checks, as an unprivileged user with a throwaway home and real files (mp3, mp4, pdf, jpg, docx, odt, zip, 7z, odd and Unicode names, broken and 2 GB files). No crash, hang, CRITICAL or "Bail out" was seen. These are the fixes:

- **Recent rows match Windows 11 (checked in a Windows 11 VM, Home > Recent).** A Recent row's menu has Copy, and Share where the desktop can share, alone in the icon strip, then Open with (Enter), Share with, Open file location, **Remove from list**, Add to Favorites, Compress to, Copy as path, Properties, the text editor's Edit in, and Show more options. Before, it had only Open, Open file location, Add to Favorites, Copy as path and Properties. Share, Share with, Open with and Edit in act on the selected row.
- **Favorites rows** have Cut, Copy, Rename and Delete in the strip, by key and in the command bar, plus Open, Open file location, Remove from Favorites, Copy as path and Properties. The VM check didn't cover them.
- **The command bar follows a selected row.** With a Recent row selected, Copy (and Share) are enabled and Cut, Paste, Rename and Delete stay grayed, as in Windows; Ctrl+C copies the file, and Delete, Shift+Delete, F2 and Ctrl+X do nothing. A click on blank space (either button) clears the selection and grays them again. Before, the whole command bar stayed grayed on Home.
- **Remove from list** takes the file off the desktop's recently used list (`recently-used.xbel`, shared by every app) and its row off Recent; the file stays.
- **The Details pane names Home.** With nothing selected it shows Home's house and "Home (N items)" with Home's own count, as Windows does. It showed a yellow folder and "Home (0 items)".
- **A renamed file stays in Recent and Favorites.** Any rename in the app now moves the file's entry in the recently used list and in Favorites (keeping its place there), also for files inside a renamed folder. Before, a renamed file dropped out of Recent and its favorite pointed at nothing.
- **Gone files leave Recent.** A Recent file that was deleted elsewhere still said "Item Not Found" when opened, as Windows does, but its row stayed until Home was drawn again. It now leaves Recent once the dialog is closed. A file deleted from a Favorites row leaves Recent at once.
- **Quick access tiles can be dragged into a new order.** A tile dropped on the left half of another goes before it, on the right half after it, and on blank space at the end. The order is the navigation pane's, so both show it, and it is saved.
- **No stuck pressed look on Wayland.** A tile or row that was right-clicked stayed drawn as pressed after its menu closed under Wayland, because the menu's grab takes the button release. The place menus now clear it when they close.

Home's blank-space menu came from pass-6-context-menu, merged here; this branch only adds the selection clearing to it.

## Merges

Merged network-like-windows, pass-3-rename, pass-4-nav-pane, pass-6-context-menu and pass-7-views (which together hold every lower pass). pass-4-nav-pane's 5cb3a27c and network-like-windows each list Network's computers in the navigation pane, in two incompatible ways (two actions, two row models). This branch keeps network-like-windows' version (SIDE-035) and reverts 5cb3a27c in a separate, labeled commit before merging pass-6 and pass-7, which carried it. That choice is the owner's to confirm; see the status file's QUESTION.

## Tests

Each fix has a test that failed before it and passes after it (the Recent row tests were rewritten after the VM check); clicks, keys and drags are real (the in-tree XTEST `Pointer`):

- `window::tests::home_recent` (new): `a_recent_rows_menu_is_windows_11s`, `remove_from_recent_drops_the_row_and_keeps_the_file`, `delete_f2_and_ctrl_x_do_nothing_on_a_recent_row`, `a_renamed_file_stays_in_recent_and_favorites`, `ctrl_c_on_a_recent_row_copies_the_file`, `a_recent_file_found_gone_leaves_recent`, `the_command_bar_offers_copy_alone_on_a_recent_row`, `the_details_pane_names_home_with_its_items`, `a_tile_dragged_onto_another_moves_there`, `a_right_click_on_blank_space_clears_the_selection`.
- Guards that passed before any change: `clearing_history_empties_recent_and_keeps_favorites` and `a_right_clicked_row_is_not_left_drawn_as_pressed`. Xvfb cannot show the Wayland pressed look, so that fix was proven by hand with xdotool in Weston, before and after.
- `ox_core::settings::tests::preferences::a_renamed_favorite_keeps_its_place_under_its_new_name`.
- `theme::stylesheets::tests::focus_rings_sit_on_controls_and_items_not_around_the_file_list` now expects pass-3-rename's `:not(.renaming)` focus rules; it failed on pass-3's own tip.

`native/tools/check.py` as an unprivileged user with Rust 1.92.0: the inventories, i18n, icon guard, rustfmt and Clippy pass; it stops at `python_fixture_matches_desktop_core_py`, which fails on unchanged main too. The remaining executables and the doctests, run with check.py's own isolation, pass except the same 27 admin tests and 2 Python-fixture tests that fail on unchanged main.

Also checked by hand: files opened from other apps (written to `recently-used.xbel`) appear live; opening a file from Home moves it to the top; with 500 recently used files Home lists the newest 50, GVfs reads the whole list in about 50 ms and 30 quick changes cost the app about 0.15 s of CPU; search from Home; light and dark at 1x.

Parity: HOME-036 (rows act on their files) and HOME-037 (tile reordering) added as done; HOME-014 and HOME-015 notes updated. The message template is regenerated; no `.po` file changed.

## Not done

- No insertion mark is drawn while dragging a tile.
- A file deleted by another app leaves Recent only once Home is drawn again or the file is opened.
- The classic menu opened low in a short window runs off the screen's bottom instead of opening upward (shared menu code, tiles too).
- Selecting a Recent row leaves the Details pane on Home; Windows may show the file there.
