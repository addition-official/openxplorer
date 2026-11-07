# fix(properties): Keep Windows' width for drives and several items, with tabs in even rows

Follows up #243, which kept Properties at Windows' width for one item, however long its name. A drive's Properties and Properties of several items could still widen the window, because the notebook's single tab row cannot shrink. This change moves the tabs onto rows the way Windows' property sheets do.

## Summary

- **Tabs in rows.** The tabs are now buttons in a new `TabRows` widget above the notebook. The notebook's own tab row is hidden, and the notebook still holds the pages. Tabs that do not fit at the window's width take another row instead of widening the window.
- **Even rows, as in Windows' multi-row tabs.** Tabs use as few rows as they need, spread over the rows evenly and in order, with the extra tab in the first rows. Every row is stretched to the full width, so no tab sits alone stretched across a row while another row is full. The row of the tab shown sits next to the page.
- **Keyboard and screen readers.** The tab rows are one stop of the Tab key. Left, Right, Home and End move between tabs. Ctrl+Tab, Ctrl+Shift+Tab, Ctrl+Page Down and Ctrl+Page Up show the next or previous tab from anywhere in the window. Screen readers hear a tab list whose shown tab is selected, and each page is labeled by its tab.
- **No Customize on the system drive.** The root (`/`, Local Disk) has no Customize tab, so its five tabs (General, Hardware, Sharing, Security, Previous Versions) fit one row. Another drive has Customize last, but only where the choice can be saved: a local drive mounted read-write whose files take GVfs metadata, which is where the Customize tab stores its choices. Network and FUSE drives and read-only drives have no Customize. Only the mount table is read before GIO, so a share that stopped answering is never touched.
- **General's check boxes wrap.** Boxes such as "Read-only (Only applies to files in folder)" now wrap instead of widening the window with a wide font or a larger text size, for one item and for several.

## Windows 11's behavior

- Local Disk (C:) Properties on Windows 11, checked in a Windows 11 VM: there is no Customize tab. The seven tabs take two rows, each filling the window's width: General, Tools, Hardware and Sharing in front, next to the page, and Security, Previous Versions and Quota behind. The window keeps its width.
- Customize appears on folders and on drives other than the system drive, per the ElevenForum tutorial [Add or Remove "Customize this folder" on Context Menu and Properties in Windows 11](https://www.elevenforum.com/t/add-or-remove-customize-this-folder-on-context-menu-and-properties-in-windows-11.9395/).
- Tools and Quota have no equivalent here and are not added.

## Testing

- rustfmt is clean, and `parity/check.py` passes.
- Clippy (`--workspace --all-targets -D warnings`) is clean for this change. It was run with Rust 1.97, not the CI toolchain 1.92. Three pre-existing lints that only the newer Clippy raises (`duration_suboptimal_units` in ox-core, `map_unwrap_or` in `window/group_day.rs`) were allowed for that run.
- Under `xvfb-run` and `dbus-run-session` with a temporary HOME, after merging main at 8fea905c: 63 tests passed. These were the `tab_rows` unit tests, `metadata::only_a_local_writable_drive_other_than_the_root_is_customized`, every `properties_width` test, every `item_dialogs` test, and the Customize tests. The new window tests are:
  - `wrapped_tabs_share_the_rows_evenly`: a window with Windows' seven drive tabs takes rows of three and four, each stretched to the full width, and the row of the tab shown moves next to the page;
  - `another_drive_has_customize_and_its_tabs_share_the_rows`: `/dev/shm`, when the machine has it as a writable mount, shows Customize last, with its six tabs in two rows of three, at the same width;
  - `every_kind_of_properties_keeps_the_width`: the root has no Customize tab and its tabs fit one row.
- `every_kind_of_properties_is_captured` writes light and dark captures of a folder, a file, several items, the root, another drive and the seven-tab window, for review with `OX_NATIVE_CAPTURE_DIR` and `GDK_SCALE=2`.
- `a_long_name_or_location_never_widens_properties` (from #243) fails now and then on its height check, also without this change. It failed 8 of 14 runs on the base.
- The full `tools/check.py` was not run.

## Overlap with other pull requests

- The fork branch `mount-point-folder` (proposed separately) shows a mount point reached as a folder with folder Properties, and only the root and This PC's drives as drives. The two merge without conflicts. With both merged, `another_drive_has_customize_and_its_tabs_share_the_rows` fails, because it opens `/dev/shm` through `properties-of`, which then shows it as a folder. Whichever lands second needs a small test change: open the other drive the way This PC does, or skip when it shows as a folder.
- #255 (folder templates) merges cleanly.
- #256 (Read-only and the Security tab) merges cleanly. Both change `general_tab.rs` and `selection_attributes.rs`, but this change only adds wrapping to the check boxes there.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
