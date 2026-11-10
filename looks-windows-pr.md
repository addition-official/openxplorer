# Title

feat(ui): Match Windows 11's tab menu, tab icons, Properties dates and navigation pane

# Body

Model: claude-opus-5-5

Four places where OpenXplorer looked different from Windows 11 File Explorer now look like Windows by default. Nothing is removed: the app's own extras are one switch away in Settings → Appearance → Layout, off by default.

- A tab's right-click menu is Windows': Close tab, Close other tabs, Close tabs to the right (new) and Duplicate tab.
- The This PC and Recycle Bin tabs show their own icons instead of a yellow folder, and the Recent tabs show the icon of their navigation pane row.
- In Properties, Created and Modified always start with the weekday, also for today and yesterday.
- The navigation pane is Windows': Home, the pinned folders, This PC with its drives, and Network.

## What changed

| Change | Before | After | Windows 11 |
|---|---|---|---|
| Tab menu | Move tab to new window, Move tab to window…, Duplicate tab, Open windows…, Close tab, Close other tabs | Close tab (X icon, Ctrl+W), Close other tabs, Close tabs to the right, Duplicate tab | Close tab (X icon, Ctrl+W), Close other tabs, Close tabs to the right, Duplicate tab |
| Close tabs to the right | Not offered | Closes every tab after the right-clicked one; grayed out on the last tab | Offered |
| This PC tab icon | Yellow folder | This PC's laptop icon | This PC icon |
| Recycle Bin tab icon | Yellow folder | Recycle Bin icon | Recycle Bin icon |
| Recent files / Recent locations tab icon | Yellow folder | The icon of their navigation pane row | No such tab |
| Address bar icon for Recycle Bin, Recent files, Recent locations | Yellow folder | The same icon as their tab and pane row, as This PC already had | Not compared |
| Properties Created and Modified | "Today, November 8, 2026, 3:56:41 AM" | "Monday, November 9, 2026, 4:02:34 PM" (24-hour clock and locale order kept) | "Monday, November 9, 2026, 4:02:34 PM" |
| Properties Accessed | "Today, …" | Unchanged | Not settled yet |
| Navigation pane | Also a usage bar under each drive, and Recent files, Recent locations and Recycle Bin rows | Home, pinned folders, This PC with its drives, Network | Home, Gallery, pinned folders, This PC, Network |
| Typing "Recycle Bin", "Recent" or "Recent locations" (or trash:///, recent:///) in the address bar | Did nothing | Opens that place; a folder of that name in the current folder still comes first | Not compared |

OpenXplorer has no Gallery, so none is listed.

## Settings that keep the extras

Settings → Appearance → Layout, both off by default and saved only when turned on:

- **Show more items in the tab menu** adds Move tab to new window, Move tab to window… and Open windows… after a separator at the end of the tab menu.
- **Show more items in the navigation pane** brings back the pane exactly as it was: the usage bar under each drive, and Recent files, Recent locations and the Recycle Bin after Network.

With the rows hidden, the Recycle Bin, Recent files and Recent locations still open from the address bar (typed by name or address), and the Recycle Bin also from the address bar's first-crumb menu.

Folder Options → General → Privacy's "Show recently used files" and "Show frequently used folders" choose whether those Recent rows show. While "Show more items in the navigation pane" is off they are grayed out, with a tooltip saying they apply when that setting is on in Settings → Appearance; Folder Options does not turn the setting on. With it on they work as before.

While a tab's menu is open, Ctrl+W (shown beside Close tab) closes that menu's tab, which can be a background tab, not the tab in front.

## Changed owner decisions

Each of these replaces an earlier choice with Windows' default; where it was a feature, the earlier behavior stays behind a setting.

- TAB-012 (tab context menu): the Python app's menu becomes Windows'; its three extra items are behind "Show more items in the tab menu" (TAB-060).
- TAB-015 (Close other tabs): now the second item, without an icon.
- TAB-010 (tab icons): This PC and the Recycle Bin had the folder icon on purpose; they now show their own icons. This is only an icon, so there is no setting for the old one.
- SIDE-001, SIDE-018, SIDE-025, SIDE-026 (navigation pane order, drive usage bars, Recycle Bin row, Recent rows): hidden by default, all behind "Show more items in the navigation pane" (SIDE-034).
- PROP-003 (Properties dates): Created and Modified no longer say "Today" or "Yesterday" (PROP-038). Only a text format, so there is no setting for the old one; Accessed keeps it.

Do you want Windows' layout as the default, or yours?

## How it was tested

- New tests written first and seen failing on the unchanged code: the tab menu's items, icons, shortcut and grayed-out items; Close tabs to the right; the extra-items setting; tab and address bar icons; the weekday dates for today, yesterday, five days ago and last year (12- and 24-hour); the default navigation pane and the setting that brings back the rows and usage bars; opening the Recycle Bin and Recent from a typed address. They pass now.
- Existing tests that use the Recycle Bin and Recent rows turn the setting on.
- `cargo fmt --check`, Clippy with the workspace lints, `python3 native/parity/check.py` and the translation template check pass. The whole workspace test suite ran on a private Xvfb display with a disposable home: every interface test passes. The remaining failures need what this test machine lacks (Python 3.13's URL parsing for the Python-app fixture, a trusted folder outside /tmp for the administrator-copy checks, user namespaces for the thumbnailer sandbox) and fail the same way on unchanged main.
- The release build was driven with real mouse and keyboard input (xdotool) on a private display with fictional files, at 1x and 2x, light and dark, normal and Compact view: every tab menu item on one tab and on the first, middle and last of three tabs, opened by right-click, Shift+F10 and the Menu key; the extra items with the setting on; tab icons for a folder, Home, This PC, Recycle Bin, Recent and search results; Properties of files from today, yesterday, five days ago and last year, a folder and several items; the navigation pane by default and with the setting on, its arrow keys, a drop onto a pinned folder, and a narrow window.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
