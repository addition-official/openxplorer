# Title

fix(open-with): Open files, choose apps and change defaults as Windows 11 does

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/08-pass-7-views)

## What changes for the user
- **A file no app opens shows the app chooser.** Double-clicking a file with an unknown extension, no extension or an executable used to end at "Could not open the item: No application is installed for this file type. Use Open with… to choose one." It now opens the app chooser at once, and lists every installed app when none is suggested for the type. "Find an app in Software" moves into the chooser, where Windows links to the Microsoft Store, and is shown only while Software is installed.
- **Always changes only what the extension names.** Always used to set the default for the file's content type, which for an unknown file is `application/octet-stream` (every unknown file) and for a file without a known extension is the type sniffed from its contents, such as `text/plain`. It now sets the default for the type GIO knows from the name alone; for an extension of several types (`.ts`), the type found for the file; and for an extension GIO does not know, a type of its own, `application/x-extension-xyz`, registered for this user only when Always or Set default is pressed (a package in `$XDG_DATA_HOME/mime/packages` and `update-mime-database`; only plain ASCII extensions are written).
- **A file without an extension gets Just once only**, under "Select an app to open 'README'". Only the current default is preselected, so Just once and Always wait for a pick.
- **Properties > Opens with > Change... only changes the default.** It used to show Just once and Always, so it opened the file. It now asks "Select a default app for .txt files", lists the current app under "Default app" and the others under "More options", and offers one Set default, which makes the chosen app the default, opens nothing and updates Opens with at once. The custom command is not offered there, since a command cannot be a default.
- **Open with lists the default app, by name, in both menus.** The compact submenu left out the file's default and said "Open with <app>"; the classic menu had separate "Open with <app>" rows. Both now show Windows 11's submenu: up to four apps, the default first, each by name and icon, then Choose another app.
- **More than 15 items ask first**, not 15 or more, as Windows asks above its `MultipleInvokePromptMinimum` of 15.
- **A disabled accent button looks disabled.** Just once before an app is chosen, or Set default for a type that cannot get a default, kept the blue fill; it is now drawn as a disabled bordered button.
- **The chooser puts the keyboard on the chosen app**, also when More apps opens by itself.
- **Type-to-select with `GTK_A11Y=none`.** PR 01 already stops this crash; this PR adds a test that runs itself again with `GTK_A11Y=none` and announces with no accessibility context.
- **Tests removed.** The "test: Drop redundant tests" commit removes PR 08's `switching_to_small_icons_keeps_the_list_at_the_top` and `home_shows_the_first_groups_heading_in_a_grouped_grid`, which other tests already cover. Three older tests of the old behavior (`a_file_without_an_application_offers_a_search_in_software`, `only_a_type_without_an_application_is_searched_for` and `the_items_other_applications_are_offered_before_open_with`) are replaced by tests of the chooser and the Open with submenu.

Checked and unchanged: text, PDF, JPEG, PNG (a Flatpak app, which gets `@@u <path> @@`), MP3, MP4, ODT, a 3 GiB MP4, names with spaces, Unicode and shell characters (passed as one literal path), `.desktop` folder links (followed), `.desktop` launchers and scripts (opened in their editor, never run, unless Settings asks), ZIP members, search results, Recent, mixed selections, and a folder's Open, Open in new tab and Open in new window.

Docs (`docs.json` and the generated guides), the UI spec, the parity notes (OPEN-003, OPEN-010, OPEN-012, OPEN-013, PROP-003) and the message template are updated; no translation is touched.

## Why
- A file no app could open left the user at a dead end instead of the chooser Windows 11 shows.
- Always changed the default for every unknown file, or for a sniffed type such as `text/plain`, instead of for the one extension.
- Change... in Properties opened the file instead of only setting the default.
- The Open with menus hid the default app and differed between the compact and classic menus.
- The open question came one item early, and a disabled Just once looked clickable.

## CHANGED OWNER DECISIONS
The parity specs in `native/parity/features.toml` record these as decided behavior, and this PR changes them:
- OPEN-003 (Open several selected items at once): before, opening 15 or more items asked first; now more than 15, as Windows asks above its `MultipleInvokePromptMinimum` of 15.
- OPEN-010 (Offer to install a handler when no application opens a file type): before, a file no app opens showed "Could not open the item: No application is installed for this file type. Use Open with… to choose one." with a Software search button; now it opens the app chooser, which carries the Software link.
- OPEN-012 (Open with launches and can set the default): before, Always set the default for the file's content type; now for what the file's extension names.
- OPEN-013 (Open With applications listed in the menu): before, the applications were rows in the menu itself, "as Nautilus has them, rather than in a submenu", without the file's default application; now an "Open with" submenu lists up to four apps, the default first, then "Choose another app", as Windows 11 does.
- PROP-003 (Properties General tab): before, Change... opened the Open with chooser; now it only sets the default app and opens nothing.

## New and changed visible text
- "Select an app to open this file" → "Select an app to open '{name}'"
- "Select a default app for '{name}'"
- "Select a default app for {extension} files"
- "Default app"
- "More options"
- "Set default"
- "No installed app can open this item."
- "A default app can be set only for files with an extension."
- "Only an installed app (.desktop) can be the default."
- "The default app could not be changed."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix with a test that failed first; `native/parity/check.py`, `tools/i18n.py check`, `tools/sync-docs.py --check`, `tools/audit-public-data.py`, rustfmt on Rust 1.92; Clippy `-D warnings` on Rust 1.92, clean apart from three spots in pass-6's code; every test executable run isolated and compared by name with unchanged main, with no test failing that passes on main (ox-app 2,608 passed, the same 17 failing as on main); the new window tests also on headless Weston (Wayland).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
