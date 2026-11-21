# Title

fix(sidebar): Fix pinning, drops, saved width, eject failures and expanding in the navigation pane

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 4 tested the navigation pane against Windows 11 with real pointer and keyboard input on a private Xvfb display, as an unprivileged user with a throwaway home. These are the fixes:

- **Pin goes at the end.** A folder pinned from its menu, the command bar or Home now goes after the last pin, as in Windows 11. It used to go above Desktop on a new setup, and a standard folder that was unpinned and then pinned again went to the top. The pin was saved without the order the pane showed.
- **Files dropped near a pin's edge go into it.** A file dragged over the top or bottom edge of a pinned folder showed the "Pin to Quick access" line, then failed with "Could not pin" and stayed where it was. A drag of this app's that holds any file now treats the whole pinned row as the folder ("Move to Pictures"). Folders still pin between pins.
- **The pane's width is saved after dragging its edge.** GtkPaned takes the press on its handle, which canceled the gesture that saved the width before the edge had moved, so a dragged width was never saved. Keyboard resizing and the double-click reset already worked.
- **A drive that can't be removed leaves the tabs where they were.** Eject, Safely remove and Disconnect move the tabs on the drive to Home first, so the window doesn't keep the drive busy. When the removal then failed, the tabs stayed at Home. They now go back to their folders. A drive that is still in use is refused with Windows' wording: "This device is currently in use. Close any programs or windows that might be using the device, and then try again."
- **Double-click expands and collapses.** A double-click on This PC or Network, or on a folder of the folder tree, collapses or expands it, as in Windows' navigation pane. It used to just open the place again. The toggle runs once GTK has handled the press: a first version collapsed the section from inside the list's press handling, and that corrupted GTK's memory, crashing a later window close (`free(): invalid pointer`, or `g_object_unref` on a finalized object) in about two of three test runs. The fix is a later commit on this branch, so squash-merging the branch brings in only the fixed version. The fixed one ran clean 14 times in a row, and the live app closed cleanly after repeated double-clicks.
- **Show all folders and Expand to open folder.** The pane's empty-space menu offers Windows' "Show all folders", which shows the folder tree, and "Expand to open folder". With Expand to open folder off, the tree no longer opens its folders down to the folder shown. It is on by default, which keeps the current behavior. The tree's own menu offers the option too, and it is saved with the tree's other options as `folderTree.expandToOpen`.
- **Home and Gallery open in a new tab or window from their menus**, like every other node. Their menus offered only Hide.

## Tests

Every fix has a test that failed before the fix and passes after it. The ones that need a real pointer use the in-tree XTEST `Pointer`:

- `window::tests::nav_pane` (new): `a_folder_pinned_from_a_menu_goes_at_the_end_of_the_pins`, `a_real_drag_of_a_file_onto_the_edge_of_a_pin_goes_into_the_pinned_folder`, `a_real_drag_of_the_pane_edge_saves_its_width`, `a_drive_that_cannot_be_ejected_leaves_the_tab_where_it_was`, `a_real_double_click_on_this_pc_collapses_and_expands_it`, `the_pane_offers_show_all_folders_and_expand_to_open_folder`, `with_expand_to_open_folder_off_the_tree_stays_closed`, `home_and_gallery_menus_open_them_in_a_new_tab_or_window`
- `window::tests::folder_tree::a_real_double_click_on_a_folder_of_the_tree_opens_and_closes_it`
- `devices::tests::a_busy_drive_is_refused_in_windows_words`
- Updated tests that relied on new pins going first: `sidebar_layout::the_selected_row_is_marked_by_its_fill_without_an_accent_bar`, `context_menus::a_pinned_folder_menu_is_windows_11_and_keeps_its_strip_nearest_the_pointer`, `drag_escape::escape_during_a_drag_onto_a_navigation_pane_row_moves_nothing` (through the `middle_of`/`top_edge_of` test helpers, which now take the last row with a name) and `home_page::home_is_spaced_as_windows_11_home`. `network::the_tabs_on_a_drive_move_home_before_it_is_ejected` now also checks that the tabs come back after the failed eject.

I also exercised the pane by hand with xdotool against the built app on a private Xvfb, using fixtures of mp3, mp4, pdf, jpg, odt, zip, 7z, a 2 GB sparse file and odd names. That covered: pin and unpin from the pane, the file list and drag; reordering among 200 pins with long Unicode names; resizing; menus of every item type; a loop-mounted drive appearing and disappearing live; and a failed Disconnect on that drive, after which the tab came back to it. The app logged no CRITICAL, panic or signal.

`native/parity/check.py` passes. `rustfmt` and Clippy are clean for the changed files. Clippy as `check.py` runs it (`-D warnings`) fails on unchanged main with the installed toolchain (1.97, `duration_suboptimal_units` in untouched ox-core and ox-app files), so the test executables were run one by one with `check.py`'s `run_isolated`. See the status file for the comparison with main.

## Limitations

- The pin line still shows for files dragged in from other apps, since a drop target can't tell folders from files until the drop. The drop then fails with "Could not pin", as before.
- "Show all folders" shows this app's folder tree under the places. It doesn't turn the whole pane into Windows' Desktop-rooted tree.
- Without udisks here, the busy wording is tested with a simulated GIO error, not a real busy USB drive.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
