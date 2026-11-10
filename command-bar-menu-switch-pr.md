# Title

feat(command-bar): Open another command bar menu with one click while one is open

# Body

Model: claude-opus-5-5

## Summary

While one of the command bar's menus is open (New, Sort, View, Search options or See more), one click on another of its menu buttons now closes the open menu and opens the one clicked, as Windows 11 File Explorer does. Before, the first click only closed the open menu and a second click was needed.

A click anywhere else now also does what it does there, as in Windows 11: a click on a file closes the menu and selects the file, a click on the address bar starts editing the address, and a click in the navigation pane goes to that place. Before, that click was spent on closing the menu. A click on the open menu's own button still just closes it, Escape still closes the menu and gives the keyboard back to its button, and choosing an item still runs it and opens nothing else.

| Action while View is open | Before | After | Windows 11 |
|---|---|---|---|
| One click on Sort | View closes, nothing opens | View closes, Sort opens | View closes, Sort opens |
| A click on View | View closes | View closes | View closes |
| A click on a file | View closes, file not selected | View closes, file selected | View closes, file selected |
| Pointer rests on Sort | Nothing | Nothing | Nothing (tooltip only) |
| Escape | View closes, keyboard on View | Same | Same |
| A click on an item | Item runs | Item runs | Item runs |

## Design

A GTK menu normally holds the pointer while it is open and spends the next click outside it on closing itself, so the click never reaches the button or file under it. The command bar's menus now hold no pointer grab (`autohide` off), so every click goes where it is; the window's existing press handler closes the open menu (it now leaves a press on a menu's own button to that button, which closes the menu itself), and the bar closes its menu when another one opens or when the window stops being active. One small module, `command_bar/menu_group.rs`, owns which of the bar's menus is open; there are no timers or idle callbacks.

Two GTK behaviors needed care without the grab. A menu without a grab is not given the keyboard by GTK, so it now takes it itself once its rows are built. And on X11, after the window lays itself out, GTK sends the window a made-up motion event at the pointer's position even when the pointer is over a menu (a grab normally prevents this), so the next click on the menu, before the pointer moves, would go to the file under it. The bar follows which surface last received pointer motion and hands such a press to the menu row under it. Wayland is not affected.

A new `command-menu=<name>` step in the snapshot hook opens a command bar menu by name, for pictures.

## Not confirmed in Windows 11

- Left and Right while a command bar menu is open: whether they move to the neighboring menu. They do not here, as before.
- Alt+Down on a focused command bar button.
- Whether a click on a plain command (such as Copy or Details) while a menu is open also runs that command. Here it does, as every click now reaches what it is on.
- A click in another app's window while a menu is open: here the menu closes when the window stops being active; not compared with Windows.

## How it was tested

- New tests with a real pointer and keyboard (XTEST on a private Xvfb display), `native/crates/ox-app/src/window/tests/command_bar_switch.rs`: every ordered pair of New, Sort, View and See more, then a click on the open menu's button; Search options and View during a search; a compact (672 pixel) window; quick clicks on Sort, View and See more while New is open; clicks on a file, the bar's empty part, the address bar and the navigation pane; a click on a menu item, including after the window lays itself out while the pointer rests on the item; Escape, Enter, Space and Down. The switching, quick-click, Search options, narrow-window and click-through tests fail on the current main.
- The same tests on Wayland (weston 13 with its X11 backend inside Xvfb, so real input reaches GTK as Wayland events). KDE Plasma was not available.
- `cargo fmt --check`, Clippy with the workspace lints, `python3 native/parity/check.py`, and every native test executable and the doctests, isolated as `native/tools/check.py` runs them.
- Pictures of Sort opened from an open View, at 1x and 2x, light and dark, normal and Compact view, with `tools/native_capture.py` and its fictional demo files.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
