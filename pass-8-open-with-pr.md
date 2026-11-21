# Title

fix(open-with): open files, choose apps and change defaults as Windows 11 does

# Body

Model: claude-opus-5-5
CLA: I agree

Pass 8 of the Windows 11 parity run: opening files and Open with. Every common type was opened by double-click and Enter in the real app under a private Xvfb (unprivileged user, throwaway HOME, recording stand-in apps), and these differences from Windows 11 are fixed, each with a test that failed first:

- **No app for a type opens the chooser.** Double-clicking a file no app opens (an unknown extension, no extension, an executable) showed "Could not open the item: No application is installed for this file type. Use Open with… to choose one.", a dead end. Windows 11 opens its app chooser at once; so does the app now, with every installed app listed when none is suggested. The old "Find an app in Software" button moves into the chooser as a link, where Windows links to the Microsoft Store.
- **Always changes only that extension's default, as checked in the Windows 11 VM.** Always called `set_as_default_for_type` with the file's content type, which for an unknown type is `application/octet-stream` (every unknown file) and for a file without a known extension is the type sniffed from its contents, such as `text/plain`. Always now changes what the extension names: the type GIO knows it by from the name alone; for an extension of several types (`.ts`), the type found for the file; and for an extension GIO does not know, a type of its own, `application/x-extension-xyz`, registered for this user only when Always or Set default is pressed (a package in `$XDG_DATA_HOME/mime/packages` and `update-mime-database`; only plain ASCII extensions are written). A file without an extension gets Just once only, under "Select an app to open 'README'", and nothing but the current default is preselected, so Just once and Always wait for a pick.
- **Properties › Opens with › Change… only changes the default.** It showed Just once and Always, so it opened the file. As in Windows it now asks "Select a default app for .txt files", lists the current app under "Default app" and the others under "More options", and offers one Set default, which makes the chosen app the default, opens nothing and updates Opens with at once. The custom command is not offered there, since a command cannot be a default.
- **Open with lists the default, by name, in both menus.** The compact submenu left out the file's default and said "Open with <app>"; the classic menu had separate "Open with <app>" rows. Both now have Windows 11's submenu: up to four apps, the default first, each by name and icon, then Choose another app.
- **More than 15 items ask first, not 15.** Windows asks above its `MultipleInvokePromptMinimum` of 15.
- **A disabled accent button looks disabled.** Just once before an app is chosen, or Set default for a type that cannot get a default, kept the blue fill; it is now drawn as a disabled bordered button, as Settings and the file picker already were.
- **The chooser puts the keyboard on the chosen app**, also when More apps opens by itself.
- **Crash: type-to-select with `GTK_A11Y=none`.** With that variable GTK gives widgets no accessibility context, and gtk-rs's `at_context()` panicked inside a signal handler on the first key typed in the file list, which aborted the app. pass-1 fixed the same crash; this branch uses pass-1's fix and adds an end-to-end test that runs itself again with `GTK_A11Y=none`.

Checked and unchanged: text, PDF, JPEG, PNG (a Flatpak app, which gets `@@u <path> @@`), MP3, MP4, ODT, a 3 GiB MP4, names with spaces, Unicode and shell characters (`$HOME;rm -rf x\`.txt`, passed as one literal path), `.desktop` folder links (followed), `.desktop` launchers and scripts (opened in their editor, never run, unless Settings asks), ZIP members (private copy in the runtime folder), search results, Recent, mixed selections (folders in background tabs, each file in its app), and a folder's Open, Open in new tab and Open in new window.

Docs (`docs.json` and the generated guides), the UI spec, the parity notes (OPEN-003, OPEN-010, OPEN-012, OPEN-013, PROP-003) and the message template are updated; no translation is touched. The lower passes are merged in (pass-1 to pass-7, and review4-diffs through pass-7).

Tests: `native/parity/check.py`, `tools/i18n.py check`, `tools/sync-docs.py --check`, `tools/audit-public-data.py`, rustfmt on the pinned Rust 1.92; Clippy `-D warnings` on 1.92 is clean apart from three spots in pass-6's merged code (left to pass 6). Every test executable was run isolated and compared by name with unchanged main: no test fails here that passes on main (ox-app 2,608 passed, the same 17 failing as on main). The new window tests also pass natively on headless Weston (Wayland).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01CQbN3NxuumeWhytvJ8bTGY
