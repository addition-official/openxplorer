# Title

fix(menus): Match Windows 11's right-click menus, Small icons, Properties and status bar

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/05-pass-5-address-bar)

This PR also carries the two commits of the `review4-diffs` branch that pass-6 had merged: "Match Windows 11's Small icons, file menu, Properties and status bar" and "Space Small icons columns 310 pixels apart, as Windows 11 does".

## What changes for the user
Menus (pass 6):
- Closing a place, Home item, crumb, address bar, Details options, group heading, status bar or Back/Forward history menu no longer crashes the app. These menus are now kept by their owner and shown again, as the file list's menu already was.
- A file's compact menu lists Share with before Add to Favorites. A search result or a Recent row lists one "Open file location" in the compact menu, after Share with; "in new tab" and "in new window" stay in the classic menu, right after Open file location.
- Network's blank space leaves out Open in Terminal and Gallery's grays it.
- Home and Gallery in the navigation pane offer Open in new tab and Open in new window; Home also offers Pin current folder to Quick access, which pins the folder shown.
- The classic menu ends with Properties: "Configure service actions…" and "Open as administrator…" move into a group of their own just above it.
- The Recycle Bin's compact menus, the menus inside a ZIP (the short classic menu in either style, with Open in bold and what a read-only ZIP cannot do grayed), and the blank space of This PC, Home and Network now follow Windows 11 (build 26200). This PC's and Home's blank space opened no menu before. In the Recycle Bin, Share with and Compress to are grayed with "Restore the item to compress or share it."
- A network computer offers Pin to Quick access in the navigation pane, and its Open in new window row has the new-window glyph. On the Network page its compact menu is Open, Open in new tab, Open in new window and Pin to Quick access, without the strip or the file commands.
- A submenu the pointer opened stays open when the pointer leaves the menus, as in Windows 11 (a close-on-leave timer added earlier on the branch is reverted).
- The test harness gains `Fixture::real_world()`, a folder of real-world file types for the menu tests.

From `review4-diffs`:
- Small icons (Ctrl+Shift+4) puts each 16 px icon left of its name, in columns filled top to bottom and scrolled sideways, 310 px apart at 100% and growing with the text size; each name fills its column. Up and Down move along the column. (PR 08 changes this to rows filled left to right.)
- Add to Favorites shows on every file's menu, compact and classic, after Open with, and turns into Remove from Favorites once the file is a favorite, whatever Home shows.
- A text file offers "Edit in <default text editor>" with the editor's icon, in its own group after Properties (right after Open in the classic menu). The editor is the default app for the file's type or for `text/plain` when that app is a text editor, else the first registered text editor. The "Open in" Visual Studio Code rows stay; an editor that is both is listed once.
- Properties > General: "Opens with:" shows the app's 16 px icon; Created, Modified and Accessed stay on one line, with a narrower label column and an 8 px gap.
- Properties > Details: under File, Windows' labels and order (Name, Type, File location, Date created, Date modified, Size, Attributes, Owner, Computer), Attributes as "A", "RA", "HA" or "RHA", and a Computer row "<host name> (this PC)".
- The status bar ends a selection with a divider: "47 items | 1 item selected 4 bytes |".

Also in this PR:
- It carries the commit that takes pass-4's computers under Network out of pass-6 in favor of network-like-windows'. In this series PR 04 already left them out, so computers under Network in the pane come with PR 07.
- The last commit drops tests of this change, and of earlier passes, that were found redundant.

## Why
- The crash: GTK 4.14 sends a fake motion to the native under the pointer when a window needs allocating. These menus were unparented in an idle after they closed, and a pointer event that reached a closed menu's hidden popup first left GTK asking a gone surface for the pointer (SIGSEGV in `gdk_surface_request_motion`).
- The other menu changes match the menus seen in Windows 11 build 26200; Network's and Gallery's Open in Terminal looked like it worked where no terminal can open.
- Add to Favorites showed only while "Open Quick access on Home" was on, which is off by default.
- With none of the code editors installed, a .txt whose default app is LibreOffice Writer had no editor row.
- "PM" wrapped onto the next line of a date, a normal file had no Attributes row, and the Details labels and the status bar differed from Windows 11.
- Small icons was a grid of tiles with a tiny icon above each name and wide gaps.

Not done: Gallery's Pin to Quick access (the app cannot pin a page) and a network computer's Create shortcut.

## CHANGED OWNER DECISIONS
From the `review4-diffs` commits:
- HOME-014: old: Add to Favorites was offered, and Favorites changed, only while "Open Quick access on Home" was on. New: it is offered on every file's menu and always saved, as in Windows 11. Tests `a_files_menu_adds_it_to_favorites_while_home_shows_them`, `a_menu_left_open_adds_no_favorite_once_the_option_is_off` and `favorites_change_only_while_home_shows_them` were replaced by `a_files_menu_adds_it_to_favorites` and `favorites_change_whatever_home_shows`.
- OPEN-015: old: only "Open in" a code editor. New: a text file also offers "Edit in" its default text editor.
- PROP-034: old: Name, Item type, Folder path, dates, Size, Owner. New: Name, Type, File location, dates, Size, Attributes, Owner, Computer.
- PROP-003: old: the dates' label column was aligned with the other groups' label column. New: it takes only the room "Created:", "Modified:" and "Accessed:" need.

## New and changed visible text
- New: "Edit in {name}"
- New: "Pin current folder to Quick access"
- New: "Restore the item to compress or share it."
- New: "{name} (this PC)"
- Removed: "Item type"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: pass 6's real-pointer tests (XTEST on `check.py`'s private display) for leaving every menu with an open submenu, the Recycle Bin's, a ZIP's and the pages' menus, Home pinning the current folder, a network computer pinning itself from the navigation pane, Home and Gallery opening in a new tab or window, and the first right-click after a menu closed, after a row was chosen and on a Ctrl+clicked selection; a sweep that opens every right-click menu with the real pointer in both styles over many item types and places and checks that every row and strip button runs an action the window has with a target of the right type (none failed); from `review4-diffs`, each failing before its fix: `window::tests::view_styles::small_icons_put_the_icon_left_of_each_name_down_columns`, `window::tests::selection::arrows_move_down_columns_in_small_icons`, `window::context_menu::entries::tests::a_text_file_offers_edit_in_its_default_text_editor`, `window::tests::context_menus::a_text_file_that_opens_in_a_word_processor_offers_edit_in_the_text_editor`, ox-core `app_catalog::the_default_text_editor_is_a_text_editor_even_when_a_word_processor_opens_text`, `window::tests::home_page::a_files_menu_adds_it_to_favorites`, ox-core `settings::tests::preferences::favorites_change_whatever_home_shows`, `window::tests::properties_width::general_keeps_each_date_on_one_line_and_shows_the_apps_icon`, `properties::general_tab::tests::pm_stays_with_its_time`, `properties::details_rows::tests::the_file_rows_follow_windows_words_and_order`, `properties::details_rows::tests::attributes_read_as_windows_letters` and the extended `window::tests::environment::the_status_bar_counts_this_pc_and_a_folder`; `python3 native/parity/check.py`, `python3 native/tools/i18n.py check`, the parity and tools unit tests, the drawn-icon guard, `cargo fmt --check` and `cargo clippy --workspace --all-targets -D warnings` with Rust 1.92; every test executable in `check.py`'s isolation on the branch and on unchanged main, with the same failures on both (permission and root cases and the Python-source comparisons); after the column-width change, rustfmt, Clippy, `parity/check.py`, the i18n check and the fonts, grid, view styles, selection and views suites; real input with xdotool on a private Xvfb with fictional files, in light and dark. Some of the tests named here are not in this branch, as this PR's last commit and the feature branch dropped them as redundant.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
