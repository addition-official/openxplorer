Model: claude-opus-5-5

# feat(drives): Copy a drive and paste or drag it into a folder as a shortcut, as Windows does

Builds on AKolenda/openxplorer#268 (`this-pc-menu`), which gave drives Windows 11's menu with Copy greyed out. This branch enables Copy on drives and makes a paste of a copied drive create a shortcut to it, so a drive's contents are never copied by Paste or by a drag.

## Summary

- **Copy is enabled on a drive** wherever Windows offers it: the drive menu's icon strip (This PC's tiles and the sidebar's drive rows), the classic menu's Copy row, Ctrl+C on a selected This PC tile or on a sidebar drive row that has keyboard focus, and the command bar's Copy while a drive tile is selected on This PC. Rename stays greyed out, as before.
- **Paste into a folder makes a shortcut** to the drive's mount point, named after the drive as This PC shows it: `Local Disk - Shortcut`, then `Local Disk - Shortcut (2)`. Windows names it `Local Disk (C:) - Shortcut`, but the app shows no drive letters. The shortcut is the app's existing kind, a symbolic link named from the `%s - Shortcut` template (the code behind Send to ▸ Desktop (create shortcut) and Create shortcuts here). Undo moves it to the Recycle Bin, and the new shortcut is selected after the paste.
- **Dragging a drive into a folder makes a shortcut**, with the drag label `Create link in <folder>`. A right-drag opens the drop menu offering only **Create shortcuts here** (in bold) and Cancel. This PC's drive tiles can now be dragged, as in Windows. A sidebar drive row is dragged with the drive icon.
- **Cut stays unavailable** on a drive: the menus have no Cut, the command bar's Cut stays greyed, and Ctrl+X on a drive does nothing.
- **Safety rule:** Paste and every drop check each item against the drives This PC lists (Local Disk and each mounted drive or device). Each drive gets a shortcut, wherever it came from, and only the other items are copied or moved. A drive's contents therefore cannot be copied through Paste or a drop, even when another app put its mount point on the clipboard.
- ox-core: `create_shortcut_named` and `create_named_shortcuts` take the name a shortcut is made from, because `/` has no file name of its own. `create_shortcuts` now calls them.

## Windows' behavior, as checked

- Dragging a drive makes a link. A MakeUseOf guide (Windows 10) shows the "Create link in Desktop" tooltip when dragging a USB drive out of the navigation pane. ReactOS, which follows Windows, sets `m_dwDefaultEffect = DROPEFFECT_LINK; // Don't copy a drive by default` (ReactOS bug CORE-17871: "When dragging disks with the left mouse button, disk shortcuts should be created").
- "Windows can't create a shortcut here. Do you want the shortcut to be placed on the desktop instead?" appears for **Create shortcut** inside This PC (Microsoft KB 179513; a 2025 Microsoft Q&A answer for Windows 11). It does not appear for a paste into an ordinary folder, so no question is asked here.
- New shortcuts have been named `<name> - Shortcut` since Windows Vista. A taken name gets ` (2)`, as the existing shortcut code already does.
- The task states that Windows 11 creates a shortcut when a copied drive is pasted. No published source confirming this was found, and an informal Windows 10 report says nothing happens. The implementation follows the task, since it is the only behavior that is both useful and safe.

## Testing

- rustfmt and Clippy are clean (Rust 1.92.0, the CI toolchain, with `-D warnings`). `python3 native/parity/check.py` passes, and the translation template was regenerated with `tools/i18n.py extract`.
- New or changed tests, run by name:
  - ox-core `ops::shortcuts`: `a_drive_shortcut_is_named_after_the_drive_and_numbered`, `local_disk_gets_a_shortcut_to_the_root`
  - ox-app: `drive_shortcuts::tests::*`, `file_ops::availability::tests::a_selected_drive_enables_copy_alone`, `tests::landing_pages::a_copied_drive_tile_pastes_as_a_shortcut_named_after_the_drive`, `tests::landing_pages::a_drive_is_copied_from_its_menu_and_never_cut`, `tests::drag_and_drop::a_dropped_drive_becomes_a_shortcut_and_the_other_items_are_copied`, and the updated drive-menu tests in `place_menus`, `landing_pages` and `sidebar`.
- Whole suites, in the isolated environment of `tools/check.py` (private X display and session bus): ox-core lib 1180/1180. ox-app lib, as an unprivileged user: 1971 passed, 2 failed. Both failures came from running as a user with a real home directory, and both pass in the same environment as root. Two earlier full runs were stopped by a crash on a GLib worker thread. It did not recur in the final full run or in a separate run of `window::tests`, and its cause is unknown.
- Real app, run under Xvfb as an unprivileged user with fictional files and driven by real pointer and key input (xdotool), at 1x and 2x, light and dark:
  - [x] Copy Local Disk with the command bar's Copy on This PC, then Ctrl+V in a folder: `Local Disk - Shortcut` → `/` is created and selected
  - [x] Paste twice: the second is `Local Disk - Shortcut (2)`
  - [x] Opening the shortcut lists the drive
  - [x] Ctrl+C on a selected This PC tile, then paste: shortcut
  - [x] The tile menu's Copy, then paste: shortcut
  - [x] The sidebar's Local Disk menu Copy, then paste: shortcut
  - [x] Ctrl+C on the sidebar's Local Disk row (keyboard focus), then paste: shortcut
  - [x] Cut disabled: the command bar's Cut and Ctrl+X on a drive leave the clipboard unchanged
  - [x] Drag the sidebar's Local Disk into a folder (`Create link in Projects`): shortcut, nothing copied
  - [x] Drag This PC's Local Disk tile onto the sidebar's Documents: shortcut in Documents
  - [x] Right-drag Local Disk into a folder: the menu offers only Create shortcuts here, which makes one
  - [x] Copying a normal folder is unchanged: a real copy

## Notes

- Clicking a sidebar row moves keyboard focus to the file list, which is existing behavior. Ctrl+C on a sidebar drive row therefore works when the row has keyboard focus (Tab, F6 or Shift+F6). In Windows, focus stays in the navigation pane after a click.
- Opening the shortcut browses the drive through the link's path (`…\Local Disk - Shortcut`), as the app's other folder shortcuts do. Windows jumps to `C:\`.
- Found during QC, and not caused by this branch: after a drop into the file list and a visit to This PC, the file list ignores the next drag. GTK logs `gtk_drop_target_async_handle_event: assertion 'self->drop == drop' failed`. It reproduces with an ordinary folder dragged from the sidebar.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
