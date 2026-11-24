# Title

fix(address-bar): Match Windows 11's address bar menu, typed paths, standard folders and history

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/04-pass-4-nav-pane)

## What changes for the user
- A path, or a place name such as "Recycle Bin", between one pair of double quotes (as Copy as path gives it) opens as the path. Completion also works after an opening quote.
- `%NAME%` is expanded as in Windows (any case; `%USERPROFILE%` stands for the home folder when it is not set), and `$NAME` (before a `/` or the end) and `${NAME}` as Linux users type them. An unset variable is looked up as typed and gets the usual "can't find" box. The history keeps what was typed.
- With the suggestion list open, Tab and Shift+Tab move through it as Down and Up do. With no list showing, Tab moves focus as before.
- The typed history behind F4 and the address bar's chevron is shared by every window and kept in `typed-addresses.json` in the settings folder (a private file, written atomically, at most 20 entries, never an address with a user name or a control character). Folder Options > Clear OpenXplorer history removes it.
- A crumb can be dragged like the folder it names. Dropped in the app it follows the usual drop rules (same drive moves, Ctrl copies, the label says which, Ctrl+Z undoes it); another app is offered Copy. The search results' crumb, pages and the address being edited drag nothing. A Ctrl or Shift click on a crumb now acts on release, so a Ctrl+drag copies without also opening a tab.
- A right-click on a crumb or on the address bar's blank space shows Windows' four commands: Copy Address, Copy Address as Text, Edit Address and Delete History. On a crumb they act on that crumb's folder. Paste and go, Open in new tab and new window, Keep address editable and Show full path leave this menu; a middle-click on blank space still goes to the selected text.
- Typing Desktop, Documents, Downloads, Music, Pictures or Videos (any case, in English or as the navigation pane names it) opens that standard folder from any folder, as Windows does. It wins over a subfolder of the same name; `./Documents` still means the one here. A standard folder and what is inside it get crumbs that start at it ("Documents › Projects").
- The unused `ox-code-20-symbolic` icon, which only the old menu used, is removed. The message template is regenerated; no translations changed.
- The last commit drops tests of this change that were found redundant.

## Why
- Pasting a quoted path said "OpenXplorer can't find…".
- Typed variables were not expanded.
- Tab with suggestions open moved focus out of the address bar and the typed text was lost.
- The typed history was kept per window in memory only, so a new window or a restart listed nothing.
- A crumb could not be dragged, unlike a navigation pane folder.
- The address bar's menu was Dolphin's, not Windows 11's (checked on build 26200).
- `Documents` typed in another folder opened that folder's own Documents or said "can't find".

Not done: `shell:` folders are not supported; after Stop on a network folder whose listing hangs, the list says "This folder is empty." (unchanged).

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- Changed: "Copy address" → "Copy Address"
- Changed: "Edit address" → "Edit Address"
- New: "Copy Address as Text"
- New: "Delete History"
- Removed: "Keep address editable", "Open “{label}” in new tab", "Open “{label}” in new window", "Paste and go", "Show full path"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: the new ox-core `location/typed.rs` (4) and `typed_addresses.rs` (4) tests, ox-app `window/tests/typed_addresses.rs` (9, one with a real XTEST pointer) and the rewritten `address_menu.rs` unit test, each failing first; two `address_input` tests changed to type `Letters` and paste `./Documents`; the address input and bar, history, search history, folder options, drag and drop, file drag, slow locations, opening and text field key suites; rustfmt and Clippy `-D warnings` with Rust 1.92.0; `parity/check.py`; every test executable in `check.py`'s isolation compared by name with unchanged main (4,932 passed, 21 failed, the same 21 that fail on main in a root-only container); real input with xdotool on a private Xvfb as an unprivileged user with fictional files. Several of these tests are dropped as redundant by this PR's last commit.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
