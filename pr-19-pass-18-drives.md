# Title

fix(drives): Handle pulled-out and busy drives, rename drives and case-only names, and add opt-in permanent delete

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/18-pass-17-shortcuts)

## What changes for the user
- Answering a question in the progress window (Skip, Try Again, Cancel or OK) no longer leaves the keyboard on a button that is being removed. GTK used to walk the freed question's widgets afterwards and read freed memory (`Gtk-CRITICAL: gtk_widget_is_ancestor`), which can crash the app; the action now lets go of the keyboard first.
- A stick pulled out during a copy says so, as Windows 11 does. The item fails once with "Interrupted Action", "An unexpected error is keeping you from copying the file" and "Error 0x800701B1: A device which does not exist was specified.", with Try Again, Skip and Cancel. After Skip nothing more is asked: a "Location is not available" box (red X, OK only) says the folder is unavailable, and OK ends the copy. A folder deleted on a drive that is still mounted is still "Item Not Found".
- Every tab on a drive of this computer that goes away moves to the Desktop at once, without a message (Home when there is no Desktop folder), as in Windows 11. A disconnected share still shows "This location is unavailable" with Try again.
- Eject, Safely remove or Disconnect on a drive a program still uses shows Windows 11's box instead of GTK's list of busy programs: a warning icon, "'<drive>' is currently in use.", Windows' two explanations, and Cancel (the default), Try Again and Continue. Continue removes the drive anyway.
- New setting, Settings > Files & folders > Deleting > "Delete files on removable drives permanently (like Windows)", off by default and stored only when on. On, Delete on a drive that can be ejected or powered off asks Windows 11's "Are you sure you want to permanently delete ..." question and deletes for good. Off, as before, the items go to the drive's own Recycle Bin (`.Trash-<uid>`). Items elsewhere in the same selection keep their way.
- Renaming a file to other case only (photo.txt to Photo.txt) works on FAT32 and exFAT drives, through a private staging name; if the second step fails, the item gets its old name back. Undo works the same way. Windows 11 silently keeps the old name here, so this goes beyond Windows.
- A drive whose FUSE driver stopped answering (ntfs-3g or exfat-fuse after the stick went away) shows, for example, "/media/demo/STICK is not accessible. The device is not ready." instead of "Transport endpoint is not connected"; the system's words are under Details.
- A removable drive the app can format can be renamed from its menu (the strip and the classic menu): a Rename dialog with the volume label, fitted as it is typed to what the file system keeps, set through UDisks' `SetLabel`. FAT32 and ext4 are relabeled while mounted; exFAT and NTFS drives are unmounted, relabeled and mounted again. Tabs on the drive follow it to its new folder. Local Disk keeps Rename grayed out, now saying "Only a removable drive can be renamed here.".
- Inside a drive mounted on this computer, the address bar's crumbs start at This PC and the drive's name as This PC shows it (This PC > STICK > Photos), and Up from the drive's root opens This PC. Local Disk, network shares and "Show the full path" are unchanged.

## Why
Removable drives were tested against Windows 11 with real drives (loop-backed FAT32, exFAT, NTFS and ext4 images under a real udisks2 and the GVfs udisks2 volume monitor, a large stick, a read-only stick, an ISO, a split image and a slow FUSE stick for pulling a drive out mid-copy). The progress window read freed widgets on about four answers in five while copying onto a stick that refuses some names. A pulled-out stick was reported as "Item Not Found" for an item still in place, then asked about every entry being copied. Windows on a gone stick stayed on it. A busy drive showed GTK's dialog. A case-only rename on FAT32 or exFAT was refused because the drive saw the new name taken by the item itself. A dead FUSE driver showed a raw transport error. Rename was grayed out on every drive. The crumbs inside a stick read "/ > media > user > STICK > Photos", and Up from its root opened /media/user.

The copy question uses PR 16's `NOT_READY` wording, and three older tests now expect PR 16's Windows wording.

## CHANGED OWNER DECISIONS
None. Delete on a removable drive still uses the drive's Recycle Bin by default, as the owner decided; permanent delete is an opt-in setting. Commands on a read-only drive stay on and show the system's error (owner decision DEV-015), as before.

## New and changed visible text
New:
- "A device which does not exist was specified."
- "{location} is unavailable. If the location is on this PC, make sure the device or drive is connected or the disc is inserted, and then try again. If the location is on a network, make sure you're connected to the network or Internet, and then try again. If the location still can't be found, it might have been moved or deleted."
- "{location} is not accessible. The device is not ready."
- "'{drive}' is currently in use."
- "Save any open files on this disc, and then close the files or programs using the files before trying again."
- "If you choose to continue, the files will be closed, which might cause data to be lost."
- "Deleting"
- "Delete files on removable drives permanently (like Windows)"
- "Delete on a USB stick or SD card asks, then deletes for good. Off: the items go to the drive's own Recycle Bin and keep using its space until it is emptied."
- Settings search words for the new setting: "delete permanently removable drive usb stick sd card memory card recycle bin trash space windows erase"
- "Rename {drive}"
- "Renaming…"
- "Only a removable drive can be renamed here."
- "This drive cannot be renamed."
- "Renaming a drive needs the UDisks service, which is not available."
- "You are not allowed to rename this drive."
- "The drive was not renamed."
- "Could not rename the drive: {error}"
- "The drive was renamed, but it could not be mounted again: {reason}"

Changed:
- "Drive names cannot be changed here." → "Only a removable drive can be renamed here."
- "This device is currently in use. Close any programs or windows that might be using the device, and then try again." → "'{drive}' is currently in use." with "Save any open files on this disc, and then close the files or programs using the files before trying again." and "If you choose to continue, the files will be closed, which might cause data to be lost."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix's test, seen failing before the fix: `window::tests::worker_questions::an_answered_question_does_not_keep_the_keyboard`, `a_destination_that_went_away_is_said_to_be_unavailable`; `transfer::item_failure::tests::an_entry_on_a_drive_that_went_away_fails_its_item_without_asking`, `an_item_on_a_drive_that_went_away_is_not_ready`, `after_skip_on_a_drive_that_went_away_the_run_ends_with_location_unavailable`, `staging_left_on_a_drive_that_went_away_is_not_told_on_its_own`; `transfer::gone_drive::tests`; `window::tests::disconnected::every_tab_on_a_pulled_out_drive_goes_to_the_desktop`, `an_unmount_seen_by_the_folder_watch_goes_to_the_desktop` and the share variants; `window::tests::devices::a_drive_still_in_use_asks_in_windows_words`; `window::tests::removable_delete::delete_on_a_removable_drive_can_delete_permanently_as_windows_does`, `delete_on_a_removable_drive_uses_the_recycle_bin_by_default`; `ops::delete_plan::tests::items_on_a_removable_drive_can_be_deleted_permanently`; `settings::preferences::tests::deleting_on_removable_drives_permanently_is_opt_in`; `settings_page::tests::the_removable_drives_delete_switch_is_off_by_default_and_saved_when_on`; `ops::rename::tests::a_case_only_rename_works_on_a_drive_that_ignores_case`; `window::empty_page::tests::a_drive_that_stopped_answering_is_not_ready`; `window::place_menus::tests::rename_changes_a_removable_drives_label`; against a fake UDisks, `integration::drive_format::tests::a_drive_is_renamed_with_a_label_its_file_system_keeps`, `an_exfat_or_ntfs_drive_is_renamed_unmounted_and_mounted_again`, `a_gone_protected_or_refused_drive_is_not_renamed`; `window::location_view::tests::crumbs_inside_a_drive_start_at_this_pc_and_the_drive`, `up_from_a_drives_root_opens_this_pc`. By hand with real input (xdotool/XTEST, private Xvfb, unprivileged user, throwaway home): a slow FUSE stick pulled out while 400 files were copied onto it; Disconnect through a real udisks2 while a program held a file on the stick; the new delete setting; case-only renames and Undo on exfat-fuse and a fusefat FAT32 mount; renaming exFAT, NTFS, FAT32 and ext4 drives through the real UDisks; live plug-in, copying onto NTFS and exFAT, formatting, drive Properties, a read-only stick, an ISO, three partitions of one image and 50,000 files on a 32 GB stick, with no CRITICAL, panic or signal under `G_DEBUG=fatal-criticals`. `native/parity/check.py`, the i18n check, rustfmt and the docs sync passed, and Clippy had no findings in the changed files. Every ox-app test isolated with `check.py`'s `run_isolated` as an unprivileged user: 2,808 passed; six failed only while the stick from the live checks was still mounted and pass with it unmounted; other failures (root-only admin copy/delete tests and two tests that need the Python app's fixtures) also fail on unchanged main.

These tests named above are not in this PR, because they were dropped as redundant: `an_item_on_a_drive_that_went_away_is_not_ready`, `after_skip_on_a_drive_that_went_away_the_run_ends_with_location_unavailable`, `staging_left_on_a_drive_that_went_away_is_not_told_on_its_own`, `a_drive_still_in_use_asks_in_windows_words`, `the_removable_drives_delete_switch_is_off_by_default_and_saved_when_on`, `a_drive_that_stopped_answering_is_not_ready`, `a_drive_is_renamed_with_a_label_its_file_system_keeps` and `crumbs_inside_a_drive_start_at_this_pc_and_the_drive`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
