Model: claude-opus-5-5

# feat(menus): Lay out right-click menus as Windows 11 does, with labeled strip buttons

## Summary

- **Strip with labels.** The compact menu's top row draws each button as an icon with its label under it, with thin lines between buttons, as Windows 11 does: **Cut, Copy, Rename, Share, Delete**. Share shows only for files, and only where the desktop's email portal can share. **Paste** shows only on one folder, and only while there is something to paste (it pastes into that folder). It is hidden, not greyed out. The navigation pane's folder strip hides Paste the same way.
- **Compact item menu in Windows 11's order:** Open (Enter), Extract all... for an archive, Mount disk image, **Open with >** (the item's apps, then Choose another app, or "Open with..." alone), a folder's Open in new tab / Open in new window / Pin to Quick access, **Share with >** (Bluetooth device, Mail recipient, where the computer has them), **Compress to >** (ZIP File, Additional options = the Compress dialog with TAR.XZ), Copy as path (Ctrl+Shift+C), Properties (Alt+Enter), then a group of app-specific items: the code editors, where Windows has "Edit in Notepad", and a folder's Open in Terminal. Show more options is last.
- **Classic menu in Windows' classic order:** Open (bold), the editors, a folder's tab, window and pin commands, Open in Terminal, Open with; Copy as path, Share, Restore previous versions; Send to >; Cut, Copy, and Paste on a folder; Duplicate (where Windows has Create shortcut), Delete, Rename, Rename items...; the app's own tools (Compress to..., Calculate folder size, Analyze disk usage, the cache toggle, Sign out of server...); Properties.
- **Left out because they are Windows-only:** Edit, Print, Scan with Microsoft Defender, Give access to, Add to Favorites, Create shortcut and Pin to Start. Restore previous versions maps to the app's snapshot Previous versions, and Open in Terminal stays.
- **Group headings:** This PC's headings get Windows 11's heading menu: Collapse group with its key Left (Expand group, Right, while collapsed), Expand all groups, Collapse all groups. Windows also selects the group and adds the selection's item menu below a divider. This PC selects one tile at a time and has no menu for several drives, so that part is left out there. It opens on right-click, the Menu key and Shift+F10, in the style the setting picks. It has no Show more options because there is nothing more to show.
- **Strip nearest the pointer:** when GTK opens a menu above the pointer, the strip moves to the bottom, as Windows 11 keeps it on the side nearest the pointer.
- **Pinned folders in the navigation pane:** the compact menu is Windows 11's: Open in new tab, Open in new window, Unpin from Quick access, Compress to > (ZIP File, Additional options), Copy as path, New >, Properties, then Open in Terminal and Show more options (Pin to Start is Windows-only). Copy as path, Compress to and New act on that folder (`win.copy-path-of`, `win.compress-to-zip-of`, `win.compress-to-of`, `win.new-in`). New opens the folder and creates the item there. The strip shows only what the folder allows, so Downloads has Copy alone, and a pin of the user's own has Cut, Copy, Rename, Delete. The app's "Hide section" stays behind Show more options.
- **Settings > Appearance > Right-click menu** is unchanged. The Windows 10 · Classic choice opens the classic menu directly on files, folders, several items, the background, the sidebar, drives and group headings. Shift+right-click still opens the classic menu.

## Menus and sources

| Menu | Order now | Source |
|---|---|---|
| File, compact | strip Cut, Copy, Rename, Share, Delete; Open, Open with >, Share with >, Compress to >, Copy as path, Properties; editors; Show more options | Owner's Windows 11 screenshots (text file, dark mode); [Microsoft Support: File Explorer in Windows](https://support.microsoft.com/en-us/help/4028272) (top row and Show more options) |
| File, classic | Open, Open with...; Copy as path, Share, Restore previous versions; Send to >; Cut, Copy; Delete, Rename; Properties | Owner's Windows 11 screenshots (Show more options) |
| Folder, compact | strip Cut, Copy, [Paste], Rename, Delete; Open, Open in new tab, Open in new window, Pin to Quick access, Compress to >, Copy as path, Properties; Open in Terminal; Show more options | The owner's Windows 11 screenshots of two folders (no Share, no Open with); [Microsoft Support](https://support.microsoft.com/en-us/help/4028272) (Pin to Quick access from the menu); Paste only with clipboard content: [Thurrott, Windows 11 Field Guide, Desktop (24H2)](https://www.thurrott.com/books/windows-11-field-guide/desktop/310916/desktop-24h2) |
| Folder, classic | as the file's, plus the tab, window and pin commands after Open, and Paste after Copy | Owner's screenshots, with folder rows as Windows lists them |
| Several items | the file or folder menu; one-item commands disabled with the reason; the strip has no Paste | as above |
| Pinned folder (sidebar), compact | Open in new tab, Open in new window, Unpin from Quick access, Compress to >, Copy as path, New >, Properties; Open in Terminal; Show more options; strip Copy only for Downloads, at the bottom when opening upward | The owner's Windows 11 screenshots (Downloads, opened upward), relayed by the manager session |
| Drive | unchanged from #268: strip Copy, Rename (greyed); Open, Open in new tab, Open in new window, Format..., Eject, Pin to Quick access, Properties; Show more options | Checked on Windows 11 on 2026-11-07 (#268) |
| Strip labels | icon with label under it | [itechtics: labels on context-menu icons](https://itechtics.com/label-icons-context-menu) |
| Compress to > | ZIP File, Additional options (Windows also has 7z File and TAR File, which the app cannot write) | [Windows Latest, 2024-05-03](https://windowslatest.com/2024/05/03/windows-11-23h2-is-getting-full-fledged-create-archives-for-zip-7z-and-tar); [Pureinfotech](https://pureinfotech.com/window-11-support-rar-7-zip-tar-gz/) |
| Group heading | Collapse group (Left), Expand all groups, Collapse all groups | Checked on a Windows 11 VM by the manager session; [Eleven Forum](https://www.elevenforum.com/t/expand-or-collapse-group-in-file-explorer-in-windows-11.6518/) |

## Captures

On the fork's `status` branch: `context-menu-match-{file,folder,several,drive}-{compact,classic}-{light,dark}.png` (2x, under Xvfb), and `context-menu-match-pinned-{down,up}-{light,dark}.png` (a pinned folder's menu opening downward with the strip on top, and Downloads' opening upward with the strip at the bottom), and `context-menu-match-realclick-{compact,classic}.png`, which are real right-clicks and a Shift+right-click with xdotool on the built app, run as a normal user. The black margins in the real-click pictures are the popover's shadow area, because Xvfb has no compositor. Share does not show in the captures because Xvfb has no email portal.

## Testing

- `cargo fmt --all`: clean.
- `cargo clippy --workspace --all-targets`: no new warnings. The same warnings as the base, in files this change does not touch.
- `python3 native/parity/check.py`, `python3 native/tools/i18n.py check`, `python3 tools/sync-docs.py` (regenerated, no further changes): pass.
- Run by name under the project's isolated runner (Xvfb, private bus and HOME): `context_menu`, `sidebar`, `place_menus`, `this_pc`, `menu_popover`, `a_group_heading_menu_collapses_and_expands_the_groups`, `the_right_click_menus_are_captured_compact_and_classic`: 156 passed, 2 failed (with the two new pinned-folder tests). Both failures fail the same way on the unmodified base: `a_folder_without_write_permission_keeps_new_and_paste_and_shows_the_error` needs a non-root user, and `reset_returns_every_windows_sidebar_to_its_default_width` needs the Python 1.x checkout.
- New and updated tests: `a_pinned_folder_menu_is_windows_11_and_keeps_its_strip_nearest_the_pointer`, `a_pinned_folder_menu_copies_compresses_and_creates_in_that_folder`, `right_clicking_a_pin_opens_its_menu`, `a_file_compact_menu_follows_windows_11`, `a_file_classic_menu_follows_windows_11`, `a_folder_classic_menu_follows_windows_order_and_keeps_its_shortcuts`, `the_compact_menu_puts_the_edit_commands_in_its_strip`, `the_compact_menu_keeps_the_applications_in_open_with`, `the_compact_menu_has_the_strip_and_show_more_options_opens_the_classic_one` (Paste appears only after a copy; the labels are drawn under the icons), `a_group_heading_menu_collapses_and_expands_the_groups`.
- `archives`, `compress`, `copy_path` (the compress code now takes explicit items): 35 passed. The 3 that fail (`a_compressed_tar_is_browsed_and_extracted_unless_archives_open_elsewhere`, `a_password_zip_says_so_and_offers_the_archive_manager`, `open_in_archive_manager_hands_the_zip_to_the_desktop`) fail the same way on the unmodified base.
- Not run: the whole suite, a deb, a real desktop session, and Share with a real email portal.

## Coordination

- Built on `this-pc-menu` (#268), which makes sidebar and drive menus use the file menu. Merge #268 first.
- Details-view group headings collapse on the `group-collapse` branch. Their heading menu can reuse `ToggleGroup`, `CollapseAllGroups` and `ExpandAllGroups` once that branch lands.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
