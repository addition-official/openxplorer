Model: claude-opus-5-5

# fix(transfers): Ask about every failed item in the progress window and remove the Operation result dialog

Builds on #263 (access-denied-continue), which moved the failed-item questions into the progress window, and is merged with newest main (#252, #254, #272 included).

## Summary

Windows 11 File Explorer has no summary dialog after a file operation. It asks about each failure while the operation runs, in the progress window. OpenXplorer still opened "Operation result" afterwards for any failure no question had shown. A reported case: restoring two items deleted from the same place said "Twin.txt: An item named "Twin.txt" already exists in its original folder. It was left in the Recycle Bin."

This PR makes every such failure ask inline, reusing the progress window's questions from #263. It then removes the dialog: `OperationSummary`, the `summarize*` functions, `RESULT_TITLE`, and their strings (also from `zh_CN.po`). Only "Operation stopped" remains, for a request refused before any item ran.

- Operations outside the transfer engine now share one helper, `ops::results::ItemQuestions`, which asks the engine's question (Try Again, Skip, Cancel, "Do this for all current items").
- A new cause, `FailureCause::NeedsAttention`, covers problems the user must fix by hand. It shows "Interrupted Action" with what happened and where, and an OK button. It also shows after Cancel, because the information is still needed.

## Each former Report source, and what it does now

| Former source | Now | Windows 11 | Source |
|---|---|---|---|
| Restore: a name taken while the restore runs, e.g. two items from the same place (the reported Twin.txt case), or a race | The item is planned again and asks **Replace or Skip Files**. Skip leaves it in the Recycle Bin silently. | Asks Replace or Skip Files. Skip leaves the second item in the Recycle Bin. | Recorded on Windows 11 for the restore-conflicts work (#248, status note `restore-conflicts.txt`). I found no Microsoft page on it. |
| Restore: any other failure of an item (permission, item gone, origin not recorded) | **File Access Denied**, **Item Not Found** or **Interrupted Action**, with Try Again, Skip, Cancel | Same questions for a failed move | [Item Not Found](https://www.winhelponline.com/blog/could-not-find-this-item-when-deleting-file-folder-windows/), [File Access Denied](https://windowsreport.com/file-access-denied-windows-8/), [Interrupted Action](https://learn.microsoft.com/en-us/answers/questions/2703053/interrupted-actions-problem) |
| Restore with Replace: the item in the way cannot go to the Recycle Bin, changed kind, or was already replaced in this restore | Interrupted Action with the reason, then OK. The item stays in the Recycle Bin. | Windows overwrites, so it has no such failure. This design keeps #248's "never overwrite" rule and asks instead of reporting afterwards. | Same note |
| Drag out of the Recycle Bin onto a taken name, and other failures of that drag | Planned as a restore into the drop folder (`scan_drag_out_conflicts`): Replace or Skip Files, and the per-item questions | Moving items onto taken names asks Replace or Skip Files | As above |
| Delete in the Recycle Bin, Empty Recycle Bin | Per-item question, e.g. Item Not Found for an item removed meanwhile. The action now reads "Deleting 1 item from Recycle Bin". | Same | [Item Not Found](https://www.winhelponline.com/blog/could-not-find-this-item-when-deleting-file-folder-windows/) |
| Copy/move/delete: failures after **Skip** with "Do this for all current items" | **Silent**: these failures are the user's own answer | Windows skips them without asking again and reports nothing | Behavior of Windows' "Do this for all current items" checkbox. I found no separate source. |
| Engine: a problem needing recovery by hand: a kept backup, a failed or unverified replacement, "copied but the original could not be removed", "leaves out items the destination cannot store, so the original was kept" | Interrupted Action with the message and where to look, then OK. Shown even after Cancel. | Windows has no staging or backups. The closest Windows behavior is asking at the point of failure, never in a summary. | [SHFILEOPSTRUCT, FOF_NOERRORUI](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-shfileopstructa): error UI is shown per error unless suppressed |
| Engine: staging left behind | Same notice, naming the folder | as above | as above |
| Engine: other failures the engine cannot ask about (a name taken while publishing, something the location cannot do) | Same notice, so nothing is left unseen | Interrupted Action with the system error | [Interrupted Action](https://learn.microsoft.com/en-us/answers/questions/2703053/interrupted-actions-problem) |
| Engine: No to "Move by copying?" | **Silent**: the user's own answer to OpenXplorer's own question | No equivalent question | — |
| Engine: a move that kept the entries the user skipped | **Silent**: the user's own Skip | Skipped items stay; nothing is said | — |
| Paste with conflicts: a replaced file left in the Recycle Bin, or a late failure of a folder | Notice at once, then OK | as above | as above |
| Batch and numbered rename | Per-item question; Skip leaves the name | **File In Use** / **File Access Denied** / **Item Not Found** with Try Again, Skip, Cancel | [File In Use](https://www.guidingtech.com/fix-action-cant-be-completed-because-file-is-open/) |
| Create links / shortcuts for several items | Per-item question | — | — |
| Duplicate: a folder that refuses the run | Question about that folder | — | — |
| Undo / Redo: rename back, moving back (re-making the folder), re-making or removing emptied folders, restoring trashed items, a request refused as a whole | Per-item questions; a refused request gets a notice | Undo runs as a normal file operation with the same questions | — |
| Undo/Redo: a folder to re-make that is already there; a folder to remove again that now holds an item | **Silent**: the first is not a failure; the second keeps what the user put there | I found no Windows equivalent or source | — |
| Restore a previous version (a copy) | Asks about failures like every copy (the asker was missing) | Same | — |

The following never produced the report and are unchanged: single rename, New folder and other new items (own error messages), extraction and compression ("Extraction stopped" / "Compression stopped" for failures of the whole operation; extracting into an existing folder goes through the transfer path above), and "Operation stopped".

### Known differences

- The question wording comes from the run's mode. A rename failure that falls into "Interrupted Action" says "...keeping you from moving the file", where Windows says "renaming".
- A restore's question asks without Continue as administrator. There is no administrator Recycle Bin path.

## Restore with Replace (deb 20 report)

The report: restoring Twin.txt (A) over an existing Twin.txt (B) with **Replace the file in the destination** seemed not to restore, and the Recycle Bin still listed a Twin.txt. It does not reproduce on this branch after merging newest main (#252, #254, #272). By real input under Xvfb:

- OX-Test/Twin.txt ends with content A and A's own older modification time.
- A leaves the Recycle Bin.
- The one "Twin" the Recycle Bin still lists is **B**, deleted at the moment of the Replace.

This also holds with the folder on a separate ext4 volume with its own `.Trash-<uid>`. The GTK test `restoring_onto_a_taken_name_asks_replace_or_skip_files` now checks all of this.

**What happens to B:** in this app, Replace on a restore sends the file in the way to the Recycle Bin, as Replace does elsewhere (#248, #252). That is why a "Twin" is still listed. For Windows 11 I found no Microsoft source:

- The only Microsoft article on the question, [KB 267422](https://ftp.zx.net.nz/pub/archive/ftp.microsoft.com/MISC/KB/en-us/267/422.HTM) (Windows 98/Me/2000), says "As soon as the file is replaced, the old file is written to the Recycle Bin."
- Community sources for current Windows say the replaced file is overwritten and is not in the Recycle Bin, e.g. [Total Commander forum](https://www.ghisler.ch/board/viewtopic.php?p=310543) and [Directory Opus forum](https://resource.dopus.com/t/move-overwritten-files-to-the-recycle-bin/59870).

So current Windows most likely overwrites B. Keeping B in the Recycle Bin is this project's deliberate safety difference. Whether to switch to overwriting is left to the maintainer.

| Check (real input, Xvfb, 2x, unprivileged user) | Light | Dark |
|---|---|---|
| Restore A over B, Replace: A on disk with its date, A out of the bin, B in the bin, no dialog | PASS | PASS |
| Same, folder on a separate ext4 volume | PASS | — |
| Restore two items from one place: Replace or Skip Files, Skip silent | PASS | PASS |
| Delete a Recycle Bin item removed meanwhile: Item Not Found, Skip silent | PASS | PASS |
| Copy an unreadable file: File Access Denied, Skip copies the other, silent | PASS | PASS |

## Testing

- rustfmt and Clippy with `-D warnings` on Rust 1.92.0 (the CI toolchain) are clean. `parity/check.py`, `i18n.py check`, `sync-docs.py --check` and the public-data audit pass.
- These ran as an unprivileged user (root ignores permissions), each in check.py's isolated session:
  - ox-core: `transfer` (135), `ops_transfer`, `ops_recycle_bin`, `ops_restore_conflicts`, `ops_undo`, `ops_duplicate`, `ops_rename`, and the `ops::` and `transfer::` unit tests.
  - ox-app: `recycle_bin`, `restore_conflicts`, `file_operations`, `clipboard`, `renaming_several`, `worker_questions`, `transfer_panel`, `failure`, `dialog`, `name_conflicts`, `file_ops`, `progress_window`, `delete`, `undo`, `history`, `middle_click` and `drag_and_drop`.
- Three ox-app `archives` tests time out in this container, also as root. They launch the desktop's archive manager or default app and touch no changed code.
- New tests:
  - `a_name_the_restore_itself_takes_is_asked_about_as_a_conflict`
  - `deleting_an_item_gone_from_the_recycle_bin_asks_about_it`
  - `a_restore_asks_about_a_failed_item_while_it_runs`
  - `ItemQuestions` unit tests
  - GTK tests `restoring_two_items_from_the_same_place_asks_about_the_second`, `deleting_an_item_gone_from_the_recycle_bin_asks_item_not_found`, `an_item_dragged_out_onto_a_taken_name_asks_replace_or_skip_files` and `an_item_that_cannot_be_renamed_is_asked_about_and_the_rest_are_renamed`
- Real app, with real input: on a private X display at 2x, as an unprivileged user with fictional files, using xdotool clicks and keys, in light and dark. Each case ended with no dialog afterwards:
  - Restoring two Twin.txt items asked Replace or Skip Files for the second.
  - Deleting a Recycle Bin item removed while Delete File asked showed Item Not Found.
  - Copying an unreadable file with a readable one showed File Access Denied; Skip copied the other.
- Not exercised in the real app: the recovery notice (covered by `cancellation_never_hides_a_backup_that_requires_manual_recovery`), Undo failures, and links.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
