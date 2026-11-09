# Title

feat(navigation): Open the Recycle Bin from the address bar and keep a removed folder's tab as Windows 11 does

# Body

Model: claude-opus-5-5

Three search and address bar behaviors now match Windows 11 File Explorer.

Typing "Recycle Bin" into the address bar and pressing Enter opens the Recycle Bin. Before, the app looked for a folder called "Recycle Bin" inside the current folder and refused the address with "No such file or directory". Case and surrounding spaces are ignored, as they already were for "This PC", which keeps working.

When the folder a tab shows is renamed or deleted by another program, the tab keeps its address instead of jumping to the nearest parent folder. Nothing is said until the user refreshes it: F5, Refresh or Try again then opens "Location is not available" with an error sign, Windows' text with the folder's own path, and an OK button. The tab stays at the old address behind it, with the "This location is unavailable" page. A tab in the background or on the other side of a split view keeps its address too. A refresh while the box is open opens no second box.

The `?` search wildcard already stood for exactly one character in names, in the current folder and its subfolders, in the search cache and with contents search. New tests pin that down with Windows' own example. A Linux file name may contain a literal `?`; a `?` typed in the search is still a wildcard, so it also matches that character, and there is no escape for it. This is noted in the parity notes and the backlog.

| Behavior | Before | After | Windows 11 File Explorer |
|---|---|---|---|
| Search `extra_??.txt` in a folder with `extra_2.txt`, `extra_3.txt`, `extra_30.txt`, `extra_31.txt` | `extra_30.txt` and `extra_31.txt` | Unchanged | `extra_30.txt` and `extra_31.txt` |
| Type "Recycle Bin" in the address bar, press Enter | Refused: "No such file or directory" | Opens the Recycle Bin | Opens the Recycle Bin |
| Type "This PC" in the address bar, press Enter | Opens This PC | Unchanged | Opens This PC |
| The open folder is renamed or deleted by another program | Jumps to the nearest existing parent and says "Current location changed, <path> is no longer accessible." | Keeps the old address and says nothing | Keeps the old address |
| F5 on that folder | (already moved away) | "Location is not available" box with the error sign, "<path> is unavailable. If the location is on this PC, ..." and OK | "Location is not available" box with an error icon, the same text with a drive letter path, and OK |

## Changed owner decisions

- NAV-039 followed Dolphin: a folder removed while shown fell back to its nearest existing parent with a warning. It now keeps its tab and address as Windows 11 does, and a refresh shows Windows' "Location is not available" box. `native/parity/features.toml` and `native/BACKLOG.md` are updated.
- NAV-033: typing "Recycle Bin" (and GNOME's "Trash") now opens the Recycle Bin from the address bar. A folder of that name in the current folder still comes first, as for Home, This PC and Network.

## Known limits

- Behind the box and after OK, the tab shows the "This location is unavailable" page; Windows' list at that point has not been compared.
- Opening the gone folder again with Back, Forward or a typed address shows that page without the box; only a refresh shows the box.
- A folder whose drive is unmounted keeps its address and shows the existing "drive or network share ... was disconnected" page at once, as before; F5 then shows the box.
- The message changed, so its existing zh_CN translation no longer applies.

## How it was tested

- Tests written first: `typing_recycle_bin_or_this_pc_opens_it`, `a_folder_removed_while_shown_keeps_its_tab_until_f5_says_it_is_not_available` and `a_background_tab_whose_folder_is_deleted_keeps_its_address` fail on the previous main and pass now. `a_question_mark_stands_for_exactly_one_character` (ox-core) and `a_question_mark_finds_names_with_one_character_there` pass on both, since `?` already worked.
- The app's listing, address bar, address input, Recycle Bin, search, history, tabs, disconnected, split view, slow locations and loading tests: 190 passed.
- rustfmt, Clippy with the pedantic workspace flags, `python3 native/parity/check.py`, `native/tools/i18n.py check`, `tools/sync-docs.py --check` and `tools/audit-public-data.py` pass.
- `python3 native/tools/check.py` passes its inventory, icon, rustfmt and Clippy steps and then stops at `python_fixture_matches_desktop_core_py`, because this machine's Python parses `smb://[V1.x]` differently. Running every test executable the same isolated way: 4400 passed and 31 failed, and the same 31 fail on the previous main (administrator copy and delete checks that need a trusted folder and administrator mounts, two sandboxed thumbnailer tests, and two Python fixture tests).
- The development build was driven with real keyboard and mouse input on a private X display with fictional files, at scale 1 and 2, light and dark: the search patterns `extra_??.txt`, `extra_?.txt`, `?xtra*`, `*_3?.txt`, `??`, a pattern with no matches and a name with a literal `?`, with and without subfolders and with contents search on and off; "Recycle Bin", "recycle bin", "  Recycle Bin  " and "This PC" from Ctrl+L and Alt+D, in a new tab, with the suggestions list open, then Back and Forward; a folder renamed, deleted or unmounted underneath the list, a split view's other side and a background tab, then F5, Back, another typed address, and closing the box with Enter, Escape and OK. No jump to a parent, no second box, no crash.
- No SMB share was used.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
