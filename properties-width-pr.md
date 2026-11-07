# fix(properties): Give every Properties window one fixed size, with tabs in even rows

Follows up #243, which kept Properties at Windows' width for one item, however long its name. A drive's Properties and Properties of several items could still widen the window, because the notebook's single tab row cannot shrink. This change moves the tabs onto rows the way Windows' property sheets do.

## Summary

- **One fixed size for every Properties window.** A file, a shortcut, several items, a folder, a drive and a mount point all open at the same width and height, on every tab, at 402 x 570 pixels at 100%. Before, only the width was fixed: two files' Properties came out about 400 x 395, a folder's 545 tall and another drive's 570, and a file's grew when Previous Versions was shown. The height under the title bar is fixed at 528 pixels, which holds the tallest page here (a drive's Customize under two rows of tabs) at the default text size. Shorter pages leave empty space at the bottom, as in Windows. The pages scroll inside the window at a larger text size, or on a screen too short for it. The Security tab's Edit and Advanced dialogs and Advanced Sharing still fit their content.
- **Tabs in rows.** The tabs are now buttons in a new `TabRows` widget above the notebook. The notebook's own tab row is hidden, and the notebook still holds the pages. Tabs that do not fit at the window's width take another row instead of widening the window.
- **Even rows, as in Windows' multi-row tabs.** Tabs use as few rows as they need, spread over the rows evenly and in order, with the extra tab in the first rows. Every row is stretched to the full width, so no tab sits alone stretched across a row while another row is full. The row of the tab shown sits next to the page.
- **Keyboard and screen readers.** The tab rows are one stop of the Tab key. Left, Right, Home and End move between tabs. Ctrl+Tab, Ctrl+Shift+Tab, Ctrl+Page Down and Ctrl+Page Up show the next or previous tab from anywhere in the window. Screen readers hear a tab list whose shown tab is selected, and each page is labeled by its tab.
- **No Customize on the system drive.** The root (`/`, Local Disk) has no Customize tab, so its five tabs (General, Hardware, Sharing, Security, Previous Versions) fit one row. Another drive has Customize last, but only where the choice can be saved: a local drive mounted read-write whose files take GVfs metadata, which is where the Customize tab stores its choices. Network and FUSE drives and read-only drives have no Customize. Only the mount table is read before GIO, so a share that stopped answering is never touched.
- **General's check boxes wrap.** Boxes such as "Read-only (Only applies to files in folder)" now wrap instead of widening the window with a wide font or a larger text size, for one item and for several.

## Windows 11's behavior

- Microsoft Learn, [Property Sheets](https://learn.microsoft.com/en-us/windows/win32/controls/property-sheets): "The property sheet is automatically sized to accommodate the largest page" and "the size of the property sheet cannot change after it has been created." The recommended medium page is 227 x 215 dialog units (`PROP_MED_CXDLG`, `PROP_MED_CYDLG` in `prsht.h`). In Windows 11 that comes to roughly 380 x 520 pixels at 100% for the whole sheet. This app's text is a little larger, so its pages need 402 x 570; the width is the one #243 set.
- Local Disk (C:) Properties on Windows 11, checked in a Windows 11 VM: there is no Customize tab. The seven tabs take two rows, each filling the window's width: General, Tools, Hardware and Sharing in front, next to the page, and Security, Previous Versions and Quota behind. The window keeps its width.
- Customize appears on folders and on drives other than the system drive, per the ElevenForum tutorial [Add or Remove "Customize this folder" on Context Menu and Properties in Windows 11](https://www.elevenforum.com/t/add-or-remove-customize-this-folder-on-context-menu-and-properties-in-windows-11.9395/).
- Tools and Quota have no equivalent here and are not added.

## Testing

- rustfmt is clean, and `parity/check.py` passes.
- Clippy (`--workspace --all-targets -D warnings`) is clean for this change. It was run with Rust 1.97, not the CI toolchain 1.92. Three pre-existing lints that only the newer Clippy raises (`duration_suboptimal_units` in ox-core, `map_unwrap_or` in `window/group_day.rs`) were allowed for that run.
- `every_kind_of_properties_has_one_size` opens Properties of a file, a shortcut, two files, a folder, the root, a mount point opened by its path, another drive listed on This PC, and a window with seven tabs in two rows. It shows every tab of each and checks that all of them have the same width and height, that the width is Windows' width, and that no page scrolls. `every_kind_of_properties_size_is_captured` captures each kind, light and dark, as `properties-size-*.png`.
- Under `xvfb-run` and `dbus-run-session` with a temporary HOME, after merging main at 8fea905c: 111 named Properties tests passed (`properties`, `tab_rows`, `item_dialogs`, `customize`, `details_tab`, `security`, `sharing`, `dialog_keys`, `late_replies`). The one failure, `mount_assistant::the_assistant_prepares_a_command_and_hands_over_the_linux_folder`, also fails without this change in this environment. These were the `tab_rows` unit tests, `metadata::only_a_local_writable_drive_other_than_the_root_is_customized`, every `properties_width` test, every `item_dialogs` test, and the Customize tests. The new window tests are:
  - `wrapped_tabs_share_the_rows_evenly`: a window with Windows' seven drive tabs takes rows of three and four, each stretched to the full width, and the row of the tab shown moves next to the page;
  - `another_drive_has_customize_and_its_tabs_share_the_rows`: a writable local mount root other than the root (a disk's file system first, else a memory drive such as `/dev/shm`), listed on This PC, shows Customize last, with its six tabs in two rows of three, at the same width. The test is skipped when the machine has no such mount;
  - `every_kind_of_properties_keeps_the_width`: the root has no Customize tab and its tabs fit one row.
- `every_kind_of_properties_is_captured` writes light and dark captures of a folder, a file, several items, the root, another drive and the seven-tab window, for review with `OX_NATIVE_CAPTURE_DIR` and `GDK_SCALE=2`.
- `a_long_name_or_location_never_widens_properties` (from #243) fails now and then on its height check, also without this change. It failed 8 of 14 runs on the base.
- The full `tools/check.py` was not run.

## Overlap with other pull requests

- `mount-point-folder` (a mount point reached as a folder shows folder Properties) merges cleanly with this change in either order, and the merged tree passes the same named tests (113, including its `a_mount_point_reached_as_a_folder_shows_as_a_folder`, with the same one pre-existing failure) and `every_kind_of_properties_has_one_size`. This change's other-drive test lists its drive on This PC before opening it, as the volume monitor would.
- #255 (folder templates) merges cleanly.
- #256 (Read-only and the Security tab) merges cleanly. Both change `general_tab.rs` and `selection_attributes.rs`, but this change only adds wrapping to the check boxes there.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
