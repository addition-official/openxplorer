# Title

fix(drives): Pulled-out drives, drive rename, drive crumbs and a progress-window crash

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 18 tested USB and other removable drives against Windows 11 with real drives: loop-backed FAT32, exFAT, NTFS and ext4 images handled by a real udisks2 2.10 and the GVfs udisks2 volume monitor, a 32 GB ext4 stick with 50,000 files, a read-only stick, an ISO image, one image split into three partitions, and a slow FUSE stick (about 2 MB/s) for pulling a drive out mid-copy. Input was real (xdotool/XTEST) on a private Xvfb, as an unprivileged user with a throwaway home. These are the fixes:

- **A crash-class bug in the progress window.** Clicking Skip, Try Again, Cancel or OK in the progress window left the window's keyboard on that button after its question had been taken out of the window. GTK then moved the keyboard on by itself after the next paint, walked the action's widgets while the old question was being freed, and read a freed widget (`Gtk-CRITICAL: gtk_widget_is_ancestor: assertion 'GTK_IS_WIDGET (widget)' failed`). It happened on about four answers in five while copying onto a USB stick that refuses some names (`a:b.txt`, `what?.txt` on NTFS). A read of freed memory can crash the app. The action now lets go of the keyboard before it hides or removes a question, or the whole action, that holds it.
- **A stick pulled out during a copy now says so.** The question used to read "Item Not Found: Could not find this item. This is no longer located in <source folder>", blaming an item that was still where it was. It then asked again about every entry of the folder being copied. The engine now notes which drive held the destination folder when the run started. When that drive has left the mount table, or its folder answers with an error only a missing device gives, the item fails once, as Windows says it: "Interrupted Action" with "The device is not ready.". The note about staging left on that drive says the same instead of "No such file or directory". A folder deleted on a drive that is still mounted is still "Item Not Found".
- **A drive whose driver stopped answering says it is not ready.** An exFAT or NTFS stick whose FUSE driver quit (ntfs-3g and exfat-fuse do when their stick goes away) showed "Error opening directory '...': Transport endpoint is not connected". The page now reads "/media/demo/STICK is not accessible. The device is not ready.", as Windows says it of a drive that is not there. The system's words are under Details.
- **Rename a removable drive.** Rename was grayed out on every drive. On a removable drive the app can format, Rename (in the strip and the classic menu) now opens a Rename dialog with the volume label, fitted as it is typed to what the file system keeps. Rename sets the label through UDisks' `SetLabel`. FAT32 and ext4 are relabelled while mounted. exfatlabel finds a mounted exFAT stick busy, and ntfslabel refuses a mounted NTFS one, so those drives are unmounted, relabelled and mounted again, as a format does. The tabs on the drive follow it to the folder its new label gives it. A drive in use is refused with "The drive is in use. Close the files and programs using it, then try again.". Local Disk keeps Rename grayed out, now saying "Only a removable drive can be renamed here.". Windows renames a drive in place in This PC; here the menu opens a dialog.
- **A drive's crumbs start at This PC.** Inside a USB stick the address bar read "/ > media > user > STICK > Photos", and Up from the stick's root opened /media/user. As in Windows 11 ("This PC > USB Drive (E:) > Photos"), the crumbs inside a drive mounted on this computer now start at This PC and the drive's name as This PC shows it, and Up from its root opens This PC. Local Disk, network shares and "Show the full path" are unchanged.

Merged the lower-numbered jobs that change the same files: pass-9-scaling (with passes 1, 2, 5 and 8), network-like-windows, pass-3, pass-4, pass-6, pass-7, pass-10, pass-11, pass-12, pass-14, pass-15, pass-16 and pass-17. The copy question uses pass-15's `NOT_READY`, and three older tests now expect pass-15's Windows wording. pass-4's commit 5cb3a27c (computers under Network in the navigation pane) conflicts throughout the sidebar with network-like-windows, which does the same thing its own way. So pass-4 came in with that one commit reverted on top, and network-like-windows' version is kept.

## Tests

Every fix has a test that failed before the fix and passes after it:

- `window::tests::worker_questions::an_answered_question_does_not_keep_the_keyboard`: the keyboard stayed on the detached Skip button.
- `transfer::item_failure::tests::an_entry_on_a_drive_that_went_away_fails_its_item_without_asking`, `transfer::item_failure::tests::an_item_on_a_drive_that_went_away_is_not_ready`, and `transfer::gone_drive::tests` (mount-table cases).
- `window::empty_page::tests::a_drive_that_stopped_answering_is_not_ready`.
- `window::place_menus::tests::rename_changes_a_removable_drives_label`. Against a fake UDisks: `integration::drive_format::tests::a_drive_is_renamed_with_a_label_its_file_system_keeps`, `an_exfat_or_ntfs_drive_is_renamed_unmounted_and_mounted_again`, `a_gone_protected_or_refused_drive_is_not_renamed`. The exFAT/NTFS unmount path was found and checked against the real UDisks.
- `window::location_view::tests::crumbs_inside_a_drive_start_at_this_pc_and_the_drive`, `up_from_a_drives_root_opens_this_pc`.

Checked by hand with real input against the built app: live plug-in (This PC and the pane update with name, icon and capacity bar); copying onto NTFS and exFAT (names NTFS cannot hold get Try Again / Skip / Cancel); pulling a stick out mid-copy (forced lazy unmount, as udisks does on removal, and a killed FUSE driver); pulling it out while 500 thumbnails were being made; Disconnect while a program held a file (GTK's busy dialog names the program); renaming exFAT, NTFS, FAT32 and ext4 drives through the real UDisks; formatting FAT32 to exFAT; drive Properties; a read-only stick; an ISO; three partitions of one image; and 50,000 files on a 32 GB stick (first rows at about 2 s, all listed by about 5.6 s cold in a debug build; sorting, End and Ctrl+A each about a second). None of these logged a CRITICAL, panic or signal under `G_DEBUG=fatal-criticals`.

`native/parity/check.py` passes, and so do the i18n check and rustfmt. Every test executable was run with `check.py`'s `run_isolated` as an unprivileged user, with the Python app's sources provided. Compared by name with unchanged main, the only new failures are older tests from other jobs that expect raw system text or paths that pass-15 now says in Windows' words (archive and transfer error names, a zip, Default apps, an offline share, a read-only folder). Those need an owner call between pass-3 and pass-15, and are listed in the status file. The Rename and Format screenshots come from a local build that counts loop devices as removable. That one line was not committed, because loop devices are never removable.

## Not changed, for the owner

- Delete on a USB stick moves the item to the stick's own `.Trash-<uid>` (OPS-018, freedesktop trash). Windows deletes permanently from a removable drive after asking. The space only comes back once the Recycle Bin is emptied, and a later copy was refused for lack of space because of it.
- Commands on a read-only drive stay on and show the system's error (owner decision DEV-015), as before.
- No AutoPlay-style notice on plugging a drive in. GNOME and KDE already notify, so an in-app one would need to be an opt-in setting.
