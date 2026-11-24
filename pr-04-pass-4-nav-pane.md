# Title

fix(sidebar): Fix pinning, drops, saved width, eject failures and expanding in the navigation pane

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/03-pass-3-rename)

## What changes for the user
- A folder pinned from its menu, the command bar or Home goes after the last pin, as in Windows 11, and the pin is saved in the order the pane shows.
- A file dragged by this app over the top or bottom edge of a pinned folder goes into that folder ("Move to Pictures"). Folders still pin between pins.
- The navigation pane's width is saved after dragging its edge.
- When Eject, Safely remove or Disconnect fails, the tabs that were moved to Home before the removal go back to their folders. A drive still in use is refused with Windows' wording: "This device is currently in use. Close any programs or windows that might be using the device, and then try again."
- A double-click on This PC or Network in the pane, or on a folder of the folder tree, collapses or expands it, as in Windows' navigation pane. The toggle runs once GTK has handled the press, and the second click no longer opens the place again.
- Right-clicking the pane's empty space starts with Windows 11's Show This PC and Show Network (checked; they hide and show those sections), Show all folders (shows the folder tree) and Expand to current folder (off by default), then a divider and the app's own items. Windows' Show libraries is left out.
- **Behavior change:** the folder tree opens its folders down to the folder shown only with Expand to current folder on, saved as `folderTree.expandToCurrent`. Before, it always did.
- Home and Gallery in the pane open in a new tab or window from their menus, like every other node.
- A commit that listed discovered computers under Network in the pane is added and then reverted in this PR, so this PR does not change what Network lists; PR 07 brings computers to the pane.
- The last two commits drop tests of this change that were found redundant, and the helpers only they used.

## Why
- New pins went above Desktop on a new setup, and a standard folder pinned again went to the top, because the pin was saved without the order the pane showed.
- A file dropped near a pin's edge showed the "Pin to Quick access" line, then failed with "Could not pin" and stayed where it was.
- GtkPaned takes the press on its handle, which canceled the gesture that saved the width before the edge had moved.
- After a failed removal the tabs stayed at Home.
- A double-click just opened the place again, where Windows expands or collapses it.
- The pane's menu lacked Windows 11's items (checked in build 26200), and Home's and Gallery's menus offered only Hide.

Limitations: the pin line still shows for files dragged in from other apps, whose drop then fails with "Could not pin" as before; "Show all folders" shows the app's folder tree under the places, not Windows' Desktop-rooted tree; a pinned folder or a drive has no chevron of its own; the busy wording is tested with a simulated GIO error, not a real busy USB drive.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- New: "Show This PC"
- New: "Show Network"
- New: "Show all folders"
- New: "Expand to current folder"
- New: "This device is currently in use. Close any programs or windows that might be using the device, and then try again."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: the new `window::tests::nav_pane` tests for this PR's fixes (`a_folder_pinned_from_a_menu_goes_at_the_end_of_the_pins`, `a_real_drag_of_a_file_onto_the_edge_of_a_pin_goes_into_the_pinned_folder`, `a_real_drag_of_the_pane_edge_saves_its_width`, `a_drive_that_cannot_be_ejected_leaves_the_tab_where_it_was`, `a_real_double_click_on_this_pc_collapses_and_expands_it`, `the_pane_menu_starts_with_windows_items_in_windows_order`, `the_tree_expands_to_the_current_folder_only_with_the_option_on`, `home_and_gallery_menus_open_them_in_a_new_tab_or_window`), `window::tests::folder_tree::a_real_double_click_on_a_folder_of_the_tree_opens_and_closes_it` and `devices::tests::a_busy_drive_is_refused_in_windows_words`, each failing before its fix; the folder-tree, sidebar, sidebar layout, context menu, drag escape, Home page and network tests updated for the option and for pins at the end; hand checks with xdotool against the built app on a private Xvfb with fictional files (pinning and unpinning, reordering among 200 pins with long Unicode names, resizing, every item type's menu, a loop-mounted drive appearing and disappearing, a failed Disconnect returning the tab), with no CRITICAL, panic or signal logged; `native/parity/check.py`; rustfmt and Clippy on the changed files; the test executables one by one with `check.py`'s `run_isolated`, since `check.py`'s Clippy step fails on unchanged main with the installed toolchain. Three of the `nav_pane` tests above are dropped as redundant by this PR's last commits. The test harness can corrupt memory when a test window is destroyed within the double-click time of a click on it (also on unchanged main); the new real-input tests move the pointer away and wait past the double-click time before ending.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
