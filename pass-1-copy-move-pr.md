# Title

fix(transfers): Copy to USB sticks, keep a move's originals until its copy is on the drive, and stop a file drag crash

# Body

Model: claude-opus-5-5

CLA: I agree

## Problem

Copy and move tested end to end as an unprivileged user on a private Xvfb (and Weston where the window manager mattered), with real keyboard and pointer input (xdotool/XTEST), fictional data (mp3, mp4, pdf, jpg, odt, zip, 7z, odd names, a 700 MB file, 100,000 small files, a sparse file) and checksums of every source and target:

1. **A cross-drive move could lose both copies.** A move to another drive copied, published the copy and removed the originals while the copy was still only in memory: read straight from the device right after the move ended, none of the moved files was there (about 21 MB still dirty). Pulling the USB stick, or losing power, in the next half minute lost the files.
2. **Copies to a FAT stick failed at once.** The engine changes the mode of its own staging folders; FAT refuses that (EPERM against the mount's masks), as do exFAT and NTFS through FUSE and other FUSE file systems (ENOSYS, ENOTSUP). The copy stopped with "Function not implemented", and Cancel then left its staging folder behind.
3. **NTFS sticks took no Windows name rules.** GIO names every FUSE file system `fuse`, so an NTFS stick mounted by ntfs-3g never matched `fuseblk`: `colon: name.txt`, `question?.txt`, `CON` and `trailing dot.` were written where Windows cannot open them, without the "Name Windows cannot use" question.
4. **The percentage jumped to 50%.** Copying a 35 KB photo and a 700 MB video showed "50% complete" as soon as the photo was done (15% had been written). The photo's byte reports, the only ones saying what the whole copy writes, were dropped within one refresh for the next item's report, so the action went by its share of the items.
5. **A file drag aborted the app with GTK accessibility turned off.** With `GTK_A11Y=none`, GTK 4.14 gives a widget no accessibility context; the drag's screen-reader announcement asked for it and gtk-rs aborted the process.

## Changes

- `transfer/engine.rs`, `transfer/node.rs`, `gio_node.rs`: a new `Node::flush`; local destinations `syncfs` their file system after the copy of a move is published and before any original is removed. A drive that cannot write it out keeps every original and says so.
- `transfer/modes.rs`, `gio_node/local_delete.rs`: `keep_without_modes`: a mode change refused with EPERM, ENOSYS or ENOTSUP on a folder the user owns counts as done; a folder of another account, or any other error, still stops the copy.
- `transfer/limits.rs`, `gio_node.rs`, `search.rs`: `mounted_kind` reads the mount table when GIO says `fuse`: `fuseblk` and the FUSE subtypes of NTFS, exFAT and FAT get their rules.
- `window/file_ops/progress.rs`: a batch report that replaces a queued file report keeps its byte counts.
- `announcement.rs`: no announcement, and no context asked for, with `GTK_A11Y=none`.
- `po/openxplorer.pot` regenerated (no `.po` changed).

## Tests run

- New tests, each failing before its fix: `the_copy_reaches_its_drive_before_an_original_is_removed`, `a_folder_of_ours_on_a_file_system_without_unix_modes_keeps_the_copy_going`, `a_fuse_drive_takes_the_rules_of_the_file_system_in_the_mount_table`, `a_small_first_file_does_not_jump_the_percentage_to_its_share_of_the_items`, `accessibility_turned_off_is_never_asked_for_a_context` (plus the existing announcement test, which aborted under `GTK_A11Y=none` and passes now).
- Real input, before and after: the cross-drive move read back from the device image with debugfs (8 of 8 files on the device the moment the move ends, 0 of 8 before); copy of the whole set to a FUSE stick; NTFS stick asking and renaming 19 odd names; percentage 14% at 15% written; plain, Ctrl, Shift and Alt drags without a crash.
- `python3 native/parity/check.py` passes. `python3 native/tools/check.py` stops at clippy on files this change does not touch (newer clippy's `duration_suboptimal_units`, same on main); every test binary was then run isolated as check.py runs them, compared with unchanged main by test name (see the status file).

## Known limitations

- A plain copy to a USB stick is not flushed when the progress window closes; only a move's originals wait for it. A sparse file is copied as a full one (GIO writes its holes).
- The "Move by copying?" question for moves to another drive stays: it is an open owner decision (XFER-011).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01CQbN3MoLus4j67poby5Q4S
