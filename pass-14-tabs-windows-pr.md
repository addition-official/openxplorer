# Title

fix(tabs): Reopen every window, keep both panes of a moved split tab, and put the keyboard in new tabs

# Body

Model: claude-opus-5-5

## Problem

Pass 14 tested tabs, several windows and split view against Windows 11 with real input (xdotool) in a private Xvfb, as an unprivileged user with a throwaway home and real-world files. Most of it already worked: Ctrl+T, +, middle-click and Ctrl+double-click on folders, Ctrl+W, middle-click and × on tabs, Ctrl+Shift+T, Ctrl+Tab and Ctrl+Shift+Tab, Ctrl+1 to Ctrl+9 (Ctrl+9 is the ninth tab, as in Windows), the tab menu, Duplicate tab, dragging tabs along the strip, out into a new window and into another window, dropping files on a tab (the tab comes to the front after a short hover), each tab's history, selection, view and scroll, Ctrl+N, closing the last window ending the app, F6 through both panes of a split tab, copy and paste between panes, and 50 tabs (memory 200 to 202 MB). These were broken:

1. **"Reopen my tabs" kept only one window.** The window in front saved its own tabs, so an app ended at logout with two windows came back with one; the other window's tabs were lost. Windows 11 reopens every folder window with its tabs (Insider build 22635.4580, KB5050094).
2. **Moving a split tab lost a pane.** "Move tab to new window", "Move tab to window" and dragging a split tab out took only its active pane. The other pane's folder and history were gone, and it was not in the reopen list.
3. **The keyboard was not in a new tab.** After Ctrl+T or + while the keyboard was on a button, or after another application opened a folder in the running app, Down moved along the command bar or the navigation pane instead of the files.
4. **A green frame round the tab strip.** While files were dragged over the tabs, GTK drew its green drop frame round the whole strip as well as the app's outline of the tab under the pointer.
5. **The Local Disk tab showed a folder.** The tab and the address bar of `/` (Local Disk), and the tab of a mounted drive's root, showed the yellow folder instead of the drive.
6. **A folder opened from outside did not ring its first item.** Checked in the Windows 11 VM: a folder another app opens, or one typed into the address bar and opened with Enter, shows the keyboard ring on its first item, unselected, so the first Down selects the second item. Here Down selected the first.

## What changes

- **Every window is saved and reopened.** The window in front saves every browsing window, its own first: every 3 s when something changed, at once when another window closes (the closed one is left out), and when the session ends. Quit saves them all before they close, so the last window to close no longer overwrites that with itself alone. A start without locations reopens every window and shows the one that was in front last. The file keeps the front window in the one-window form and adds the others under `otherWindows` (at most 20; a damaged one is skipped), so an older version still reopens the front window. The Settings description now reads "Starting OpenXplorer without a folder reopens every window with the tabs and split panes it had." (.pot regenerated; no .po touched).
- **Both panes move.** A moved split tab takes both panes, each with its history, selection and scroll position, and the pane that was active stays active, whether moved from the tab menu or by dragging.
- **A tab opened in front takes the keyboard into its list**, as clicking a place or typing an address already does. A tab opened behind leaves the keyboard where it is. After Ctrl+T, Down selects the first item, as in Windows 11 (checked in the VM).
- **A folder opened from outside or typed rings its first item.** Its first listing puts the keyboard on its first item without selecting it and shows the ring; the next plain Up or Down in Details moves on from that item, where GTK would select the ringed item itself. Another app's folder opens in a new tab, the app's default; Settings > "Open folders from other apps in a new window" (NAV-042) gives it a window, as the Windows 11 build in the VM did.
- **Only the tab under a file drag is outlined.**
- **Drives show the drive** on their tab, and `/` in the address bar too.
- Merged pass-1 to pass-13 and network-like-windows, which change the same files. pass-4-nav-pane (5cb3a27c) and network-like-windows (2af9384b) each added the computers under Network to the navigation pane, with conflicting actions, types and row classes; pass-4's copy is reverted in its own commit before the merge, keeping network-like-windows' version. Two tests broken by merged branches are updated: the stylesheet test now expects pass-3's `:not(.renaming)` focus rules, as pass-10 also does.

## Tests run

- New tests, each failing before its fix: `every_saved_window_comes_back_with_the_front_one_first` (ox-core), `the_window_in_front_saves_the_tabs_of_every_window`, `quit_keeps_every_window_for_the_next_start`, `a_start_reopens_every_saved_window`, `a_split_tab_moves_to_another_window_with_both_panes`, `a_tab_opened_in_front_puts_the_keyboard_in_its_folder`, `a_folder_opened_from_outside_or_typed_rings_its_first_item_and_down_selects_the_second` (real keys through XTEST), `a_drag_over_the_tab_strip_draws_no_frame_round_it`, `the_tab_of_a_drive_shows_the_drive`, and the Local Disk case of `tabs_and_the_address_bar_show_each_locations_own_icon`.
- Real input on a private Xvfb with Openbox, as an unprivileged user with a throwaway home: two windows ended with SIGTERM after a minute reopen both; Quit with two windows reopens both; a split tab dragged out of its window arrives with both panes; Down after Ctrl+T selects in the new tab; a file drag over a tab outlines only that tab; the Local Disk tab shows the drive. Light and dark captures at 1x.
- `native/parity/check.py` passes. `native/tools/check.py`'s stages with Rust 1.92 (CI's toolchain) pass: inventories, icon guard, rustfmt, Clippy. Every test executable through `run_isolated` as an unprivileged user: 87 of 89 pass and the doctests pass; ox-app 2773 passed, 0 failed. `location_fixture_drift` and `network_python` fail the same way on unchanged main 5d21fd3e on this machine (its Python parses `smb://[V1.x]` differently from the committed fixture).

## Not tested

Wayland (the tab-drag tests use XTEST under Xvfb); the Windows 11 behavior on the VM CHECK list; translations.

CLA: I agree
