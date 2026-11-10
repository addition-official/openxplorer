# Title

fix(ui): Open the keyboard menu at the focused item, end a rename on No and put duplicated tabs next to the original

# Body

Model: claude-opus-5-5

Three small differences from Windows 11 File Explorer, each compared with Windows 11 (build 26200):

- **Shift+F10 and the Menu key with several items selected** now open the menu at the selected item the keyboard is on, the one it moved to last, instead of at the first selected item. When the keyboard is on an item that is not selected, the menu still opens at the first selected item. An item scrolled out of sight comes into view first and the menu opens once it is drawn there.
- **No in the "Rename" extension question** now ends the rename: the name box closes and the row shows the old name. Escape, the box's close button, N and Alt+N do the same. Y and N now answer the question without Alt too. Yes, Y, Alt+Y and Enter still rename. In Properties, No puts the old name back in the name field without the "The name was not changed." message.
- **Duplicate tab** now puts the copy right after the tab it copies and brings it to the front, with the same folder, view and Back and Forward history. This works from the tab's right-click menu, from Shift+F10 or the Menu key on a tab, and from a double-click on a tab. Right-clicking a tab behind still leaves it behind.

| Case | Before | After | Windows 11 |
| --- | --- | --- | --- |
| Click `extra_2.txt`, Shift+Down three times, Shift+F10 | Menu at `extra_2.txt` (first selected) | Menu just below `extra_31.txt` (focused) | Menu just below `extra_31.txt` |
| Rename `extra_2.txt` to `extra_2.md`, answer No | Name box stays open with `extra_2.md` | Name box closes, row shows `extra_2.txt` | Name box closes, row shows `extra_2.txt` |
| Plain N in the Rename question | Nothing happens | Answers No | Answers No |
| Duplicate tab on the first of three tabs | Copy goes after the active tab (or at the end), with a fresh history | Copy right after the first tab, in front, same history | Copy right after the original, in front |

## Changed owner decisions

These parity entries described the old behavior. They now follow Windows 11:

- CMD-014 (and the CMD-013 wording): the keyboard menu pointed at the first selected item; it now points at the focused selected item.
- OPS-049: after No the in-place field stayed open with the typed name; it now closes with the old name, and plain Y and N answer the question.
- PROP-005: No in Properties showed "The name was not changed."; it now puts the old name back silently, and the other changes on the page still apply.
- TAB-012, TAB-013: Duplicate tab opened where tabs opened from the active tab go, with a fresh history; it now goes right after the original, in front, with its history.
- TAB-014: a double-click on a tab copies it the same way as Duplicate tab.
- TAB-017: Duplicate tab and the double-click ignore "New tabs open", as Windows has no such setting for duplicates.

The docs (Rename, Tabs and the context menu sections of the interface guide) say the same.

## How it was tested

- The new tests were written first and failed on the unchanged main branch for the expected reasons, then passed with the change.
- New tests drive the app on the private test display with its real keyboard and pointer, in the light and the dark theme:
  - Shift+F10 and the Menu key with one item, Shift+Down, Shift+Up, Ctrl+Space selecting and unselecting the focused item, and Ctrl+A, in Details, List, Tiles, Large icons and the compact view; and with the focused row scrolled out of sight in each view.
  - The Rename question answered with No, Escape, the close button, the window's close request, N, Alt+N, Yes, Y, Alt+Y and Enter, each followed by F2.
  - Duplicate tab on the first, middle and last of three tabs from the tab menu, from the keyboard (Shift+F10 on the tab, Down, Enter) and by double-click.
- The same real-input tests also ran with `GDK_SCALE=2`.
- Updated tests for the extension question in place and in Properties, and for the tab menu.
- `python3 native/tools/check.py` (rustfmt, Clippy with the workspace lints, all native tests) and `python3 native/parity/check.py`.
- `python3 tools/sync-docs.py --check` and `python3 tools/audit-public-data.py`.
- Pictures of the menu at the focused row and of the tab strip after duplicating the middle tab were taken with the repository's capture tool at 1x and 2x, light and dark, with fictional files.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
