fix(native): Fix a crash, a silent recursive change, two freezes and smaller gaps found in the 2.0 audit

Model: claude-opus-5-5
CLA: I agree

## Summary

Reliability first, then Windows look-alikes. Where the current behavior was a clear design choice, it stays the default and the Windows behavior is a setting. One commit per fix, on top of main 5d21fd3e.

### Crash, data safety and hangs

- **fix(a11y): Never crash an announcement when GTK_A11Y is none.** With `GTK_A11Y=none`, GTK 4.14 gives widgets no accessibility context and the binding wraps the missing context as a real one, so the app crashed when a file drag started or ended, when `:` was typed while renaming in place, and when the status bar read the type-to-select hint. All of these go through one helper, which now reads nothing aloud when `GTK_A11Y` is `none`. A new test runs the test program again with `GTK_A11Y=none`; it fails without the fix.
- **fix(properties): Ask before Read-only reaches into a folder.** Ticking or clearing Read-only on a folder (or a selection with a folder) changed every file in the tree without asking. OK and Apply now show "Confirm Attribute Changes" in Windows 11's words ("set read-only" or "unset read-only"), with "Apply changes to this folder only" or "Apply changes to this folder, subfolders and files" (selected, as in Windows), then OK and Cancel. Cancel changes nothing and keeps the choice to apply.
  - Windows greys out "this folder only" for Read-only. Here it stays available on purpose: on Linux a folder's own Read-only changes nothing ("Only applies to files in folder"), so this answer is the one way to confirm without touching any file inside. Greying it out would leave recursion or Cancel as the only choices, and it was kept for data safety.
  - Not changed: the box's click order. Windows cycles mixed → clear → ticked; the current order (mixed → ticked → clear) is left as is.
- **fix(address): Never freeze the window on a crumb of a huge folder.** A crumb's subfolder menu sorted every subfolder and built one row per subfolder on the main thread. Sorting now runs on a worker thread, the menu lists at most 1,000 subfolders followed by "Show all <n> folders" (opens the folder), and any menu builds its first 100 rows at once and the rest 100 at a time while idle. It still scrolls.
- **fix(dnd): Read a drag's items once instead of at every motion.** Every drag-motion event cloned and scanned all dragged URIs (up to 100,000). The label and drop facts are now read once per drag.

### Behavior fixes

- **fix(window): Leave scroll bar troughs and edge controls to their presses.** The 4 px inner resize band now stands aside over a whole scroll bar (thumb and trough) and over buttons, check boxes, switches, scales, text fields and drop-downs. Corners and the outer band still resize.
- **fix(defaults): Let Restore previous undo only what Make default changed.** Restore previous turned off the Open and Save dialog handling even when the user had enabled it separately. Make default now marks when it is the one that turned the dialogs on, and Restore previous gives them back only then.
- **fix(versions): Start no new SMB lookup while an abandoned one still runs.** A Previous Versions lookup's SMB exchange runs on its own thread and could keep waiting on a server that stopped answering, and every reopen started another. At most one abandoned exchange now runs per server; a new lookup waits up to 2 seconds for it, then reports that the server did not list its versions.
- **feat(versions): Add "Restore over the original (Windows)" to Settings.** Off by default, so Restore still asks for another folder. On, Restore copies the version back into the item's folder and asks "Replace or Skip Files" like a paste.

### Smaller gaps

- **fix(views):** a folder's infotip adds Size, Folders and Files (first seven names, then "...", as Windows 11 lists seven of eight files), or "Empty folder". Contents are read on a worker thread, only for local folders, and a size that takes over a second is left out.
- **fix(network):** Map network drive has a Browse... button beside Folder.
- **fix(properties):** a local or removable drive's label box on its General tab can be edited; OK or Apply sets the label through UDisks (polkit may ask). Shares and discs stay read-only.
- **fix(search):** a word starting at the extension's dot follows the same rule as one without it: `.doc` no longer finds `report.docx`, just as `doc` does not; `.docx` and `docx` still do.
- **feat(search):** new switch in Settings > Search, "Match words at the start of file extensions (like Windows)", off by default. Windows 11 finds report.docx for doc, .doc and docx, and the PDF for pd; with the switch on, a word may also start at an extension's dot or first character (never inside it). The owner's rule stays the default.
- **fix(previews):** a `remotePreviews: true` that 2.0.18 saved as its default goes back off once, unless the user turned it on since (marked with `remotePreviewsChosen`).

The message template (`native/po/openxplorer.pot`) is regenerated in each commit that adds strings; no translation file is changed.

### Further fixes

- **fix(clipboard): Decode a pasted file list off the main thread.** A file list of up to 32 MiB was parsed on the main thread (#223).
- **fix(open): Read a .desktop link without holding the main loop.** The read waited up to 400 ms on a share that stopped answering (#29); it now runs on a worker while the window goes on.
- **fix(network): Forget only the signed-out server's saved credentials.** Sign out deleted the saved passwords of every port of the host (#126); it now deletes those of that host and port. Credentials are kept per server, so one share alone cannot be forgotten.
- **fix(format): Add Allocation unit size and Restore device defaults** (#133), and ask in Windows' words: "WARNING: Formatting will erase ALL data on this disk. To format the disk, click OK. To quit, click CANCEL."
- **feat(settings):** each owner choice that differs from Windows keeps its default and gains a setting, also set by "Behave like Windows Explorer": "Show all search matches (like Windows)" (#116), "Neutral selection colors (like Windows)" (#227), "Show Recent files in the navigation pane" (#273), "Connect to shares opened by other apps right away (like Windows)" (#191), "Keep Open/Save dialogs modal on GNOME (like Windows)" (#382), and "Show check boxes only while Ctrl is held" (#106). "Allow Linux file name characters when renaming" (off, as in Windows) applies to renaming in place, the Rename dialog and Properties alike (#212).
- **feat(search):** with "Show all search matches (like Windows)" on, the search cache returns every match too; the first 500 show at once and the rest are added 500 at a time without holding the window.
- **fix(defaults):** "Also handle Open and Save dialogs" starts off, as native actions stay opt-in (#132, #143).
- **fix(dialogs):** GTK 3 apps get `GTK_USE_PORTAL=1` from a systemd user environment generator the host packages install, which reads an opt-in mark in the settings folder, instead of a file in `~/.config/environment.d`; uninstalling removes it, and no maintainer script touches a home folder. The app replaces an unchanged file of an earlier version with the mark, once (#143).
- **Folder Options > Search** gains "Include system directories" (on) and "Include compressed files (ZIP, 7Z, TAR...)" (off), Windows' defaults (#231).
- **style:** three Clippy findings in code this branch builds on.

## Testing

- `python3 native/parity/check.py` and `python3 native/tools/check.py`, as an unprivileged user with Rust 1.92.0, on a private display and session bus: see the results below.
- New or updated tests for every fix (named in `native/parity/features.toml`).
- Real input with xdotool on a private Xvfb with a throwaway HOME and generated sample files, with `GTK_A11Y=none` and with it unset: file drag, `:` while renaming, type-to-select, and an MP3 and an MP4 shown in the Preview pane, then renamed and deleted.

## Results

- Further fixes: rustfmt and workspace Clippy with `-D warnings` PASS; `python3 native/parity/check.py` PASS; message template check PASS; the ox-core tests of the changed modules (596) PASS; the ox-app tests of the changed modules PASS except the two column-header tests listed below. No full run.

- `python3 native/parity/check.py`: PASS (exit 0).
- `python3 native/tools/check.py`: inventories, driver tests, drawn-icon guard, rustfmt, workspace Clippy with `-D warnings` and the message template check all PASS. It stops at the first failing test program, so all 88 test programs and the doctests were then run with check.py's own isolation helpers, carrying on past failures: 4,971 tests passed, 1 ignored. The ox-app program (every GTK test) passed in full.
- FAIL, in files this branch does not change: `location_fixture_drift::python_fixture_matches_desktop_core_py` (the installed Python 3.13 `urlparse` accepts `[V1.x]`, which the committed fixture expects to be rejected); `network_python::fuse_export_paths_match_native_opening_py` (the system `gi` module does not import under Python 3.13); nine `admin_copy` tests (the isolated run puts their folders under the world-writable `/tmp`). Not re-run on upstream main.
- Real input (xdotool, release build, private Xvfb, throwaway HOME, generated files), with `GTK_A11Y=none` and with it unset, two runs each: file drag, `:` while renaming, type-to-select, an MP3 playing in the Preview pane then renamed and deleted, an MP4 selected then renamed and deleted. The app stayed up after every step. The MP4 showed only its icon in the pane in this environment, so no video played.

## Checked on Windows 11

The Read-only dialog wording, the folder infotip format and empty-folder text, and Windows' search matches for doc, .doc, docx, pd, txt and mp were checked in a Windows 11 VM, and the branch follows them as described above. Still not checked: the Read-only dialog's wording for a selection of several items.

## Known test failures not caused by this branch

`window::tests::column_header_style::windows_style_keeps_the_chevron_clear_of_the_text_in_any_column_width` and `windows_style_shows_one_small_chevron_over_the_sorted_column` fail in this environment, the same way on the base commit.


`window::tests::search_history::a_tab_on_results_keeps_its_selection_and_scroll_position` times out intermittently in this environment; it fails the same way on main 5d21fd3e (3 of 3 runs).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
