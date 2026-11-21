# Title

fix(rename): Keep a typed name on a click away and refuse the same characters everywhere

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 3 of tonight's Windows 11 comparison: renaming. F2, a slow second click, right-click Rename and the Properties name box were tried in Details and the icon views, with real keys and clicks (xdotool and the test harness's XTEST pointer) on a private Xvfb display, as an unprivileged user with a throwaway home, on fictional files (mp3, mp4, pdf, jpg, odt, zip, 7z, tar.gz, a 3 GB sparse image, odd and accented names). Five problems are fixed, each with a test that fails first:

- **A click away lost the typed name.** In Windows 11 a click on blank space or on another item commits the name. Here the click changed the selection, GTK bound the row again, and that closed the name field before it could commit: the typed name was silently dropped. A row bound again to the same item now keeps its field. The name is committed and the selection stays as the click left it: nothing after a click on blank space, the clicked item after a click on another item, so the next command acts on what the user clicked. A quick click (a touchpad tap, press and release together) lost it too; both are tested.
- **A slow second click on a tile's name did nothing** in Large and Medium icons. The click was measured against text starting at the tile's left edge, but tile names are centered. It is now measured against where the text is drawn.
- **F2 opened the name field out of sight.** With the selected item scrolled just out of view, GTK still keeps its row alive, and the field opened there, taking whatever the user typed unseen. The row is now scrolled back into sight and the field opens once it is laid out there, as a new folder far down the list already was. An item with no row at all still gets the Rename dialog (OPS-009).
- **The Properties name box and the Rename dialog took `\ / : * ? " < > |`.** Renaming in place already refuses them as Windows does, with Windows' balloon. Properties renamed a file to `Pic:2. .jpg`. Both now refuse them the same way, and a name changed in Properties loses trailing periods and spaces as in place.
- **A refused rename showed GIO's message with the whole path** ("Error moving file /…/locked.txt: Permission denied"). It now says "You don't have permission to rename “locked.txt” here.", also for Undo.

A stale focus outline that stayed on the first row after Tab moved a rename on was seen before these changes and not after; no automated test reproduces it, so it is not claimed as fixed by a test.

## Forbidden characters on Linux

Linux file systems refuse only `/` and NUL. The app refuses Windows' whole set (`\ / : * ? " < > |`) while a name is typed, on every drive, in all three places a name is typed (in place, the Rename dialog, Properties). A name with one of them cannot be copied as it is to a Windows share, an NTFS or exFAT drive or a FAT USB stick, so refusing it at the moment of typing, with Windows' own balloon, avoids a later copy failure. Existing names that already contain one keep it while they are edited. Trailing periods and spaces are trimmed as the Windows shell trims them ([Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file)). Leading spaces are kept (see "Not confirmed").

## Checked and unchanged

- Esc, Enter, Tab and Shift+Tab (also across a re-sort), the name without its extension selected with extensions shown and hidden, the whole name of folders.
- Several items: "Batch (1).mp4", "Batch (2).pdf", "Batch (3).mp3", each keeping its extension; one Undo renames all back.
- A taken file name asks Windows' "Rename File" question with the next free number ("Todo (2).txt"); Yes renames, Undo renames back.
- Case-only renames on ext4 in the app; on a FAT image through GIO (fusefat). exfat-fuse turned out case-sensitive, so it does not stand in for real exFAT.
- Names over 255 bytes are refused with a message and the field stays.
- Renaming a playing mp3 in the preview pane: renamed, no crash or warning; playback stops and the preview starts again at 0:00.
- Undo of a single rename, a batch and a Properties rename.

## Tests

- New: `window/tests/renaming_by_pointer.rs` (click on blank space and on another item, each with a normal click and a quick tap, in Details, Large and Medium icons; slow second click on a tile's name; F2 on an item scrolled out of sight is scrolled back and edited where it shows; the Rename dialog's balloon), `item_dialogs::the_name_in_properties_refuses_forbidden_characters_and_trims_its_end`, ox-core `ops_rename::a_rename_the_folder_refuses_names_the_item_without_its_path` (run as an unprivileged user; it returns early as root) and `rename::tests::a_denied_rename_names_the_item_not_its_path`.
- Each failed before its fix and passes after.
- The test pointer gained `Pointer::tap`, a press and release sent together.

- `native/tools/check.py` stops at Clippy here: this container's Clippy 1.97 reports `duration_suboptimal_units` in five ox-core files and two other lints in `group_day.rs` and `picture_guard.rs`, none touched by this branch. With that one lint allowed, Clippy reports nothing else. rustfmt, `parity/check.py` and the i18n check pass.
- Every test executable was then run in check.py's isolation (private Xvfb, D-Bus and home): 4,885 passed, 27 failed. Each failing test was rerun on unchanged main 5336b108. 21 fail there too; they need a non-root user, the Python app's comparison sources, an administrator helper, or a display scale. Of the other six, five were renames of a new folder or new text document broken by a first version of the off-screen fix; that fix was reworked, and the five now pass. The sixth, `dialog_keys::the_file_keys_wait_for_a_dialog_window_in_front`, passes when rerun on the branch.
- `renaming_by_pointer::a_click_on_another_item_keeps_the_typed_name_and_selects_that_item` failed once in a combined run and passed in about ten other runs; the failure message was not captured.

## Not confirmed

The Windows behaviors this relies on that no Microsoft page states are listed as steps to try in the Windows 11 VM in the status file.

## Limitations

- Renaming a folder in the navigation pane still uses the Rename dialog; Windows edits it in place. The sidebar and tree belong to pass 4, so this is left to it.
- A renamed item is sorted into place at once; Windows may keep it where it was until a refresh (VM check).
- Renaming a folder to the name of another folder is refused with a message; Windows offers to merge them (VM check). Merging was not added.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
