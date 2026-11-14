# Title

fix(desktop): Never give Super+E back to an uninstalled app

# Body

Model: claude-opus-5-5

When the release package and the preview package are installed side by side, either one can take Super+E on KDE Plasma. Uninstalling a package never cleaned up after it: KDE kept the removed package's Super+E entry and the user's settings folder kept its record of where Super+E came from. Turning Super+E off in the package that remained could then hand the shortcut back to the removed package, so Super+E opened nothing.

Uninstalling still changes nothing in anyone's home folder. Instead, the package that remains now tidies up when it starts and each time its Super+E switch is used:

- An app counts as removed once its desktop file is gone from the `applications` data folders. Components that are not desktop files (such as KWin's) always count as installed.
- If a removed OpenXplorer package still has Super+E, Super+E goes back to the app that package took it from (for example Dolphin). It never goes to the package doing the cleanup, so that package's switch stays off until the user turns it on.
- A record that points at a removed app is followed through the removed package's own record to an app that is still installed, or dropped if there is none. A removed package's own record is deleted.
- Turning the switch off never gives Super+E to a removed app. If no installed app is left to give it to, Super+E is left free.
- If the app that had Super+E was itself removed (for example Dolphin), turning the switch on takes Super+E from it without recording it.
- On other desktops and in the Flatpak, nothing is read or written. When there is no record and no removed package has Super+E, the settings folder is not touched.

## What is written where

| | Before | After |
|---|---|---|
| System files (deb scripts) | Nothing for Super+E. postinst/postrm refresh caches; the release package also adds/removes its polkit update rule | Unchanged. A new packaging test checks that neither package's scripts mention home folders or KDE shortcuts, and that remove and purge succeed repeatedly without KDE |
| `~/.config/kglobalshortcutsrc` (written by KDE's shortcut service when the switch is used) | A removed package kept `Meta+E` on its `NewWindow` action | When the other package starts, `Meta+E` moves back to the app it was taken from, or is freed |
| `~/.config/winspace/launch-shortcut-<desktop file>.json` | Could point at a removed package or app forever | Mended to point past the removed app, or deleted. A removed package's own record is deleted |

## Testing

- 13 new tests in `native/crates/ox-core/tests/integration_launch_shortcut.rs`, run against an in-memory model of KDE's shortcut service where a removed app's actions and keys stay saved. 10 of them fail on main (with only the two new API names stubbed in so they compile). The other 3 cover cases that already worked (preview only, removing a package that never had Super+E, other desktops). All 45 tests in the file pass.
- A unit test for the desktop-file lookup (`an_app_is_installed_while_its_desktop_file_is`) and a packaging test in `native/tools/test_build_deb.py`.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `python3 native/parity/check.py`, `tools/sync-docs.py` and `native/tools/test_build_deb.py` pass.
- Every Rust test executable, run with the isolation `native/tools/check.py` uses: 4556 passed on this branch and 4542 on unchanged main, with the same 11 failures on both. Those 11 come from this test machine (9 administrator-copy tests that need a trusted mount, and 2 tests that compare against the retired Python app). `check.py` itself stops at the first of them, so a wrapper ran every executable with the same isolation.
- End-to-end in a container, never on a live desktop: locally built debs of both packages installed, removed (`dpkg -r`, `apt-get remove`) and purged (`dpkg -P`, `apt-get purge`) for real, with KDE's own shortcut service (kglobalaccel 5.115) on a private Xvfb display and D-Bus bus, with a fake home folder. A small harness called the same `LaunchShortcut` functions the Settings switch and startup call, against the real service:
  - Only one package installed: on, then off, gives Super+E back to Dolphin.
  - Release or preview removed or purged while it had Super+E (directly, or after taking it from the other package): when the other package starts, Super+E goes back to Dolphin, its own switch stays off, and no record names the removed package. With the code from main, Super+E stays on the removed package.
  - Turning the remaining package off when its record pointed at the removed package gives Super+E to Dolphin. On main it went to the removed package.
  - Removing a package that never had Super+E changes nothing in the settings folder.
  - Reinstalling after the other package tidied up starts with the switch off.
  - Dolphin removed: turning off leaves Super+E free.
  - Old records pointing at a removed package are mended or deleted at startup.
  - Repeated remove and purge succeed without KDE.
  - On a non-KDE desktop nothing is written.

  The GUI switch and Plasma 6's kglobalacceld were not exercised.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
