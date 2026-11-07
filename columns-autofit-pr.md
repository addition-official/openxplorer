# Title

feat(details): Size all columns to fit with Ctrl+keypad plus and the column titles' menu, as Windows File Explorer does

# Body

Model: claude-opus-5-5

## Summary

In Details, Ctrl+plus on the number pad now fits every shown column to its content, as Windows File Explorer does. The column titles' menu starts with "Size column to fit" and "Size all columns to fit", as Windows 11's does. Ctrl+plus on the main keyboard keeps changing the text size.

## How Windows behaves

Microsoft's "Keyboard shortcuts in Windows" page lists, under File Explorer: "Ctrl+Plus (+)": "Resize all columns to fit text." with the note "This shortcut requires using a numeric keypad." (https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec). In Windows 11, right-clicking a column title opens a menu that starts with "Size column to fit" and "Size all columns to fit", then lists the columns.

## Changes

- `DetailsView::fit_all_columns` fits every shown column the way a double-click on its edge does: the widest of up to 2,000 loaded items, within the column's limits. A fitted column is never narrower than its title, for a double-click on an edge too. Hidden columns keep their widths. The widths are saved like a drag.
- `win.fit-all-columns` is bound to `<Primary>KP_Add` and `win.fit-column` takes a column key. Both act only in Details on a listing page, so in Icons, List, Tiles and Content the key does nothing.
- `KP_Add` is removed from the text-larger keys. Ctrl+plus and Ctrl+= keep making the text larger, and Ctrl+keypad minus still makes it smaller.
- The titles' menu starts with "Size column to fit" (the right-clicked column; unavailable when no title is under the pointer) and "Size all columns to fit", then a divider and the column choices. Windows' trailing "More..." is not added, as the app has no column chooser dialog.
- Fixed: a real right-click on a column title never opened the titles' menu, because its gesture waited for the press in the bubble phase. The gesture now takes the press in the capture phase. The existing test called `show_column_menu` directly, so it did not catch this.
- Fixed: GTK reports a column title's natural width as the column's fixed width, so a title measure never let a column shrink. The room a title needs is now its label and sort indicator plus its padding.
- The Keyboard shortcuts window lists "Size all columns to fit" under View with "Ctrl+Plus (numeric keypad)". It used to hide every keypad key, which would have left this command with no key. It now hides a keypad key only when the same command also has a main-keyboard key.
- Parity: new VIEW-072; VIEW-043 updated. User guide (`apps/web/lib/docs.json`, regenerated Markdown) and `openxplorer.pot`/`zh_CN.po` updated.

Known differences from Windows: GTK draws no header to the right of the last column, so a right-click there after a fit opens the folder's own menu. Fitting keeps the existing column minimums (Name 140, Date 100, Type 80 and Size 70 pixels).

## How it was tested

- New tests in `window/tests/column_fitting.rs`: `ctrl_keypad_plus_is_size_all_columns_to_fit_and_ctrl_plus_keeps_the_text_size`, `ctrl_keypad_plus_fits_every_shown_details_column`, `grouped_details_with_the_details_pane_open_fit_as_well`, `ctrl_keypad_plus_does_nothing_in_the_other_views`, `the_titles_menu_sizes_one_column_or_all_to_fit`, and with real XTEST input `a_real_right_click_on_a_title_opens_the_menu_that_sizes_all_columns` and `the_real_ctrl_keypad_plus_fits_and_the_main_ctrl_plus_resizes_text`. `the_shortcuts_window_lists_every_live_binding_and_searches_them` checks the new row, and that Larger text and Reset text size show no keypad keys. The real right-click test fails without the gesture fix. Also `sizing_all_columns_fits_each_shown_one_and_keeps_its_title_whole` in `column_fit.rs`, which also checks that a widened column shrinks back.
- rustfmt, parity, i18n and sync-docs checks pass. Clippy reports nothing in the changed code. Its Rust 1.97 lints `duration_suboptimal_units` and `map_unwrap_or` fire only in files this change does not touch.
- Related tests were run by name in an isolated Xvfb and D-Bus session: column fitting, column choice and widths, text size, view options, views, view styles, grouping, Explorer keys, accessibility, details, narrow windows, details pane, context menus, pane layout, right-drag, split view, sidebar and the Keyboard shortcuts window, after merging current upstream main. 203 passed. 3 failed as root and fail the same way on the base. Two of those 3 pass as an unprivileged user, and the third needs the Python 1.x checkout. The full `native/tools/check.py` run was not done.
- The built app was run as an unprivileged user under Xvfb with real xdotool pointer and keys, on a fictional folder, in light and dark at scale 1 and 2. All 60 checks passed: Ctrl+keypad plus with long and short names, grouped by Type, with the Details pane open and in a narrow window; both header menu items from a real right-click; Ctrl+plus, Ctrl+= and Ctrl+0 on the main keyboard; and Ctrl+keypad plus in Icons, List, Tiles and Content.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
