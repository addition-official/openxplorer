# fix(properties): Show a folder that is a mount point as a folder

## Summary

- Properties showed the drive page for any local folder in the kernel's mount table: Local Disk, the file system, the used and free ring, and a Hardware tab. So `/tmp`, a tmpfs mount point, opened from its parent's listing as a drive.
- `PropertiesTarget` now carries `as_drive`. An item of a folder's listing never shows as a drive. Properties of the folder shown, and Properties opened from the navigation pane, This PC or another application, show the drive page only for the root (Local Disk) and the drives This PC lists, when they are mount points. Any other mount point shows the folder's own Properties: General with Size, Contains and its attributes, Sharing, Security, Previous Versions and Customize.
- The root keeps its drive page.

## Windows 11's behavior

- In Windows, the drive page (Type, File system, used and free space, the ring, and the Hardware tab) belongs to the volumes This PC lists. A folder in a listing opens as a folder.
- On Windows a volume can also be mounted in an empty NTFS folder ([Mount a drive in a folder](https://learn.microsoft.com/en-us/windows-server/storage/disk-management/assign-a-mount-point-folder-path-to-a-drive), Microsoft Learn), and the drive is then reached through that folder path. That page does not describe the Properties dialog of such a folder. This change shows a Linux mount point reached as a folder as a folder; it does not add a separate "Mounted Volume" page.

## Testing

- rustfmt is clean, and `parity/check.py` passes.
- Clippy (`--workspace --all-targets -D warnings`) is clean for this change. It was run with Rust 1.97, not the CI toolchain 1.92. Three pre-existing lints that only the newer Clippy raises (`duration_suboptimal_units` in ox-core, `map_unwrap_or` in `window/group_day.rs`) were allowed for that run.
- Under `xvfb-run` and `dbus-run-session` with a temporary HOME, after merging main at 8fea905c: 56 tests passed. These were every `item_dialogs` test, every `properties_width` test, and the Customize tests. The new `a_mount_point_reached_as_a_folder_shows_as_a_folder` test opens a mount point the machine has (`/dev` or `/proc`) from the root's listing and through the navigation pane's action, and expects folder Properties. `properties_show_a_links_target_and_a_drives_space` checks that the root keeps its drive page. The branch also captures a mount point as a folder and the root as a drive.
- The full `tools/check.py` was not run.

## Overlap with other pull requests

- The fork branch `properties-width` (proposed separately as a follow-up to #243) merges without conflicts. With both merged, its new test `another_drive_has_customize_and_its_tabs_share_the_rows` fails, because it opens `/dev/shm` through `properties-of`, which this change shows as a folder. Whichever lands second needs a small test change.
- #255 (folder templates) merges cleanly.
- #256 (Read-only and the Security tab) conflicts only in `native/crates/ox-app/src/window/tests/item_dialogs.rs`, where both add tests. Keeping both sides resolves it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
