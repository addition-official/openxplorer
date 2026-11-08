# Title

Make "Install updates without asking for a password" an opt-in setting, off by default

# Body

This builds on #298 and keeps its update helper, its policy and the whole update flow as they are. The one change: updating without a password is now something an administrator turns on, instead of something every installation gets.

AGENTS.md asks to "keep all native actions opt-in", and a polkit rule that skips the password is one of those. So:

- **Nothing active is installed.** The stable .deb ships the helper and its policy (`auth_admin_keep`) as before, so updates still work and ask for an administrator's password. The rule now goes only to `/usr/share/openxplorer/install-update.rules`, a template that polkit does not read. Nothing is installed in `/usr/share/polkit-1/rules.d` or `/etc/polkit-1/rules.d`.
- **A new setting on Settings › About, under Updates:** "Install updates without asking for a password", off by default. Polkit rules cannot read app settings, so the switch runs the existing helper through pkexec with one of two fixed arguments, `--enable-passwordless` or `--disable-passwordless`. That asks for an administrator's password once. The helper keeps its clean environment and root-only check. It copies the shipped template to `/etc/polkit-1/rules.d/60-io.winspace.Development.install-update.rules` (root-owned, 0644, atomic write, never through a link), or deletes it. It takes no path and no text.
- **The switch shows the real state, and a cancelled prompt puts it back.** Ubuntu and Debian keep `/etc/polkit-1/rules.d` unreadable to users (`0750 root:polkitd` on Ubuntu 24.04). So the helper also writes a small readable record, `/var/lib/openxplorer/install-updates-without-password`. The switch reads the rule file when it can see it, and the record only when it cannot.
- **Uninstalling turns it off.** The stable postrm deletes the rule and the record on `remove` and `purge`. An upgrade keeps them, and with them the user's choice.
- **Only where it applies.** The setting is shown only in the stable Debian package, which has the helper. It is hidden in RPM, Arch, Flatpak, the preview and source builds.
- **Docs updated.** SECURITY.md, the packaging README, docs/installation.md (regenerated with `tools/sync-docs.py`) and the UPD-003 parity note now say it is off by default and turned on by this setting.

Signature verification is untouched here; it was suggested separately.

## Checklist

| # | Check | Result |
|---|---|---|
| 1 | Fresh install of the stable .deb: no rule active, setting OFF | PASS |
| 2 | Setting OFF: the update action needs the administrator (`auth_admin_keep`) | PASS (pkaction, pkcheck) |
| 3 | Turn ON: one password prompt, fixed rule in `/etc/polkit-1/rules.d`, switch ON, no prompt for an active local sudo member | PASS (last part shown by evaluating the installed rule, see below) |
| 4 | Cancel the prompt when turning ON: switch back to OFF, nothing written | PASS |
| 5 | Turn OFF: rule removed, polkit asks again | PASS |
| 6 | Restart the app: the switch matches the file, ON and OFF | PASS |
| 7 | Remove and purge with the setting ON: rule gone; reinstall keeps it | PASS |
| 8 | Keyboard: Tab reaches the switch with a focus ring, Space toggles; reduced motion | PASS |
| 9 | Hidden in a build without the helper | PASS |

## How it was tested

- **Tests first.** New tests in `test_package_data.py` and `test_build_deb.py` failed on the #298 head (16 failures and errors). New Rust tests: `ox-core` `update::passwordless` and `install_helper`, `tests/update_passwordless.rs`, and GTK tests in `ox-app` `settings_page::tests::passwordless`. Those fail to build on the #298 head, because the setting does not exist there.
- **Checks on this branch:** `native/tools/check.py`, `native/parity/check.py`, rustfmt, clippy, `tools/sync-docs.py --check` and the public-data audit. The parity check, rustfmt, clippy, the docs check, the audit and the translation check all pass.
  - `check.py` normally stops at the first failing test program. It was run in full on this branch and on the unchanged #298 head, carrying on past failures. This branch: 4254 passed, 17 failed. #298 head: 4212 passed, 35 failed.
  - 16 of the branch's 17 failures fail on the #298 head too. The container causes them: its Python 3.13.18 answers two comparisons differently, sandboxed thumbnailers and administrator copies cannot run here, and three search tests depend on the indexing cache.
  - The 17th, `search::service_tests::a_folder_that_becomes_a_virtual_environment_leaves_the_cache`, passed 5 times out of 5 when run alone.
  - None of the failures is in code this change touches.
- **The built stable .deb in a container:** passes `verify_deb.py`, and its payload has no rules file.
  - It was installed with apt, and polkitd ran on a private system bus.
  - The app ran under Xvfb at 1x and 2x, light and dark, driven with xdotool keys only: Ctrl+, then arrows to About, Tab to the switch, Space.
  - `pkttyagent` on a pseudo-terminal stood in for the desktop's password dialog. It answered the one prompt, or cancelled it.
  - After each step the rule, the record and polkitd's rule count (5, then 6, then 5) were checked. The app was restarted with the setting on and with it off.
  - Removal, purge and reinstall were run with the setting on.
- **What a container cannot show.** It has no systemd-logind, so polkit never sees an active local session, and `pkcheck` keeps asking there even with the rule present. For that one part of line 3, the installed rule file was evaluated with a mocked polkit subject. It gives YES for an active local sudo or wheel member, and leaves remote, inactive, non-admin and other actions to the defaults. The real update download was not run, because GitHub is not reachable from the container.
- **Reduced motion.** With `gtk-enable-animations=false` the switch is in its new position at once; with animations on it is still moving 50 ms later.
- **Hidden case.** The stable binary run from outside `/opt/openxplorer` shows no such row. A GTK test covers the same. No preview .deb was built.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
