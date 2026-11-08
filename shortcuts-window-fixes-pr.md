# Title
fix(shortcuts): Find shortcuts by their keys and keep the list clear of the scroll bar

# Body
Model: claude-opus-5-5

## Summary
Two problems in Settings > About > Keyboard shortcuts:

- **Searching by keys found nothing.** Typing "ctrl shift e", "ctrl+shift+e", "Ctrl+Shift+E" or "shift e" did not find Ctrl+Shift+E, because the search compared the typed text as one string against the row. It now reads the keys however they are typed: case does not matter; spaces, "+" and "-" between keys count the same; a "+", "-" or "*" on its own is that key (so "keypad +" finds Num +); "control", "escape", "delete" and "keypad" stand for Ctrl, Esc, Del and Num; and the modifiers can come in any order. It also finds a shortcut by words of its name and of its section (for example "tabs and windows"). A one-letter word has to be a key, so "shift e" does not also show every Shift shortcut whose name has a word starting with "e". Results update as you type, as before.
- **The scroll bar covered the keys.** The key column ran to the right edge of the list, so the scroll bar was drawn over it: an overlay bar when the pointer is on it, and an always-shown bar (GTK's overlay scrolling off) all the time, which cut the keys off. The list now keeps 12 pixels clear at its end, the width of an overlay bar under the pointer and of an always-shown one, at any scale.

Names and keys also wrap now, so the window keeps its usual 510-pixel width however long they are. Before, at the largest text size, it grew to 978 pixels over a 670-pixel window. 670 pixels is the narrowest the main window gets (its minimum width), so I tested at 670, 800 and 1,200 pixels instead of 480. The change adds a margin, label wrapping and a search matcher. It doesn't touch the focus ring, animations or the window's breakpoints.

The developer snapshot hook gains a `scroll=end` scene step, used for the pictures of the bottom of the list.

## Checklist
| Check | Before (main) | After |
|---|---|---|
| Search "ctrl shift e" shows Ctrl+Shift+E | FAIL | PASS |
| Search "ctrl+shift+e" | FAIL | PASS |
| Search "Ctrl+Shift+E" | FAIL | PASS |
| Search "shift e" | FAIL | PASS |
| Search "control-shift-e", "Shift Ctrl E", "e ctrl shift", "keypad +", "control c", section words | FAIL | PASS (unit test) |
| Search "ctrl shift x" does not show Ctrl+Shift+E or Copy | PASS | PASS |
| Keys at least 12 px clear of the right edge (overlay bar) | FAIL (keys reached the edge) | PASS |
| Keys left of an always-shown bar | FAIL in the pictures (bar over the keys) | PASS |
| Window stays within a 670-pixel window at the largest text size | FAIL (978 px) | PASS |
| Scroll to the end with the mouse wheel; last row shown | PASS | PASS |

## How it was tested
- **Unit tests first.** `the_shortcuts_window_finds_a_shortcut_by_its_keys_however_typed`, `the_scroll_bar_never_covers_the_keys` and `the_shortcuts_window_fits_a_narrow_window` were added first and fail on main, then pass with the fix. The matcher has its own tests (`the_search_reads_keys_however_typed`, `a_query_finds_a_shortcut_by_its_keys_or_words`).
- **Real input.** xdotool on a private Xvfb display with openbox running, run as an unprivileged user, at 1x and 2x, with app-window widths of 670, 800 and 1,200 pixels, with overlay and with always-shown scroll bars: open the window with Ctrl+?, type each of the four query forms, check over the accessibility bus that the Ctrl+Shift+E row is shown, then scroll to the end with the mouse wheel. Main: 72 checks, 48 failed (every search form). This branch: 72 of 72 pass. Label positions read over the accessibility bus did not match the window, so the key column's position is checked by the unit test and the pictures, not by the real-input run.
- **Pictures** from the native capture path (`tools/native_capture.py` and the snapshot hook) with the fictional demo tree, at 1x and 2x, light and dark: the top and the end of the list at each width, the end with always-shown scroll bars, the largest text at 670 pixels, and each search form; plus before pictures from main.
- **Checks** run as an unprivileged user with Rust 1.92. `native/parity/check.py`, the i18n check, rustfmt and Clippy pass. In `native/tools/check.py`, the ox-app library has 2,149 passed and 2 failed: the sandboxed-thumbnailer tests, which also failed on unchanged main in this container (bubblewrap sandbox). check.py stops at the first failing test executable, so I ran the remaining 27 executables and the doctests the same way; all pass. The ox-core tests ran with `OX_TEST_TRUSTED_BASE` set to a folder only its owner can write, as they ask; the Python fixture test ran with Python 3.12 and node on the PATH.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
