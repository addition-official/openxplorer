Model: claude-opus-5-5

# feat(drives): Copy a drive and paste or drag it into a folder as a shortcut, as Windows does

Builds on AKolenda/openxplorer#268 (`this-pc-menu`), which gave drives Windows 11's menu with Copy greyed out. This branch enables Copy on drives and makes a paste of a copied drive create a shortcut to it, so a drive's contents are never copied by Paste or by a drag.

## Summary

- **Copy is enabled on a drive** wherever Windows offers it: the drive menu's icon strip (This PC's tiles and the sidebar's drive rows), the classic menu's Copy row, Ctrl+C on a selected This PC tile or on a sidebar drive row that has keyboard focus, and the command bar's Copy while a drive tile is selected on This PC. Rename stays greyed out, as before.
- **Paste into a folder makes a shortcut** to the drive's mount point, named after the drive as This PC shows it: `Local Disk - Shortcut`, then `Local Disk - Shortcut (2)`. Windows names it `Local Disk (C:) - Shortcut`, but the app shows no drive letters. The shortcut is the app's existing kind, a symbolic link named from the `%s - Shortcut` template (the code behind Send to ▸ Desktop (create shortcut) and Create shortcuts here). Undo moves it to the Recycle Bin, and the new shortcut is selected after the paste.
- **Dragging a drive into a folder makes a shortcut**, with the drag label `Create link in <folder>`. A right-drag opens the drop menu offering only **Create shortcuts here** (in bold) and Cancel. This PC's drive tiles can now be dragged, as in Windows. A sidebar drive row is dragged with the drive icon.
- **Several drives:** Select all on This PC (AKolenda/openxplorer#259) can select several drive tiles. Copy copies all of them, and a paste makes one shortcut per drive. A pinned drive's Quick access row has the drive menu (from #268), so its Copy is enabled too.
- **Dragging tiles:** since #259 the tile boxes select several tiles, and a drag that starts on a tile would draw the box's rubber band. The drive drag source sits on the This PC page in the capture phase, so a drag that starts on a drive tile carries the drive, while a drag from empty space still draws the band.
- **Cut stays unavailable** on a drive: the menus have no Cut, the command bar's Cut stays greyed, and Ctrl+X on a drive does nothing.
- **Other apps never get a drive as a folder:** a copy that names a drive is published only in this app's own clipboard format and as its path in text, never as GNOME's or the standard file list, which another file manager would paste as a folder, copying the whole drive. A drag of a drive offers other apps a link alone, as a drive's drop effect in Windows is a link.
- **No drive in the Recycle Bin:** a drive dropped on the Recycle Bin is refused before anything runs ("A drive cannot be moved to the Recycle Bin."), alone or with other items.
- **Phones and cameras:** their Copy is greyed out, saying why ("A phone or camera cannot be copied. Open it and select what to copy."), because their root has no path a shortcut could point to. Ctrl+C skips them.
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
  - ox-app: `file_drop::tests::a_drive_dropped_on_the_recycle_bin_is_refused`, `file_drop::action::tests::a_drag_that_offers_a_link_alone_is_answered_with_the_link`, `place_menus::tests::a_phone_cannot_be_copied_and_a_drive_can`, `drive_shortcuts::tests::*`, `tests::landing_pages::every_selected_drive_pastes_as_its_own_shortcut`, `file_ops::availability::tests::a_selected_drive_enables_copy_alone`, `tests::landing_pages::a_copied_drive_tile_pastes_as_a_shortcut_named_after_the_drive`, `tests::landing_pages::a_drive_is_copied_from_its_menu_and_never_cut`, `tests::drag_and_drop::a_dropped_drive_becomes_a_shortcut_and_the_other_items_are_copied`, and the updated drive-menu tests in `place_menus`, `landing_pages` and `sidebar`.
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

- After merging upstream main up to bab3fb5 (AKolenda/openxplorer#274) and the newest `this-pc-menu`: rustfmt and Clippy (1.92.0, `-D warnings`) are clean and `parity/check.py` passes. The named tests (ox-core `ops::shortcuts`; ox-app `drive_shortcuts`, `file_ops::availability`, `place_menus`, `tests::landing_pages`, `tests::sidebar`, `tests::drag_and_drop`, `tests::right_drag`, `tests::clipboard`, `file_drop`) pass in the isolated environment as an unprivileged user (8 and 177 tests). The real-input checklist below passes again at 1x light and 2x dark. The user tested deb 20 and found that a pasted drive made "Local Disk - Shortcut", which opened the drive.

- After the review fixes (02c903f): rustfmt and Clippy (1.92.0, `-D warnings`) are clean and the translation template is regenerated. The named ox-app tests pass (180) in the isolated environment as an unprivileged user. The real-input checklist passes 32/32 at 1x light and 2x dark, including: a copied drive offers other apps no file-list format (only `application/x-winspace-files` and text, read with xclip) while Ctrl+V in this app still makes the shortcut, and Local Disk dropped on the Recycle Bin is refused with nothing trashed.
- **Not changed:** a paste whose clipboard holds a drive together with ordinary items records two Undo steps, the shortcuts and then the copy. Each step undoes its own part correctly.

## Notes

- Clicking a sidebar row moves keyboard focus to the file list, which is existing behavior. Ctrl+C on a sidebar drive row therefore works when the row has keyboard focus (Tab, F6 or Shift+F6). In Windows, focus stays in the navigation pane after a click.
- Opening the shortcut browses the drive through the link's path (`…\Local Disk - Shortcut`), as the app's other folder shortcuts do. Windows jumps to `C:\`.
- Found during QC, and not caused by this branch: after a drop into the file list and a visit to This PC, the file list ignores the next drag. GTK logs `gtk_drop_target_async_handle_event: assertion 'self->drop == drop' failed`. It reproduces with an ordinary folder dragged from the sidebar. The same bug also leaves GTK's drop highlight around the sidebar after a drop on it, with an ordinary file too.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
