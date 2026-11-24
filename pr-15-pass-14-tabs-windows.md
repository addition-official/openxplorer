# Title

fix(tabs): Reopen every window, keep both panes of a moved split tab, and put the keyboard in new tabs

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/14-pass-13-startup)

## What changes for the user
- **Every window is saved and reopened.** The window in front saves every browsing window, its own first: every 3 s when something changed, at once when another window closes (the closed one is left out), and when the desktop logs out. Quit saves them all before they close, so the last window to close no longer overwrites that with itself alone. A start without locations reopens every window and shows the one that was in front last. The saved file keeps the front window in the one-window form and adds the others under `otherWindows` (at most 20; a damaged one is skipped), so an older version still reopens the front window. The Settings description is updated to match.
- **A moved split tab keeps both panes.** "Move tab to new window", "Move tab to window" and dragging a split tab out take both panes, each with its history, selection and scroll position, and the pane that was active stays active.
- **A tab opened in front takes the keyboard into its list**, as clicking a place or typing an address already does. A tab opened behind leaves the keyboard where it is. After Ctrl+T, Down selects the first item, as in Windows 11.
- **A folder opened from outside or typed rings its first item.** Its first listing puts the keyboard on its first item without selecting it and shows the ring; the next plain Up or Down in Details moves on from that item. Another app's folder opens in a new tab, the app's default; Settings > "Open folders from other apps in a new window" (NAV-042) gives it a window.
- **Only the tab under a file drag is outlined**, without GTK's green drop frame around the whole tab strip.
- **Drives show the drive** on their tab, and `/` (Local Disk) shows it in the address bar too.
- The interface guide mentions that every window comes back.
- Two tests are updated for changes made earlier in the series: the stylesheet test expects the rename work's `:not(.renaming)` focus rules, and the accessibility test no longer exempts `window/loading.rs`, since the New menu work waits for painted frames without a tick callback.

## Why
- "Reopen my tabs" kept only the window in front, so an app ended at logout with two windows came back with one. Windows 11 reopens every folder window with its tabs (Insider build 22635.4580, KB5050094).
- Moving a split tab took only its active pane; the other pane's folder and history were lost, and it was not in the reopen list.
- After Ctrl+T or + while the keyboard was on a button, or after another application opened a folder in the running app, Down moved along the command bar or the navigation pane instead of the files.
- While files were dragged over the tabs, GTK drew its drop frame around the whole strip as well as the app's outline of the tab under the pointer.
- The tab and address bar of `/`, and the tab of a mounted drive's root, showed the yellow folder.
- In the Windows 11 VM, a folder another app opens, or one typed into the address bar, rings its first item unselected, so the first Down selects the second item; here Down selected the first.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Starting OpenXplorer without a folder reopens the tabs and split panes the last window had." → "Starting OpenXplorer without a folder reopens every window with the tabs and split panes it had."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: new tests, each failing before its fix: `every_saved_window_comes_back_with_the_front_one_first` (ox-core), `the_window_in_front_saves_the_tabs_of_every_window`, `quit_keeps_every_window_for_the_next_start`, `a_start_reopens_every_saved_window`, `a_split_tab_moves_to_another_window_with_both_panes`, `a_tab_opened_in_front_puts_the_keyboard_in_its_folder`, `a_folder_opened_from_outside_or_typed_rings_its_first_item_and_down_selects_the_second` (real keys through XTEST), `a_drag_over_the_tab_strip_draws_no_frame_round_it`, `the_tab_of_a_drive_shows_the_drive`, and the Local Disk case of `tabs_and_the_address_bar_show_each_locations_own_icon` (some of these were later removed as redundant and are not in this branch); real input on a private Xvfb with Openbox as an unprivileged user with a throwaway home (two windows ended with SIGTERM reopen both, Quit with two windows reopens both, a split tab dragged out arrives with both panes, Down after Ctrl+T, a file drag over a tab, the Local Disk tab; light and dark captures at 1x); `native/parity/check.py`; `native/tools/check.py`'s inventories, icon guard, rustfmt and Clippy with Rust 1.92; every test executable through `run_isolated` as an unprivileged user (87 of 89 pass, ox-app 2773 passed, 0 failed; the 2 others fail the same way on unchanged main on that machine) and the doctests. Wayland was not tested.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
