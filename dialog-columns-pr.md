# Title

fix(details): Keep a column at its narrowest when its edge is dragged past the column's start

# Body

Model: claude-opus-5-5

## Summary

Dragging a column title's edge past the start of its column no longer makes the column disappear. The column stops at its narrowest width, as it does for any shorter drag.

## What was wrong

When a title's end was dragged past the start of its column, GTK set the column's fixed width to 0. This is easy to do in a Save dialog, where the list starts near the window's left edge. The width handler applied the column limits only to widths above 0, so Name folded to nothing: the next title covered "Name", the rows showed dates where the names were, and no width was saved. This happened in the main window and in the dialogs.

## Change

In `folder_view/details/column_fit.rs`, a width of 0 now gets the column's narrowest width (Name 140 pixels plus its end). Only -1 still means a column without a width of its own.

## How it was tested

- `a_dragged_width_stays_within_the_column_limits` now also sets a width of 0. Without the fix it fails with 0 where 154 is expected.
- rustfmt is clean. Clippy reports nothing in the changed code; its Rust 1.97 `map_unwrap_or` lint fires only in `group_day.rs`, which this change does not touch.
- Related tests were run by name: column fit, column keys, column widths, details, narrow windows, view styles, accessibility, views and the file dialog. 162 passed. One failed as root and fails the same way without this change.
- The built app ran as an unprivileged user under Xvfb with real xdotool drags, before and after the change, in the main window and in the Save dialog opened through the portal backend's `SaveFile` call. Before, Name folded to nothing. After, Name stops at 140 pixels with its names shown, and the width is saved.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
