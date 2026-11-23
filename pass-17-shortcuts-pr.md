# Title

fix(shortcuts): Open shortcuts' targets, ask Problem with Shortcut, add Create shortcut and the Create Shortcut wizard

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 17 tested the app's shortcuts against Windows 11: creating them (New › Shortcut, Alt+drag, right-drag "Create shortcuts here", the classic menu, Send to › Desktop (create shortcut)), their names, the arrow overlay, opening shortcuts to a file, a folder, an app, a web page and a network share, renaming, copying, Properties, broken shortcuts (deleted target, link loops, a drive that is not plugged in), launchers from Downloads and archives, and the Type column. It fixes what was broken or unlike Windows:

- **A shortcut to a folder opened at the link's own path.** Opening "Projects - Shortcut" on the Desktop showed Desktop › Projects - Shortcut, so Up went back to the Desktop. Windows opens the folder itself. It now opens at the folder's own address (Home › Projects).
- **A shortcut to a file handed the application the link's path.** The player got ".../Desktop/Morning Walk.mp3 - Shortcut", a name with no extension. It now gets the file itself, as in Windows. A chain of links is followed to its end (at most 40); a symbolic link named for a type ("photo.jpg") still opens at its own path, as Windows opens a symbolic link.
- **A broken shortcut said only "This item is not a regular file or a readable folder."** A link to a deleted item, a loop of links, or a `.desktop` link to a deleted folder now asks Windows' Problem with Shortcut question: "The item ‘Old plan.txt’ that this shortcut refers to has been changed or moved, so this shortcut will no longer work properly. Do you want to delete this shortcut?" in a message box with the warning icon, with Yes, the default, and No (Y and N alone answer it). Yes deletes the shortcut, never anything else, permanently, as Windows 11 does. A shortcut onto a removable drive or share that is not connected says instead that "the drive or network connection that the shortcut … refers to is unavailable", with OK, and deletes nothing.
- **The classic menu had no Create shortcut.** Duplicate sat where Windows has Create shortcut. Create shortcut now comes before Delete and Rename, makes "report - Shortcut" beside the item ("report.txt - Shortcut" while extensions are shown) and leaves the item selected. Where the folder takes none (read-only, a search, Recent), it asks whether to place the shortcut on the desktop instead. Duplicate stays, after Rename.
- **New › Shortcut was a "New link" box.** It is now Windows' two-page Create Shortcut wizard: "Type the location of the item:" with Browse… and Next, then "Type a name for this shortcut:" with the name Windows proposes, Back and Finish. It also takes an Internet address or a network share (`\\server\share`, `smb://`), which becomes a `.desktop` link that opens it and runs nothing. Nothing is ever replaced.
- **Names, selection and columns, as checked in a Windows 11 VM.** With file name extensions hidden, as by default, a new shortcut is "song - Shortcut"; with them shown, "song.mp3 - Shortcut". After Create shortcut the item stays selected. A folder's shortcut sorts among the files, and a shortcut's Size and Date modified are its own (1 KB), not its target's.
- **Properties of a shortcut** open on its Shortcut tab; General gives the target's name as its Description and shows the icon with the arrow.
- **Launchers did not look like shortcuts.** A `.desktop` launcher showed a plain document with no arrow, and with file name extensions shown it read "Calculator.desktop". Launchers now carry the arrow, and `.desktop` is never shown, as Windows never shows `.lnk`; renaming keeps it.
- **Running app launchers is opt-in.** Settings › Confirmations › "Run installed apps' launchers on double-click", off by default, starts an installed app from its launcher, or a shortcut to one, as Windows runs a program's shortcut; other launchers still ask.
- **The trust question.** A launcher that is neither installed nor marked trusted (`metadata::trusted`) asks before its command runs, as GNOME asks about an untrusted application launcher. Trust and Launch marks it trusted, so it is not asked about again.

The interface guide describes the wizard, Create shortcut and broken shortcuts.

## Tests

Each fix has a test that was run and seen failing before the fix:

- `window::tests::shortcuts::a_folder_shortcut_opens_the_folder_at_its_own_address`, `a_file_shortcut_opens_the_file_itself`
- `window::tests::shortcuts::a_broken_shortcut_offers_to_delete_it` (deleted target and a loop), `a_shortcut_onto_a_drive_that_is_not_connected_says_so`, `a_desktop_link_to_a_deleted_folder_offers_to_delete_it`, and `ox_core::ops::shortcut_destination` tests
- `window::tests::send_to::create_shortcut_makes_one_beside_the_item_and_selects_it`, `create_shortcut_in_a_read_only_folder_offers_the_desktop`, and the classic menu order tests
- `icons::emblem::tests::links_and_read_only_items_carry_their_emblems`, `folder_view::shown_name::tests::a_launchers_desktop_extension_is_always_hidden`
- `window::tests::shortcuts::new_shortcut_is_the_create_shortcut_wizard`, `new_shortcut_to_an_internet_address_is_a_desktop_link`, and `ox_core::ops::links` wizard tests
- `window::file_drop::targets::tests::a_downloaded_launcher_asks_before_it_runs`, `items_dropped_on_a_trusted_launcher_start_its_application`

Real input (xdotool/XTEST, private Xvfb, unprivileged user, throwaway HOME with fictional mp3, mp4, jpg, broken jpg, pdf, docx, odt, zip, a 2 GB file and odd names): Alt+drag, Send to › Desktop, F2 rename of a link and a launcher, copy and paste of a folder shortcut (the link is copied, not the folder), opening each kind of shortcut, the new dialogs, the wizard typed and finished from the keyboard, a drop on a downloaded launcher. No Bail out, CRITICAL, panic or signal in the app's output.

Checks (Rust 1.97 here; CI pins 1.92.0): `native/parity/check.py`, `i18n.py check`, `tools/sync-docs.py --check`, rustfmt and the public-data, sync-docs and release-source tests pass. Clippy fails only on newer lints in files this branch does not touch. Every test executable was run in check.py's isolation as an unprivileged user: the failures that remain also fail on unchanged main (Python and JavaScript fixture checks that need node and the Python app's environment) or on origin/pass-15-errors built alone (five tests that pin GIO's error text), which this branch merges.

## Known limitations

- A launcher's own `Icon=` picture is not drawn: the view draws only the bundled Fluent art (VIEW-074 partial).
- Change Icon is not offered for shortcuts: a symbolic link has no icon of its own. A link's Shortcut tab has only Target and Open File Location (no Start in, Shortcut key, Run or Comment to keep).
- Renaming a shortcut to "song.mp3" makes it an MP3-typed link, where Windows keeps it a shortcut (kept by decision).
- The wizard proposes the item's name with its extension ("Morning Walk.mp3"), as Windows does; a symbolic link so named is typed by its extension (MP3 File, with the arrow), as the app types symbolic links, where Windows shows such a shortcut as Shortcut.
- Not tested: shortcuts on a share that stops answering (all new look-ups run off the main thread), Wayland (Weston).
