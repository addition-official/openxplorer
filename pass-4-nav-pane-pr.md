# Title

fix(sidebar): Fix pinning, drops, saved width, eject failures, expanding and Network in the navigation pane

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 4 tested the navigation pane against Windows 11 with real pointer and keyboard input on a private Xvfb display, as an unprivileged user with a throwaway home. These are the fixes:

- **Pin goes at the end.** A folder pinned from its menu, the command bar or Home now goes after the last pin, as in Windows 11. It used to go above Desktop on a new setup, and a standard folder that was unpinned and then pinned again went to the top. The pin was saved without the order the pane showed.
- **Files dropped near a pin's edge go into it.** A file dragged over the top or bottom edge of a pinned folder showed the "Pin to Quick access" line, then failed with "Could not pin" and stayed where it was. A drag of this app's that holds any file now treats the whole pinned row as the folder ("Move to Pictures"). Folders still pin between pins.
- **The pane's width is saved after dragging its edge.** GtkPaned takes the press on its handle, which canceled the gesture that saved the width before the edge had moved, so a dragged width was never saved. Keyboard resizing and the double-click reset already worked.
- **A drive that can't be removed leaves the tabs where they were.** Eject, Safely remove and Disconnect move the tabs on the drive to Home first, so the window doesn't keep the drive busy. When the removal then failed, the tabs stayed at Home. They now go back to their folders. A drive that is still in use is refused with Windows' wording: "This device is currently in use. Close any programs or windows that might be using the device, and then try again."
- **Double-click expands and collapses.** A double-click on This PC or Network, on a computer under Network, or on a folder of the folder tree, collapses or expands it, as in Windows' navigation pane (checked in Windows 11 26200). It used to just open the place again. The toggle runs once the button is released, and the second click no longer opens the place a second time.
- **The pane's menu matches Windows 11 (checked in 26200).** Right-clicking the empty space now starts with Show This PC and Show Network (both checked; they hide and show those sections), Show all folders (shows the folder tree) and Expand to current folder, off by default. Windows' Show libraries is left out, since there are no libraries. The app's own items (Add entry, Show all entries, icon sizes) follow after a divider. With Expand to current folder off, the folder tree doesn't open its folders down to the folder shown. It's saved with the tree's other options as `folderTree.expandToCurrent`. **Behavior change:** the tree used to always follow the folder shown, and now does so only with the option on.
- **Discovered computers under Network (checked in Windows 11 26200).** Network in the pane now lists the computers that discovery found, after its network locations, each with its own chevron. Going to Network, or expanding Network in the pane, runs discovery; nothing is discovered at startup. A computer's chevron, Right/Left or a double-click lists its shared folders under it, mounting the computer and asking for a password through the app's sign-in, as opening it does. Computers and shares open in a new tab or window from their menus.
- **Home and Gallery open in a new tab or window from their menus**, like every other node. Their menus offered only Hide.

## Tests

Every fix has a test that failed before the fix and passes after it. The ones that need a real pointer use the in-tree XTEST `Pointer`:

- `window::tests::nav_pane` (new): `a_folder_pinned_from_a_menu_goes_at_the_end_of_the_pins`, `a_real_drag_of_a_file_onto_the_edge_of_a_pin_goes_into_the_pinned_folder`, `a_real_drag_of_the_pane_edge_saves_its_width`, `a_drive_that_cannot_be_ejected_leaves_the_tab_where_it_was`, `a_real_double_click_on_this_pc_collapses_and_expands_it`, `the_pane_menu_starts_with_windows_items_in_windows_order`, `the_tree_expands_to_the_current_folder_only_with_the_option_on`, `home_and_gallery_menus_open_them_in_a_new_tab_or_window`, `network_lists_the_discovered_computers_each_with_a_chevron`, `expanding_network_in_the_pane_discovers_the_computers`, `a_computer_s_chevron_lists_its_shares_under_it`, `right_and_left_open_and_close_a_computer`, `a_real_double_click_on_a_computer_lists_its_shares`. Discovery in tests is the in-tree fixed `Discoverer`, and the "computer" is a local fixture folder, so no test reaches a network.
- Folder-tree tests that rely on the tree following the folder shown now turn Expand to current folder on (`folder_tree`, `more_explorer_keys`, `expand_arrows`, `drop_frames`, `file_operations` and `file_drop::targets` tests), and `sidebar::add_entry_pins_a_typed_location_and_edit_changes_it_in_place` finds Add entry after Windows' items.
- `window::tests::folder_tree::a_real_double_click_on_a_folder_of_the_tree_opens_and_closes_it`
- `devices::tests::a_busy_drive_is_refused_in_windows_words`
- Updated tests that relied on new pins going first: `sidebar_layout::the_selected_row_is_marked_by_its_fill_without_an_accent_bar`, `context_menus::a_pinned_folder_menu_is_windows_11_and_keeps_its_strip_nearest_the_pointer`, `drag_escape::escape_during_a_drag_onto_a_navigation_pane_row_moves_nothing` (through the `middle_of`/`top_edge_of` test helpers, which now take the last row with a name) and `home_page::home_is_spaced_as_windows_11_home`. `network::the_tabs_on_a_drive_move_home_before_it_is_ejected` now also checks that the tabs come back after the failed eject.

I also exercised the pane by hand with xdotool against the built app on a private Xvfb, using fixtures of mp3, mp4, pdf, jpg, odt, zip, 7z, a 2 GB sparse file and odd names. That covered: pin and unpin from the pane, the file list and drag; reordering among 200 pins with long Unicode names; resizing; menus of every item type; a loop-mounted drive appearing and disappearing live; and a failed Disconnect on that drive, after which the tab came back to it. The app logged no CRITICAL, panic or signal.

`native/parity/check.py` passes. `rustfmt` and Clippy are clean for the changed files. Clippy as `check.py` runs it (`-D warnings`) fails on unchanged main with the installed toolchain (1.97, `duration_suboptimal_units` in untouched ox-core and ox-app files), so the test executables were run one by one with `check.py`'s `run_isolated`. See the status file for the comparison with main.

## A crash the tests surfaced (also on unchanged main)

The test harness can corrupt memory when a test window is destroyed within the double-click time of a click on it. The crash shows up later, in another test: `free(): invalid pointer`, `g_object_unref` on a finalized object, or SIGSEGV. A probe on unchanged main reproduces it in 10 of 25 runs: open This PC from the pane, click its chevron, let the test end, then drag a file in a new window. It drops to 0 of 25 when the probe waits 700 ms before its window is destroyed. Memcheck showed nothing (it serializes threads). The new real-input tests therefore move the pointer away and wait past the double-click time before ending, like the existing `drag_escape` test. After that, the navigation-pane group ran clean 16 times in a row. In the built app, 10 rounds of the same steps, with Ctrl+W right after the click, didn't crash. So it hasn't been seen outside the harness, but its root cause in GTK isn't found.

## Limitations

- The pin line still shows for files dragged in from other apps, since a drop target can't tell folders from files until the drop. The drop then fails with "Could not pin", as before.
- "Show all folders" shows this app's folder tree under the places. It doesn't turn the whole pane into Windows' Desktop-rooted tree.
- A pinned folder or a drive still has no chevron of its own; only This PC, Network and the discovered computers do.
- Without udisks here, the busy wording is tested with a simulated GIO error, not a real busy USB drive.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
