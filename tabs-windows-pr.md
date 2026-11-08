# Title

fix(tabs): Open a tab's menu from the keyboard and clear the drop outline on the tab strip

# Body

Model: claude-opus-5-5

This makes the tab menu reachable by keyboard and fixes a leftover outline after files are dropped on a tab. It adds no menu items and changes no drop behavior. Microsoft's documentation only confirms "Duplicate tab" on the tab menu, which the app already has, and that files can be dragged "across tabs", which the app already supports (TAB-018). Everything else the job asked about is listed below as not confirmed, or as a conflict with an existing owner decision. Neither kind was built.

What changes:

- **Menu key and Shift+F10 on a focused tab** open that tab's menu under it, with the keyboard on the first item. Before, a focused tab ignored both keys, so its menu needed a pointer.
- **Escape gives the keyboard back to the tab** after a menu opened that way. Before, it landed on the "+" button.
- **F6 lands on the tab in front.** It used to land on the first tab's close button, so the Menu key opened the wrong tab's menu and the arrow keys didn't move along the row. ACC-015 already said "focus lands on the front tab".
- **No outline stays on the tab strip after a drop.** GTK left its own thin drop-target box on the strip, visible behind every tab but the one in front, until the window closed.

| Behavior | Microsoft source | How it was tested |
| --- | --- | --- |
| Shift+F10 opens the context menu of the selected item; here, the focused tab | https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec ("Shift+F10: Display the context menu for the selected item") | `the_menu_key_and_shift_f10_open_the_focused_tabs_menu` (fails on main); real keys under Xvfb |
| F6 includes the tab row; arrow keys move along it | https://blogs.windows.com/windows-insider/2022/07/05/announcing-windows-11-insider-preview-build-22621-290-22622-290/ ("The row of tabs should now be included in the keyboard focus cycle when pressing Tab or F6", "you can use left or right arrow keys to navigate through them") | `f6_into_the_tab_row_focuses_the_tab_in_front` (fails on main); real F6 and Left under Xvfb |
| Duplicate tab is on the tab menu (already present, unchanged) | https://blogs.windows.com/windows-insider/2024/05/03/announcing-windows-11-insider-preview-build-22635-3570-beta-channel/ | real right-click with 1, 2 and 5 tabs |
| Files can be dropped across tabs (already present, unchanged; this PR clears the leftover outline) | https://blogs.windows.com/windows-insider/2023/02/15/announcing-windows-11-insider-preview-build-25300/ ("Drag and dropping files and folders across tabs should work again") | extended `a_drop_on_a_tab_goes_into_its_folder_and_hovering_shows_the_tab` (fails on main); real drags in Details, Tiles and Content |
| Ctrl+drag copies, Shift+drag moves (already present, unchanged) | https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec | real Ctrl drag onto a tab: "Copy to Archive", source kept |
| A drop on another volume copies by default (already present, unchanged) | https://learn.microsoft.com/en-us/windows/win32/shell/clipboard | existing tests only, not re-run with real input |

Not built, because no Microsoft source confirms it: New tab, Close tabs to the right, other item names or order, where a duplicate opens, whether a moved tab keeps its history, and hover-to-switch on a tab during a drag. The status file lists what to check on Windows.

How it was tested:

- Each new or extended test was run before its fix and failed, then passed after it.
- rustfmt and clippy are clean. `python3 native/parity/check.py` passes. The public-data audit is clean.
- Every test executable ran with check.py's own isolation. ox-app passed 2106 of 2106, including the real-pointer tab-dragging tests. check.py itself stops at an ox-core Python fixture. Run one by one, three ox-core executables fail: location_fixture_drift and network_python, both from the container's Python 3.13, and nine admin_copy tests that depend on fstab and root. This PR changes no ox-core file.
- Real input was xdotool on a private Xvfb display with openbox. The branch's debug build ran as an unprivileged user on a fictional /home/demo tree, at GDK_SCALE 1 and 2, in the light and dark themes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
