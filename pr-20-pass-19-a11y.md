# Title

fix(a11y): Make the app heard by screen readers, usable from the keyboard and readable in high contrast

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/19-pass-18-drives)

## What changes for the user
Screen readers (checked through the real accessibility bus and with Orca 46):
- Properties values are heard with their captions ("Type of file: Text Document (.txt)"), also on the drive tabs, key-value grids and "Opens with". They follow later changes such as counted sizes, and the caption beside them is not heard twice.
- Check boxes are no longer named with their access-key underscore ("_Read-only", "_Hidden").
- Menu buttons (New, Sort, View, See more, Places, Settings drop-downs) are buttons that open a menu, collapsed or expanded, each with its own name, as Microsoft documents Narrator reading "New button, collapsed". A Settings drop-down's keyboard stop carries its row's title and the chosen option.
- The status bar's Details view and Large icons buttons are toggle buttons, pressed for the view shown.
- Column titles are buttons that sort, and the sorted one is described "Sorted ascending." or "Sorted descending.". GTK maps no role for column headers, so they used to reach Orca as "filler".
- A Details row is described with the other columns shown ("Date modified: ..., Type: PDF Document, Size: 8 KB"), and an icon-view tile with its infotip (type, size and date).
- The navigation pane is a tree of tree items, with levels where GTK sends them (4.16 and later).
- An empty folder's list is named Items View and described "This folder is empty.".

Keyboard:
- Settings' theme cards form one radio group and one Tab stop; Tab leaves them and the arrow keys move and choose.
- Settings' category list is one Tab stop: Tab goes on to Back to files, Shift+Tab back to the search.
- After a new tab or a typed address, the keyboard moves onto the selected item once the folder is listed (else the first shown), without selecting or scrolling.
- F6 reaches the status bar after the Details pane, as Microsoft lists Windows 11's F6 order.
- New setting, Settings > Appearance > Layout > "F6 moves like Windows", off by default. On, F6 visits only the tabs, navigation pane, file list (each pane of a split tab), Details pane and status bar, and Tab reaches the address bar, search box and command bar. Off, F6 keeps its own cycle through those three too.

High contrast:
- The portal's `org.freedesktop.appearance contrast` and a chosen HighContrast or HighContrastInverse GTK theme now turn the high-contrast rules on, not only GNOME's key, so Flatpak installs and other desktops get them. HighContrastInverse asks for the dark palette.
- With high contrast on, a selected item is ringed 1 px in the text color.

The interface guide describes F6, Shift+F6 and the new setting. CONTRIBUTING.md says what the screen-reader tests need, and the backlog's ACC-003 note is updated.

## Why
GTK 4.14's AT-SPI bridge sends roles, names, descriptions and states and little else (no column-header role, sort order, level, row count or position), so several things GTK's own accessible properties claimed never reached Orca: Properties read "Type of file: Type of file:" and skipped the value, menu buttons were "toggle button not pressed" and, while open, were named after everything in their menu, rows and tiles were heard by name only, the navigation pane was a flat list and an empty folder left the keyboard on an unnamed "list". The theme cards' GtkFlowBox claimed every Tab and the arrow keys, so the theme could not be changed from the keyboard. The category list took Tab as Down, so each Tab opened the next page. After a new tab the keyboard landed on the still-empty list. Only GNOME's key turned high contrast on, and the selection in high contrast was 1.5:1 in dark and 1.2:1 in light, close to the hover fill.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
New:
- "Sorted ascending."
- "Sorted descending."
- "{column}: {value}"
- "F6 moves like Windows"
- "When on, F6 moves only between the tabs, the navigation pane, the file list, the Details pane and the status bar, as in Windows 11, and Tab reaches the address bar, search box and command bar. When off, F6 visits those three too."
- Settings search words for the new setting: "f6 shift+f6 keyboard focus panes regions cycle windows 11 address bar search box command bar tab"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each new test, seen failing before its fix, through `test_support/atspi.rs`, which reads the test process's widgets back over the accessibility bus as Orca does: `screen_reader::the_command_bar_menus_are_heard_as_menu_buttons_with_their_state`, `properties_values_are_heard_with_their_names`, `check_boxes_are_heard_without_their_access_key_underscore`, `the_status_bar_view_buttons_are_heard_pressed_for_the_view_shown`, `the_navigation_pane_is_heard_as_a_tree`, `the_column_titles_are_heard_as_buttons_with_the_sort_order`, `a_details_row_is_heard_with_its_type_size_and_date`, `a_tile_is_heard_with_its_type`, `a_new_tab_or_address_puts_the_keyboard_on_an_item_once_listed`, `an_empty_folder_is_heard_as_empty`; with real keys (XTEST) on the private display, `settings_page::tests::tab_leaves_the_category_list_without_opening_another_category`, `tab_leaves_the_theme_cards_and_the_arrows_still_choose`, `accessibility::real_f6_visits_the_address_bar_search_and_command_bar_as_it_comes` and `real_f6_like_windows_leaves_the_address_bar_search_and_command_bar_to_tab`; `theme::contrast::tests::the_portal_and_a_high_contrast_theme_ask_for_more_contrast` (fake portal), `high_contrast_themes_are_told_by_name`, `theme::system::tests::the_inverse_high_contrast_theme_is_dark`, `window::tests::high_contrast::high_contrast_rings_the_selection_in_the_text_color` (painted pixels, light and dark); `accessibility::f6_moves_focus_round_the_regions_and_back_to_the_same_item` with the status bar; `settings_page::tests::the_f6_like_windows_switch_is_off_and_saves_the_choice`; `focus_regions::tests::f6_goes_on_from_a_region_it_passes_over`; ox-core `the_f6_like_windows_choice_is_off_and_saved_only_when_on`; `window::tests::focus::closing_a_window_with_a_tooltip_pending_logs_no_critical`. Orca ran headless with a speech log, not on a desktop with speech; Narrator was not available. The test executables were run one by one, because `tools/check.py` stopped at Clippy on two files from the drives work with Rust 1.92.

These tests named above are not in this PR, because they were dropped as redundant: `check_boxes_are_heard_without_their_access_key_underscore`, `a_tile_is_heard_with_its_type`, `a_new_tab_or_address_puts_the_keyboard_on_an_item_once_listed`, `an_empty_folder_is_heard_as_empty`, `high_contrast_themes_are_told_by_name` and `high_contrast_rings_the_selection_in_the_text_color`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
