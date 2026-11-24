# Title

fix(rename): Keep a typed name on a click away, merge folders as Windows asks and refuse the same characters everywhere

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/02-pass-2-undo-redo)

## What changes for the user
- A click on blank space or on another item while renaming now commits the typed name, as in Windows 11, also with a quick tap. The selection stays as the click left it: nothing after a click on blank space, the clicked item after a click on another item.
- A slow second click on a tile's name in Large and Medium icons starts renaming. It is now measured against where the centered name is drawn.
- F2 on a selected item scrolled just out of view scrolls it back into sight and opens the name field there once it is laid out. An item with no row at all still gets the Rename dialog (OPS-009).
- The Properties name box and the Rename dialog refuse `\ / : * ? " < > |` with Windows' balloon, as renaming in place already did, and a name changed in Properties loses trailing periods and spaces as in place. Names that already contain one of these characters keep it while they are edited. Leading spaces are kept.
- Down and Up while renaming move the text cursor to the very end of the name (past the extension) and to its start, as End and Home do, in every view, and the row being renamed draws no focus outline, as Windows draws none.
- Renaming a folder in place to another folder's name asks Windows 11's "Confirm Folder Replace" question, with both folders, their icons and dates created, and Yes and No. Yes moves the folder's items into the other one through the normal move path (a taken file asks Replace or Skip, same-named subfolders merge), removes the renamed folder once empty (a skipped item keeps it) and selects the merged folder. One Undo ("Undo Rename") makes the folder again and moves everything back, a replaced file returning from the Recycle Bin; Redo merges again. No returns to editing. Properties and the Rename dialog still refuse a taken folder name.
- A refused rename says "You don't have permission to rename “locked.txt” here." instead of GIO's message with the whole path, also for Undo.
- The test pointer gained `Pointer::tap`, a press and release sent together.
- The last commit drops tests of this change that were found redundant (see Tests).

## Why
- A click away changed the selection, GTK bound the row again, and that closed the name field before it could commit, so the typed name was silently dropped.
- The slow second click was measured against text starting at the tile's left edge, but tile names are centered.
- With the item just out of view GTK still keeps its row alive, and the field opened there, taking whatever the user typed unseen.
- Properties renamed a file to `Pic:2. .jpg`. Linux file systems refuse only `/` and NUL, but a name with any of Windows' forbidden characters cannot be copied as it is to a Windows share, an NTFS or exFAT drive or a FAT stick, so refusing it while typing avoids a later copy failure. Trailing periods and spaces are trimmed as the Windows shell trims them ([Naming Files, Paths, and Namespaces](https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file)).
- Down and Up committed the name and renamed the next item in Details, and left the field in the icon views. In Windows 11 the name box is a one-line edit.
- A folder renamed to another folder's name was refused, where Windows 11 offers to merge.
- The permission error showed the full path in GIO's wording.

Limitations: renaming a folder in the navigation pane still uses the Rename dialog, where Windows edits it in place; a renamed item is sorted into place at once; the folder merge is offered when renaming in place only.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- New: "Confirm Folder Replace"
- New: "This destination already contains a folder named '{name}'.\n\nIf any files have the same names, you will be asked if you want to replace those files.\n\nDo you still want to merge this folder"
- New: "with this one?"
- New: "You don't have permission to rename “{name}” here."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: the new `window/tests/renaming_by_pointer.rs` tests (click on blank space and on another item, each with a normal click and a quick tap, in Details, Large and Medium icons; slow second click on a tile's name; F2 on an item scrolled out of sight; the Rename dialog's balloon), `item_dialogs::the_name_in_properties_refuses_forbidden_characters_and_trims_its_end`, ox-core `ops_rename::a_rename_the_folder_refuses_names_the_item_without_its_path` and `rename::tests::a_denied_rename_names_the_item_not_its_path`, `renaming_by_pointer::down_and_up_while_renaming_move_the_cursor_to_the_ends`, `renaming_by_pointer::the_row_being_renamed_draws_no_focus_outline` (painted, light and dark), the four `window/tests/renaming_folder_merge.rs` tests and ox-core `a_merge_by_rename_is_undone_by_making_the_folder_then_moving_back`, each failing before its fix; after the undo work was merged, 158 rename, conflict, new item, archive, undo and file operation tests passing, the 5 failures also failing on unchanged main; rustfmt, `parity/check.py` and the i18n check; every test executable in `check.py`'s isolation (4,885 passed, 27 failed; 21 also fail on unchanged main, 5 were fixed by reworking the off-screen fix, and `dialog_keys::the_file_keys_wait_for_a_dialog_window_in_front` passed when rerun), with `check.py` itself stopping at a newer Clippy's lints in files this branch does not touch. `renaming_by_pointer::a_click_on_another_item_keeps_the_typed_name_and_selects_that_item` failed once in a combined run and passed in about ten other runs. This PR's last commit drops the Rename dialog balloon test, the Down and Up test, the focus outline test and the two ox-core denied-rename tests as redundant.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
