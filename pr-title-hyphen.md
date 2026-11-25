# Title

feat(appearance): Title every window with Windows' hyphen

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: test-combined-60 (14a8bf9f), which holds pass-14-system-menu-classic (bc428e0b) and its optional hyphen setting; branch `title-hyphen-default`, one commit (1d01125f).

## What changes for the user
- **This replaces the owner's em dash with Windows' hyphen.** Every window title now separates its parts with a hyphen and spaces, for everyone, with no setting: "Documents — OpenXplorer" becomes "Documents - OpenXplorer", "Settings — OpenXplorer" becomes "Settings - OpenXplorer", and an archive's window "Photos.zip — Compressed folder" becomes "Photos.zip - Compressed folder". The title shows in KDE's and other desktops' title bars, the taskbar, Alt+Tab and the app's own window lists ("Move tab to window", the Windows menu).
- **The optional setting is gone.** Settings > Appearance > Layout no longer has "Separate window titles with a hyphen (like Windows)", and Behave like Windows Explorer no longer lists it. A `hyphenInWindowTitles` key saved by a build that had the setting is ignored when settings load and dropped on the next save; nothing else changes for such a settings file.
- Titles without a dash are unchanged: Properties ("<name> Properties"), the progress window, Open with, Software updates and the other dialogs.
- The interface guide and native/README.md say that titles read "Documents - OpenXplorer", as Windows writes "Documents - File Explorer".

## Why
- KDE's title bar, the taskbar and Alt+Tab show the window title, and Windows 11 writes it with a hyphen: "Documents - File Explorer". The em dash was the one visible place where the title differed from Windows' form.
- The user decided to make the hyphen the plain default rather than an option, and to leave the final choice to the owner in review: if the owner prefers the em dash, this PR can simply be refused, and the optional setting from pass-14-system-menu-classic can stay or go on its own.

## CHANGED OWNER DECISIONS
- The em dash in window titles is the owner's own choice, from the 2.0.0 release (e0286b27) and kept in his later commits (8b702ed6, 26684791). This PR replaces it with Windows' hyphen in every window title.
- LOOK-033 ("Window titles with Windows' hyphen") in `native/parity/features.toml`: before, an optional setting, off by default; now the behavior for everyone, with no setting. TAB-045's native note now says the title is "<active tab title> - OpenXplorer".

## New and changed visible text
- Window titles: "{place} — OpenXplorer" → "{place} - OpenXplorer"; "{name} — Compressed folder" → "{name} - Compressed folder".
- Removed Settings row (Appearance > Layout), title: "Separate window titles with a hyphen (like Windows)"; description: "Titles a window "Documents - OpenXplorer", as Windows writes "Documents - File Explorer", instead of "Documents — OpenXplorer"."; its search keywords "window title caption taskbar alt tab hyphen dash em dash separator windows explorer". The same title is removed from the Behave like Windows Explorer list.
- Interface guide, "Use your desktop's window frame": "Separate window titles with a hyphen (like Windows), just below, titles windows "Documents - OpenXplorer" as Windows writes "Documents - File Explorer", instead of "Documents — OpenXplorer"; Behave like Windows Explorer turns it on." → "Window titles read "Documents - OpenXplorer", with a hyphen, as Windows writes "Documents - File Explorer"."
- `native/po/openxplorer.pot` is regenerated: it drops "{place} — OpenXplorer", "{name} — Compressed folder" and the removed row's three messages. No `.po` file is changed. zh_CN translated the two em-dash titles; until it translates "{name} - Compressed folder", a Chinese archive window reads "Photos.zip - Compressed folder" in English ("{place} - OpenXplorer" has nothing to translate).

## Tests
Run on this branch with Rust 1.92, each test binary through `run_isolated` as an unprivileged user on a private Xvfb: new `the_retired_title_hyphen_setting_is_ignored` (ox-core) and `window_titles_take_windows_hyphen` (replacing `window_titles_take_a_hyphen_like_windows_when_asked`); ox-core `settings` 151 passed (including the Behave like Windows Explorer guard test `every_line_reads_and_writes_the_same_setting` and the backup test); ox-app `title_bar` 14, `settings_page` 134, `window_decorations` 7, `windows_mode` 3, `history` 40, `gallery` 30, `search_layout` 14, `search_history` 7, `home_page` 33, `address_bar` 36, `tab_moves` 27, all passing; `archives` 27 of 30, including the archive window's title, with the 3 others timing out while waiting for a default application, as they do on unchanged test-combined-60 in this container. Also `cargo fmt --all -- --check`, Clippy `-D warnings` on ox-core and ox-app, `tools/i18n.py check`, `tools/sync-docs.py` and `native/parity/check.py`. The full suite was not run, and no live desktop check was made for this change.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
