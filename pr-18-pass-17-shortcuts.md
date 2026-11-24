# Title

fix(shortcuts): Open shortcuts' targets, ask Problem with Shortcut and add the Create Shortcut wizard

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/17-pass-16-clipboard)

## What changes for the user
- A shortcut to a folder opens the folder at its own address (Home › Projects), not at the link's path under the Desktop, so Up goes to the folder's parent, as in Windows.
- A shortcut to a file hands the application the file itself, not the link's extensionless path. A chain of links is followed to its end (at most 40); a symbolic link named for a type ("photo.jpg") still opens at its own path, as Windows opens a symbolic link.
- A broken shortcut (deleted target, a loop of links, or a `.desktop` link to a deleted folder) asks Windows' Problem with Shortcut question in a message box with the warning icon, with Yes (the default) and No; Y and N alone answer it. Yes deletes the shortcut permanently, never anything else, as Windows 11 does. A shortcut onto a removable drive or share that is not connected says the drive or network connection is unavailable, with OK, and deletes nothing.
- The classic menu has Create shortcut before Delete and Rename. It makes "report - Shortcut" beside the item ("report.txt - Shortcut" while extensions are shown) and leaves the item selected. Where the folder takes no shortcut (read-only, a search, Recent) it asks whether to place the shortcut on the desktop instead. Duplicate stays, after Rename.
- New › Shortcut is Windows' two-page Create Shortcut wizard instead of the "New link" box: "Type the location of the item:" with Browse… and Next, then "Type a name for this shortcut:" with the name Windows proposes, Back and Finish. It also takes an Internet address or a network share (`\\server\share`, `smb://`), which becomes a `.desktop` link that opens it and runs nothing. Nothing is ever replaced.
- Shortcut names follow File Explorer: "song - Shortcut" with file name extensions hidden (the default), "song.mp3 - Shortcut" with them shown. A folder's shortcut sorts among the files, and a shortcut's Size and Date modified are its own, not its target's.
- A shortcut's Properties open on its Shortcut tab; General gives the target's name as its Description and shows the icon with the arrow.
- `.desktop` launchers carry the shortcut arrow, and `.desktop` is never shown, as Windows never shows `.lnk`; renaming keeps it.
- New setting, Settings › Confirmations › Launchers › "Run installed apps' launchers on double-click", off by default. On, an installed app's launcher, or a shortcut to one, starts the app, as Windows runs a program's shortcut. Off, it opens in its editor. Other launchers still ask.
- A launcher that is neither installed nor marked trusted (`metadata::trusted`) asks before its command runs, as GNOME asks about an untrusted launcher. Trust and Launch marks it trusted, so it is not asked about again.
- The interface guide describes the wizard, Create shortcut and broken shortcuts.

## Why
Shortcuts were tested against Windows 11: creating them (New › Shortcut, Alt+drag, right-drag "Create shortcuts here", the classic menu, Send to › Desktop (create shortcut)), their names, the arrow overlay, opening shortcuts to a file, a folder, an app, a web page and a network share, renaming, copying, Properties, broken shortcuts and launchers. Folder and file shortcuts opened the link's own path instead of the target; a broken shortcut said only "This item is not a regular file or a readable folder."; the classic menu had Duplicate where Windows has Create shortcut; New › Shortcut was a plain "New link" box; launchers showed a plain document with no arrow and read "Calculator.desktop" with extensions shown; and names, sort and sizes of shortcuts differed from a Windows 11 check.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
New:
- "Create Shortcut"
- "What item would you like to create a shortcut for?"
- "This wizard helps you to create shortcuts to local or network programs, files, folders, computers, or Internet addresses."
- "Type the location of the item:"
- "Type the location of a file or folder on this computer, a network share such as \\server\share, or an Internet address that starts with http:// or https://."
- "Browse for Files or Folders"
- "Click Next to continue."
- "Next"
- "What would you like to name the shortcut?"
- "Type a name for this shortcut:"
- "Click Finish to create the shortcut."
- "New Internet Shortcut"
- "The location could not be read. Try again."
- "OpenXplorer can’t create a shortcut here.\n\nDo you want the shortcut to be placed on the desktop instead?"
- "Problem with Shortcut"
- "The item ‘{name}’ that this shortcut refers to has been changed or moved, so this shortcut will no longer work properly.\n\nDo you want to delete this shortcut?"
- "The drive or network connection that the shortcut ‘{name}’ refers to is unavailable. Make sure that the disk is properly inserted or the network resource is available, and then try again."
- "“{name}” is a launcher that has not been marked as trusted. Launching it runs the command it holds. Launch it only if you trust where it came from."
- "Trust and Launch"
- "Launchers"
- "Run installed apps' launchers on double-click"
- "Like Windows: a shortcut to an installed app starts it. Off: it opens in its editor. Other launchers still ask before they run."
- Settings search words for the new setting: "run installed apps launchers double click desktop entry shortcut application start execute trust windows"

Removed (the old New link box): "New link", "Link to", "For example ~/Documents", "A link opens the file or folder it points to. Leave the name empty to use the name of that item."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix's test, seen failing before the fix: `window::tests::shortcuts::a_folder_shortcut_opens_the_folder_at_its_own_address`, `a_file_shortcut_opens_the_file_itself`, `a_broken_shortcut_offers_to_delete_it`, `a_shortcut_onto_a_drive_that_is_not_connected_says_so`, `a_desktop_link_to_a_deleted_folder_offers_to_delete_it`, `new_shortcut_is_the_create_shortcut_wizard`, `new_shortcut_to_an_internet_address_is_a_desktop_link`; the `ox_core::ops::shortcut_destination` and `ox_core::ops::links` wizard tests; `window::tests::send_to::create_shortcut_makes_one_beside_the_item_and_selects_it`, `create_shortcut_in_a_read_only_folder_offers_the_desktop` and the classic menu order tests; `icons::emblem::tests::links_and_read_only_items_carry_their_emblems`; `folder_view::shown_name::tests::a_launchers_desktop_extension_is_always_hidden`; `window::file_drop::targets::tests::a_downloaded_launcher_asks_before_it_runs`, `items_dropped_on_a_trusted_launcher_start_its_application`. Real input (xdotool/XTEST, private Xvfb, unprivileged user, throwaway home with fictional files): Alt+drag, Send to › Desktop, F2 rename of a link and a launcher, copy and paste of a folder shortcut, opening each kind of shortcut, the new dialogs, the wizard from the keyboard and a drop on a downloaded launcher, with no CRITICAL, panic or signal in the app's output. `native/parity/check.py`, `i18n.py check`, `tools/sync-docs.py --check`, rustfmt and the public-data, sync-docs and release-source tests (Rust 1.97 there; Clippy failed only on newer lints in files the branch does not touch). Every test executable in `check.py`'s isolation as an unprivileged user, with the remaining failures also failing on unchanged main or on pass-15-errors built alone.

These tests named above are not in this PR, because they were dropped as redundant: `a_desktop_link_to_a_deleted_folder_offers_to_delete_it` and `new_shortcut_to_an_internet_address_is_a_desktop_link`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
