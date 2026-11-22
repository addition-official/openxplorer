# Title

feat(network): Make Network work like Windows: one row per device, discovery banner, computers in the navigation pane

# Body

Model: claude-opus-5-5

## Problem

Network worked "completely differently" from Windows 11 (review 5b, U1): it listed one row per advertised service, so one NAS showed as three or four computers and its WebDAV server was dropped (B3); an empty Network gave no hint that discovery was off; the navigation pane's Network listed the shares visited this session instead of computers; `$` shares showed; share names took GVfs's mounted spelling and Recent locations kept both spellings; a mapped drive letter was only a label (B1); the SFTP host-key question could clip its buttons (B2); a computer's Properties showed an empty folder (B4); and SFTP opened at the server's `/` (B6).

## What changes

- **One row per device.** Discovery merges every SMB, SFTP, FTP and WebDAV service of a host into one computer, named by its SMB server's NetBIOS name, else its host name. GVfs lists WebDAV as `dav+sd://Name._webdav._tcp.local`; its host, port and folder are now asked of Avahi over D-Bus (`ResolveService`), so it joins its host; an unresolved one joins the host whose services carry the same advertised name. Network starts in Tiles grouped by type ("Computer") and keeps its own style.
- **Opening a computer** lists its SMB shares and its other services as rows ("Harbor Files (SFTP)"). The tab says `\\PC`, the address "Network > PC", the search box "Search PC". Its menu is Open, Open in new tab, Open in new window, Pin to Quick access and Create shortcut (on the Desktop), then Show more options, with no Properties; Alt+Enter does nothing on it. Network's blank space offers View, Sort by, Group by and Show more options.
- **Discovery off.** Without GVfs's network browser, or with neither Avahi nor wsdd and nothing found, Network shows Windows' yellow bar ("Network discovery is turned off. Network computers and devices are not visible. Click to change..."), with the reason as its tooltip and an × that hides it until discovery runs again. Its menu offers "How to turn on network discovery" (a new manual topic) and "Open Help". The app never starts or changes a system service.
- **Navigation pane.** Network starts collapsed, as in Windows, and expands to the computers (discovering when nothing found them yet), each with its own chevron that expands it to its shares and services, read off the main loop; Right and Left work on it. Browsed or mounted shares sit inside their computer. Saved locations and shares mapped to a letter move under This PC.
- **`$` shares** are hidden items, shown with "Show hidden files".
- **Share spelling.** Inside a share, folders keep the share's spelling instead of GVfs's mounted one; Recent locations lists a share once whatever its case.
- **B1.** `Z:` and `Z:\path` in the address bar open the share mapped to Z. Disconnect ends the mapping (saved or for the session), and Disconnect is now the default button of its question.
- **B2.** The question page scrolls its message and keeps the answers in a row that is always shown; the answers are cleared before a question is shown again.
- **B6.** An SFTP server opened at its root shows the account's home, from the mount's default location, looked up on a worker thread.
- Merged `pass-4-nav-pane` and `pass-6-context-menu`, then `pass-9-scaling` (passes 4 to 9). Each Network piece has one code path: computers come from discovery, the navigation pane from `network_tree`; a computer uses pass-6's server menus (with Create shortcut added) and Network's blank space pass-9's page menu without Refresh. Three Clippy 1.92 findings in merged menu code are fixed.

## Tests run

- New unit and window tests (all pass): device grouping, NetBIOS/DNS-SD merging, `dav+sd` resolution and Avahi's reply, unresolved services, discovery sources, discovery passes, `$` shares, drive letters, forgetting mappings, share spelling, Recent dedupe, the navigation pane tree, the info bar, the computer listing and naming, the long sign-in question, and updated tests for the deliberate changes (SMB crumbs start at Network, `\\server` titles, Network collapsed by default, saved shares under This PC). The two 1.x JavaScript parity tests now apply those two deliberate differences explicitly.
- `native/parity/check.py` passes; `tools/i18n.py check` passes (only the .pot changed).
- `native/tools/check.py` as an unprivileged user with Rust 1.92 (CI's toolchain): unit tests of the driver and parity, the i18n check, the icon guard, rustfmt and Clippy pass; see the status file for the test executables.
- Real input (xdotool) on a private Xvfb, unprivileged user, throwaway HOME, against local Samba (password, guest and a `Backup$` share), SFTP, FTP and WebDAV advertised over Avahi as one fictional host, plus a second host: Network, a computer, the navigation pane, the banner and its help, SFTP sign-in and home, light and dark.
- Frozen server: with smbd and Avahi stopped (SIGSTOP) while browsing, Refresh, Network, expanding a computer and going Home kept the main loop answering (slowest 54 ms over 277 pings), and Ctrl+Q quit in 145 ms.

## Known failure, not from this change

`menu_sweep::every_page_menu_runs_actions_the_window_has` (pass-6, real pointer) segfaults when run alone, on unchanged `pass-9-scaling` too.

## Not tested

A real Windows PC (WS-Discovery) or NetBIOS browsing; the website's pictures were not recaptured (the "pinned-sidebar" picture still shows a saved share under Network); no translations.

CLA: I agree
