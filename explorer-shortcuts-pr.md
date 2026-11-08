# Title
Add more of Windows 11 File Explorer's keyboard shortcuts: Ctrl+1 to 9, Ctrl+Shift+E, Num Lock keys, Ctrl+F5, Ctrl+Insert, typed Ctrl+Shift+1 to 8

# Body
Model: claude-opus-5-5

## Summary
I went through every shortcut in Microsoft's page "Keyboard shortcuts in Windows" that acts in File Explorer, and tried each one on unchanged main with real key presses (xdotool on Xvfb). Most already worked. This adds the ones that were missing:

- **Ctrl+1 to Ctrl+9** show the tab of that number (Alt+1 to Alt+9 still work). In the Properties window they show its tab of that number, as Windows dialogs do.
- **Ctrl+Shift+E**, while the navigation pane is shown, opens it down to the folder you are in and moves the keyboard there: in the folder tree when it is shown, otherwise to the place's row (opening This PC or Network if it was collapsed).
- **Num Lock + plus, minus, asterisk** in the navigation pane open, close, or open everything below the selected folder (and expand or collapse This PC / Network). Before, these keys were typed into the file list.
- **Ctrl+F5** refreshes, like F5 and Ctrl+R.
- **Ctrl+Insert / Shift+Insert** copy and paste files in the file list. Text fields keep these keys for their own text.
- **Ctrl+Shift+1 to 8 now work when typed.** A keyboard sends Ctrl+Shift with the shifted symbol of the digit key (for example % for 5), which the accelerators never matched, so typing these keys did nothing. The window now also accepts each digit key's shifted symbol, read from the keyboard layout.

Keys typed in the address bar, the search box or the rename box still edit text there. The new keys appear in the keyboard shortcuts window (the keypad keys as "Num +", "Num -" and "Num *"), the built-in manual, the guide and the parity list (CMD-039).

Two Windows behaviors differ from choices the app already made, so they stay as they are and need a decision:
- **Alt+mouse drag**: Windows makes a shortcut; here Alt+drop opens the drop menu (drag-and-drop rule).
- **Backspace in an Open or Save dialog**: Windows goes up one folder; here Backspace goes back everywhere (NAV-004).

The developer snapshot hook (used for the review pictures) gained `key=`, `zoom=` and `type=` scene steps, `OPENXPLORER_DIALOG=1` (save the dialog a scene opened) and `OPENXPLORER_SAVE_DIALOG=1` (show the window as a Save dialog that nobody waits for).

## Checklist
| Shortcut | What Windows does (Microsoft) | Before (main 2.0.15) | After | Result |
|---|---|---|---|---|
| Alt+D | "Select the address bar." | works: address text selected | same | PASS |
| Alt+Enter | "Display properties for the selected item." | Properties opens | same | PASS |
| Alt+Left arrow / Backspace | "Navigate to the previous folder." | works (Backspace from the list too) | same | PASS |
| Alt+mouse drag a file | "When you drop the file, create a shortcut to the original file in that location." | the owner's drop menu opens (DND rule: Alt asks); no shortcut | same | CONFLICT |
| Alt+P | "Show or hide the preview pane." | works | same | PASS |
| Alt+Right arrow | "View the next folder." | works | same | PASS |
| Alt+Shift+P | "Show or hide the details pane." | works | same | PASS |
| Alt+Up arrow | "Move up a level in the folder path." | works (main window, navigation pane, Save dialog) | same | PASS |
| Ctrl+Arrow key (to move to an item)+Spacebar | "Select multiple individual items." | works in all five views | same | PASS |
| Ctrl+D / Delete | "Delete the selected item and move it to the Recycle Bin." | works | same | PASS |
| Ctrl+E / Ctrl+F | "Select the search box." | works | same | PASS |
| Ctrl+L | "Focus on the address bar." | works | same | PASS |
| Ctrl+mouse drag a file | "When you drop the file, create a copy in that location." | works | same | PASS |
| Ctrl+mouse scroll wheel | "Change the size and appearance of file and folder icons." | works | same | PASS |
| Ctrl+N | "Open a new window." | works | same | PASS |
| Ctrl+Number (1-9) | "Move to that tab number." | nothing (only Alt+1..9) | shows that tab; Alt+1..9 kept | PASS (was FAIL) |
| Ctrl+Plus (+) (keypad) | "Resize all columns to fit text." | works (upstream 2.0.15) | same | PASS |
| Ctrl+Shift+E (while the navigation pane is shown) | "Expands all folders from the tree in the navigation pane." (Windows 10: "Display all folders above the selected folder.") | nothing | opens the navigation pane down to the folder shown, opens its subfolders, focus moves there (folder tree, else the place's row) | PASS (was FAIL) |
| Ctrl+Shift+N | "Create a new folder." | works, name box open | same | PASS |
| Ctrl+Shift+Number (1-9) | "Change the view style. For example, 2 is Large icons and 6 is Details." | typed keys did nothing (GTK got the shifted symbol, such as % for 5) | 1-8 switch the view in all views, main window and Save dialog | PASS (was FAIL) |
| Ctrl+Shift+Tab | "Move to the previous tab." | works | same | PASS |
| Ctrl+T | "Open a new tab and switch to it." | works | same | PASS |
| Ctrl+W | "Closes the active tab. Closes the window if there's only one tab open." | works | same | PASS |
| Ctrl+Tab | "Move to the next tab." | works | same | PASS |
| End | "Scroll to the bottom of the active window." | works | same | PASS |
| F2 | "Rename the selected item." | works | same | PASS |
| F3 | "Search for a file or folder." | works | same | PASS |
| F4 | "Select the address bar to change the current path." | works | same | PASS |
| F5 | "Refresh the window." | works | same | PASS |
| F6 | "Cycle through elements in the window." | works | same | PASS |
| F11 | "Maximize or minimize the active window." | full screen and back (as Windows 11 File Explorer does) | same | PASS |
| Home | "Scroll to the top of the active window." | works | same | PASS |
| Left arrow | "Collapse the current selection (if it's expanded), or select the folder that the folder was in." | works on This PC / Network heads; folder tree reachable only by mouse | same keys; Ctrl+Shift+E now reaches the tree by keyboard | PASS |
| Right arrow | "Display the current selection (if it's collapsed), or select the first subfolder." | same as Left | same as Left | PASS |
| Shift+Arrow keys | "Select multiple items." | works in all five views | same | PASS |
| Shift+Delete | "Permanently delete the selected item without moving it to the Recycle Bin." | asks, then deletes for good | same | PASS |
| Shift+F10 | "Display the context menu for the selected item." | works | same | PASS |
| Shift+mouse drag a file | "When you drop the file, move it to that location." | works | same | PASS |
| Shift+mouse right-click | "Display the Show more options context menu for the selected item." | classic menu | same | PASS |
| Windows key+E | "Open File Explorer." | - | - | SKIPPED: a desktop key; the desktop opens its own default file manager |
| Num Lock+asterisk (*) | "Display all subfolders under the selected folder." | nothing (key went to type-to-select) | opens every folder below the selected tree folder; expands This PC / Network | PASS (was FAIL) |
| Num Lock+plus (+) | "Display the contents of the selected folder." | nothing | opens the selected tree folder; expands a section | PASS (was FAIL) |
| Num Lock+minus (-) | "Collapse the selected folder." | nothing | closes it; collapses a section | PASS (was FAIL) |
| Backspace | "View the previous folder." | works | same | PASS |
| Ctrl+A | "Select all items in a window." | works | same | PASS |
| Ctrl+F5 / Ctrl+R | "Refresh the current window." | Ctrl+R works, Ctrl+F5 nothing | both refresh | PASS (was FAIL) |
| Ctrl+Y | "Redo an action that was previously undone with Ctrl+Z." | works | same | PASS |
| Ctrl+Z | "Undo the previous action." | works | same | PASS |
| Esc | "Stop or leave the current task, or dismiss a dialog box." | works | same | PASS |
| Alt+F4 | "Close the active window..." | - | - | SKIPPED: the desktop's window manager closes windows on Linux; the app binds nothing |
| Alt+Spacebar, Alt+Esc | window menu, cycle windows | - | - | SKIPPED: window manager keys |
| Alt+Page Down / Alt+Page Up | "Move down/up one screen." | not tested | not tested | NOT TESTED |
| Alt+underlined letter | "...Press Alt and then the letter to do that action." | not tested | not tested | NOT TESTED |
| Alt+Shift+arrow, Ctrl+Shift+arrow (Start tiles) | Start menu tiles | - | - | SKIPPED: Start menu only |
| Ctrl+F4 | "In apps that are full-screen ... close the active document but not the entire app." | not added | not added | SKIPPED: not in the File Explorer list; File Explorer closes tabs with Ctrl+W |
| Ctrl+Spacebar | "Enable or disable the Chinese input method editor (IME)" | - | - | SKIPPED: the desktop's input method; in the file list Ctrl+Space toggles an item (tested above) |
| F10 | "Activate the menu bar in the active window." | - | - | SKIPPED: Windows 11 File Explorer has no menu bar, nor has this app |
| Ctrl+A | "Select all text." | works in all three fields | same | PASS |
| Home / End | "Move the cursor ... beginning/end of the line." | works | same | PASS |
| Backspace / Del | delete a character | works; the window does not go back | same | PASS |
| Ctrl+C / Ctrl+Insert, Ctrl+V / Shift+Insert | copy / paste text | works; no files copied | same | PASS |
| Ctrl+Z | "Undo the last typing." | works | same | PASS |
| Other text-editing rows (Ctrl+arrows, Shift+Ctrl+arrows, Ctrl+Del, Ctrl+Shift+V, Page Up/Down) | GTK's own field keys | not tested | not tested | NOT TESTED |
| Ctrl+Insert / Shift+Insert in the file list | copy / paste (Microsoft's text-editing rows; File Explorer also copies and pastes files with them) | nothing | copy and paste the selection | PASS (was FAIL) |
| Ctrl+Tab / Ctrl+Shift+Tab | "Move forward/back through tabs." | works in Properties | same | PASS |
| Ctrl+1, 2, 3,... | "Move to that tab number." | nothing in Properties | shows that tab | PASS (was FAIL) |
| Esc | dismiss | works | same | PASS |
| Backspace | "Open a folder one level up if a folder is selected in the Save As or Open dialog box." | goes Back (owner rule NAV-004) | same | CONFLICT |
| F4, Tab, Shift+Tab, Spacebar, Arrow keys, Alt+underlined letter | dialog controls | not tested | not tested | NOT TESTED |

## How it was tested
- **Real input.** xdotool on a private Xvfb display with its own D-Bus session and a small window manager, a disposable home with fictional files, run as an unprivileged user. Results were read back over the accessibility bus, from the files on disk and from the settings file. File-list keys were tried in Details, List, Tiles, Content and Large icons; window, text-field, navigation-pane, Save dialog, mouse-drag and Properties keys separately; everything at 1x and 2x scale. Unchanged main: 382 cells, 44 failed (each one of the gaps above). This branch: 382 cells, 378 pass; the other 4 are the two conflicts at 1x and 2x.
- **Unit tests first.** New tests in `window/tests/more_explorer_keys.rs` and `window/tests/properties_width.rs` were added first: the 8 in `more_explorer_keys.rs` fail on upstream main with only the tests added (commit 651611fc), and the Properties test fails without its fix; all 9 pass with the change.
- **Checks**, run as an unprivileged user with Rust 1.92: `python3 native/parity/check.py`, the i18n check, rustfmt and Clippy pass. `python3 native/tools/check.py` ran every test executable plus the doctests. In ox-app, 2,113 tests passed and 2 failed: the sandboxed thumbnailer tests, which also fail on unchanged main in this container. The ox-core library's link-safety tests need a folder that only its owner can write, so they ran with `OX_TEST_TRUSTED_BASE` set (1,193 passed). The Python-compatibility fixture test was run with Python 3.12 (the system Python is 3.13). The release-source and public-data tests and `tools/audit-public-data.py` pass.
- **Not run:** `pnpm check` and `pnpm build` (no website code changed; the docs Markdown was regenerated with `tools/sync-docs.py`). No .deb was built.
- **Num Lock on.** The navigation pane's keypad keys and Left/Right ignore Caps Lock and Num Lock bits in the key state, as the picker and Properties handlers do; unit tests that send those bits failed before the fix. With Num Lock really on (xdotool, X11), the keys worked before and after: X11 did not deliver the lock bits to these handlers here, so please also check on your desktop.
- **Pictures** were taken with the native capture path (`tools/native_capture.py` and the snapshot hook) at 1x and 2x, light and dark, with the fictional demo tree.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
