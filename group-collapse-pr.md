Model: claude-opus-5-5

Follows up [AKolenda/openxplorer#228](https://github.com/AKolenda/openxplorer/pull/228) (Tiles and Content) and builds on [AKolenda/openxplorer#266](https://github.com/AKolenda/openxplorer/pull/266) (tiles-content-fix: Tiles and Content keep a folder's groups), which this branch merges; review after #266.

## Problem and resulting behavior

A grouped folder's headings in Details, Tiles and Content could not be collapsed, and Details drew them with GTK's section headers, which take no keyboard focus. Windows 11 heads each group with a chevron that collapses and expands it.

Now, in Details, Tiles and Content:

- Each heading shows a chevron (down while expanded, right while collapsed), the group's title and a line. Following Windows 11 22H2 and later, it shows no item count.
- The whole heading row is one target. The pointer anywhere on it lights the row in the items' hover colour.
- A click on the chevron collapses or expands the group without selecting.
- A click anywhere else on the row selects every item of the group and gives the heading the keyboard, with a 1px outline. Ctrl adds the group or takes it out; Shift extends the selection to it.
- A double-click collapses or expands the group and leaves the selection and the keyboard where they were.
- The arrow keys stop on headings and select the heading's group. On a heading, Left collapses, Right expands, and Space or Enter selects the group's items.
- A collapsed heading whose items are all selected is filled; an expanded one is outlined only.
- A right-click on a heading selects the group and opens the items' menu, headed by Collapse group (Left) or Expand group (Right), Expand all groups and Collapse all groups. The folder menu's View lists the last two while the view is grouped.
- A collapsed group lists only its heading. Its items keep their selection and Select all still selects them; the arrows, Shift ranges and type-to-select skip them.
- Collapsed groups last while the folder stays shown in its pane. Another folder, another tab or another Group by expands them, as no source was found that Windows saves them.
- A heading row is as tall as an item row in Details (30 px, 24 px compact, at 100%) and one line of title in Tiles and Content (26 px). One 8 px gap comes before every heading, the first one included, outside the hover box. No Microsoft source gives these sizes, so they follow the app's own item rows.
- The chevron changes without animation.
- Not as in Windows, by choice: Ctrl+Numpad- and Ctrl+Numpad+ keep changing the text size. Microsoft documents Ctrl+Numpad+ in File Explorer as resizing the columns to fit.

Implementation:

- Details now lists the folder through the same `GroupedSelection` as Tiles and Content, in one column, with each heading a row of its own. The Name cell draws the chevron and title, and the other columns carry the line on.
- `SlotLayout` lists a collapsed group as its heading alone. `FolderModel::collapsed_groups` holds the titles for the views.
- `window/group_headings.rs` handles the heading clicks, keys, menu and actions.
- The grouped grid keys rest on headings and skip collapsed items.
- When the list takes the keyboard, it starts on an item, not on the first heading.
- Hover, focus and fill are drawn on a heading's cells from the model's own state, not from GTK's row states.

Checked on a Windows 11 VM (File Explorer, Tiles and Details, grouped by Date modified), and matched here:

- The pointer over a heading lights the whole row, with rounded corners, and the whole row is the target.
- A single click selects every item of the group in place of the selection. The heading then shows a 1px outline and no fill; a second click later changes nothing.
- A double-click folds or unfolds the group and changes neither the selection nor the focus.
- A click on the chevron folds or unfolds at once, with no change to the selection or focus.
- With the keyboard on a heading, Left folds and Right unfolds.
- The arrows reach headings, Up from a group's first item and Down from the previous group's last, and moving onto a heading selects its group's items, hidden ones included.
- Down from an expanded heading selects its first item alone; Down from a collapsed heading goes to the next heading.
- A folded heading whose items are selected shows a gray fill and the outline; an expanded one shows the outline only.
- A collapsed group's items stay selected (the status bar counts them).
- A right-click on a heading selects the group and opens the items' menu, headed by Collapse group (shown with "Left") or Expand group, Expand all groups and Collapse all groups.
- Headings show no item count.

Not settled on the VM: whether Ctrl+click on a heading adds its group (here it does, or takes a selected group out) and what Enter does on a heading (here it selects the group's items, as Space does).

Sources: Microsoft Learn, Win32 list view (`LVGROUP` states `LVGS_COLLAPSED`, `LVGS_FOCUSED`, `LVGS_SELECTED`; `LVHT_EX_GROUP_HEADER` and `LVHT_EX_GROUP_COLLAPSE`). Microsoft Support, Keyboard shortcuts in Windows (Ctrl+Plus on the numeric keypad). ElevenForum, "Expand or Collapse Group in File Explorer in Windows 11".

## Validation

- `cargo fmt --all --check`: passes.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passes for this change. The container's newer clippy (Rust 1.97) also flags `duration_suboptimal_units` and `map_unwrap_or` in files this change does not touch; those two lints were allowed for the run.
- Named ox-app tests on Xvfb with a private D-Bus and a temporary home (`folder_view::`, `theme::`, `window::tests::{look,group_by,tiles_and_content,view_styles,views,selection,context_menus}`, the grouped-view capture test, `grid_keys`, `group_headings`): 244 passed, 2 failed. Both failures check file permissions and fail because the container runs as root: `views::listed_links_and_read_only_files_carry_their_emblems` and `context_menus::a_folder_without_write_permission_keeps_new_and_paste_and_shows_the_error`. Afterwards, `selection::a_group_collapses_under_its_heading_and_its_items_are_passed_by` gained two assertions and was re-run alone: passed.
- `python3 native/parity/check.py`, `python3 native/tools/i18n.py check` and `python3 tools/sync-docs.py --check`: pass.
- Native GTK with real input: the app ran on Xvfb (X11) as an unprivileged user and was driven with xdotool at 1x and 2x, on a fictional Downloads folder grouped by date, in Details, Tiles and Content. Each step was captured from the screen and passed:
  - hover on the title and on the line
  - a click on the line and on the title
  - after a click: Left, Right, Down and Up
  - the chevron, and a double-click
  - the arrows onto headings
  - a fold of a selected group, and a selection of a folded group
  - 600 pointer moves over the headings, after which the next key was answered within 0.1 s
- Heading and gap sizes were measured from the screen's pixels at 1x and 2x.
- Not run here: a Wayland session, and the full suite (`python3 native/tools/check.py`). Built packages were tested separately on KDE Plasma (Wayland) at 100%: one outline after a click, the keys, double-click, and the fill of a folded, selected heading all passed; those checks are not part of the commands above.

## Compatibility and release impact

- No persisted data changes; collapsed groups are not saved.
- No new dependencies, desktop integration or license changes.
- Four new translatable strings: Collapse group, Expand group, Collapse all groups, Expand all groups. `openxplorer.pot` is regenerated.
- The user guide (`apps/web/lib/docs.json` and its generated Markdown) describes the headings, and the VIEW-022 and VIEW-071 parity notes are updated.
- Test captures use fictional file names and a temporary folder, as `docs/PRIVACY.md` asks. No captures are committed to the repository.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
