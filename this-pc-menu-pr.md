# Title

feat(sidebar): Give This PC, Network and drives Windows 11's menus, and remove the Map network drive button

# Body

Model: claude-opus-5-5

## Summary

Brings the right-click menus of the navigation pane and of This PC's drive tiles in line with Windows 11's File Explorer, and removes the "Map network drive" button that was pinned below the navigation pane. The menus were compared with Windows 11's on a Windows 11 machine.

## Removed: the Map network drive button below the navigation pane

The button at the bottom-left of the navigation pane (`#connect-sidebar`, `.sidebar-bottom`, with its CSS and font-size rule) is removed. Windows 11's navigation pane has no button there. Windows reaches Map network drive from the right-click menu of This PC and Network, and from the command bar's "…" menu in This PC and Network. Both are kept: "…" still lists Map network drive, and the new menus below list it too.

## This PC's and Network's menus

This PC's right-click menu lists Windows 11's items in Windows' order, where the app has the feature:

- Expand / Collapse
- Open in new tab
- Open in new window
- Add a network location
- Map network drive…
- Disconnect network drive (the same submenu of network mounts as "…", disabled with a reason while none is connected)
- Properties (opens the desktop's system information app, as "…" > Properties does; left out where there is none)

Network's menu is the same without Add a network location and Properties.

Left out because the app has no counterpart: Pin to Start, Pin to Quick access (the app cannot pin a page such as This PC), and Network's Properties (Windows opens the Network and Sharing Center). "Hide section" stays last, after a divider: it sets the saved `hiddenSidebarSections` preference, and Folder Options has no Windows option it could move to.

## Sidebar menus use the file list's menu

Every navigation-pane menu now opens as the file list's right-click menu: the same popover in the style chosen in Settings ("Right-click menu"), the classic menu with Shift held, and the Menu key and Shift+F10 on the focused row. The pane's row rules (`.sidebar list > row` and similar) also styled the rows of its menu, because the menu is a child of the pane. They are now scoped to the pane's own list (`list.places`), so width, padding, row height, icon column and dividers come from the shared menu rules.

In the compact style:

- This PC, Network, Home, Recent files, Recent locations and the Recycle Bin show Rename and Delete in the icon strip, greyed out, as Windows greys them on This PC.
- A pinned folder or a network location shows Cut, Copy, Paste, Rename and Delete, acting on the row's location through the folder tree's existing actions. Cut, Rename and Delete are enabled only for a folder the user pinned, not for shares or the standard folders.
- Each menu ends with "Show more options", which opens the classic menu in its place, with the strip's commands as rows.

## Drive menus

A drive's menu is now the same wherever it opens: This PC's drive tiles and the sidebar's drive rows. Its compact menu follows Windows 11's drive menu as checked on a Windows 11 machine (Copy and Rename in the strip, then Open, Open in new tab, Open in new window, Turn on BitLocker, Format…, Pin to Quick access, Pin to Start, Properties, Show more options), without Turn on BitLocker and Pin to Start:

- Strip: Copy and Rename, greyed out with the reason. The app does not change drive names, and pasting a copied drive would copy its whole contents (Windows creates a shortcut instead).
- Open, Open in new tab, Open in new window
- Format… where the app can format the drive
- Eject, Safely remove or Disconnect where the drive allows them (placed after Format…; Eject's exact position in Windows' compact menu was not confirmed)
- Pin to Quick access (new: `win.pin-location`), or Unpin from Quick access while pinned
- Properties
- Show more options, which opens the classic menu in place with Open in Terminal, Cache this folder for search and Open in Disks, which leave the compact menu, and Copy and Rename as rows

Alt+Enter on a focused sidebar row now opens its Properties.

## Docs, translations and parity

The manual, the docs guides (regenerated with `tools/sync-docs.py`), the message template and the parity notes (SIDE-001, SIDE-014, SIDE-017, SIDE-033, NET-001, CMD-007, HOME-003, HOME-005, HOME-009) are updated. The parity notes no longer cite the removed button's test.

## Tests

- New: `this_pc_and_network_menus_list_windows_items_in_its_order`, `nothing_is_pinned_below_the_navigation_pane`, `sidebar_menus_take_the_file_menus_style_strip_and_show_more_options` (window/tests/sidebar.rs); `this_pc_menu_lists_windows_items_with_properties_and_the_drives`, `network_menu_has_no_add_a_network_location_or_properties` (window/sidebar/menu.rs); `a_drive_menu_follows_windows_11_in_the_compact_style` (window/place_menus/tests.rs); `a_drive_tile_menu_is_the_file_menu_with_windows_drive_items` (window/tests/landing_pages.rs).
- Capture tests for visual review, which save pictures only when `OX_NATIVE_CAPTURE_DIR` is set: `the_sidebar_and_its_menus_are_captured_light_and_dark`, `the_drive_tile_menu_is_captured_light_and_dark`.
- Removed: `map_network_location_sits_below_the_list`, as the button is gone.
- Run: rustfmt, clippy (no new warnings), `native/parity/check.py`, `native/tools/i18n.py check`, `tools/sync-docs.py --check`, and the sidebar, menu, landing, place menu, snapshot, Quick access, context menu, focus, Recycle Bin, accessibility, stylesheet, font and command bar tests by name under xvfb-run and dbus-run-session: 362 pass. 4 unrelated tests fail the same way on main in that container (it runs as root and lacks the `v2.0.0` tag). The full suite was not run locally.

## Overlap with open pull requests

- #258 (Map network drive letters) edits the same Map network drive sentence in the manual and the network shares guide, which this branch rewords from "below the navigation pane" to the right-click menus. Merging either one after the other needs that sentence combined and the guides regenerated.
- #259 (This PC selection) touches `landing.rs` next to this branch's tile menu change and merges cleanly there. It conflicts only in HOME-003's parity note and the generated `markdown.json`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
