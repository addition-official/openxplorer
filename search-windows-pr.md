# Title
feat(search): Size and Last week search options, and kind:, datemodified: and size: typed into the search box

# Body
Model: claude-opus-5-5

## Summary
While a search shows, the search strip already had Kind and Date modified lists. This adds a **Size** list with the size groups Microsoft documents for Windows (Empty, Tiny, Small, Medium, Large, Huge, Gigantic), and **Last week** in Date modified. You can now also type the options into the search box, as Windows' search syntax allows: `kind:picture`, `kind:folder`, `datemodified:today`, `datemodified:last week`, `size:tiny`, `size:>500`. Typed options and the lists' options narrow together. An option typed alone, such as `kind:folder`, lists every such item in the folder and its subfolders, even in a folder of 10,000 files. Other text is searched for as before.

Nothing the owner already decided was changed: the "Search Results in <folder>" title (SRCH-042), the 120 ms typing pause (SRCH-002), the no-results messages (SRCH-013) and the place of the search options (the strip, SRCH-037) stay as they were. Where Windows may differ, it is listed under "Not confirmed" and "Conflicts" below.

The snapshot hook gains three scene steps (`type=`, `option=`, `choose=`) so the repo's capture path can picture a running search and the open option lists.

## Behaviours and their Microsoft sources
| Behaviour | Built? | Microsoft source |
|---|---|---|
| Search options offer date modified, kind and size | Size added; kind and date already there | https://support.microsoft.com/en-us/office/foundations-experiences/find-what-you-need-with-microsoft-search ("Select Search options on the top toolbar [...] You can also select date modified, kind (like document, picture, or video), size, system, or zipped files.") |
| Size groups Empty, Tiny, Small, Medium, Large, Huge, Gigantic, with ranges | Yes: Windows 10 1809+ minValues 0, 1, 16,383, 1,048,577, 134,217,729, 1,073,741,825, 4,294,967,297 | https://learn.microsoft.com/en-us/windows/win32/properties/props-system-size |
| Date modified: Last week | Yes | https://learn.microsoft.com/en-us/windows/win32/lwef/-search-2x-wds-aqsreference ("date:last week", "modified:last week") |
| Typed `datemodified:` / `modified:` with today, yesterday, this week, last week | Yes | Advanced Query Syntax page above ("Date modified \| datemodified or modified", "date:today ... date:yesterday", "date:this week date:last week") |
| Typed `kind:` (folder(s), document/docs, picture/pics, music, video(s)) | Yes | AQS page above (kind:docs, kind:pics, kind:videos, kind:folders, kind:music); https://learn.microsoft.com/en-us/windows/win32/properties/props-system-kind (document, folder, picture, music, video) |
| Typed `size:` groups and `size:>N` / `size:<N` bytes | Yes | AQS page above ("size:>500", "size:<500"); System.Size page above |
| Results update as you type, no Enter needed | Already built (SRCH-002), unchanged | https://blogs.windows.com/windows-insider/2022/09/14/announcing-windows-11-insider-preview-build-25201/ ("File Explorer search will now show results as you type. The full search results page will update live without needing to press enter.") |
| "Search Results in <folder>" title in tab, window and address bar | Already built (SRCH-042), unchanged | No Microsoft page found: not confirmed |
| Empty-results text, progress indicator, typing delay, Escape/clear returning to the folder | Kept as the owner built them | No Microsoft page found: not confirmed |

## Conflicts kept as the owner decided
- Title "Search Results in <folder>" (SRCH-042) kept; no Microsoft page states the wording.
- The search options stay in the search strip (SRCH-037); Windows puts them under Search options in the command bar.
- "No matching items" (SRCH-013) kept; "No items match your search." is not on a Microsoft page.
- 120 ms typing pause (SRCH-002) kept; Microsoft documents no delay.

## Not confirmed, needs a Windows check
Title case and placement; the Search options menu's place and labels; Windows 11's Date modified and Kind values; the Size labels and the 16,383-byte boundary; whether folders pass a size filter; whether menu choices write syntax into the box; size units and other dates in the syntax; typing delay, progress and empty text; Escape, clear and Back from results.

## How it was tested
- Tests first: the new tests in `ox-core/src/search/facets.rs`, `ox-core/src/search/syntax.rs` and `ox-app/src/window/tests/search_options.rs`; the two window tests that compile on main (`a_typed_option_alone_lists_everything_of_it`, `a_typed_option_finds_items_in_a_big_folder`) fail there.
- `python3 native/tools/check.py`: passed (75 test executables plus doctests) with Python 3.12. One rename test (`the_name_fields_own_menu_keeps_the_rename_going`) failed in one of three full runs and passed in the other two.
- `python3 native/parity/check.py`, `cargo fmt --check` and clippy: passed. `tools/audit-public-data.py`: no issues.
- Real input with xdotool under Xvfb, in the capture tools' sandbox with fictional files, at 1x and 2x, light and dark: every Size, Date and Kind value alone, combined filters, typed syntax, keyboard access to each list, Escape, the clear button, Back and Forward, empty results, and a 10,000-file folder. Emptying the box in that folder stalls input for about 0.5 to 0.9 s while the folder lists again; unchanged main does the same (0.71 to 0.96 s).
- Pictures: 44 captures through `tools/native_capture.py` and the snapshot hook (1x/2x, light/dark).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
