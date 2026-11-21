# Title

fix(menus): Match Windows 11's right-click menus in search results, the navigation pane and the classic menu

# Body

Model: claude-opus-5-5
CLA: I agree

A pass over every right-click menu of the app, in both styles, with the real pointer and keyboard.

Fixes:

- A file's compact menu now lists Share with before Add to Favorites, as Windows 11's does.
- A search result or a Recent row lists one "Open file location" in the compact menu, after Share with and before Add to Favorites, as Windows 11's Gallery lists it. It listed three ("in new tab" and "in new window" too); those two stay in the classic menu, right after Open file location.
- In the compact menu, a submenu the pointer opened (Open with, Compress to, View, Sort by, Group by, New) now closes once the pointer has stayed off every menu for the 400 ms hover delay. It stayed open until a click. Coming back onto the menu or the submenu within the delay keeps it. The classic menu keeps its submenus open, as Windows' classic menus do, and so does a submenu the keyboard opened.
- The navigation pane's Home and Gallery rows offered only Hide. They now offer Open in new tab and Open in new window, as This PC and Network do.
- The classic menu listed "Configure service actions…" and "Open as administrator…" after Properties. They now sit in a group of their own just above Properties, which ends Windows' classic menu.
- The folder menu offered Open in Terminal on Network's and Gallery's blank space, where no terminal can open (it then only said so in the status line). It is now grayed there, as the navigation pane's menus already gray it.

Tests:

- New real-pointer tests (XTEST on check.py's private display): leaving every menu closes the compact menu's submenu; Home and Gallery open in a new tab or window from the navigation pane; the first right-click after a menu closed, after a menu row was chosen, and on a Ctrl+clicked selection opens the next menu.
- A sweep that opens every right-click menu with the real pointer in both styles (one file, several files, a folder, a mixed selection, MP3, MP4, PDF, JPEG, ODT, ZIP, 7z, odd names, a 5 GB sparse image, folder background, Home tiles, Gallery, This PC and a drive, Network, the Recycle Bin and an item in it, inside a ZIP, search results, every navigation pane row, the address bar, the column titles, a tab) and checks that every row and strip button, submenus included, runs an action the window has with a target of the type that action takes. A menu ignores a failed activation, so such a row would silently do nothing. None was found.
- `Fixture::real_world()` in the test harness: a folder of real-world file types for the menu tests.
