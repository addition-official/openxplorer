# Title
feat(search): Show Windows 11's search layout by default, with Size, typed options and faster results

# Body
Model: claude-opus-5-5

## Summary
Searching now looks and works like Windows 11 File Explorer by default. While a search shows there is no filter bar above the results. The command bar shows **Search options**, with All subfolders or Current folder, Date modified, Kind and Size submenus, and File contents, plus **Close search**. The tab reads "notes - Search Results in Documents", the words you searched for are marked in yellow in the names, and the results keep the folder's grouping.

The current filter bar is still there behind a new setting, Settings > Search > **Show the search filter bar** (off by default). With it on, searching looks as it did before.

Also:
- A **Size** option, with the size groups Microsoft documents for Windows (Empty, Tiny, Small, Medium, Large, Huge, Gigantic), and **Last week** in Date modified.
- `kind:`, `datemodified:` (or `modified:`) and `size:` typed in the search box work as in Windows, for example `kind:picture`, `datemodified:last week`, `size:tiny`, `size:>500`. Typed options and the menu's options narrow together. An option typed alone, such as `kind:folder`, lists every such item in the folder and its subfolders, even in a folder of 10,000 files.
- **Faster results.** Results no longer take a second to show in a big folder. Stack samples showed the time going into making row cells: each new cell scanned every registered cell, and its check box set a tooltip that waits on the X server. Both are fixed. A search whose walk finds nothing beyond the folder's own matches now keeps the rows typing already filtered instead of swapping in a copy, so the list doesn't jump. A new keystroke still cancels the running search, and the window never waits on it.

Question for the maintainer: this PR makes Windows 11's search layout (the "Search options" menu and "Close search" button) the default, and keeps the current filter bar behind the new setting "Show the search filter bar". Would you prefer Windows 11's layout as the default, or your filter bar as the default? Either way it is a one-line change to the setting's default.

## Behaviours and their sources
| Behaviour | Source |
|---|---|
| Search options on the toolbar: current folder or all subfolders, date modified, kind, size | Microsoft Support, [Find what you need with Microsoft Search](https://support.microsoft.com/en-us/office/foundations-experiences/find-what-you-need-with-microsoft-search) ("Select Search options on the top toolbar and narrow your search to the all folders from the current one on down or just the current folder. You can also select date modified, kind (like document, picture, or video), size, system, or zipped files.") |
| Search options and Close search in the command bar, the tab "Notes - Search Results in Downloads", matches marked in names, results grouped as the folder | Windows 11 File Explorer, checked on a Windows 11 PC |
| Size groups and their first byte sizes (Windows 10, version 1809 and later) | [System.Size](https://learn.microsoft.com/en-us/windows/win32/properties/props-system-size) |
| Last week; `datemodified:` / `modified:` with today, yesterday, this week, last week; `kind:docs/pics/videos/folders/music`; `size:>500`, `size:<500` | [Advanced Query Syntax](https://learn.microsoft.com/en-us/windows/win32/lwef/-search-2x-wds-aqsreference) |
| `kind:` document, folder, picture, music, video | [System.Kind](https://learn.microsoft.com/en-us/windows/win32/properties/props-system-kind) |
| Results update as you type, no Enter needed (already so; now without the wait) | [Windows Insider build 25201](https://blogs.windows.com/windows-insider/2022/09/14/announcing-windows-11-insider-preview-build-25201/) ("File Explorer search will now show results as you type.") |

Left out of Search options: System files (no system files are kept apart from hidden ones, which follow Show hidden files), Zipped (compressed) folders (no search inside ZIP files) and Change indexed locations (folders are indexed in Settings > Search). The typing pause (120 ms), the no-results text and the progress caption are unchanged.

## How it was tested
- Tests first: the new tests in `ox-core/src/search/facets.rs`, `ox-core/src/search/syntax.rs`, `ox-core/src/settings/preferences.rs`, `ox-app/src/folder_view/cells/marks.rs`, `ox-app/src/settings_page/tests.rs`, `ox-app/src/window/tests/search.rs`, `search_options.rs` and `search_layout.rs` failed before the changes they cover.
- `python3 native/tools/check.py`: passed (75 test executables plus doctests). `python3 native/parity/check.py`, `cargo fmt --check` and clippy: passed. `listing::a_slow_listing_says_loading` failed once in one full run and passed three times on its own.
- Real keyboard and mouse input with xdotool under Xvfb, with fictional files, at 1x and 2x, light and dark:
  - Search options opens by mouse and keyboard, and each item works: All subfolders, Current folder, each date, kind and size, File contents.
  - Close search, Escape and Back return to the folder.
  - The tab and address bar text, the marks in names, and grouped results in Downloads.
  - With the setting on, the window looks as before.
- Time from the last keystroke to the final results, release build:

  | Folder | Before | After |
  |---|---|---|
  | 10,000 files, "item 0999" | 1,688 ms | 475 ms |
  | 10,000 files, `kind:picture` | 762 ms | 280 ms |
  | Small folder with subfolders, "e" | 272 ms | 261 ms |

  The small folder is mostly the 120 ms typing pause.
- In this environment, every command bar menu, Sort included, focuses no row the first time it is opened from the keyboard, so the arrow keys do nothing until it is opened again. The menu code is not changed here.
- Screenshots: taken with `tools/native_capture.py` and the snapshot hook, which gains `search=`, `option=` and `choose=` steps.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
