# Title

fix(native): Fix a crash, a silent recursive change, two freezes and smaller gaps found in the 2.0 audit

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/26-pass-7-views-2)

Reliability first, then Windows look-alikes. Where the current behavior was a clear design choice, it stays the default and the Windows behavior is a setting.

## What changes for the user
- **No crash with `GTK_A11Y=none`.** Starting or ending a file drag, typing `:` while renaming in place, and the status bar's type-to-select hint no longer crash the app. All of them go through one announcement helper, which now reads nothing aloud when `GTK_A11Y` is `none`.
- **Read-only on a folder asks first.** Ticking or clearing Read-only on a folder (or a selection with a folder) and pressing OK or Apply now shows "Confirm Attribute Changes" in Windows 11's words ("set read-only" or "unset read-only"), with "Apply changes to this folder only" or "Apply changes to this folder, subfolders and files" (selected, as in Windows), then OK and Cancel; a selection gets the "selected items" wording. Cancel changes nothing and keeps the choice to apply. Unlike Windows, "this folder only" stays available: on Linux a folder's own Read-only changes nothing, so this answer is the one way to confirm without touching any file inside. The box's click order (mixed, ticked, clear) is unchanged.
- **A crumb of a huge folder no longer freezes the window.** A crumb's subfolder menu sorts on a worker thread, lists at most 1,000 subfolders followed by "Show all {count} folders" (which opens the folder), and any menu builds its first 100 rows at once and the rest 100 at a time while idle.
- **Drags read their items once.** The label and drop facts are read once per drag instead of cloning and scanning all dragged URIs (up to 100,000) at every motion event.
- **Scroll bars and edge controls keep their presses.** The 4 px inner resize band stands aside over a whole scroll bar (thumb and trough) and over buttons, check boxes, switches, scales, text fields and drop-downs. Corners and the outer band still resize.
- **Restore previous undoes only what Make default changed.** It no longer turns off the Open and Save dialog handling when the user had turned it on separately.
- **Previous Versions no longer piles up SMB lookups.** At most one abandoned exchange runs per server; a new lookup waits up to 2 seconds for it, then reports that the server did not list its versions.
- **"Restore over the original (Windows)"**, a new setting in Settings > Files & folders, off by default, so Restore still asks for another folder. On, Restore copies the version back into the item's folder and asks Replace or Skip Files like a paste. The Interface guide says so.
- **Folder infotips** add the folder's size, "Folders:" and "Files:" (the first seven names, then "...", as Windows 11 lists them), or "Empty folder". Contents are read on a worker thread, only for local folders, and a size that takes over a second is left out.
- **Map network drive** has a Browse... button beside Folder, which opens "Browse For Folder".
- **A drive's label can be changed** on its General tab in Properties, for a local or removable drive; OK or Apply sets it through UDisks (polkit may ask). Shares and discs stay read-only.
- **Search words starting at the extension's dot** follow the same rule as without the dot: `.doc` no longer finds `report.docx`, just as `doc` does not; `.docx` and `docx` still do.
- **"Match words at the start of file extensions (like Windows)"**, a new switch in Settings > Search, off by default. On, a word may also match from an extension's dot or first character, never inside it, as Windows 11's search finds report.docx for doc, .doc and docx.
- **Network thumbnails go back off once** where 2.0.18 saved `remotePreviews: true` as its default, unless the user turned them on since (marked with `remotePreviewsChosen`).

## Why
- With `GTK_A11Y=none`, GTK 4.14 gives widgets no accessibility context, and the binding wrapped the missing context as a real one, so announcements crashed.
- Read-only on a folder changed every file in the tree without asking.
- The crumb menu sorted every subfolder and built one row per subfolder on the main thread.
- Every drag-motion event rescanned the whole drag.
- The resize band took presses meant for scroll bar troughs and controls at the window's edge.
- Restore previous turned off a setting it had not turned on.
- A Previous Versions exchange with a server that stopped answering kept waiting on its own thread, and every reopen started another.
- Windows restores a version over the original, shows a folder's contents in its infotip, offers Browse in Map network drive, lets a drive's label be edited, and matches words at an extension's start; the app's own choices stay the defaults where they were deliberate.
- 2.0.18 had stored network thumbnails on as a short-lived default.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Confirm Attribute Changes"
- "You have chosen to make the following attribute changes:"
- "set read-only"
- "unset read-only"
- "Do you want to apply this change to this folder only, or do you want to apply it to all subfolders and files as well?"
- "Apply changes to this folder only"
- "Apply changes to this folder, subfolders and files"
- "Do you want to apply this change to the selected items only, or do you want to apply it to all subfolders and files as well?"
- "Apply changes to the selected items only"
- "Apply changes to the selected items, subfolders and files"
- "Show all {count} folders"
- "Restore over the original (Windows)"
- "Previous Versions' Restore copies the version back over the item, asking before it replaces anything, as in Windows. Off: Restore asks where to put a copy, and the item is never touched."
- "Copy this version back over the item; you are asked before anything is replaced."
- "Empty folder"
- "Folders: {names}"
- "Files: {names}"
- "Browse For Folder"
- "Match words at the start of file extensions (like Windows)"
- "A word also finds names whose extension starts with it, as in Windows: “doc” finds “report.docx”. Off: a word finds an extension only when it is the whole extension."
- Search keywords for the new rows: "previous versions restore snapshot shadow copy overwrite original replace windows", "search word extension suffix type docx pdf partial match windows file name"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: new or updated tests for every fix (named in `native/parity/features.toml`), including a test that runs the test program again with `GTK_A11Y=none` and fails without the fix; `python3 native/parity/check.py`; `python3 native/tools/check.py` as an unprivileged user with Rust 1.92.0 (inventories, driver tests, drawn-icon guard, rustfmt, workspace Clippy with `-D warnings` and the message template check passed); all 88 test programs and the doctests with check.py's own isolation helpers, carrying on past failures: 4,971 passed and 1 ignored, the ox-app program in full; failures in files the branch does not change: `location_fixture_drift::python_fixture_matches_desktop_core_py`, `network_python::fuse_export_paths_match_native_opening_py` and nine `admin_copy` tests; `window::tests::search_history::a_tab_on_results_keeps_its_selection_and_scroll_position` timed out intermittently, as on main; real input with xdotool on a private Xvfb display with a throwaway HOME and generated files, with `GTK_A11Y=none` and with it unset, two runs each: file drag, `:` while renaming, type-to-select, an MP3 playing in the Preview pane then renamed and deleted, an MP4 selected then renamed and deleted (the app stayed up after every step). The Read-only dialog wording, the folder infotip format and Windows' search matches for doc, .doc, docx, pd, txt and mp were checked in a Windows 11 VM; the wording for a selection of several items was not.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
