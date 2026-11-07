# fix(look): Outline the file list's focused item only while the list has the keyboard, as Windows 11 does

Follows up #227, which drew the file list's selection, hover and keyboard ring like Windows 11.

## What was wrong

After #227, a click in the file list drew the selected row's fill but no outline, in light and in dark. The only focus mark was the two-tone ring, which `:focus-visible` shows only during keyboard navigation. An item that kept keyboard focus after a Ctrl+click had cleared its selection showed nothing at all.

## How Windows 11 behaves

This was compared against Windows 11 File Explorer's Details view, in light and dark mode:

1. Selected, with the list focused: the selection fill plus a 1px outline around the row. Light: light blue fill, dark outline. Dark: gray fill, near-white outline.
2. Focused but not selected (after Ctrl+click): the outline only, no fill.
3. Selected and under the pointer: fill plus outline.
4. Under the pointer only: a fainter fill, no outline.

A selected item in a list without keyboard focus keeps its fill and shows no outline, for example while the navigation pane or the address bar has the keyboard. Keyboard navigation keeps the two-tone focus visual described in Microsoft's focus visual guidelines: https://learn.microsoft.com/en-us/windows/apps/design/input/guidelines-for-visualfocus

## Change

- The focused row (Details) or tile draws `outline: 1px solid @ox_item_focus_outer; outline-offset: -1px` in #227's ring color: dark in light, white in dark.
- The outline is drawn only while the list holds the window's keyboard focus. A focus controller gives the view a `keyboard-in` class while focus is inside it (`window/input.rs`), and the outline rules require that class. A row GTK still marks `:focus` gets no outline once the keyboard has moved elsewhere.
- The selection fill never depends on focus.
- Keyboard navigation keeps #227's two-tone `:focus-visible` ring, which still takes precedence. #227's colors are unchanged, and drop-target outlines still win.
- High contrast draws the outline 2px in the text color.
- ui-spec.md §4.13, the row states table and ACC-009 in the parity inventory are updated.

## How it was tested

- `items_take_windows_11s_fill_outline_and_ring_in_each_state` (look.rs) checks the painted pixels in light and dark for each state: selected in an unfocused list (fill, no edge), selected and focused (fill plus outline), selected, focused and hovered, focused and unselected (outline, no fill), hovered only (fainter fill, no outline), and keyboard navigation (two-tone ring). It fails without the CSS change.
- `a_selected_row_keeps_its_fill_and_outlines_only_while_the_list_has_the_keyboard` (look.rs) opens the parent with Up so the folder left is selected (NAV-011). In light and dark it checks fill plus outline while the list has the keyboard, and fill without outline while the navigation pane or the address bar has it.
- The real app was run under Weston (Wayland, scale 2, real pointer clicks and Ctrl+click) on a fictional Downloads folder, grouped and ungrouped, in light and dark. All four states, an unfocused list, and the folder left after the navigation pane, Up and a typed address all matched the list above.
- The change was also tried in a full build on KDE Plasma 6 (Wayland): the fill always shows, and the outline shows only while the list has the keyboard.
- `native/tools/check.py` and `native/parity/check.py` pass. After merging current main, rustfmt, clippy and the look, stylesheet, focus, selection, pointer and history tests pass.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
