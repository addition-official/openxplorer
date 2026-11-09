# Title

feat(home): Show Quick access, Recent and Favorites on Home, as Windows 11 does, behind an option

# Body

Model: claude-opus-5-5

## Summary

Home in OpenXplorer opens the home folder, like any other folder. Windows 11 File Explorer's Home instead shows Quick access (the pinned folders), a Recent list of recently opened files and a Favorites list of files the user marked. This change adds that Home page, off by default:

- **Settings > General > Start up > "Show Quick access, Recent and Favorites on Home"** (saved as `homeSections`, stored only when on). With it off, nothing changes: Home, Alt+Home, new tabs and new windows still open the home folder (NAV-038).
- With it on, the navigation pane's Home row, Alt+Home, the HomePage key, a new tab, a new window without a startup folder and a typed `home:` open the Home page (`ox:home`).
- **Quick access** shows the navigation pane's own Quick access list (one list, so pinning or unpinning anywhere updates both) as tiles. A tile opens its folder (double-click or Enter, middle-click for a background tab) and has the pin's usual right-click menu, Unpin from Quick access included.
- A **Recent / Favorites** switcher sits below it:
  - **Recent** lists the files GVfs already lists at `recent:///`, the same source as the navigation pane's Recent files, newest first. It updates as the desktop's list changes. A row opens the real file, and its menu has Open file location.
  - **Favorites** lists the files added with the new **Add to Favorites** command. The command is on a file's right-click menu (only while the option is on) and on Recent rows; **Remove from Favorites** removes a file. Favorites are saved in the settings file (`favorites`).
- Both sections **fold** with their chevron, by click or with Left and Right, and stay folded (`homeFolded`). The arrow keys move between headings, tabs, tiles and rows, with a visible focus ring. Compact view draws the tiles and rows closer.
- Nothing is sent anywhere. Recent shows only what GVfs already lists, and Favorites is a list in the settings file.

## Behaviors and their Microsoft sources

| Behavior | Microsoft source |
|---|---|
| Home's sections are Quick access, Favorites and Recent | https://blogs.windows.com/windows-insider/2023/06/07/announcing-windows-11-insider-preview-build-23475/ |
| Quick access on Home and in the navigation pane is one list; "Pin to Quick access" / "Unpin from Quick access" | https://support.microsoft.com/en-us/windows/file-explorer-in-windows-ef370130-1cca-9dc5-e0df-2f7416fe1cb1 |
| Favorites holds files: pinning was extended to files, and pinned files became Favorites | https://blogs.windows.com/windows-insider/2022/02/16/announcing-windows-11-insider-preview-build-22557/ and https://blogs.windows.com/windows-insider/2022/04/06/announcing-windows-11-insider-preview-build-22593/ |
| A Recent / Favorites / Shared switcher on Home (Shared is left out: it needs a Microsoft account) | https://blogs.windows.com/windows-insider/2024/07/12/announcing-windows-11-insider-preview-build-22635-3930-beta-channel/ |
| Open file location on Home's files | https://blogs.windows.com/windows-insider/2025/10/21/releasing-windows-11-builds-26100-7015-and-26200-7015-to-the-release-preview-channel/ |
| Sections fold and Home remembers which are folded; middle-click on a Home folder opens a new tab | https://blogs.windows.com/windows-insider/2026/08/17/improving-file-explorer-context-menu-faster-simpler-and-more-customizable/ |
| "Open File Explorer to: Home" (here: a new window with no startup folder opens Home while the option is on) | https://support.microsoft.com/en-us/windows/file-explorer-in-windows-ef370130-1cca-9dc5-e0df-2f7416fe1cb1 |

## Not confirmed by Microsoft's documentation (to check in Windows 11)

These are drawn in the app's own way, or left out:

- Layout and sizes: tile size, row height, spacing, whether Home's tiles look like the Tiles view, and the switcher's look.
- The order on screen of Quick access, the switcher and the tabs, and which tab is shown first (Recent here).
- Recent's columns (none here: icon and name only) and how many files it lists (up to 50 here).
- The exact wording "Add to Favorites" and "Remove from Favorites", and where they sit in a file's menu.
- Whether Recent has "Remove from list" (not built).
- Whether frequent folders still appear in Home's Quick access (documented for Windows 10 only; not built).
- Heading counts, empty-state wording (the app's own sentences here) and the chevrons.
- Whether a single click opens a Quick access tile (a double-click opens it here, as on This PC).
- Keyboard order between Home's sections (Tab stops at each tile here, as on This PC).
- Which menu items the Recent and Favorites rows have besides Open file location.

## Owner decisions kept

- NAV-038 (Home opens the real home folder) stays the default. The Windows-style Home page is opt-in.
- Pin to Quick access still takes folders only, so Favorites has its own commands rather than Windows' pinning of files.
- HOME-011 noted that showing recent files on Home would be a new feature for the owner to schedule. It is here, behind the option.

## How it was tested

- New tests in `native/crates/ox-app/src/window/tests/home_page.rs` (7 tests). They were committed before the feature and do not compile on main, since the option and the page do not exist there. They all pass with the change.
- Real input with xdotool on a private Xvfb display, with a throwaway home, fictional files and a mock default app: the Home page with its sections; folding by click and by keyboard; pin from a folder's menu and unpin from a tile's menu; double-click and middle-click on a tile; Recent's double-click opening the real file in the mock app; Open file location; Add to Favorites from a file's menu and Remove from Favorites; the empty states; Back and Forward; F5; the keyboard between sections; a 670 px window.
- Pictures taken with `tools/native_capture.py` (fictional demo home) at 1x and 2x, light and dark, normal and Compact view.
- `cargo fmt --check`, Clippy (CONTRIBUTING.md command), `python3 native/parity/check.py`, `python3 native/tools/i18n.py check` and `python3 native/tools/check.py`; results in the test notes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
