## Classic right-click menu drawn as Windows 11's Win32 menu

"Show more options", Shift+right-click and the classic place menus now look and behave like Windows 11's classic menu, measured by script against `reference/ref-05-menu-file-classic.png` (1x, dark).

### Look
- 24 px rows (was 32), 1 px separators in (62,62,62) inset 5 px, 20 px from the text above and 11 px to the text below, as in Windows.
- 1 px #a0a0a0 border, square corners, no shadow and no slide-in motion; background #2c2c2c in dark.
- Icon 11 px and text 36 to 37 px from the menu's edge; the submenu arrow 14 px from the right edge; the menu is as wide as its longest label plus Windows' 76 px.
- Icons only where Windows draws them: an application's own icon (as Edit in Notepad) and Share; every other row is text alone, with the icon column kept so the text lines up. Submenus keep their icons.
- Hover fills the whole row (#414141 dark, #dadada light). The light colors follow Windows 10's immersive light menu until a light reference exists.

### Behavior
- Access keys use Windows' letters ("Cu&t", "P&roperties", ...). They are underlined when the keyboard opened the menu (Shift+F10, the Menu key, or Show more options chosen with the keyboard) and hidden when the mouse opened it. They work either way; a shared letter moves between its rows.
- Up/Down wrap, Right opens a submenu, Left/Esc close one level, and submenus open after Windows' 400 ms hover delay and stay open while the pointer heads for them.
- As in Windows 11, a submenu closes 400 ms after the pointer leaves every menu, in the compact, classic and command bar menus; the menu it opens from stays open until a click outside or Escape.

### Items
- File: the open group first (Open, Edit in <editor> where installed, Add to Favorites, Open with…), then Copy as path, Share, Restore previous versions; Send to; Cut, Copy; Create shortcut, Delete, Rename; the app's own commands (Duplicate, Open containing folder in Terminal, Compress to…, service actions) above Properties.
- A file with fewer than two applications shows "Open with…" instead of an "Open with" submenu, as Windows does for a text file.
- Empty space, the Recycle Bin and a drive follow Windows' groups. The Recycle Bin item menu reads Restore | Cut | Delete | Properties.
- No Windows-only entries are listed (Defender, Media Player, Cast to Device, Pin to Start, OneDrive, Microsoft Store).

### Captures and checks
- Before/now captures for a file, a folder, empty space and a drive (1x, dark and light) are on the status branch under `ours-classic-menu/`, with the measured before/now/Windows table in `status/classic-menu-windows.txt`.
- Menu tests: 154 passed; the 1 failure comes from the test environment (tests run as root, which ignores a read-only folder).
- rustfmt, Clippy on the changed code, `native/parity/check.py` and `tools/sync-docs.py --check` pass.
- New snapshot scene steps `right-click=` and `place-menu=` open menus as a right-click does, for captures.

Model: claude-opus-5-5
CLA: I agree
