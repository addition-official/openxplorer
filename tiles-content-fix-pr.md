Model: claude-opus-5-5

# fix(views): Keep a folder's groups in Tiles and Content, with Windows 11's spacing

## Summary

Tiles and Content (#228) list a grouped folder without its groups. Windows 11 File Explorer keeps a folder's grouping in both views: each group starts on a row of its own under a heading with the group's title, a chevron and a line to the right edge, as in Details. This was compared side by side with Windows 11 File Explorer (System32, grouped by Date modified, dark mode). This change keeps the groups in Tiles and Content and brings both views' spacing closer to Windows 11.

## Changes

- **Grouping.** GTK's `GtkGridView` has no section headers, so the grid lists the folder's selection model through a new `GroupedSelection` (`folder_view/grid/grouped.rs`). It puts a heading row before each group (the title and count as Details draws them, then a line across the row) and blanks after the group's last item, so the next group starts a row. `SlotLayout` (`folder_view/group_slots.rs`) holds the layout and maps grid slots to listing positions and back. Selection passes through to the folder's own `MultiSelection`, so Details, the status bar and the window's commands see one selection. The cell registry maps grid positions back to listing positions (`PositionMap` in `cells/cell_owners.rs`), and activation and `scroll_to` map them too. Ungrouped folders and the other icon layouts use the selection model directly, as before.
- **Keys.** GTK's own grid keys would stop at a heading, so in grouped Tiles and Content the window handles the arrows, Home, End, Page Up and Page Down, with Shift (extend from the anchor) and Ctrl (move the focus alone), and moves between items across the headings (`window/grid_keys.rs`). Up and Down keep to the column, as elsewhere. Ctrl+Shift+7 and Ctrl+Shift+8 are unchanged.
- **Content.** Rows are 48 px at 100% text size, with 16 px of padding at each side and a 1 px line under each row. Next to a 32 px icon, a file shows its name with its type under it; a folder shows its name alone, as in Windows 11. "Date modified:" and "Size:" are in a column at the right.
- **Tiles.** Cells are 256 px wide (three per 1000 px), with a 48 px icon, 8 px of padding, 8 px between tiles and 6 px between rows.
- **Styling.** Both views draw an item's padding, hover, selection and focus ring on its cell instead of the grid's slot, so heading rows never highlight. The high-contrast focus ring follows.
- **Parity and docs.** The VIEW-071 behavior and note are updated, and the docs regenerated with `tools/sync-docs.py`.

## Differences from Windows 11

- Group headings cannot be collapsed, and they have no chevron. The Details view's headings have neither yet.
- Tiles shows no version or description for programs, which Windows reads from their resources.
- Large icons, Medium icons and List stay ungrouped, without headings.

## Overlap with open PRs

- #253 changes how Tiles and Content write sizes (`ItemLines::of` in `folder_view/cells.rs`), adds a case to `tiles_and_content_show_each_items_type_and_size_beside_its_name` and edits the VIEW-071 `native_note`. This PR edits the same test (a folder's type in Content) and the same note, so expect small textual conflicts there. The two changes do not otherwise interact.
- #252 and #254-#260 touch `native/parity/features.toml` in other entries. #261 does not touch the files changed here.

## Testing

- `cargo fmt`; `cargo clippy -p ox-app --all-targets`: no warnings in the changed files.
- New tests: `folder_view/group_slots.rs` (`each_group_starts_a_row_under_its_heading`, `content_has_a_heading_row_then_one_row_per_item`, `up_and_down_cross_the_headings_and_keep_the_column`, `a_change_covers_the_slots_that_differ`, `equal_titles_in_a_row_make_a_group`), `window/grid_keys.rs` (`grouped_keys_move_between_the_items_across_the_headings`), `window/tests/tiles_and_content.rs` (`tiles_and_content_keep_the_folders_groups_under_headings`) and `window/tests/selection.rs` (`arrows_cross_the_group_headings_in_tiles_and_content`).
- After merging main at 8fea905, the tests were run by name under `xvfb-run` and `dbus-run-session`, with the environment `native/tools/check.py` sets. They cover Tiles and Content, slot layout, keys, sizes, fonts, the icon view and cells, selection, grouping, views, item check boxes, rubber band, renaming, view styles, drag, input, focus, the empty-folder look, the Details pane and the file dialogs. 298 passed and 1 failed. The failure is `views::listed_links_and_read_only_files_carry_their_emblems`, and it fails in Details too: the container runs tests as root, and GIO reports a 0444 file as writable for root (`access::can-write: TRUE`).
- `native/parity/check.py` passes. The full `native/tools/check.py` was not run.
- Pictures of grouped Tiles and Content, in light and dark at 2x, were taken with a fictional folder of mixed files and folders on a private Xvfb display. They are a visual check of the GTK app only, not SMB validation.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
