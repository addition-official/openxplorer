# Title

feat(network): Make Network work like Windows with one row per device, a discovery banner and computers in the navigation pane

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/06-pass-6-context-menu)

This PR comes ahead of pass 7 in this series because it depends only on passes 1 to 6.

## What changes for the user
- **One row per device.** Discovery merges every SMB, SFTP, FTP and WebDAV service of a host into one computer, named by its SMB server's NetBIOS name, else its host name. WebDAV services that GVfs lists as `dav+sd://Name._webdav._tcp.local` are resolved through Avahi over D-Bus (`ResolveService`) so they join their host; an unresolved one joins the host whose services carry the same advertised name. Network starts in Tiles grouped by type ("Computer") and keeps its own view style.
- **Opening a computer** lists its SMB shares and its other services as rows ("Harbor Files (SFTP)"). The tab says `\\PC`, the address "Network > PC", the search box "Search PC". Its menu is Open, Open in new tab, Open in new window, Pin to Quick access and Create shortcut (on the Desktop), then Show more options, with no Properties; Alt+Enter does nothing on it. Network's blank space offers View, Sort by, Group by and Show more options (Refresh is no longer listed there).
- **Discovery off.** Without GVfs's network browser, or with neither Avahi nor wsdd and nothing found, Network shows Windows' yellow bar ("Network discovery is turned off. Network computers and devices are not visible. Click to change..."), with the reason as its tooltip and an × that hides it until discovery runs again. Its menu offers "How to turn on network discovery" (a new manual topic) and "Open Help". The app never starts or changes a system service.
- **Navigation pane.** Network starts collapsed, as in Windows, and expands to the discovered computers (discovering when nothing found them yet), each with its own chevron that expands it to its shares and services, read off the main loop; Right and Left work on it. Browsed or mounted shares sit inside their computer. Saved locations and shares mapped to a letter move under This PC.
- **Hidden shares.** Shares whose names end in `$` are hidden items, shown with "Show hidden files".
- **Share spelling.** Inside a share, folders keep the share's spelling instead of GVfs's mounted one, and Recent locations lists a share once whatever its case.
- **Drive letters.** `Z:` and `Z:\Plans` in the address bar open the share mapped to Z and a folder in it. Disconnect ends the mapping, whether it was saved or not, and Disconnect is the default button of its question.
- **Sign-in questions.** The question page scrolls its message and keeps the answers in a row that is always shown; the answers are cleared before a question is shown again.
- **SFTP home.** An SFTP server opened at its root shows the account's home, from the mount's default location, looked up on a worker thread.
- The user guide (`docs/network-shares.md` and the website's copies) and the in-app manual describe the new behavior.
- Each Network piece has one code path: computers come from discovery, the navigation pane from `network_tree`, and a computer uses PR 06's server menus with Create shortcut added. Three Clippy 1.92 findings in the merged menu code are fixed.
- **Tests removed.** The "test: Drop redundant tests" commit removes four tests that earlier PRs in this series add and that other tests already cover: PR 04's `a_busy_drive_is_refused_in_windows_words`, and PR 06's `a_right_click_after_a_menu_closed_opens_the_next_menu`, `the_folder_menu_grays_open_in_terminal_where_no_terminal_opens` and `small_icons_put_the_icon_left_of_each_name_down_columns` (review4-diffs' column-first Small icons test, which PR 08 replaces with a row-first one). The sidebar test of SMB mounts listed under Network becomes `rows_follow_windows_11_order_and_saved_shares_appear_once_under_this_pc`, since saved shares now sit under This PC and browsed ones inside their computer.

## Why
Network worked very differently from Windows 11: it listed one row per advertised service, so one NAS showed as three or four computers and its WebDAV server was dropped; an empty Network gave no hint that discovery was off; the navigation pane's Network listed the shares visited since the app started instead of computers; `$` shares showed; share names took GVfs's mounted spelling and Recent locations kept both spellings; a mapped drive letter was only a label; the SFTP host-key question could clip its buttons; a computer's Properties showed an empty folder; and SFTP opened at the server's `/`.

Not tested: a real Windows PC (WS-Discovery) or NetBIOS browsing; the website's pictures were not recaptured (the "pinned-sidebar" picture still shows a saved share under Network); no translations.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- Changed: "Network discovery is turned off. Network computers and devices are not visible. Click to learn more..." → "Network discovery is turned off. Network computers and devices are not visible. Click to change..."
- New: "Avahi (DNS-SD) is not running and wsdd (WS-Discovery) is not installed."
- New: "The GVfs network browser is not installed."
- New: "How to turn on network discovery"
- New: "Open Help"
- New: "Turning on network discovery"
- New: "Create shortcut"
- New: the help topic under "Turning on network discovery":

  ```text
  OpenXplorer finds computers through your desktop's own network discovery services, and never turns them on or changes them by itself. When Network says that network discovery is turned off, one of them is missing or stopped:

  - GVfs's network backends, which list the network: install gvfs-backends on Ubuntu, Debian and Zorin OS, gvfs-smb on Fedora and Arch Linux, or gvfs-backends and gvfs-backend-samba on openSUSE.
  - Avahi, which finds Linux computers, NAS devices and Macs (DNS-SD): install avahi-daemon and start it with sudo systemctl enable --now avahi-daemon.
  - wsdd, which finds Windows PCs (WS-Discovery): install the wsdd package.
  - Your firewall must let discovery through: mDNS on UDP port 5353 and WS-Discovery on UDP port 3702.

  Then open Network again or press F5. You can always open a computer by typing its address, such as \\server or \\server\share, in the address bar.
  ```

- Changed: the Network section of the manual (one message). Its first three bullets change as below; its last two bullets (Enter network credentials, and Sign out of server) are unchanged.

  Bullet 1, old:

  ```text
  - Open Network in the navigation pane to see the computers and servers on your network, as in Windows; open one to see its shared folders. Refresh (F5) looks again. Or type an address such as \\server\share or smb://server/share in the address bar.
  ```

  Bullet 1, new:

  ```text
  - Open Network in the navigation pane to see the computers and devices on your network, one row per device, as in Windows; open one to see its shared folders, and its other services (SFTP, FTP, WebDAV) as rows of their own. Shares whose names end in $ are hidden until Show hidden files is on. Refresh (F5) looks again. In the navigation pane, Network expands to the same computers, and each computer to its shares. Or type an address such as \\server\share or smb://server/share in the address bar.
  ```

  Bullet 2, old:

  ```text
  - Network finds only devices that advertise themselves, through the desktop's GVfs network services (DNS-SD through Avahi, Windows computers through WS-Discovery); firewalls and separate networks can hide them. If Network says that network discovery is turned off, those services are missing: install your distribution's GVfs backends (gvfs-backends on Ubuntu and Debian) and Avahi, or type the server's address instead.
  ```

  Bullet 2, new:

  ```text
  - Network finds only devices that advertise themselves, through the desktop's GVfs network services (DNS-SD through Avahi, Windows computers through WS-Discovery); firewalls and separate networks can hide them. If Network says that network discovery is turned off, those services are missing or stopped: click the bar and choose How to turn on network discovery, or type the server's address instead.
  ```

  Bullet 3, old:

  ```text
  - Map network drive, on the right-click menu of This PC and Network in the navigation pane or in their "…" menu, connects a share such as \\server\share under a drive letter, as in Windows, offering only the letters still free; Save in the sidebar · reconnect when opened (Windows has Reconnect at sign-in) saves it in the navigation pane. The letter is only part of its name. Its link Connect to a Web site, and Add a network location in that menu, connect SFTP, FTP, WebDAV and NFS folders too. Disconnect network drive there disconnects a connected share.
  ```

  Bullet 3, new:

  ```text
  - Map network drive, on the right-click menu of This PC and Network in the navigation pane or in their "…" menu, connects a share such as \\server\share under a drive letter, as in Windows, offering only the letters still free; it is listed under This PC. Type the letter in the address bar, such as Z: or Z:\Plans, to open the share or a folder in it. Save in the sidebar · reconnect when opened (Windows has Reconnect at sign-in) keeps it after OpenXplorer quits. Its link Connect to a Web site, and Add a network location in that menu, connect SFTP, FTP, WebDAV and NFS folders too. Disconnect network drive there disconnects a connected share and ends its mapping.
  ```

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here (that branch also had passes 4 to 9 merged): new unit and window tests for device grouping, NetBIOS and DNS-SD merging, `dav+sd` resolution and Avahi's reply, unresolved services, discovery sources and passes, `$` shares, drive letters, forgetting mappings, share spelling, Recent dedupe, the navigation pane tree, the info bar, the computer listing and naming and the long sign-in question, plus tests updated for the deliberate changes (SMB crumbs start at Network, `\\server` titles, Network collapsed by default, saved shares under This PC), and the two 1.x JavaScript parity tests applying those two differences explicitly; `native/parity/check.py` and `tools/i18n.py check`; `native/tools/check.py` as an unprivileged user with Rust 1.92, whose driver and parity unit tests, i18n check, icon guard, rustfmt and Clippy passed; real input with xdotool on a private Xvfb as an unprivileged user with a throwaway home, against local Samba (password, guest and a `Backup$` share), SFTP, FTP and WebDAV advertised over Avahi as one fictional host plus a second host, in light and dark; with smbd and Avahi stopped (SIGSTOP) while browsing, Refresh, Network, expanding a computer and going Home kept the main loop answering (slowest 54 ms over 277 pings) and Ctrl+Q quit in 145 ms. `menu_sweep::every_page_menu_runs_actions_the_window_has` segfaulted when run alone there, also without this change.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
