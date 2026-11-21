# Title

fix(address-bar): Match Windows 11's address bar menu, typed paths, standard folders and history

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 5 tested the address bar against Windows 11 File Explorer with real input (xdotool on a private Xvfb, an unprivileged user, a throwaway home with fictional mp3, mp4, pdf, jpg, odt, zip, 7z and 300 MB files). Seven differences are fixed, each with a test that failed first. The last two come from a check on Windows 11 build 26200:

- **Quoted paths.** "Copy as path" puts a path between double quotes, and pasting it into the address bar said "OpenXplorer can't find…". A path (or a place name such as "Recycle Bin") between one pair of double quotes now opens as the path. Completion also works after an opening quote.
- **Environment variables.** `%NAME%` is expanded as in Windows (any case; `%USERPROFILE%` stands for the home folder when it is not set), and `$NAME` (before a `/` or the end) and `${NAME}` as Linux users type them. An unset variable is looked up as typed, so it gets the usual "can't find" box. The history keeps what was typed.
- **Tab in the suggestions.** With the suggestion list open, Tab moved focus out of the address bar and the typed text was lost. Tab and Shift+Tab now move through the list as Down and Up do. With no list showing, Tab moves focus as before.
- **Typed history.** The history behind F4 and the address bar's chevron was kept per window, in memory only, so a new window or a restart listed nothing. It is now shared by every window and kept in `typed-addresses.json` in the settings folder (private file, written atomically, at most 20 entries, never an address with a user name or a control character). Folder Options > Clear OpenXplorer history removes it.
- **Dragging a crumb.** A crumb can now be dragged like the folder it names, as a navigation pane folder can. Dropped in the app, it follows the usual drop rules: on the same drive it moves, Ctrl copies, the label under the pointer says which, and Ctrl+Z undoes it. Another app is offered Copy. The search results' crumb, pages and the address being edited drag nothing. A Ctrl or Shift click on a crumb now acts on release, so a Ctrl+drag copies without also opening a tab.
- **Address bar menu.** A right-click on a crumb or on blank space of the address bar now shows exactly Windows' four commands: Copy Address, Copy Address as Text, Edit Address and Delete History. On a crumb they act on that crumb's folder. Copy Address puts the folder on the clipboard as Copy does; Copy Address as Text puts its path; Delete History forgets the typed addresses. Dolphin's Paste and go, Open in new tab / new window, Keep address editable and Show full path left this menu; the two toggles remain in the edit box's own menu and in Settings, and a middle-click on blank space still goes to the selected text.
- **Standard folders by name.** Typing Desktop, Documents, Downloads, Music, Pictures or Videos (any case, in English or as the navigation pane names it) opens that folder from any folder, as Windows opens the user's Documents from C:\Windows. It wins over a subfolder of the same name; `./Documents` still means the one here. A standard folder and what is inside it now get crumbs that start at it ("Documents › Projects") instead of "Home › Documents › Projects".

No new dependency. No translations changed (the message template is regenerated). The unused `ox-code-20-symbolic` icon, which only the old menu used, is removed.

## Before and after

| Case | Before | After |
|---|---|---|
| `"/home/demo/Team files"` + Enter | "OpenXplorer can't find…" | Opens Team files |
| `%HOME%/Music`, `$HOME/Music`, `${HOME}/Music` | "can't find" | Opens Music |
| `%NOT_SET%/Music` | "can't find" | "can't find '%NOT_SET%/Music'" (unchanged) |
| Tab with suggestions open | Focus leaves, typed text lost | Next suggestion goes into the address |
| F4 in a new window or after a restart | Empty | The typed addresses, most recent first |
| Drag the "Music" crumb onto Desktop | Nothing | "→ Move to Desktop" (Ctrl: "+ Copy to Desktop") |
| Ctrl+drag of a crumb | (no drag) | Copies; no extra tab |
| Right-click a crumb | Copy address, Paste and go, Open in new tab / window, Edit address, Keep address editable, Show full path | Copy Address, Copy Address as Text, Edit Address, Delete History |
| `Documents` typed in another folder | That folder's own Documents, or "can't find" | The user's Documents, one "Documents" crumb |

## Checked and unchanged

Breadcrumb click and middle-click (background tab), chevron menus between crumbs, the « overflow menu at 640 px, right-click on a crumb, Alt+D / Ctrl+L / F4 / blank-space click editing with the whole path selected, Escape restoring, `~`, relative paths, trailing slashes, `file://` URIs (escaped and not), a typed file path opening the file, the not-found box, Back/Forward right-click lists, Up, Refresh/Stop, This PC / Network / Home / Recycle Bin / Gallery typed by name, search results as the location, and network addresses that hang (FTP and SFTP to a server that accepts and never answers): the window stays responsive throughout.

## Tests

- New: ox-core `location/typed.rs` (4), `typed_addresses.rs` (4); ox-app `window/tests/typed_addresses.rs` (9, one with a real XTEST pointer), `address_menu.rs` unit test rewritten. Two address_input tests now type `Letters` and paste `./Documents`, since a bare `Documents` now means the standard folder.
- Related suites (address input and bar, history, search history, folder options, drag and drop, file drag, slow locations, opening, text field keys) pass.
- rustfmt and clippy `-D warnings` with Rust 1.92.0, `parity/check.py`.
- Full run of every test executable in check.py's isolation, compared by name with unchanged main: 4,932 passed / 21 failed, the same 21 that fail on main in this root-only container.

## Not done

- `shell:` folders (`shell:downloads` and the like) are not supported; Windows opens them.
- After Stop on a network folder whose listing hangs, the list says "This folder is empty." (unchanged; what Windows shows there is not confirmed).
- The second Ctrl+L toggling the address back to crumbs is fixed in the open review-4 PR and is not touched here.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
