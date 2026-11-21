# Title

fix(layout): Keep commands, Settings and thumbnails usable in narrow windows, at large text and on high scales

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 9 tested the app against Windows 11 at display scales 1x and 2x (X11), 1.25x, 1.5x and mixed 2x + 1x (Mutter, Wayland), the app's text size from 100 % to 200 %, the 670 x 470 minimum window, 3840 x 2160 and 5120 x 1440 windows, maximize and restore, menus and tooltips at the screen's edges, Properties, Settings and the progress window. It fixes what was broken or unlike Windows:

- **"See more" could be cut off.** The commands from New to "…" sat in a scroller without a scroll bar. In a narrow window, at a large text size, in the Recycle Bin (Empty Recycle Bin, Restore) or during a search, the last commands were clipped, "…" among them, so its menu (Select all, Properties, Options and the rest) could not be reached. Now, as in Windows 11, the commands that do not fit leave the bar from the right and are listed at the top of "…", Sort and View as submenus; New and "…" always stay. The commands a compact window hides (Cut, Rename, Share) are listed there too, and Tab skips the commands that moved.
- **Opening Settings could push the window off the screen.** At 200 % text in a window at its 670-pixel least width, opening Settings grew the window to 1363 x 697 pixels: no Settings page could narrow, so its widest row set the window's least width. A page now scrolls sideways before it widens the window, a row's controls move onto further lines when they do not fit on one, the folder fields narrow to 12 characters, and a long choice ends in "…". At 150 % text every page fits beside the categories; at 200 % only the rows whose button labels are longer than the room still scroll.
- **Thumbnails were blurred at Large and Extra large icons.** A picture first shown in Details got a 128-pixel thumbnail, and GIO then reported it for every view, so Extra large icons (256 pixels) drew it stretched, and so did Large icons on a 200 % screen. A cached thumbnail now counts only when it is at least the size asked for, and Extra large icons at 150 % or 200 % use the standard's `x-large` and `xx-large` sizes (asked of the desktop's thumbnailer service only when it lists them).
- **The navigation pane did not give way to the files.** It kept its width as the window narrowed, limited only while dragged; at 200 % text its 420-pixel default left the files 250 pixels of a 670-pixel window. It now narrows so the files keep 300 pixels and widens back to the width chosen as the window widens. Nothing is saved.
- **A late thumbnail could soften a Gallery tile.** With larger thumbnails for larger views, a 256-pixel thumbnail could arrive after the photo decoded for the tile's exact device pixels and replace it, as the tile kept whichever picture had more pixels. A picture decoded for the tile is now never replaced by a thumbnail.
- **The progress window mixed text sizes.** At 200 % text its header, its percentage and the speed graph's labels, and the File Conflict window's heading and names, stayed at their 100 % pixel sizes beside text twice as large. They now follow the text size.

The interface guide describes the narrow-window behavior.

## Tests

Each fix has a test that failed on the code before it (run and seen failing):

- `window::tests::narrow_windows::commands_that_do_not_fit_move_into_see_more` (200 % text, 700 px: "…" shows whole, View is in "…", Tab from Sort goes to "…"), `a_compact_windows_hidden_commands_are_in_see_more`, and `window::command_bar::overflow::tests::commands_leave_from_the_right_and_separators_follow_them`
- `narrow_windows::settings_at_the_largest_text_size_fit_the_least_window_width`, `settings_pages_at_150_percent_text_need_no_sideways_scrolling`, `settings_page::wrap_layout::tests::controls_move_onto_another_line_only_when_they_do_not_fit`
- `thumbnails::tests::a_larger_view_does_not_stretch_a_smaller_thumbnail` (real cache, GIO lookup, 256 and 512 pixels) and the ox-core flavor tests
- `narrow_windows::the_navigation_pane_narrows_with_the_window_and_widens_back`
- `theme::fonts::tests::the_progress_and_conflict_windows_follow_the_text_size`
- `folder_view::gallery_view::tests::a_late_thumbnail_never_replaces_the_sharp_picture`, and `window::tests::gallery_view::small_print_is_drawn_as_sharp_as_the_original_shrunk_to_the_tile`, which failed with the thumbnail change alone

Real input with xdotool under a private Xvfb with Openbox, as an unprivileged user with a throwaway home holding mp3, mp4, pdf, jpg, png, odt, zip, 7z, a 3 GB file and odd names: opening "…" and choosing View › Large icons from it at 200 % text; the Recycle Bin at 800 and 1000 pixels; Settings at 670 x 700 at 100 % and 200 %; a 2-3 GB copy's progress window at 200 %; Extra large thumbnails at 1x and 2x; maximize, restore and double-clicking the title bar; a context menu and an info tip at the screen's bottom-right corner. No "Bail out", CRITICAL, panic or signal from the app.

`native/tools/check.py` with Rust 1.92.0: every stage before the test executables passed on this work (parity tests, parity check, i18n check, tools tests, icon guard, rustfmt, Clippy `-D warnings`); Clippy now reports three findings in that branch's own files (`window/context_menu.rs`, `window/tests/menu_sweep.rs`). It then stops at `display_scale`, whose snapshots start the app, which refuses to run as root in this container; `display_scale` passes when run as an unprivileged user in the same isolation, and every other executable was run in check.py's isolation without stopping. After merging main 0be6bf33: 87 executables, 5,056 passed, 23 failed; every failure also fails on unchanged main by name (root ignoring file permissions, Python fixtures, the bubblewrap thumbnailer tests, `display_scale` and `launch` started as root). `native/parity/check.py` passes.

## Not covered

- Mixed scales were run on Mutter headless (2x + 1x, 1.25x, 1.5x) for crashes only: moving a window between monitors needs input that headless Mutter does not take here.
- The File icons rows of Settings › Appearance are left as they are; at 200 % text in a 670-pixel window that row still scrolls sideways.
- Windows' exact overflow order and the navigation pane's narrowing are from memory, not a Microsoft source; the status file lists steps to confirm them in Windows 11.
