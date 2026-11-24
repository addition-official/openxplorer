# Title

fix(undo): Ask when Undo Move finds the old name taken, and make extraction undoable

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/01-pass-1-copy-move)

## What changes for the user
- When Undo Move finds an item's old name taken by another item, the progress window now asks, with Windows' Interrupted Action layout and Try Again, Skip and Cancel: "An item named “Report.pdf” is already in “Work”, so Undo left it in “Sub folder”." Try Again moves the item back once the name is free; Skip leaves both items as they are. Nothing is ever overwritten. "Do this for all current items" appears only when more than one item is blocked.
- Extract here and Extract all into a new folder record "Undo Copy" for the extracted folder.
- Extract all into an existing folder records the items moved in from the private staging folder as "Undo Copy". Files a Replace answered there still come back from the Recycle Bin, and the staging folder's emptied-folder step is dropped, so it is never made again.
- Undo of an extraction moves things to the Recycle Bin (never a permanent delete), asks first when the extracted files changed since (OPS-030), and Redo brings the extraction back.
- `native/po/openxplorer.pot` is regenerated for one new message; no `.po` file changed.
- The last commit drops a test of this change that was found redundant (`try_again_moves_an_item_back_once_its_old_name_is_free`).

## Why
- After a cut and paste or a drag, if another program created an item with the old name, Ctrl+Z moved nothing, said nothing and used up the step, so the next Ctrl+Z reversed an older operation.
- Extractions were not recorded for Undo, so Ctrl+Z right after one reversed the operation before it. In testing it moved a file restored from the Recycle Bin straight back there.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- New: "An item named “{name}” is already in “{folder}”, so Undo left it in “{left_in}”."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: failing first, then passing, `ox-core/tests/ops_undo.rs` `undoing_a_move_puts_items_back_but_never_over_a_new_item` (now expects the question) and `try_again_moves_an_item_back_once_its_old_name_is_free` (since dropped as redundant), `ox-core/src/ops/undo.rs` `an_extraction_is_undone_as_a_copy`, and the `ox-app` window tests `undoing_a_move_onto_a_taken_name_asks_and_try_again_moves_it_back`, `undo_takes_back_an_extraction_into_an_existing_folder` and `undo_and_redo_walk_extract_here`; real input with xdotool under a private Xvfb, as an unprivileged user with a throwaway home and fictional mp3, mp4, pdf, jpg, odt, zip and 7z files, odd names and a 150 MB file.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
