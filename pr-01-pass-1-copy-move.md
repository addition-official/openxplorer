# Title

fix(transfers): Copy to USB sticks, keep a move's originals until its copy is on the drive, and stop a file drag crash

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: upstream main (0fed5d4a)

## What changes for the user
- A move to another drive writes its copy out to that drive (`syncfs` on local destinations) after the copy is published and before any original is removed. If the drive cannot write it out, every original is kept and the progress window says: "The copy at {uri} could not be written to its drive, so the original was kept. {error}".
- Copies to a FAT stick, and to exFAT, NTFS or other FUSE file systems, no longer stop at once with "Function not implemented". A mode change that such a file system refuses (EPERM, ENOSYS or ENOTSUP) on a folder the user owns counts as done; a folder of another account, or any other error, still stops the copy.
- An NTFS, exFAT or FAT stick mounted through FUSE now gets Windows' name rules and the "Name Windows cannot use" question. GIO names every FUSE file system `fuse`, so the app now reads the mount table to find `fuseblk` and the FUSE subtypes of NTFS, exFAT and FAT.
- The percentage of a copy no longer jumps to its share of the items when a small file comes first (a 35 KB photo before a 700 MB video showed "50% complete" at 15% written). A batch report that replaces a queued file report keeps its byte counts.
- Dragging a file no longer aborts the app when GTK accessibility is turned off (`GTK_A11Y=none`): the drag's screen-reader announcement is skipped and no accessibility context is asked for.
- `po/openxplorer.pot` is regenerated for the new message; no `.po` file changed.

## Why
- A cross-drive move removed the originals while its copy was still only in memory: read straight from the device right after the move ended, none of the moved files was there. Pulling the stick or losing power in the next half minute lost the files.
- The copy engine changes the mode of its own staging folders, which FAT and FUSE file systems refuse, so the copy stopped and Cancel left its staging folder behind.
- NTFS sticks mounted by ntfs-3g never matched `fuseblk`, so names such as `colon: name.txt`, `question?.txt`, `CON` and `trailing dot.` were written where Windows cannot open them.
- The first file's byte reports, the only ones saying what the whole copy writes, were dropped within one refresh for the next item's report.
- With `GTK_A11Y=none`, GTK 4.14 gives a widget no accessibility context, and asking for it aborted the process.

Known limitations: a plain copy to a USB stick is not flushed when the progress window closes; only a move's originals wait for it. A sparse file is copied as a full one. The "Move by copying?" question for moves to another drive stays, as it is an open owner decision (XFER-011).

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- New: "The copy at {uri} could not be written to its drive, so the original was kept. {error}"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: the new tests `the_copy_reaches_its_drive_before_an_original_is_removed`, `a_folder_of_ours_on_a_file_system_without_unix_modes_keeps_the_copy_going`, `a_fuse_drive_takes_the_rules_of_the_file_system_in_the_mount_table`, `a_small_first_file_does_not_jump_the_percentage_to_its_share_of_the_items` and `accessibility_turned_off_is_never_asked_for_a_context`, each failing before its fix, plus the existing announcement test under `GTK_A11Y=none`; real input on a private Xvfb as an unprivileged user with fictional files: the cross-drive move read back from the device image with debugfs (8 of 8 files on the device when the move ends, 0 of 8 before), a copy of the whole set to a FUSE stick, an NTFS stick asking about and renaming 19 odd names, the percentage at 14% with 15% written, and plain, Ctrl, Shift and Alt drags without a crash; `python3 native/parity/check.py`; every test binary run isolated as `native/tools/check.py` runs them and compared by test name with unchanged main (`check.py` itself stopped at a newer Clippy's `duration_suboptimal_units` lint in files this change does not touch, the same on main).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
