# Title

fix(native): Fix a crash, a silent recursive change, freezes and smaller gaps found in the 2.0 audit

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
- **A pasted file list no longer freezes the window.** A file list read from the clipboard (up to 32 MiB) is decoded on a worker thread instead of the main thread.
- **Opening a .desktop link no longer holds the window.** The link is read on a worker thread for at most 400 ms while the window goes on, so a share that stopped answering no longer stalls it.
- **Sign out forgets only that server's saved credentials.** It deletes the saved passwords (OpenXplorer's and GVfs's) of the host and port signed out of, no longer those of every port of the host. Credentials are kept per server, so one share alone cannot be forgotten.
- **Format** adds "Allocation unit size" ("Default allocation size", then the file system's own sizes in bytes or kilobytes) and "Restore device defaults", and asks in Windows' words before it erases anything: "WARNING: Formatting will erase ALL data on this disk." / "To format the disk, click OK. To quit, click CANCEL.", with OK (no longer "Format") and Cancel.
- **Windows' way for the owner's choices.** Each choice that differs from Windows keeps its default and gains a setting, listed for the "Behave like Windows Explorer" switch that PR 31 adds: "Show all search matches (like Windows)" (Settings > Search), "Neutral selection colors (like Windows)" and "Show Recent files in the navigation pane" (Settings > Appearance), "Connect to shares opened by other apps right away (like Windows)" (Settings > Default apps, under "Requests from other apps"), "Keep Open/Save dialogs modal on GNOME (like Windows)" (Settings > Default apps) and the existing "Show check boxes only while Ctrl is held". "Allow Linux file name characters when renaming" (Settings > Files & folders, off as in Windows) applies to renaming in place, the Rename dialog and Properties alike; the slash and backslash stay refused.
- **All search matches.** With "Show all search matches (like Windows)" on, a search and the search cache return every match; the first 500 show at once and the rest are added 500 at a time without holding the window.
- **"Also handle Open and Save dialogs" starts off**, as native actions stay opt-in.
- **GTK 3 apps' Open and Save dialogs** get `GTK_USE_PORTAL=1` from a systemd user environment generator that the host packages install, which reads an opt-in mark in the settings folder (`~/.config/winspace/file-dialogs/gtk-apps`), instead of a file the app wrote in `~/.config/environment.d`. Uninstalling removes it, and no maintainer script touches a home folder. The app replaces an unchanged file of an earlier version with the mark, once.
- **Folder Options > Search** adds "Include system directories" (on) and "Include compressed files (ZIP, 7Z, TAR...)" (off), Windows' defaults.
- **Tests.** The "test: Drop redundant tests" commit of the feature branch removes no test that is in this series before this PR. Five existing tests from upstream main are renamed or rewritten by the fixes above: `a_link_on_a_stalled_mount_is_given_up_at_once`, `forgetting_a_host_waits_for_a_custom_port_save_in_flight`, `signing_out_forgets_every_port_and_window_in_the_chosen_scope`, `gtk_apps_are_covered_by_an_environment_file` and `the_right_edge_leaves_the_file_lists_scroll_thumb_to_the_scroll_bar`.

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
- A pasted file list and a .desktop link on a stalled share were read on the main thread; Sign out deleted the saved passwords of every port of the host.
- Windows' Format dialog has Allocation unit size and Restore device defaults and warns before erasing.
- Several of the owner's choices differ from Windows with no way to get Windows' behavior.
- "Also handle Open and Save dialogs" started on, though native actions stay opt-in; the GTK 3 dialog setting was a file in the home folder that uninstalling left behind.

## CHANGED OWNER DECISIONS
The owner's own choices keep their defaults; Windows' behavior is added as settings that start off. The parity specs in `native/parity/features.toml` change in these recorded decisions:
- INT-009 (Make-default options): before, the native option "Also handle Open and Save dialogs" started on; now it starts off (native actions stay opt-in).
- INT-032 (GTK 3 apps' Open and Save dialogs): before, the app wrote `~/.config/environment.d/60-openxplorer-file-dialogs.conf` setting `GTK_USE_PORTAL=1`; now the host packages install a user environment generator that sets it only while the opt-in mark `~/.config/winspace/file-dialogs/gtk-apps` exists, and the old file is replaced by the mark once, only while unchanged.
- SRCH-004 (Wildcards in the folder filter): before, a plain word found only in the extension did not match, but `.doc` still found `report.docx`; now a word or pattern starting at the extension's dot follows the same rule (`.doc` no longer finds `report.docx`, `.docx` and `docx` do), and the new switch "Match words at the start of file extensions (like Windows)" can turn on Windows' matching.
- PROP-003 (Read-only on a folder): before, Read-only on a folder changed every file inside without asking; now Confirm Attribute Changes asks first.

## New and changed visible text
Changed:
- "Item check boxes stay out of the way until you hold Ctrl to pick several items. Turn this off to show them whenever you point at an item." → "Item check boxes stay out of the way until you hold Ctrl to pick several items. Turn this off to show them whenever you point at an item, as in Windows."
- "GTK 3 apps keep GTK's dialog: ~/.config/environment.d/60-openxplorer-file-dialogs.conf is not OpenXplorer's, so it was left alone." → "GTK 3 apps keep GTK's dialog: ~/.config/winspace/file-dialogs/gtk-apps is not OpenXplorer's, so it was left alone."
- "Formatting erases all data on this drive. Copy anything you want to keep to another drive first." → "WARNING: Formatting will erase ALL data on this disk." followed by a line break and "To format the disk, click OK. To quit, click CANCEL."
- "Format" (the format question's button) is removed; the button reads "OK".

New:
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
- "Allocation unit size"
- "Default allocation size"
- "{size} bytes"
- "{size} kilobytes"
- "Restore device defaults"
- "Show all search matches (like Windows)"
- "A search shows every match, as in Windows, adding them while you keep working. Off, it shows the first 500 and asks you to narrow your search."
- "Neutral selection colors (like Windows)"
- "Hover and selection in the file list and the navigation pane keep Windows' colors whatever your accent color, as in Windows 11. Off, your accent color tints them."
- "Show Recent files in the navigation pane"
- "Lists Recent files in the navigation pane while it shows more items. Off, recent files are on Home only, as in Windows 11."
- "Requests from other apps"
- "Connect to shares opened by other apps right away (like Windows)"
- "When another app opens a folder or item on a network share that is not connected, OpenXplorer connects to it at once, as Windows does. Off, the tab waits until you click Try again."
- "Keep Open/Save dialogs modal on GNOME (like Windows)"
- "On GNOME, another app's Open or Save dialog stays modal, so that app waits until the dialog closes, as in Windows. GNOME then moves the dialog with that app's window. Off, the dialog moves on its own and the app stays usable."
- "Allow Linux file name characters when renaming"
- "Names you type may contain : * ? " < > | as Linux allows, when renaming in place, in the Rename dialog and in Properties alike. Such a name cannot be copied to a Windows share or a USB drive as it is. Off, these are refused as in Windows."
- "Include system directories"
- "Include compressed files (ZIP, 7Z, TAR...)"
- Search keywords for the new rows: "previous versions restore snapshot shadow copy overwrite original replace windows", "search word extension suffix type docx pdf partial match windows file name", "search results matches all limit 500 first more cap live folder windows", "selection hover highlight color colour accent tint neutral blue grey file list windows look", "recent files navigation pane sidebar history row hide windows home", "connect share smb network mount other apps show folders show items d-bus request automatically windows", "open save dialog file picker modal attached gnome mutter zorin wayland move window windows", "rename name characters colon asterisk question mark quote pipe forbidden invalid linux windows balloon"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: for the later fixes (clipboard, .desktop link, Sign out, Format, the settings for the owner's choices, all search matches, the Open and Save dialogs, Folder Options > Search), rustfmt, workspace Clippy with `-D warnings`, `python3 native/parity/check.py`, the message template check and the tests of the changed modules (596 ox-core tests) passed, except two column header tests that fail the same way on the base commit, with no full run; for the first fixes, new or updated tests for every fix (named in `native/parity/features.toml`), including a test that runs the test program again with `GTK_A11Y=none` and fails without the fix; `python3 native/parity/check.py`; `python3 native/tools/check.py` as an unprivileged user with Rust 1.92.0 (inventories, driver tests, drawn-icon guard, rustfmt, workspace Clippy with `-D warnings` and the message template check passed); all 88 test programs and the doctests with check.py's own isolation helpers, carrying on past failures: 4,971 passed and 1 ignored, the ox-app program in full; failures in files the branch does not change: `location_fixture_drift::python_fixture_matches_desktop_core_py`, `network_python::fuse_export_paths_match_native_opening_py` and nine `admin_copy` tests; `window::tests::search_history::a_tab_on_results_keeps_its_selection_and_scroll_position` timed out intermittently, as on main; real input with xdotool on a private Xvfb display with a throwaway HOME and generated files, with `GTK_A11Y=none` and with it unset, two runs each: file drag, `:` while renaming, type-to-select, an MP3 playing in the Preview pane then renamed and deleted, an MP4 selected then renamed and deleted (the app stayed up after every step). The Read-only dialog wording, the folder infotip format and Windows' search matches for doc, .doc, docx, pd, txt and mp were checked in a Windows 11 VM; the wording for a selection of several items was not.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
