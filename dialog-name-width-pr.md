# Title

fix(details): Let Name take the room the other columns leave, so a narrow list shows Date modified, Type and Size

# Body

Model: claude-opus-5-5

## Summary

In a narrow file list, Name no longer pushes the other columns out of sight. It now takes the room that Date modified, Type and Size leave, cuts long names with an ellipsis, and stops at 100 pixels. This matches Windows 11's Save and Open dialogs.

## What was wrong

A Save dialog about 980 pixels wide with the Details pane open has a file list about 530 pixels wide. Name took 274 pixels of it, Type was cut off, Size was out of sight, and the list scrolled sideways.

The Name title had `min-width: 235px` in `folder-views.css`. With its padding that is 260 pixels plus the 14-pixel end, kept from the web app's `minmax(260px,1fr)` by the 2.0.0 port. A GTK column is never narrower than its title, so any list narrower than about 690 pixels scrolled sideways. Name has no default width of its own, so the minimum came only from this rule.

## Change

- The Name title's minimum is now 86 pixels plus the 14-pixel end, enough for the icon and the first letters. Name still expands, so it takes exactly the room the other columns leave.
- Date modified, Type and Size keep their widths. Only a list too narrow for 100 pixels of Name scrolls sideways.
- A width the user sets still overrides this. The main window and the dialogs share the stylesheet, so both change.
- The LOOK-022 parity note is updated.

## How it was tested

- `the_name_column_takes_the_room_left_and_keeps_100_pixels` replaces `the_name_column_keeps_260_pixels_and_the_list_scrolls_sideways`. With the Details pane open at 1000 and 990 pixels, Name is the list width minus the other columns, which keep their widths, and nothing scrolls sideways. At 700 pixels Name stops at 100 pixels and the list scrolls. The test fails with the old stylesheet.
- rustfmt is clean and the parity check passes. Clippy reports nothing in the changed code; its Rust 1.97 `map_unwrap_or` lint fires only in `group_day.rs`, which this change does not touch.
- Related tests were run by name: narrow windows, the file dialog, look, geometry, views, details pane, pane layout, grouping, view styles, split view, accessibility, details, column fit, column widths and search. 260 passed. One failed as root and fails the same way without this change; it passes as an unprivileged user.
- The built app ran as an unprivileged user under Xvfb. The Save dialog was opened through the portal backend's `SaveFile` call at 980 by 640, and the Details pane was opened with a real click, on a fictional Downloads folder with a long name. This was done before and after the change, in light and dark, at scale 1 and 2. After the change, all four columns show and there is no horizontal scrollbar.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
