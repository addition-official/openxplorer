# Title

fix(a11y): Make the app heard, operable from the keyboard and readable in high contrast

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 19 walked the whole app as a screen reader sees it, through the real accessibility bus (pyatspi) and with Orca 46 run headless with a speech log, then did the daily tasks with the keyboard alone and looked at GTK's high-contrast themes, GNOME's high-contrast setting, 200 % text and reduced motion. GTK 4.14's AT-SPI bridge sends roles, names, descriptions and states, and little else (no column-header role, sort order, level, row count or position), so several things GTK's own accessible properties claimed never reached Orca. Fixed:

**Screen readers**

- **Properties values were never read.** Each value was named after its caption alone, so Orca read "Type of file: Type of file:" and skipped the value. Values are now named "Type of file: Text Document (.txt)" (also the drive tabs, key-value grids and "Opens with"), follow later changes such as counted sizes, and the caption beside them is not heard twice.
- **Check boxes were named "_Read-only" and "_Hidden".** GTK names a check box after its label as written, access-key underscore included.
- **Menu buttons** (New, Sort, View, See more, Places, Settings drop-downs) were heard as "toggle button not pressed", and while a menu was open GTK named the button after everything in it ("View Extra large icons Large icons ..."). They are now buttons that open a menu, collapsed or expanded, each with its own name, as Microsoft documents Narrator reading "New button, collapsed". A Settings drop-down's keyboard stop now carries its row's title and the chosen option.
- **The status bar's Details view and Large icons** are toggle buttons, pressed for the view shown.
- **Column titles** reached Orca as "filler" with no sort order (GTK maps no AT-SPI role for column headers, through 4.20). They are buttons that sort, as Narrator reads "Date modified, split button", and the sorted one is described "Sorted ascending." or "Sorted descending.".
- **File rows and tiles** were heard by name only. A Details row is described with the other columns shown ("Date modified: ..., Type: PDF Document, Size: 8 KB"), a tile with its infotip.
- **The navigation pane** was a flat list; it is a tree of tree items, as Narrator reads "Navigation pane tree", with levels where GTK sends them (4.16 and later).
- **An empty folder** left the keyboard on an unnamed list ("list"); it is named Items View and described "This folder is empty.".

**Keyboard**

- **Settings' theme could not be changed from the keyboard.** The theme cards sit in a GtkFlowBox, which claimed every Tab while a card followed (though the next radio declined the keyboard), so Tab never left the chosen card, and took the arrow keys for a cursor no card could hold. The cards now form one radio group and one Tab stop; the arrows move and choose.
- **Settings' category list** took Tab as Down, so every category was a Tab stop and each Tab opened the next page. It is one stop now: Tab goes on to Back to files, Shift+Tab back to the search.
- **After a new tab or a typed address** the keyboard went to the list while it was still empty and stayed on the list itself: no item drew the focus and Orca heard only "list". It now moves onto the selected item if shown, else the first shown, without selecting or scrolling.
- **F6** now reaches the status bar after the Details pane, as Microsoft lists Windows 11's F6 order.

**High contrast**

- Only GNOME's key turned the high-contrast rules on, so a Flatpak install (which cannot read the host's keys), other desktops and a chosen HighContrast or HighContrastInverse GTK theme got the normal skin. The portal's `org.freedesktop.appearance contrast` and the GTK theme chosen before the skin was installed now count too; HighContrastInverse asks for the dark palette.
- With high contrast on, the selection was still #3d3d3d on #202020 (1.5:1) in dark and #cce8ff on white (1.2:1) in light, close to the hover fill. A selected item is now ringed 1px in the text color.

## Tests

`native/crates/ox-app/src/test_support/atspi.rs` reads the test process's own widgets back over the accessibility bus as Orca does, so the tests in `window/tests/screen_reader.rs` check what a screen reader gets. They need at-spi2-core's bus launcher and return early without it; `OX_REQUIRE_ATSPI=1` makes that a failure (CONTRIBUTING.md). Every new test was run and seen failing on the code before its fix, then passing:

- `screen_reader::the_command_bar_menus_are_heard_as_menu_buttons_with_their_state`, `properties_values_are_heard_with_their_names`, `check_boxes_are_heard_without_their_access_key_underscore`, `the_status_bar_view_buttons_are_heard_pressed_for_the_view_shown`, `the_navigation_pane_is_heard_as_a_tree`, `the_column_titles_are_heard_as_buttons_with_the_sort_order`, `a_details_row_is_heard_with_its_type_size_and_date`, `a_tile_is_heard_with_its_type`, `a_new_tab_or_address_puts_the_keyboard_on_an_item_once_listed`, `an_empty_folder_is_heard_as_empty`
- With real keys (XTEST) on the private display: `settings_page::tests::tab_leaves_the_category_list_without_opening_another_category`, `tab_leaves_the_theme_cards_and_the_arrows_still_choose`
- `theme::contrast::tests::the_portal_and_a_high_contrast_theme_ask_for_more_contrast` (fake portal), `high_contrast_themes_are_told_by_name`, `theme::system::tests::the_inverse_high_contrast_theme_is_dark`, `window::tests::high_contrast::high_contrast_rings_the_selection_in_the_text_color` (painted pixels, light and dark)
- `accessibility::f6_moves_focus_round_the_regions_and_back_to_the_same_item` now includes the status bar

The new real-key Settings tests left the test pointer over a category row, so the next test's window showed the navigation pane's Gallery tooltip and `update::tests::browsing_never_checks_and_untrusted_text_stays_text` failed. They now rest the pointer on their window's edge, where nothing has a tooltip. (A first fix moved the pointer to the screen's corner whenever a test dropped it. That motion reached GTK 4.14's tooltip timer while the harness destroyed the windows, and its `gdk_surface_get_device_position` critical, fatal in tests, killed the ox_app binary about halfway through every full run. It is reverted.) `window::tests::focus::closing_a_window_with_a_tooltip_pending_logs_no_critical` closes and hides a window while the real pointer waits on a control's tooltip and checks that no critical is logged.

Merged, no rebase, the lower passes that touch these files (every lower pass and network-like-windows, through pass-12, pass-14, pass-15, pass-18, pass-11, pass-13 and pass-16).

## Not covered

- Orca was run headless with its speech log, not on a desktop with speech; Narrator was not available. The status file lists steps to compare with Windows 11.
- GTK 4.14 sends no level, position or row count to screen readers; the navigation pane's levels reach AT-SPI only from GTK 4.16.
- `tools/check.py` stops at Clippy on two pass-18 files with Rust 1.92; the tests were run per executable instead. Failures that come with the merged passes, and a crash in ox-core's restore tests when they run in parallel (inherited, on pass-18's tip too), are in the status file.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01CQbN3PoioEiAg9HNGsaJ1E
