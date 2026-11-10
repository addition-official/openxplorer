# Title
feat(keys): Make Shift+F10, Ctrl+T, Ctrl+W and Esc work as in Windows 11 File Explorer

# Body
Model: claude-opus-5-5

This changes four keyboard behaviors so they match Windows 11 File Explorer. Each one is covered by tests that fail on the previous main. The key tests press real keys and move the real pointer on the test display.

**Shift+F10 opens the same menu as a right-click.** In Windows 11, the Menu key and Shift+F10 on a selected file both open the new menu, with the Cut, Copy, Rename, Share and Delete strip and Show more options. The classic menu shows only through Show more options or Shift+right-click. OpenXplorer's Shift+F10 used to open the classic menu. It now opens the menu style chosen under Settings › Appearance › Right-click menu, as the Menu key does. This applies to the file list in every view, the navigation pane, This PC's tiles and group headings. The menu still opens next to the selected item, not at the pointer. Shift+right-click and Show more options still open the classic menu. The keyboard shortcuts window now lists Shift+F10 as "Context menu". Show more options no longer shows "Shift+F10" beside it, as in Windows 11, where that row shows no key.

**A setting keeps the old Shift+F10.** Settings › Appearance › Layout › **Shift+F10 opens the classic menu** is off by default. When it is on, Shift+F10 opens the Windows 10 menu again on the file list, the navigation pane, This PC's tiles and group headings. The Menu key is not affected. While the setting is on, Show more options shows "Shift+F10" again, since that key then opens the same classic menu. The setting is saved as `shiftF10Classic` and written only while it is on.

**Ctrl+T and Ctrl+W work from the address bar and the search box.** In Windows 11, Ctrl+T in the address bar or the search box opens a new Home tab, and Ctrl+W closes the tab, or the window when it is the last tab. Ctrl+N does nothing in those fields. OpenXplorer used to leave Ctrl+T and Ctrl+W to the text field, where they did nothing. They now work while the address bar is being edited and while the search box has focus, and they leave the field's text alone. Ctrl+N, Ctrl+H, Ctrl+Tab and Alt+Left/Right/Up still stay with the field. Ctrl+A, Ctrl+C, Ctrl+V and Ctrl+Z still act on the field's text. Other text fields, such as the rename box and dialogs, are unchanged.

**Esc closes the Replace or Skip Files question.** In Windows 11, Esc on Replace or Skip Files closes it and nothing is copied. Esc used to do nothing there. It now cancels that job, as the window's close button does: the files with the same names stay as they were, nothing is moved, and the progress window closes. Esc in the File Conflict window still closes only that window and brings the question back. As before, files without a name conflict are copied before the question appears, and a cancel keeps them.

**Esc cancels a drag.** In Windows 11, pressing Esc while a file is dragged with the button still held cancels the drag, so releasing over a folder does nothing. In OpenXplorer the release still moved the file. On X11, GTK's drag never sees that Esc, which goes to the window that has the keyboard. Every OpenXplorer window now takes Esc while a drag of the app's files, folders or drives is in progress, including a drive dragged from This PC, and cancels the drag the way GTK cancels one. A drag started afterwards works normally.

A navigation pane row whose menu was opened with the Menu key or Shift+F10 now gets the keyboard back when the menu closes. Before, focus went to the pane splitter.

| Behavior | Before | After | Windows 11 |
|---|---|---|---|
| Shift+F10 on a file, folder, blank space or navigation pane row | Classic menu | Right-click menu style (the new menu by default) | New menu |
| Menu key | Right-click menu style | Unchanged | New menu |
| Shift+right-click, Show more options | Classic menu | Unchanged | Classic menu |
| Key text beside Show more options | "Shift+F10" | None ("Shift+F10" while the setting is on) | None |
| Ctrl+T in the address bar or search box | Nothing | New Home tab | New Home tab |
| Ctrl+W in the address bar or search box | Nothing | Closes the tab, or the window with its last tab | Closes the tab or window |
| Ctrl+N in the address bar or search box | Nothing | Nothing | Nothing |
| Esc in Replace or Skip Files | Nothing | Closes it and cancels the job | Closes it, nothing copied |
| Esc while dragging, then release over a folder | File moved | Nothing happens | Nothing happens |

**Changed owner decisions.** The parity inventory recorded these as decided, and they now follow Windows 11 instead:

- CMD-013: Shift+F10 opened the classic menu. It now opens the right-click menu's style. The previous behavior is kept as the "Shift+F10 opens the classic menu" setting. The CMD-008, CMD-014 and Send to (classic menu) notes now say the classic menu comes from Shift+right-click.
- CMD-017 (with TAB-001 and TAB-002): Ctrl+T and Ctrl+W were ignored in text fields, as app.js ignored them. They now work from the address bar while it is edited and from the search box. The backlog entry and the CMD-017 note also said Ctrl+H, Ctrl+N, Ctrl+T, Ctrl+W, Ctrl+Tab and Alt+Left/Right/Up already acted from text fields, which `window_keys.rs` never did. Both now describe what the code does.

DND-029 (Esc during a drag) is a new parity item. The user guide, the backlog and the translation template are updated. The new setting text and the changed messages need their Chinese translation.

**How it was tested.**

- Tests were written first and run on the previous main, where they fail: Shift+F10 opened the classic menu, Ctrl+T and Ctrl+W did nothing from the fields, Esc left Replace or Skip waiting, and Esc during a real drag still let the release move the file.
- With the changes, these tests pass. They cover Shift+F10 in Details, List, Tiles, Large icons and the compact view, for one file, one folder, several items and blank space, with the pointer moved away, and Esc returning focus. They also cover the navigation pane, a tab, the setting off and on, the Menu key, Shift+right-click, and the key text beside Show more options. For the text fields they cover Ctrl+T, Ctrl+W and Ctrl+N from the address bar, the empty search box and the search box with text, with 1 and 3 tabs, plus Ctrl+W on the last tab and the text-editing keys. Esc is tested in Replace or Skip for a copy and a move, with one name conflict and with several, and Enter on a focused row is checked. Esc during a drag is tested onto a folder, a navigation pane row, a tab and another window, and with a This PC drive tile dragged onto a folder, and a later drag is checked to work.
- The app was also checked by hand with xdotool on an Xvfb display, using fictional files, at 1x and 2x and in light and dark.
- rustfmt, Clippy with the workspace lints, `native/parity/check.py`, the translation template check and the docs sync check pass.
- Not tested: Wayland. The new keyboard tests cover X11 only.
- On KDE Wayland the compositor cancels the drag on Esc and keeps its drag cursor until the pointer moves.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
