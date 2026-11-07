Model: claude-opus-5-5

# fix(focus): Follow Windows Explorer's Tab order after Esc and show every Tab stop

This targets the branch of AKolenda/openxplorer#257 (`win11/address-collapsed-keyboard`, head 50d06d8b) and fixes the Tab order and the visible focus around its chevron and «.

## The problem

Steps, in a build of #257 on KDE Plasma Wayland:

1. Open OX-Test > Template test > Inside A.
2. Click an empty spot in the address bar.
3. Press Esc, then press Tab.

The first Tab showed no focus highlight anywhere, and the second landed on the Details and Tiles view buttons at the right end of the status bar. The keyboard never visibly reached the chevron, the « or the crumbs.

On a private X display the same steps showed why. Esc leaves the keyboard in the file list, but Tab then followed GTK's widget tree:

- the status bar's view buttons;
- the tab's Close button and New tab;
- the title bar's minimize, maximize and close buttons;
- the navigation buttons;
- only then the chevron and the crumbs.

Every row of the navigation pane was a stop of its own. In an empty folder, Esc left the keyboard on the empty list, which drew nothing.

## Windows' order

- Microsoft's tip [Navigate Windows Explorer More Quickly with These Keyboard Shortcuts](https://learn.microsoft.com/en-us/previous-versions/technet-magazine/ee851673(v=msdn.10)) (TechNet Magazine, written for Windows 7) gives Tab's order as "address bar, search box, toolbar, navigation pane, file list, column headings (available in Details view only)". It gives F6's order as the same list without the search box.
- Microsoft's current [Keyboard shortcuts in Windows](https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec) (Windows 11 and 10) lists F6 in File Explorer as "Cycle through elements in the window". It has no Tab row for File Explorer.
- Neither source says where Windows 11's tabs and its Back, Forward, Up and Refresh buttons fall. Here they come just before the address bar, where they sit on screen.

## Changes

- New `window/tab_order.rs`. It takes the window's `move-focus` for Tab and Shift+Tab. GTK emits that signal only after the focused widget has let the key go, so these keep their own Tab handling: Tab between split panes (VIEW-059), a rename in place, and the unavailable page. Dialogs, menus and the Settings page keep GTK's order.
- Tab now goes round in this order:
  1. the address bar: the chevron or « first, as #257 has it, then each crumb shown, then the history chevron;
  2. the search box;
  3. the command bar;
  4. the navigation pane, as one stop (the arrow keys move inside it);
  5. the folder tree;
  6. Map network drive;
  7. the file list, as one stop on its current item (each pane of a split tab);
  8. the Details pane;
  9. the status bar's buttons;
  10. the Open or Save bar;
  11. the tab in front, then its Close button;
  12. New tab;
  13. Back, Forward, Up and Refresh.

  The caption buttons are not stops.
- Esc still returns the keyboard to the file list, as before. From there, Tab reaches the view buttons, the tab row, the navigation buttons and then the chevron.
- When an empty list has the keyboard, "This folder is empty." takes the item focus ring, for keyboard focus only. ACC-009's "no ring around the list" still holds.
- Crumbs now draw the 2 px focus ring the other buttons draw. Their old 1 px edge was hard to see.
- `test_support/pointer.rs` converts GDK's logical pixels to the X server's device pixels. Nothing changes at scale 1. Real clicks now also land at `GDK_SCALE=2`.
- `features.toml` has a new ACC-016 entry and a note on NAV-018. `ui-spec.md` §4.13 is updated.

## Testing

- `tab_after_escaping_the_address_goes_round_in_explorers_order_with_every_stop_shown` (`tests/focus.rs`) runs the user's steps with real input from XTEST on the private display:
  - a click on blank space in the address bar, then Esc, Tab and Shift+Tab;
  - in light and dark;
  - it asserts the order above and checks that each stop's pixels change when it takes the keyboard;
  - in a narrow window, it checks that the « is the address bar's first stop.
- `the_keyboard_on_an_empty_list_rings_its_note` covers the empty folder.
- `tab_tries_every_other_stop_once_and_wraps_round` unit-tests the cycle.
- With the new handler switched off, both window tests fail.
- These pass on the private display: accessibility (12), address_bar (12), address_input (19), focus (14), split_view (7) and tabs (14), plus the crumb strip, focus region and tab order unit tests (85 in all). The theme, look, chrome and empty folder look tests pass too.
- rustfmt and Clippy with `-D warnings` on Rust 1.92.0 are clean. `parity/check.py`, `i18n.py check` and `sync-docs.py --check` pass.
- Three tests in other modules fail in this container, and they fail the same way on 50d06d8:
  - `ctrl_v_pastes_nothing_into_a_folder_that_could_not_be_read` and `new_and_paste_stay_off_while_an_unavailable_folder_is_listed_again`: the tests ran as root, which ignores folder permissions;
  - `reset_returns_every_windows_sidebar_to_its_default_width`: the Python test helper.
- Not tested: a real KDE Plasma Wayland session, a screen reader, and Windows 11's own order for its tabs and navigation buttons.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
