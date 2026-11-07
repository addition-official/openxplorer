Model: claude-opus-5-5

# fix(recycle-bin): Replace the first of two items restored to one name with the second

This targets the `win11/restore-collisions` branch of #254. It fixes Replace for the second of two Recycle Bin items with the same original path.

## Problem

With #254, restoring two items with the same original path asks Replace or Skip Files about the second. Choosing **Replace** then fails with an Operation result report:

> Twin.txt: An item named "Twin.txt" already exists in its original folder. It was left in the Recycle Bin.

Steps:

1. In a folder, create `Twin.txt` and delete it to the Recycle Bin.
2. Create `Twin.txt` again and delete it.
3. Open the Recycle Bin, select both `Twin.txt` items and choose Restore.
4. Choose **Replace** for the second.

`replace()` refused any name this restore had already restored an item into.

## Change

- Replace sends the first restored item back to the Recycle Bin, where it came from, and restores the second in its place. The first item is not recorded as a replaced item, so Undo puts the second back in the Recycle Bin and leaves the folder as it was before the restore.
- When several conflicts for one name are answered Replace (for example, "Replace the files in the destination" for three items from one place, or two items over a newer file), only the last is restored and the earlier ones stay in the Recycle Bin. Each earlier Replace would end in the Recycle Bin anyway. This way the item first in the way is the only replaced item, and Undo still finds and restores it. The Recycle Bin dates deletions to the second, so with more than one replaced item from one path, Undo could not tell them apart.
- Skip and keeping both from Compare info were already correct and are now covered by tests.
- The parity note for OPS-041 names the new tests.

## Windows behavior

In Windows 11, Replace restores the second item over the first, Skip leaves the second in the Recycle Bin, and Compare info works as for other conflicts. This branch keeps #248's design, in which Replace moves the item in the way to the Recycle Bin rather than overwriting it. I did not find a reliable source saying that Explorer's Replace uses the Recycle Bin. Microsoft documents `FOFX_RECYCLEONDELETE` only for deletes, and community answers say Explorer overwrites the old file.

## Testing

- New `ox-core` test `replace_restores_the_second_twin_over_the_first` follows the steps above for Replace, Skip and Replace for three items, then checks Undo. It fails before this change with the report quoted above.
- New GTK test `replace_restores_the_second_of_two_twins_over_the_first` follows the same steps in the Recycle Bin window for Replace, Skip and both kept from Compare info. It fails before this change because the Operation result dialog appears.
- `two_items_from_one_free_place_ask_about_the_second` and `replacing_one_name_twice_restores_the_last_item_and_undo_brings_the_newer_file_back` (renamed) were updated to the new Replace behavior.
- The GTK test was also run once with Replace clicked by an XTEST pointer on a private Xvfb display. That variant is not committed.
- rustfmt and Clippy (`-D warnings`, Rust 1.92.0) are clean, and `parity/check.py` passes.
- Passing tests: `ops_restore_conflicts` (10), `ops_recycle_bin` (11), and `window::tests::recycle_bin` with `window::tests::restore_conflicts` (18). The full `tools/check.py` run was not done.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
