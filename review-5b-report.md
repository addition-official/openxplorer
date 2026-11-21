# Review 5b: everything outside tonight's nine fix passes, OpenXplorer 2.0.21

Upstream main `5336b108` (2.0.21), `cargo build --release --locked`, default application ID `io.winspace.Development.Native`. Report only: no app code was changed.

Areas left out because tonight's nine passes own them: copy/move and the progress window, undo/redo, rename, the navigation pane's own behavior, the address bar, right-click menus, views/sort/group, Open with and default apps, scaling and window sizes. Where a finding here touches one of those areas (for example a missing right-click row), it is listed because of what it breaks in an area this review owns (Network, Home), and that pass is named.

## How it was run

- An unprivileged user `oxtest` with a throwaway HOME, on a private Xvfb display (1600x1000, Openbox, no compositor), its own `dbus-run-session`, GVfs (with its FUSE export), gnome-keyring (Secret Service), a PulseAudio null sink and the AT-SPI bus. Never the live desktop.
- Real input with xdotool. stderr of every run was kept and read.
- Local test servers on the same machine, advertised by Avahi (mDNS/DNS-SD):
  - Samba 4.19, NetBIOS name `HARBORNAS`, shares `Projects`, `Media` (1,500 small files plus media), `ReadOnly` (guest), `Private` (password), and a hidden `HiddenShare$`.
  - OpenSSH SFTP on port 2222, vsftpd FTP on port 2121, Apache WebDAV on 8080 (`/dav`, password), all three also advertised over DNS-SD as "Harbor Files".
  - Fictional account `nasuser`; the host answers to its mDNS name `vm.local`.
  - NFS was not available (no kernel NFS server in this container).
- Fictional files, all generated: mp3 (tagged and untagged), flac, mp4 (H.264), truncated mp3/mp4/jpg; PDFs of 3 and 300 pages, a truncated PDF and a random-bytes "PDF"; docx and xlsx with a fictional author; rtf, md, json, html, py; a 50 MB binary; 1920x1080, 6000x4000 and WebP pictures; zip, 7z, tar, tar.gz, password zip/7z, a corrupt zip; a `~/Templates/Letter.docx`.
- Environment: Ubuntu 24.04 libraries, GTK 4.14.5, GLib 2.80, GVfs 1.54, Samba 4.19.5, bubblewrap installed part-way (see Preview pane).
- Checks: `native/tools/check.py` and `native/parity/check.py` were not run, because no code was changed. No translations were read or tested.

- A main-loop watchdog (`org.gtk.Actions.List` on the app's D-Bus name every 250 ms) ran during the frozen-server tests and logged any reply slower than 1 s.

Paths below are written as `~/...`. "Real" input means xdotool keys and clicks on the window, not API calls.

## 1. Crashes, hangs and data loss

**None found in these areas.** Tried and held up:

- **20-minute soak** (40 rounds, each: Documents, Pictures in two views, a Samba share plus a search there, Network with discovery, an SFTP folder, inside a ZIP, the Recycle Bin, a new tab with Music and the Details pane, This PC, Gallery, home). Open files went from 31 to 33, threads stayed at 24, memory from 421 MB to 435 MB, flat after the first three rounds. No crash, no Rust panic, no Gtk-CRITICAL.
- **Frozen Samba server** (`kill -STOP` on smbd) while a share was open: search in the share, Refresh, going to Documents, Network with discovery, and Ctrl+Q. The watchdog saw no stall over 1 s, and Ctrl+Q quit at once.
- **Name clashes do not overwrite.** Compress to > ZIP File next to an existing `Notes 1.zip` made `Notes 1 (2).zip`; the old zip kept its md5. New > Compressed (zipped) Folder, Rich Text, Text and Bitmap made fresh files.
- **Recycle Bin Restore** into a folder that no longer exists recreated the folder with the file. Restore over a newer file of the same name asked "Replace or Skip Files"; Skip kept the newer file ("NEW") and its contents.
- **Broken and odd files** in the Preview and Details panes: a truncated PDF, random bytes named `.pdf`, a 300-page PDF, a 50 MB binary, a truncated jpg, mp3 and mp4. Each showed a preview or "No preview available.", with no warning on stderr.
- **Archives on a share**: Compress to ZIP and Extract all both worked on `\\vm.local\Projects`.

stderr over the whole session: only `Gtk-WARNING: GtkGizmo (progress) reported min width -2` (80 times in the soak, once per visit to This PC; review 4 already reported it), and LibreOffice's own javaldx warning.

## 2. Broken everyday actions

Ranked by how soon a Windows user hits them.

### B1. A mapped drive letter can't be used: typing `Z:\` is refused

- **Steps**
  1. This PC > "..." > Map network drive.
  2. Drive `Z:`, Folder `\\vm.local\Projects`, Finish.
  3. The drive shows as "Projects (Z:)" under Network and under This PC > Network locations.
  4. Alt+D, type `Z:\Agenda.rtf` (or just `Z:`), Enter.
- **What happens**: a message says "Windows drive letters are not Linux paths. Use /home/… or \\server\share." The letter is only a label. Other apps can't use `Z:` either. With mapped drives, typing the letter is the everyday way to get there.
- **What Windows does**: the mapped letter is a real drive. Microsoft's page says "In the Drive list, select a drive letter" (https://support.microsoft.com/en-us/windows/experience/connectivity-networking/file-sharing-over-a-network-in-windows), and `Z:\` works in the address bar and in every app.
- **Suggested fix**: in the address bar's normaliser, resolve `X:` and `X:\rest` against the network rows whose label ends in `(X:)` (`drive_letter()` already parses that), and open `smb://…/rest`. Show the error only when no mapping has that letter. Overlaps pass 5 (address bar), but the mapping belongs to this area.
- **Code**: `native/crates/ox-core/src/location/normalise.rs` lines 76-80 (`is_windows_drive_path` refuses before any lookup). `native/crates/ox-app/src/dialogs/network_map.rs:95` (`drive_letter`).

### B2. The SFTP host-key question cuts off its buttons

- **Steps**: Network > double-click "Harbor Files (Remote Login)" (`sftp://vm.local:2222/`) the first time.
- **What happens**: the "Network connection" window asks whether to trust the host key. Its long message fills the window, and the row of answer buttons is clipped at the bottom, so Cancel is cut off. The choices are a FlowBox inside a fixed-height window.
- **What Windows does**: Windows has no SFTP. Every Windows dialog keeps its buttons visible.
- **Suggested fix**: give the question page a scroller for the message, or let the window grow to fit its content. Keep the choices in a footer that is always visible.
- **Code**: `native/crates/ox-app/resources/ui/network-sign-in.ui` lines 303-345 (`question` page, `GtkFlowBox id="choices"`). `native/crates/ox-app/src/dialogs/network_sign_in.rs:219` (`ask_question`).

### B3. A WebDAV server advertised on the network never shows on the Network page

- **Steps**: a server advertising `_webdav._tcp` (here Apache, "Harbor Files"). Open Network, wait for the three passes.
- **What happens**: the SFTP and FTP services of the same server appear, but WebDAV does not. GVfs lists it with target `dav+sd://Harbor%20Files._webdav._tcp.local`. `advertised_server` keeps only targets that `split_location` accepts as `smb` or remote, and `dav+sd` is neither, so the entry is dropped.
- **What Windows does**: Windows lists devices, not services (see U1 below). A WebDAV NAS shows as its computer.
- **Suggested fix**: accept `dav+sd` and `davs+sd`. Resolve them through GVfs (open the `dav+sd://` URI and use the mount's root) or list them as is, since GVfs mounts them directly.
- **Code**: `native/crates/ox-core/src/network/discovery.rs` (`advertised_server`, about line 168). `native/crates/ox-core/src/location/parts.rs:31` (`REMOTE_SCHEMES` has no `dav+sd`).

### B4. Properties of a computer on the Network page shows an empty, read-only "folder"

- **Steps**: Network, select HARBORNAS, Alt+Enter.
- **What happens**: the window is titled "HARBORNAS Properties". Its name box says "Windows shares on vm.local" and can be edited. It also shows "Type: File folder", "Size: 0 bytes (0 bytes)", "Contains: 0 Files, 0 Folders", "Created: Not provided", and a checked, grayed "Read-only" box. None of that describes a computer.
- **What Windows does**: needs VM. Windows shows a computer's own Properties (device name and kind), or no Properties row at all.
- **Suggested fix**: for `Entry::network_computer` rows (and `smb://host/`), either hide Properties or show a small page: name, address (`\\host`), protocols found, and "Open".
- **Code**: `native/crates/ox-app/src/window/item_dialogs.rs:214-330` (`open_properties` → `show_properties` treats the virtual row as a folder). `native/crates/ox-core/src/entry.rs:266` (`network_computer`).

### B5. The PDF preview silently needs bubblewrap, and only notices it at startup

- **Steps**
  1. On a system without `bwrap`, turn on the Preview pane (Alt+P) and select `Invoice 1042.pdf`.
  2. It says "No preview available.".
  3. Install bubblewrap: still the same until the app restarts. After a restart, the pages draw.
- **What happens**: the sandbox lookup is cached for the life of the process, and nothing tells the user what is missing. I found no package dependency on `bubblewrap` or `poppler-utils` in `native/packaging` (the RPM spec has none; I could not find the .deb control file), so the user's install may be in this state.
- **What Windows does**: the PDF preview works once a PDF previewer is installed.
- **Suggested fix**: depend on (or at least recommend) `bubblewrap` and `poppler-utils` in the .deb/.rpm. Retry the lookup when it last failed instead of caching `None` forever. When the sandbox is missing, say "Install bubblewrap and poppler-utils to preview PDFs" instead of "No preview available.".
- **Code**: `native/crates/ox-app/src/thumbnails/thumbnailers.rs:550` (`static PDF_SANDBOX: OnceLock<Option<Sandbox>>`) and `:622` (`pdf_sandbox`).

### B6. SFTP opens at the server's root folder, not the account's home

- **Steps**: Network > "Harbor Files (Remote Login)", sign in as nasuser.
- **What happens**: the folder shown is the server's `/` (bin, etc, home, usr…). The user's files are in `home/nasuser`, three clicks away. Pinning or Keep in Network then saves `/`.
- **What Windows does**: no SFTP in Explorer, so this is a usability point, not parity.
- **Suggested fix**: after mounting an `sftp://host/` root, go to the remote home (GVfs exposes it through `standard::target-uri`, or query `~` on the mount's default location: `gio::Mount::default_location()`).
- **Code**: `native/crates/ox-app/src/window/network_actions.rs` (`navigate_or_report` after connect), and the Network row's open path in `native/crates/ox-app/src/folder_view/loader.rs:113` (`computer_entry` uses the bare server URI).

## 3. Unlike Windows

### Microsoft-confirmed

**U1. The Network page's model is not Windows'.** This is the "works completely differently" the user feels. These are the concrete differences, each seen tonight:

| | Windows 11 | OpenXplorer 2.0.21 (seen tonight) |
|---|---|---|
| What a row is | A **device**: one row per computer, media device or router | One row per **advertised service**. One machine gave three rows: "HARBORNAS" (SMB), "Harbor Files (ftp)" and "Harbor Files (Remote Login)". Its WebDAV was dropped (B3) |
| How devices are found | Network discovery (WS-Discovery, NetBIOS, SSDP/UPnP) | Only what GVfs's `network:///` returns. Here that was mDNS/DNS-SD. A Windows PC that advertises only through WSD or NetBIOS shows only if GVfs's wsdd or SMB browser finds it. SMB1 browsing is off by default in Samba 4.19 (`SMB1 disabled -- no workgroup available`). Needs VM to see a real Windows PC |
| Discovery off | A yellow banner: "Network discovery is turned off…", which turns it on. Microsoft: "If you open File Explorer, go to Network, and see an error message ("Network discovery is turned off…."), you'll need to turn on network discovery…" (https://support.microsoft.com/en-us/windows/experience/connectivity-networking/file-sharing-over-a-network-in-windows) | With Avahi and GVfs's network backends stopped, Network was **empty with no banner**. The banner shows only when `network:///` itself errors, not when it reads fine and holds nothing |
| Navigation pane under Network | Expands to the same computers | Lists something else: the shares mounted, mapped or visited this session ("Projects (Z:)", "private on vm.local", "nasuser on vm.local:2121", "vm.local:8080", "vm.local"). Two rows were both named "vm.local" (the SFTP mount and the SMB server), and the FTP row shows the account name |
| Opening a computer | Lists its shares and printers, hiding `$` shares | Lists its shares, **including `HiddenShare$`** (AT-SPI rows: `HiddenShare$`, Media, Private, Projects, ReadOnly). Microsoft: "Shared resource names that end in a $ character do not appear when you browse the local computer from a remote computer." (https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-server-2012-r2-and-2012/hh750728(v=ws.11)) |
| Share names | As the server spells them | GVfs's mount spelling: typed `Projects`, shown "projects" in the tab and breadcrumb; "private on vm.local". Recent locations got both `smb://vm.local/Private` and `smb://vm.local/private` |
| Mapped drive | A drive letter that works everywhere | A label (B1). It shows under This PC > Network locations and also under Network |
| Disconnect | This PC > right-click the drive > Disconnect: the mapping is gone | "Disconnect this mount?" with Cancel as the default button (my first Enter only canceled). After Disconnect, "Projects (Z:)" stays in the navigation pane and This PC, and clicking it reconnects. Only "Remove saved location" removes the mapping |

**Suggested fixes, smallest first:**
1. Hide shares ending in `$` from server listings unless "Show hidden files" is on (`ox-core/src/network` share listing, or the folder filter for `smb://host/` listings).
2. Show the banner also when discovery ran but found nothing *and* the GVfs dnssd/wsdd backends are missing or Avahi is not running. Word it "No computers were found. Network discovery may be off…", and open Help.
3. Group the Network page's services by host (`DiscoveredServer.host`) into one computer row. Opening it lists that host's SMB shares, then its other services as folders ("FTP", "SFTP", "WebDAV").
4. Keep the navigation pane's Network children as computers (the discovered hosts) plus mapped drives. Move session-visited shares under the computer they belong to.
5. Show the share name as the server spells it (from the browse listing or the typed address), not the GVfs mount name.
6. Make Disconnect on a mapped drive also forget the mapping (or label the current action "Unmount" and add Windows' Disconnect).

**Code**: `native/crates/ox-core/src/network/discovery.rs` (`servers_among`, `advertised_server`); `native/crates/ox-app/src/folder_view/loader.rs:95-115` (`list_network`, `computer_entry`); `native/crates/ox-app/src/window/network_info_bar.rs:64-73` (`show_network_info_bar` shows only on `discovery_problem`); `native/crates/ox-app/src/places.rs:63-90` (`compose`: Network rows = saved + mounts + visited); `native/crates/ox-app/src/window/mounting.rs:110-125` (`confirm_disconnect`) and `native/crates/ox-app/src/window/network_actions.rs`.

**U2. Search can't find words inside Office documents or PDFs.**
- **Steps**: Documents, search options > search in names and contents, type `Maple` (a word inside `Quarterly Plan.docx`).
- **What happens**: no result. The same option finds words in `.txt`/`.md` files. Contents search reads only local plain-text files (a NUL byte in the first 8 KB means "binary"), and docx/xlsx/pdf are zip or binary.
- **What Windows does**: Microsoft: "For files with text, their contents are indexed to allow you to search for words within the files." and "properties and content" (https://support.microsoft.com/en-us/windows/experience/performance-optimization/search-indexing-in-windows). Office and PDF contents are found through their IFilters.
- **Suggested fix**: extract text for docx/xlsx/pptx/odt (unzip `word/document.xml` etc. and strip tags) and PDF (`pdftotext` in the same bubblewrap sandbox as the preview) in `lowercase_text`, with the same size limit.
- **Code**: `native/crates/ox-core/src/search/contents.rs:44-60` (`lowercase_text`: `file://` only, text only).

### Needs VM

- **V1.** Network page with a real Windows 11 PC and a NAS on the same LAN: are they listed, once each, and under which category? (U1)
- **V2.** Does the "Enter network credentials" box start with "Remember my credentials" checked? OpenXplorer's sign-in has "Remember" on by default. No Microsoft page says either way.
- **V3.** Properties of a computer in Windows' Network folder: what it shows. (B4)
- **V4.** Properties > Details of a .docx and a .pdf. OpenXplorer shows only the File group: no Title, Authors or page count, though the test docx has a title and author and the PDF has a Title and Author. The mp3, jpg and mp4 Details tabs are full.
- **V5.** Does Windows search find a file inside a ZIP (`b.txt` in `Report.zip`)? OpenXplorer does not.
- **V6.** High contrast. With GNOME's `high-contrast` set to true, the Network page was pixel-identical apart from outlines (ImageMagick `compare -metric AE` gave 0 on a 760x300 crop of the window). The app keeps its own light colors. Windows' contrast themes replace every color (black background, yellow or cyan text). Check how a contrast theme looks in Explorer and decide whether the app should switch to a full high-contrast palette.
- **V7.** Does a mapped drive also appear in the navigation pane under Network, or only under This PC? (No Microsoft page found.)
- **V8.** Does Disconnect ask first, and does the drive stay listed afterwards? (U1, last row)

## 4. Looks

- **L1.** The navigation pane shows two rows named "vm.local" (an SFTP mount and the SMB server). The FTP row says "nasuser on vm.local:2121" and the WebDAV row "vm.local:8080". The GVfs mount names leak through. Name rows like Windows' network locations ("Harbor Files (FTP)", "nasuser's files"), or at least add the protocol. Code: `native/crates/ox-app/src/window/sidebar/entries.rs` (`network_entry`).
- **L2.** Network rows show a long dash (U+2014) under Date modified and an empty Size, and screen readers read the dash. Windows' Network folder in Details view shows Name and Category. Code: `native/crates/ox-core/src/entry.rs:266`.
- **L3.** `GtkGizmo (progress) reported min width -2` on every visit to This PC (review 4, still there: 80 times in the soak). Code: `native/crates/ox-app/resources/skin/landing.css:64`.
- **L4.** The Gallery and the WebP thumbnail were blank for `Beach.webp`. That came from this machine's missing gdk-pixbuf WebP loader, not the app, but the tile shows a plain light-blue box with no "can't show" icon.

## 5. Worked well

- **Network sign-in.** A wrong password asks again with the user name kept, the right one opens the share, and after a restart the saved password (gnome-keyring) opened `Private` without asking.
- **SMB, SFTP, FTP and WebDAV** all browse, open, search and preview. A share whose server is down shows "This location is unavailable" with Try again.
- **Map network drive** maps `\\vm.local\Projects` as "Projects (Z:)", saves it, and lists it under This PC > Network locations with a capacity bar.
- **Search**: live results while typing, `kind:music` and `size:>1MB` filters, search in a share and in subfolders, and stays responsive with a frozen server.
- **Home** with "Open Quick access on Home": Quick access, Recent (a file opened in LibreOffice showed up with its folder), Favorites (Add to Favorites in the right-click menu, then Remove).
- **Gallery**: pictures from all known folders by month; the Details pane works, and Set as background and Rotate appear for a selected picture (not clicked).
- **Recycle Bin**: Restore, Restore all items, conflicts and a missing original folder (see section 1).
- **Properties**: General, Security, Details and Previous Versions tabs for mp3 (title, artist, album, length, bit rate), jpg (dimensions, bit depth), mp4 (frame size, data rate, frame rate), docx, pdf and zip, with correct sizes.
- **Preview pane**: PDFs (all pages, scrollable, after bubblewrap), text, json, html, md, rtf; broken files say "No preview available.".
- **Details pane**: one file, a folder and 18 items ("18 items selected").
- **Archives**: Compress to ZIP, 7z and TAR, on local disks and on a share; Extract all on a share; name clashes add " (2)".
- **Tabs and windows**: Ctrl+Shift+T reopens a closed tab, Duplicate tab, dragging a tab out makes a new window, Ctrl+N and Ctrl+W, F11, and "Reopen my tabs" restores both tabs after Ctrl+Q.
- **Settings**: General, About; "Check for updates" explains a failed check ("Couldn't check for updates (HTTP 403). Try again later.") with Check again. The 403 was this sandbox's proxy.
- **New**: Bitmap image, Rich Text Document, Text Document and Compressed (zipped) Folder each made a valid file (checked with `file`), and `~/Templates/Letter.docx` appeared as "Letter" in the menu. Folder and Shortcut were not tried.
- **Clipboard with other apps**: Ctrl+C offers `x-special/gnome-copied-files`, `text/uri-list`, text and the KDE cut flag. Pasting files copied by another app (`gnome-copied-files` and `text/uri-list`) works; pasting plain text does nothing (correct).
- **Drag and drop to another app**: a GTK 4 window accepting `GdkFileList` got `file:///home/oxtest/Desktop/Chime.flac`; a text drop target got the path.
- **Keyboard**: Ctrl+D deletes to the Recycle Bin and Ctrl+Z brings it back; Ctrl+E, Alt+D, Alt+P, Alt+Shift+P, Alt+Enter, Shift+F10, Ctrl+T, Ctrl+W, Ctrl+N, F5 and F11 all did what Windows does.
- **Screen-reader names** (AT-SPI): toolbar buttons with their shortcuts, the search box ("Search Network"), the navigation pane rows with each path or share address and its state ("\\vm.local\Projects · Connected"), the tabs, and the file rows and cells.
- **Long session**: see section 1. No growth in open files or threads.

## 6. Counts

| Section | Count |
|---|---|
| Crashes, hangs, data loss | 0 found (20-minute soak, frozen server, name clashes, Restore conflicts, broken files) |
| Broken everyday actions | 6 (B1-B6) |
| Unlike Windows, Microsoft-confirmed | 2 (U1 with 7 parts, U2) |
| Unlike Windows, needs VM | 8 (V1-V8) |
| Looks | 4 (L1-L4) |
| Worked well | 19 |
| stderr | 0 Gtk-CRITICAL, 0 panics; 1 known Gtk-WARNING kind (80 times) |

## Not tested

A real Windows PC or NAS on the network (WS-Discovery, NetBIOS), NFS, kernel CIFS mounts, Orca speaking (names were read through AT-SPI only), reduced motion in practice (only the code path that checks `gtk-enable-animations` was read), Wayland, HiDPI, Office previews (no previewer in Windows without Office either), the packaged .deb, and translations.

## VM CHECK (Windows 11, tomorrow)

Use fictional names. Do Network first.

**Network**
1. On the same LAN, put one Windows 11 PC with a shared folder and one NAS or Linux Samba box (advertising mDNS and WSD). Leave "Network discovery" on.
   - Open File Explorer > Network.
   - Write down every row, its Category and Type, and whether each machine appears once.
   - Switch to Details view and note the columns.
2. Turn network discovery off (Settings > Network & internet > Advanced network settings > Advanced sharing settings).
   - Reopen Network. Copy the banner's exact text and what clicking it offers.
3. Double-click the Samba box.
   - Note whether a share named `Test$` is listed. Turn on "Show hidden items" and check again.
4. Right-click the computer > Properties. Note what opens, or whether there is no Properties row.
5. Expand Network in the navigation pane. Note what the children are: computers or shares.
6. Open a share that needs a password.
   - At the prompt, note whether "Remember my credentials" starts checked, and where the focus is.
   - Type a wrong password: note the message and whether the user name is kept.
7. Map network drive.
   - This PC > "..." > Map network drive, `Z:` to `\\<server>\<share>`, Reconnect at sign-in on.
   - Note where `Z:` appears: under This PC only, or also under Network?
   - Type `Z:\` in the address bar.
8. Right-click `Z:` > Disconnect.
   - Note whether a question appears, and whether `Z:` stays anywhere afterwards.
9. Add a network location (This PC > right-click empty space > Add a network location) for an FTP or WebDAV address.
   - Note where it is listed and how it is named.

**Other**
10. Search in Documents for a word that is only inside a .docx's text, and for a file name that is only inside a .zip. Note whether each is found.
11. Properties > Details of a .docx and of a .pdf that have a title and author set. List the rows shown.
12. Settings > Accessibility > Contrast themes > "Night sky". Open File Explorer and note the background, text and selection colors.
