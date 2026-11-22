# Title

fix(menus): Match Windows 11's right-click menus for files, pages, the Recycle Bin, ZIPs and the navigation pane

# Body

Model: claude-opus-5-5
CLA: I agree

A pass over every right-click menu of the app, in both styles, with the real pointer and keyboard.

Fixes:

- A crash (SIGSEGV in GTK 4.14's gdk_surface_request_motion during a window layout) after closing a place, Home item, crumb, address bar, Details options, group heading, status bar or Back/Forward history menu. Those menus were unparented once they closed, and a pointer event that reached the hidden popup first left GTK asking its gone surface for the pointer. They are now kept by their owner and shown again, as the file list's menu already was.
- A file's compact menu now lists Share with before Add to Favorites, as Windows 11's does.
- A search result or a Recent row lists one "Open file location" in the compact menu, after Share with and before Add to Favorites, as Windows 11's Gallery lists it. It listed three ("in new tab" and "in new window" too); those two stay in the classic menu, right after Open file location.
- Network's and Gallery's blank space no longer offer a working-looking Open in Terminal: Network's menu leaves it out, as Windows 11's does, and Gallery's grays it.
- The navigation pane's Home and Gallery rows offered only Hide. They now offer Open in new tab and Open in new window, as This PC and Network do.
- The classic menu listed "Configure service actions…" and "Open as administrator…" after Properties. They now sit in a group of their own just above Properties, which ends Windows' classic menu.

Checked against Windows 11 build 26200:

- The Recycle Bin's compact menus: an item has Cut and Delete in the strip, then Share with, Restore, Compress to and Properties, then Show more options. Share with and Compress to are grayed with a reason, since Compress to did nothing there. Blank space has View, Sort by, Group by and Refresh, then Empty Recycle Bin, Undo while it can run and Properties, then Show more options.
- Inside a ZIP, an item has the short classic menu in either style, with Open in bold: Cut, Copy, Copy as path, Delete, Extract all… and Properties, with the ones the app can't do in a read-only ZIP grayed. Blank space has View, Sort by, Group by and Refresh, then Extract all… and Undo, then Show more options.
- This PC's and Home's blank space opened no menu at all. This PC now has View, Sort by and Group by, then Add a network location, Undo and Redo while they can run, and Properties where a system information app is installed. Home has View, with Sort by and Group by grayed, then Undo and Redo while they can run. Network's has View, Sort by, Group by and Refresh. Each compact menu ends with Show more options.
- The navigation pane's Home offers Pin current folder to Quick access, which pins the folder shown.
- A network computer offers Pin to Quick access in the navigation pane. On the Network page its compact menu is Open, Open in new tab, Open in new window and Pin to Quick access, without the strip or the file commands. Its "Open in new window" row in the navigation pane had the share glyph; it now has the new-window glyph.
- A submenu the pointer opened stays open when the pointer leaves the menus, as it does in Windows 11.

Not done: Gallery's Pin to Quick access (the app cannot pin a page) and a network computer's Create shortcut.

Tests:

- New real-pointer tests (XTEST on check.py's private display): leaving every menu keeps an open submenu; the Recycle Bin's, a ZIP's and the pages' menus; Home pins the current folder and a network computer pins itself from the navigation pane; Home and Gallery open in a new tab or window from the navigation pane; the first right-click after a menu closed, after a menu row was chosen, and on a Ctrl+clicked selection opens the next menu.
- A sweep that opens every right-click menu with the real pointer in both styles (one file, several files, a folder, a mixed selection, MP3, MP4, PDF, JPEG, ODT, ZIP, 7z, odd names, a 5 GB sparse image, folder background, Home tiles, Gallery, This PC and a drive, Network, the Recycle Bin and an item in it, inside a ZIP, search results, every navigation pane row, the address bar, the column titles, a tab) and checks that every row and strip button, submenus included, runs an action the window has with a target of the type that action takes. A menu ignores a failed activation, so such a row would silently do nothing. None was found.
- `Fixture::real_world()` in the test harness: a folder of real-world file types for the menu tests.
