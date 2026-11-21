# Title
fix(undo): Ask when Undo Move finds the old name taken, and make extraction undoable

# Body
Model: claude-opus-5-5
CLA: I agree

## What was wrong

- **Undo Move left items behind in silence.** After a cut and paste (or a drag), if another program created a file with the item's old name, Ctrl+Z moved nothing, said nothing and used up the step. The next Ctrl+Z then reversed an older operation. Nothing was overwritten, but the user had no way to know the Undo hadn't happened.
- **Extraction wasn't undoable.** Extract here and Extract all were not recorded for Undo, so Ctrl+Z right after an extraction reversed the operation *before* it. In testing it moved a file restored from the Recycle Bin straight back there.

## What changed

- Undo Move now asks in the progress window, with Windows' Interrupted Action layout and Try Again / Skip / Cancel: "An item named “Report.pdf” is already in “Work”, so Undo left it in “Sub folder”." Try Again moves the item back once the name is free. Skip leaves both files as they are. Nothing is ever overwritten. "Do this for all current items" appears only when more than one item is blocked.
- Extract here and Extract all into a new folder record "Undo Copy" for the extracted folder.
- Extract all into an existing folder records the items moved in from the private staging folder as "Undo Copy". Previously Undo was disabled for that step, which made Ctrl+Z reverse the step before. Files a Replace answered there still come back from the Recycle Bin. The private folder's emptied-folder step is dropped, so it is never recreated.
- Undo of an extraction moves things to the Recycle Bin (never a permanent delete), asks first when the extracted files changed since (OPS-030), and Redo brings the extraction back.
- Only `native/po/openxplorer.pot` was regenerated (one new message). No .po edits.

## Tests

Failing first, then passing:
- `ox-core/tests/ops_undo.rs`: `undoing_a_move_puts_items_back_but_never_over_a_new_item` (now expects the question), `try_again_moves_an_item_back_once_its_old_name_is_free`
- `ox-core/src/ops/undo.rs`: `an_extraction_is_undone_as_a_copy`
- `ox-app` window tests: `undoing_a_move_onto_a_taken_name_asks_and_try_again_moves_it_back`, `undo_takes_back_an_extraction_into_an_existing_folder`, `undo_and_redo_walk_extract_here`

Real input (xdotool under a private Xvfb, unprivileged user, throwaway HOME, with mp3/mp4/pdf/jpg/odt/zip/7z, odd names and a 150 MB file) is listed in the status file.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01CQbN3QRPtdSgfs1zG3Kxvr
