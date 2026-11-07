Model: claude-opus-5-5

# fix(focus): Follow Windows Explorer's Tab order and show the keyboard focus after Esc in the address bar

A follow-up to AKolenda/openxplorer#257, which made the address bar's chevron and « take keyboard focus. In practice the keyboard never visibly reached them after leaving the address bar with Esc.

## The problem

Steps, in a build of main on KDE Plasma Wayland:

1. Open OX-Test > Template test > Inside A, in Large icons view.
2. Click an empty spot in the address bar.
3. Press Esc, then press Tab.

No focus ring showed anywhere. The next Tab rang the status bar's view buttons, and the chevron, the « and the crumbs never visibly took the keyboard.

Two causes:

- **Esc left the keyboard on the icon view itself.** In Large icons and every other icon view, GTK's grid view keeps keyboard focus on itself until one of its tiles has had it, and it draws nothing there. The Details view focuses a row, so the problem did not show there.
- **Tab followed GTK's widget tree.** After the file list it went to the status bar's view buttons, the tab's Close button, New tab, the title bar's minimize, maximize and close buttons, and the navigation buttons, and only then to the chevron and the crumbs. Every row of the navigation pane was a separate stop.

## Windows' order

- Microsoft's tip [Navigate Windows Explorer More Quickly with These Keyboard Shortcuts](https://learn.microsoft.com/en-us/previous-versions/technet-magazine/ee851673(v=msdn.10)) (TechNet Magazine, written for Windows 7) gives Tab's order as "address bar, search box, toolbar, navigation pane, file list, column headings (available in Details view only)".
- Microsoft's current [Keyboard shortcuts in Windows](https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec) lists F6 in File Explorer as "Cycle through elements in the window". It has no Tab row for File Explorer.
- Neither source says where Windows 11's tabs and its Back, Forward, Up and Refresh buttons fall, or what Esc does in the address bar. Here the tabs and buttons come just before the address bar, where they sit on screen, and Esc keeps sending the keyboard to the file list, as before.

## Changes

- **Esc** returns to the crumbs with the keyboard on an item of the file list: its selected item, else its first. The item draws its ring in every view. Tab back into the list does the same. A click still focuses what it clicks.
- **Tab and Shift+Tab** go round in this order (`window/tab_order.rs`):
  1. the address bar: the chevron or « first, then each crumb shown, then the history chevron;
  2. the search box;
  3. the command bar;
  4. the navigation pane, as one stop (the arrow keys move inside it);
  5. the folder tree;
  6. Map network drive, where the navigation pane shows it;
  7. the file list, as one stop (each pane of a split tab);
  8. the Details pane;
  9. the status bar's buttons;
  10. the Open or Save bar;
  11. the tab in front, then its Close button;
  12. New tab;
  13. Back, Forward, Up and Refresh.

  The caption buttons are not stops.
- **What keeps its own Tab handling:** the handler takes the window's `move-focus`, which GTK emits only after the focused widget lets Tab go. Tab between split panes, a rename in place and the unavailable page therefore keep their own Tab handling. Dialogs, menus and Settings keep GTK's order.
- **Focus rings:** in an empty folder, "This folder is empty." takes the item ring while the list has the keyboard. Crumbs draw the 2 px focus ring the other buttons draw, instead of a 1 px edge.
- **Test support:** the test pointer (`test_support/pointer.rs`) converts GDK's logical pixels to the X server's device pixels. Nothing changes at scale 1, and real clicks also land at `GDK_SCALE=2`.
- **Docs:** `features.toml` has a new ACC-016 entry and a note on NAV-018. `ui-spec.md` §4.13 is updated.

## Testing

- `tab_after_escaping_the_address_goes_round_in_explorers_order_with_every_stop_shown` (`tests/focus.rs`) follows the steps above with real input from XTEST on the private X display:
  - a click on blank space in the address bar, then Esc, Tab and Shift+Tab;
  - in Large icons and Details, light and dark;
  - it checks the order above and that the focused widget's pixels change at each stop, the item after Esc included;
  - in a narrow window, it checks that the « is the address bar's first stop.
- `the_keyboard_on_an_empty_list_rings_its_note` covers the empty folder, and `tab_tries_every_other_stop_once_and_wraps_round` unit-tests the cycle.
- The real app was also driven with real keys, as an unprivileged user:
  - on X11 under Xvfb with Openbox and xdotool;
  - on Wayland under weston, with keys sent through an RDP client.

  Before the fix, Esc in Large icons left nothing ringed. After it, the first tile is ringed, and each Tab rings the next stop.
- The whole ox-app test binary ran under `check.py`'s isolation. All 14 focus tests pass. 14 others failed as root: 9 of them pass as an unprivileged user, and the other 5 need the retired Python app's sources (`OX_PYTHON_APP`).
- rustfmt and Clippy with `-D warnings` on Rust 1.92.0 are clean. `parity/check.py`, `i18n.py check` and `sync-docs.py --check` pass.
- Not tested: KWin itself, a screen reader, and Windows 11's own behavior for Esc in the address bar and for its tabs and navigation buttons in the Tab order.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
