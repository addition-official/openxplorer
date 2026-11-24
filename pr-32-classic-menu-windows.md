# Title

feat(menus): Draw the classic right-click menu as Windows 11's Win32 menu

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/31-windows-mode-switch)

## What changes for the user
- **"Show more options", Shift+right-click and the classic menus of the navigation pane and the landing pages** now look and behave like Windows 11's classic (Win32) menu, measured by script against the Windows reference capture of a file's classic menu (1x, dark).
- **Look:** 24 px rows at 100% text (were 32), as wide as the longest label plus Windows' room for the submenu arrow, with no least width. A 1 px `#a0a0a0` border, square corners, no shadow and no slide-in motion; background `#2c2c2c` in dark. Separators are 1 px, 4 px below the row above and 2 px above the next. The icon sits 10 px and the text 36 px in from the menu's edge, and a submenu's arrow 14 px in from the right edge. Hover fills the whole row (`#414141` dark, `#dadada` light). The light colors follow Windows 10's light menu until a light Windows 11 reference exists.
- **Icons only where Windows draws them:** an application's own icon (as on "Edit in Notepad") and Share; every other row, Properties, Send to and Open with included, is text alone, with the icon column kept so the text lines up. Submenus such as Send to and New keep their icons.
- **Access keys** use Windows' letters (Cu**t**, P**r**operties, ...); a label Windows does not list takes the first free letter. They are underlined when the keyboard opened the menu (Shift+F10, the Menu key, or "Show more options" chosen with the keyboard) and hidden when the pointer opened it, and they work either way; a letter that several rows share moves between them. Up and Down wrap, Right opens a submenu, Left and Esc close one level, and submenus open after Windows' hover delay, as before.
- **A file's classic menu** follows Windows' groups: the open group first (Open, "Edit in" an editor where one is installed, Add to Favorites, and Open with last); Copy as path, Share, Restore previous versions; Send to; Cut, Copy; Create shortcut, Delete, Rename; then the app's own commands, which Windows does not have (Duplicate, a file's "Open containing folder in Terminal", the tools), above Properties. A folder keeps Open in Terminal where Windows lists it.
- **A file with fewer than two applications** shows "Open with…" (the chooser) instead of an "Open with" submenu, as Windows does for a text file.
- **Empty space** follows Windows' groups: View, Sort by, Group by and Refresh; Paste, Undo and Redo; Open in Terminal and the folder's applications; New; the app's own folder tools; Properties.
- **The Recycle Bin:** an item's classic menu reads Restore | Cut | Delete | Properties, and the folder's empty space reads View, Sort by, Group by, Refresh | Empty Recycle Bin | Properties. A drive's or place's Open is in bold, as Open is for a file.
- For developers: the `legacy` look in `window/menu_popover.rs` (`set_legacy_look`, the access-key table and `keyboard_used_last`), `popover.ox-menu.legacy` in `resources/skin/menus.css` with its colors in `dark.css` and `light.css`, and the 24 px row in `theme/fonts.rs`; `native/docs/ui-spec.md` S09 and E03 describe it. New snapshot scene steps `right-click=` and `place-menu=` open menus as a right-click does, for captures. In this series the legacy menu's row icons use the 100% size, where the feature branch called an icon-scaling helper from the icon-growth work this series leaves out.

## Why
- "Show more options" and Shift+right-click opened the app's classic style, with 32 px rows, a shadow, a slide-in and icons on most rows, which did not match the menu Windows 11 shows there; its access keys and item groups differed from Windows' too.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: the menu tests (154 passed; the one failure comes from the test environment, where tests run as root, which ignores a read-only folder); rustfmt; Clippy on the changed code; `native/parity/check.py` and `tools/sync-docs.py --check`; before and after captures of a file, a folder, empty space and a drive (1x, dark and light), measured by script against the Windows reference.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
