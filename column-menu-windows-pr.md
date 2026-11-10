# Title

feat(views): Show Windows 11's column titles menu and Choose Details

# Body

Model: claude-opus-5-5

Right-clicking the column titles in Details now opens the same menu as Windows 11 File Explorer, and its new "More..." item opens a "Choose Details" window where columns are shown, hidden, put in order and given a width.

The menu lists "Size Column to Fit", "Size All Columns to Fit", the columns with check marks only, and "More...". Name is listed checked and grayed because it cannot be hidden. The columns follow Windows' order (Name, Date modified, Type, Size, Date created), then the app's own File extension, Owner and Permissions. Windows' Authors, Tags and Title are left out because the app does not read that metadata.

Choose Details has Windows' text and layout: the columns as check boxes (shown ones first, in their order, then the rest alphabetically), Move Up, Move Down, Show and Hide (grayed where they do not apply), the selected column's width in pixels, and OK and Cancel. As in Windows, Name cannot be hidden (its check box is drawn like the others and stays checked) but it can be moved down, and other columns can then sit before it. A Name that is not first is saved among the columns; a Name in front is not, so existing settings keep the same keys. OK applies and saves the columns, their order and widths the same way the menu does, for the folder or for every folder, as Folder Options says. Cancel, Escape and the close button change nothing. The window has a fixed size and works with the keyboard alone: arrows move through the columns, Space checks one, Tab moves through the buttons, Enter is OK and Escape is Cancel. The width field takes digits only and keeps each column within its limits, so 0 gives the narrowest width and a very large number the widest.

Two fixes came with this work:

- The menu could not take the keyboard. It was attached to the titles' row, which takes no focus, and it drew its rows before it was in the window, so every row started grayed. It now belongs to the details view, builds its rows once attached, and opens on the button's release, as Windows and the file list's menu do. Before, the menu opened on the press and lost the keyboard when the button came up. The view keeps one menu and shows it again, as the file list does, instead of destroying it as it closes while the pointer is still over it.
- A right-click on the empty part of the titles' row after the last column now opens the menu too, with "Size Column to Fit" grayed.

| | Before | After | Windows 11 |
|---|---|---|---|
| Fit items | "Size column to fit", "Size all columns to fit" | "Size Column to Fit", "Size All Columns to Fit" | "Size Column to Fit", "Size All Columns to Fit" |
| Name in the menu | Not listed | Checked and grayed | Checked and grayed |
| Column order in the menu | Date modified, Date created, Type, Size, ... | Name, Date modified, Type, Size, Date created, ... | Name, Date modified, Type, Size, Date created, Authors, Tags, Title |
| Icons on the rows | None on the fit items, an icon on each column | Check marks only | Check marks only |
| "More..." | Missing | Opens Choose Details | Opens Choose Details |
| Choose Details | Missing | Check boxes, Move Up/Down, Show/Hide, width, OK/Cancel | Same |
| Name in Choose Details | Missing | Checked, drawn normally, can move down, cannot be hidden | Same |
| Keyboard in the menu | Rows could not take focus | Starts on the first item | Starts on the first item |
| Right-click after the last title | Nothing | Opens the menu | Opens the menu |

## Changed owner decisions

- VIEW-033 left Name out of the titles' menu; it is now listed checked and grayed, as in Windows.
- VIEW-034 kept Name always first; Choose Details can now move it down, as in Windows. A drag still puts Name back in front unless it was moved down this way.
- VIEW-072 noted that the menu does not end with "More..." because the app had no column chooser; the menu now ends with "More...", which opens Choose Details, and the two fit items use Windows' capitals.

## How it was tested

Tests were written first and fail on main: the menu tests fail by assertion, and the Choose Details tests do not compile without the window. All tests run on a private Xvfb display with fictional files.

- New tests for the menu's items, order, check marks and lack of icons; for the Choose Details list order, the grayed buttons, Show, Hide, Move Up and Move Down; for width limits (0, a huge number, letters, an empty field); for OK, Cancel and closing, and reopening; for moving Name down and back; and for saving per folder and in the shared view.
- With real pointer and keyboard input (XTEST): a right-click on every title and on the empty part of the row, a click on every column row (Name changes nothing), a click on "More...", and a keyboard-only run through Choose Details (arrows, Space, Tab, Shift+Tab, Enter, Escape).
- The menu also opens in search results and the Recycle Bin. Recent files was not checked, since the test setup has no recent items.
- `cargo fmt --check`, Clippy with the workspace lints, `python3 native/parity/check.py` and the translation template check pass. Every test binary and the doctests ran to the end: four tests unrelated to this change fail, and fail the same way on main (an address-parsing comparison with the Python app, a FUSE path comparison and two sandboxed thumbnailer tests); `python3 native/tools/check.py` stops at the first of them.
- A new `titles-menu=` step for the snapshot hook opens the menu for pictures.
- Pictures of the menu and the window at 1x and 2x, light and dark, in normal and Compact view.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
