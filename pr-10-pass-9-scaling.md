# Title

fix(layout): Keep commands, Settings and thumbnails usable in narrow windows, at large text and on high scales

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/09-pass-8-open-with)

## What changes for the user
- **Commands that do not fit move into See more.** As in Windows 11, the commands that do not fit leave the command bar from the right and are listed at the top of "...", with Sort and View as submenus; New and "..." always stay. The commands a compact window hides (Cut, Rename, Share) are listed there too, and Tab skips the commands that moved. See more is rebuilt on opening only while it lists moved commands.
- **Settings stays within a narrow window at large text sizes.** A Settings page now scrolls sideways before it widens the window, a row's controls move onto further lines when they do not fit on one, the folder fields narrow to 12 characters, and a long choice ends in "...". At 150 % text every page fits beside the categories; at 200 % only the rows whose button labels are longer than the room still scroll.
- **Large and Extra large icons get a sharp thumbnail.** A cached thumbnail now counts only when it is at least the size asked for, and Extra large icons at 150 % or 200 % use the standard's `x-large` and `xx-large` sizes (asked of the desktop's thumbnailer service only when it lists them).
- **The navigation pane narrows with the window** so the files keep 300 pixels, and widens back to the chosen width as the window widens. Nothing is saved.
- **A Gallery tile keeps its sharp picture.** A picture decoded for the tile is never replaced by a thumbnail that arrives later.
- **The progress and File Conflict windows follow the text size**: the progress header, its percentage and the speed graph's labels, and the File Conflict window's heading and names.
- The interface guide describes the narrow-window behavior.

## Why
- In a narrow window, at a large text size, in the Recycle Bin or during a search, the last commands were clipped, "..." among them, so its menu (Select all, Properties, Options and the rest) could not be reached.
- At 200 % text in a window at its 670-pixel least width, opening Settings grew the window to 1363 x 697 pixels, because no Settings page could narrow.
- A picture first shown in Details got a 128-pixel thumbnail that was then used for every view, so Extra large icons (256 pixels), and Large icons on a 200 % screen, drew it stretched.
- The navigation pane kept its width as the window narrowed; at 200 % text its 420-pixel default left the files 250 pixels of a 670-pixel window.
- With larger thumbnails, a 256-pixel thumbnail could arrive after the tile's exact-size picture and replace it.
- At 200 % text, parts of the progress and conflict windows stayed at their 100 % pixel sizes.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix with a test seen failing before it: `window::tests::narrow_windows::commands_that_do_not_fit_move_into_see_more`, `a_compact_windows_hidden_commands_are_in_see_more`, `window::command_bar::overflow::tests::commands_leave_from_the_right_and_separators_follow_them`, `narrow_windows::settings_at_the_largest_text_size_fit_the_least_window_width`, `settings_pages_at_150_percent_text_need_no_sideways_scrolling`, `settings_page::wrap_layout::tests::controls_move_onto_another_line_only_when_they_do_not_fit`, `thumbnails::tests::a_larger_view_does_not_stretch_a_smaller_thumbnail` and the ox-core flavor tests, `narrow_windows::the_navigation_pane_narrows_with_the_window_and_widens_back`, `theme::fonts::tests::the_progress_and_conflict_windows_follow_the_text_size`, `folder_view::gallery_view::tests::a_late_thumbnail_never_replaces_the_sharp_picture`, and `window::tests::gallery_view::small_print_is_drawn_as_sharp_as_the_original_shrunk_to_the_tile` (some of these were later removed as redundant and are not in this branch); real input with xdotool under a private Xvfb with Openbox as an unprivileged user with a throwaway home (See more at 200 % text, the Recycle Bin at 800 and 1000 pixels, Settings at 670 x 700, a progress window at 200 %, Extra large thumbnails at 1x and 2x, maximize and restore, menus at the screen's corner); `native/tools/check.py` with Rust 1.92 through the stages before the test executables (Clippy reported three findings in `window/context_menu.rs` and `window/tests/menu_sweep.rs`), and every test executable in check.py's isolation, with every failure also failing on unchanged main by name; `native/parity/check.py`. Mixed scales on headless Mutter were checked for crashes only.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
