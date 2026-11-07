Model: claude-opus-5-5

# fix(address-bar): Show the selected suggestion as Windows 11 does

## The problem

Typing a path in the address bar, for example `~/OX-Test/`, opens a list of suggestions under it. Pressing Down moves into the list and puts the row's address into the box, but the row itself showed no highlight, so there was no way to see which suggestion was current. The rows were also short and tight compared with Windows.

## What Windows 11 does

Source: a Windows 11 File Explorer screenshot from the user, typing `c:/inetpub` in the address bar.

- The suggestions drop down under the box, with rounded rows about 40 px tall and comfortable padding.
- The current row has a light gray fill (the list's hover and selection color) and a short vertical blue accent bar at its left edge, Windows 11's list selection indicator, as in its navigation pane.
- The box shows the current suggestion's text while you arrow through the list.

For the keys, Windows' autocomplete list (the shell's IAutoComplete drop-down, as ReactOS's `CAutoComplete::OnListUpDown` and `OnEditKeyDown` reimplement it): Down with no row selected selects the first row and Up the last; Down on the last row and Up on the first leave the list with no row selected and put the typed text back, so the next key wraps round; Esc puts the typed text back and closes the list. These were not checked on a Windows machine.

## The change

- Suggestion rows are 40 px at 100% text, with 4 px corners, 4 px in from the flyout's sides, 2 px apart, and the text 16 px in.
- The selected row has the subtle fill and a 3 x 16 accent bar with 2 px corners at its left edge, in the app's accent color. It is the same bar the sidebar's selected row and the Settings categories use.
- The pointer's hover draws the same fill without the bar. A click still opens the row at once.
- Up and Down move as described above, wrapping through the typed text. Enter opens the selected row, as before.
- Esc puts back the typed text before it closes the list. A second Esc still ends editing.
- High contrast adds an accent outline to the selected row, as for the sidebar.
- The fill fades in 83 ms, like the other menu rows. It appears at once when the desktop turns animations off.

The address bar's focus and Tab handling are unchanged.

## Testing

- rustfmt and clippy `-D warnings` (Rust 1.92.0) are clean, and `native/parity/check.py` passes.
- `cargo test -p ox-app address` ran 70 tests. 68 pass. `f4_lists_the_typed_addresses_most_recent_first_and_a_row_goes_there` now also covers Up, both wraps and Esc.
- Two tests fail the same way on unmodified main under `xvfb-run`: `a_long_path_collapses_its_first_crumbs_into_a_button_as_the_width_allows` and `the_keyboard_reaches_the_collapsed_crumbs_through_the_overflow_button`, both with "timed out waiting for the new width".
- The new unit test `up_and_down_wrap_through_the_typed_text` and the `theme::` tests pass. The theme tests include the 40 px row rule.
- Real input under Xvfb (xdotool keys and pointer, run as an unprivileged user on fictional folders), at 1x and 2x, light and dark:
  - typing a partial path
  - Down ×1 and ×2, Up
  - wrapping past the first and the last row
  - Esc
  - hover, and hover together with a selected row
  - Enter and a click
  - row height
  - reduced motion

🤖 Generated with [Claude Code](https://claude.com/claude-code)
