# Review 6: the areas no job owns, OpenXplorer 2.0.23

Upstream main `5d21fd3e` (2.0.23), `cargo build --release --locked`, default application ID `io.winspace.Development.Native`. Report only: no app code was changed.

Left out because other jobs own them: copy/move, undo, rename, the navigation pane, the address bar, right-click menus, views/sort/group, Open with, scaling, Network, icons, the sorted header, owners, Home/Recent, New, the Details pane and status bar, startup/memory, tabs/windows/split view, error messages, clipboard, shortcuts, USB/drives, accessibility, fuzzing, the soak test, the thumbnail cache and office previews. Where a finding below touches one of those (for example an error message shown on a page this review owns), it is listed because of what it breaks here, and the owning area is named.

## How it was run

- Two unprivileged users (`oxtest`, `oxtest2`), each with a throwaway HOME, its own private Xvfb display (1600x1000, Openbox, xcompmgr so menus draw as on a desktop), its own D-Bus session, GVfs, gnome-keyring, the AT-SPI bus, and later `xdg-desktop-portal` with the GTK backend. Never the live desktop.
- The app ran under `gdb -batch` so a crash would print a backtrace (`SIGPIPE` passed through; see Counts). stderr of every run was kept and read.
- Real input with xdotool: clicks, drags, keys and typing.
- A small GTK 4 drop-target window (a test app written for this review) received drags from OpenXplorer and logged the formats and URIs it was offered.
- Environment: Ubuntu 24.04 libraries, GTK 4.14.5, GLib 2.80.0 (`6ubuntu3.10`), LibreOffice as the desktop's default for text.
- Fictional files, all generated: documents (txt, csv, md, rtf, json, html, docx, xlsx, a real PDF, a broken PDF, random bytes named `.docx`, an empty file, a file with no extension, `archive.tar.gz.part`), 24 JPEGs with EXIF dates 2019 to 2025 plus panorama, tall, transparent PNG, WebP, BMP, TIFF, animated GIF, SVG, a truncated JPEG and text named `.png`; mp3, ogg, flac, wav, webm; ZIP, 7z, tar and tar.gz/xz/bz2/zst, password ZIP, header-encrypted 7z, corrupt ZIP, Unicode ZIP, `../` traversal ZIP, backslash ZIP, gz and xz single files.
- Names: Japanese, Russian, Arabic, Hebrew, Greek, Korean, emoji, NFC and NFD "café", Windows-forbidden characters (`: ? * | " < > \`), trailing dot and space, leading space, three case variants, `CON`, `nul`, a 254-byte name, a zero-width space, a name that is only a zero-width space, a newline, a tab, a right-to-left override and invalid UTF-8; a `.hidden` list.
- Special files: relative and absolute symbolic links to files and folders, a dangling link, a two-link loop, `self -> .`, links to `/`, `/etc/shadow` and `/dev/null`, a hard link, FIFOs named `.txt`, `.zip`, `.jpg`, a socket, a 4 GB sparse file.
- Permissions: folders with modes 000, 444, 111 and 555; files with 000, 444 and 111.
- Dates: 1960, 1970-01-01 00:00 UTC, 1999-12-31 23:59:59, the 2025 US DST start, 2038-01-19, 2100, yesterday, last week, a minute ago.
- Paths: 30 nested 41-byte folders plus 80 more (3,633 bytes), then more levels to 4,338 bytes (past Linux's 4,096-byte `PATH_MAX`).
- A 49,920-file archive tree for search speed.
- Locales `en_US`, `en_GB`, `de_DE`, `ja_JP` (date formats only; UI translations were not looked at), time zones America/New_York and Asia/Kolkata, and a system time-zone change while the app ran.
- The one-hour mixed session: see "One hour of mixed use" under section 5.

Paths below are written as `~/...`.

## 1. Crashes, hangs and data loss

No crash, hang or Rust panic in these areas. The main-loop watchdog of the one-hour session saw no stall over 1 s. One silent data gap:

### D1. Compressing a folder leaves out every symbolic link, FIFO and socket in it without saying so, so a "backup" archive can lack files the user sees

- **Steps**
  1. A folder holding normal files and symbolic links, here `~/Documents/Special` (a link to `../Meeting notes.txt` shown as "link to notes", Type "Text Document", 2 KB; a link to the `Projects` folder; plus FIFOs and a socket).
  2. Right-click the folder > Compress to > ZIP File (or TAR File, or Additional options > tar > Create).
  3. List the archive: `unzip -l ~/Documents/Special.zip`.
- **What happens**: the archive holds only `hardlink notes.txt` and `sparse 4G.bin`. All 12 links and special files are missing, including "link to notes" and the whole linked `Projects` folder. No message is shown: Compress to > ZIP File says nothing at all, and the Additional options dialog closes silently. The app does count them (`skipped_count`) and builds the sentence "N links or special files were left out.", but nothing shows it.
- **What Windows does**: Windows 11's Create archive dialog has "Retain symbolic links" (unchecked by default). Unchecked, the link's target is stored, as far as I know; this needs a VM check (VM CHECK 4). Either way Windows does not drop the files without a word.
- **Suggested fix**: at least show the existing sentence when `skipped_count > 0`, on both paths (a message, not only a toast, since this is about missing data). Better: follow links to regular files and folders inside the selection when "Retain symbolic links" is off (as the checkbox label implies), guarding against loops and links that leave the selected tree, and keep FIFOs/sockets skipped.
- **Code**:
  - `native/crates/ox-app/src/window/archive_actions.rs` about lines 421-457 (`conclude_compression` never looks at `skipped_count`).
  - `native/crates/ox-app/src/window/compress_dialog.rs` lines 240-244 build `compression_success_text` (with the count), then `report_in_folder` (`archive_actions.rs` about line 488) drops every success text ("success is not mentioned").
  - `native/crates/ox-app/src/archive_view/compress.rs` lines 59-77 (the unused sentence).
  - `native/crates/ox-core/src/archive/create.rs` line 21 and lines 486-555 (links and special files skipped).
  - `compress_dialog.rs` lines 145-151: "Retain symbolic links" and "Retain hard links" are always unticked and greyed.

## 2. Broken everyday actions

Ranked by how soon a Windows user hits them.

### B1. A new, empty text file can't be opened: "No application is installed for this file type"

- **Steps**
  1. Any folder, New > Text Document, press Enter to keep the name "New Text Document.txt".
  2. Press Enter again (or double-click it).
- **What happens**: "Could not open the item. No application is installed for this file type. Use Open with… to choose one." The same for any 0-byte `.txt` (`~/Documents/Empty file.txt`). Properties > General says "Opens with: No default application", while a non-empty `.txt` next to it says "Opens with: LibreOffice Writer". The Type column correctly says "Text Document" for both.
- **What Windows does**: opens it in Notepad (the `.txt` default). Creating a text file and opening it to type in it is one of the most common things people do.
- **Why**: GIO sniffs every empty file as `application/x-zerosize` (`gio info` shows it), and nothing is registered for that type. The Type column already special-cases it (`EMPTY_FILE_TYPE` in `ox-core/src/entry/type_label.rs` line 44) and uses the name; opening does not.
- **Suggested fix**: when the content type is `application/x-zerosize`, use the type guessed from the name alone (`gio::content_type_guess(Some(name), None)`) for the default app, the Open with list and Properties' "Opens with", as the Type column does. Nautilus does the same for empty files.
- **Code**: `native/crates/ox-core/src/integration/opening.rs` lines 139-141 (`entry.content_type` passed straight to `default_for_type`); `native/crates/ox-app/src/properties/item_window.rs` about line 533 and `properties/metadata.rs` about line 328 ("Opens with"); `native/crates/ox-app/src/integration/applications.rs` about lines 231 and 272 (Open with).

### B2. Share does nothing at all when the desktop can't compose the email, and never says why

- **Steps**
  1. A desktop with `xdg-desktop-portal` and its GTK backend but no email app set up (here a fresh Ubuntu 24.04 session; on a real desktop: no Thunderbird/Evolution, or webmail only).
  2. Select `~/Desktop/New Text Document.txt`, click Share in the command bar.
- **What happens**: nothing visible. Three tries; the window was checked several times over two minutes after each, and after the third it was filmed once a second from 112 s to 134 s after the click (every frame identical, no toast or message). `dbus-monitor` shows the portal's answer arriving about 1 ms after the call: `org.freedesktop.portal.Request.Response` on `/org/freedesktop/portal/desktop/request/1_48/openxplorer_email1` with code `2` (failed). The app's code turns code 2 into an error message, so the answer is not reaching it.
- **What Windows does**: Share opens the share window (Microsoft: "To share a file, select it, then select **Share** on the ribbon", https://support.microsoft.com/en-us/windows/file-explorer-in-windows-ef370130-1cca-9dc5-e0df-2f7416fe1cb1), and an email target that can't open says so.
- **Likely cause**: `compose_email` subscribes to `Response` with the sender set to the well-known name `org.freedesktop.portal.Desktop`. Since the GLib fix for CVE-2024-34397 (backported to Ubuntu's 2.80.0), GDBus matches a well-known sender only after it has learned the name's owner, which it asks for asynchronously; an answer that comes back within a millisecond can be dropped. The 120 s timeout should then show "Unanswered", but no message appeared within 134 s either, so the timeout path deserves a look too. The same pattern may affect a fast successful answer (code 0): the email app would open, but the app might show an error two minutes later; not tried here (no email app).
- **Suggested fix**: subscribe with no sender (or the portal's unique name from `GetNameOwner` before the call) and match the object path, which the portal documentation lets the caller compute in advance from its own unique name and the token. Show "Opening your email app…" right away and a clear message on failure ("No email app is set up").
- **Code**: `native/crates/ox-core/src/integration/email_portal.rs` lines 196-234 (`subscribe_to_signal(Some(portal), …)`, then the wait); `native/crates/ox-app/src/window/share.rs` lines 224-230.

### B3. "Index pinned folders automatically" is on, Documents shows a pin, yet nothing is indexed

- **Steps**
  1. First run. Settings > Search: "Index pinned folders automatically" is on; "Instant search is off". The navigation pane shows Desktop, Downloads, Documents, Pictures, Music and Videos with pin marks.
  2. Indexed folders > Manage…: "No folders are indexed yet. Add one below, or pin a folder to Quick access."
- **What happens**: the built-in Quick access folders are not pins in the settings file (`"pins": []`), so the setting does nothing for them, while the page tells the user to pin folders that already look pinned. Searches in Documents walk the disk (fast enough here: about 1.5 s for 50,000 files) and file contents are off until chosen per search.
- **What Windows does**: indexes the common user folders out of the box. Microsoft: Classic mode "indexes your Documents, Pictures, and Music folders plus the desktop by default." (https://support.microsoft.com/en-us/windows/search-indexing-in-windows-da061c83-af6b-095c-0f7a-4dfecda4d15a)
- **Suggested fix**: treat the default Quick access folders as pins for indexing (or index the standard folders on first run, opt-out), and word the empty state so it does not ask for something the user seems to have done.
- **Code**: `native/crates/ox-app/src/search/pin_sync.rs` lines 44-62 (only settings pins count, and the first reading only records); `native/crates/ox-app/src/settings_page/indexed_folders.rs` (empty-state text).

## 3. Unlike Windows

### Microsoft-confirmed

**U1. Hidden renames the file.** Properties > General > Hidden > Apply turned `Meeting notes.txt` into `.Meeting notes.txt` (I renamed it back from a terminal). Any reference to the old name breaks: other apps' recent lists, scripts and links (the corpus's `link to notes.txt` points at the old name). Microsoft: hidden is an attribute, "The file or directory is hidden. It is not included in an ordinary directory listing." (https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants). Renaming onto an existing dot-name is refused safely ("An item named ".Plan.txt" already exists. Nothing was overwritten."). Suggested fix: add the name to the folder's `.hidden` file instead (GIO and the app already honor it), and keep the rename only where no `.hidden` file can be written. Code: `native/crates/ox-app/src/properties/selection_attributes.rs` lines 10-14 and the General tab's Hidden box.

**U2. Dates at or before 1970-01-01 show the "unknown" dash.** A file dated 1970-01-01 00:00 UTC and one dated 1960 show the unknown-date dash (U+2014) in Date modified, the Recycle Bin and Properties ("Not provided"), and sort as unknown. Windows shows them: file times count "100-nanosecond intervals since January 1, 1601 (UTC)" (https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime). Cause: times are `u64`, `0` is read as "none", and GIO's signed time read through `attribute_uint64` overflows for negative values. Suggested fix: keep times as `i64` (`time::modified` is signed in GIO; read it with `info.modification_date_time()`), and use `None` only when the attribute is absent. Code: `native/crates/ox-core/src/entry/attributes.rs` lines 33-39; `native/crates/ox-core/src/format.rs` lines 292-297 and 419-422.

### Needs a VM check (exact steps in VM CHECK)

**U3. Copying a file symbolic link copies the link, so a relative link breaks.** Copy `~/Documents/Special/link to notes.txt` (`-> ../Meeting notes.txt`, shown as "Text Document", 2 KB) and paste it on the Desktop: the Desktop gets a dangling link (`cat` fails), shown as "Text Document, 1 KB". A copied folder link (`link to Projects`) is still a link to the same folder, so anything deleted inside the "copy" is deleted from the original (inferred; I did not delete inside it). The engine copies links as links on purpose (XFER-017). Windows Explorer copies the target's contents as far as I know (VM CHECK 5). Suggested fix: when the source is a link to a regular file or folder on the same computer, copy what it points to (as Windows does), or at least resolve relative links against the source folder so the copy still works. Code: `native/crates/ox-core/src/transfer/copy.rs` line 197, `transfer/node.rs` line 26.

**U4. A tab whose folder is deleted elsewhere stays on a raw error page.** Open `~/Work/New folder` in a tab, delete it from a terminal, switch to the tab: "This location is unavailable" over "Error opening directory '/home/…/Work/New folder': No such file or directory", under the Network icon, with Try again. Windows moves the window to the nearest folder that still exists (VM CHECK 6). Code: `native/crates/ox-app/src/window/empty_page.rs` lines 66-82 (every non-server error is shown as GIO's text) and 252 (`Icon::Organization` for every unavailable page). The wording is the error-messages job's; the navigation is this review's.

**U5. A running window ignores a system time-zone change.** With the app open on New York time, change `/etc/localtime` to Asia/Tokyo (what Settings > Date & Time does through systemd-timedated), press F5: Date modified still shows New York times; only a restart picks Tokyo up. Windows redraws times at once (VM CHECK 7). GLib caches the local zone. Suggested fix: watch `/etc/localtime` (a `gio::FileMonitor`) or timedated's `PropertiesChanged`, then build the local zone again from the file (`glib::TimeZone::from_identifier`) and redraw. Code: `native/crates/ox-core/src/format.rs` lines 419-422 (`DateTime::from_unix_local`).

**U6. No Print.** Neither the modern menu nor Show more options has Print for a `.txt`, `.pdf` or picture. Windows' classic menu has Print for printable types (VM CHECK 8). No Microsoft page found that documents it.

**U7. Search options > File contents is not Folder Options > Search > "Always search file names and contents", and it is forgotten on restart.** Ticking File contents in Search options found `find-me.txt` by its text, but Folder Options > Search still showed "Always search file names and contents" unticked, and after a restart File contents was unticked again. In Windows these are one setting that stays (VM CHECK 9). Code: the File contents item in `native/crates/ox-app/src/window/command_bar/menus.rs`, and `window/imp.rs` lines 283-285 (`searches_contents`, which follows Folder Options but is not saved back by the menu).

**U8. Japanese dates use 年月日.** With `LANG=ja_JP.UTF-8` Date modified reads "2000年01月01日 05時29分"; Windows' ja-JP short form is "2000/01/01 5:29" (VM CHECK 10). The code says it chose this (`format.rs` line 290); en_GB ("01/01/2000 05:29") and de_DE ("01.01.2000 05:29") match Windows.

**U9. Folder links are typed "Shortcut"; file links take their target's type.** In `~/Documents/Special`, `link to Projects`, `self` and `loop-a` read "Shortcut", while `link to notes` reads "Text Document" and `dangling` reads "Text Document, 1 KB". Windows shows a directory symbolic link as "File folder" with the arrow overlay (VM CHECK 11).

**U10. A "Give us a star on GitHub" popup after a few launches.** "Enjoying OpenXplorer? Give us a star on GitHub to let us know!" appeared over Settings after a few starts (the settings file counts launches, `starPromptLaunches`). Windows never asks. Suggest showing it only from About, or once and only when the user opens About. Code: `native/crates/ox-app/src/window/star_prompt.rs`.

## 4. Looks

**L1. Local errors use the Network icon and GIO's raw text, and the PATH_MAX page selects all of it.** Opening a folder past 4,096 bytes (`~/Documents/Long paths/…/over 08 …`) shows "This location is unavailable" over the Network icon, then the full 4 KB path ending "File name too long", all of it selected in blue. "Open as administrator…" without an authentication agent shows the same icon with "Permission denied". Windows: "The file name(s) would be too long for the destination folder" / "Location is not available" with a folder icon. Suggested fix: a folder icon for local paths, a short sentence ("The path is too long."), the raw text under Details, and do not focus (select) the label. Code: `native/crates/ox-app/src/window/empty_page.rs` lines 66-82 and 252.

**L2. The "Interrupted Action" window for a too-long path is all path.** Copy `~/Documents/Long paths` to the Desktop: the window's body is "Error creating directory /home/oxtest/Desktop/Long paths/very long … : File name too long" (about 4,300 characters), so Try Again, Skip and Cancel are scrolled out of sight until the user scrolls. Skip then worked (120 of 126 folders copied, the too-deep part left out). Windows names the item and says "Destination Path Too Long". Suggested fix: show only the item's name and the reason; keep the path under More details, ellipsized in the middle. Code: `native/crates/ox-app/src/window/file_ops/failure_dialog.rs`. (Copy/move's area; listed for long paths.)

**L3. Hebrew and Arabic names sit at the far right of the Name column.** `עברית.txt` and `مستند عربي.txt` are drawn right-aligned against the Date column, far from their icons, in the folder list and in the Recycle Bin (screens of `~/Documents/Names`). Windows keeps every name next to its icon and lets the text itself run right to left. Suggested fix: give the name label a left-to-right base direction (prefix the displayed text with U+200E, or set the layout's auto-direction off) so only the characters run RTL. Code: the name labels in `native/crates/ox-app/src/folder_view/cells.rs` (about lines 640-715).

**L4. A local indexed folder reads "Incomplete / offline" when one subfolder can't be read.** After indexing `~/Documents` (50,334 names), its row says "Incomplete / offline" because `Restricted/no access` (mode 000) could not be listed. Nothing is offline. Suggest "Indexed (1 folder could not be read)" with the folder named on hover. Code: `native/crates/ox-core/src/search/crawl.rs` lines 254-261 and `search/root.rs` line 110.

**L5. Previous Versions of a local file mentions "your server".** "No matching previous versions were found in readable snapshot folders. This does not prove that your server has no snapshots or backups." for `~/Documents/Meeting notes.txt`. Windows: "There are no previous versions available." Code: `native/crates/ox-core/src/versions/lookup.rs` about line 37.

**L6. Create archive dialog.** With format tar and method None, the Compression level slider stays active. Method offers None, GZip, BZip2 and XZ, but not Zstandard, though `.tar.zst` opens and extracts (VM CHECK 4 for Windows' list). Code: `native/crates/ox-app/src/window/compress_dialog.rs` (`offer_methods`, `follow_method`).

**L7. Alt+D reaches the address bar behind an in-window dialog.** With Extract Probe.zip open (or "Compression stopped"), Alt+D turns the dimmed address bar into a selected edit field behind the dialog; typing still goes to the dialog. Harmless, but it looks like the dialog lost focus. Code: `native/crates/ox-app/src/dialog/layer.rs` (window accelerators are not blocked while the layer is up).

**L8. Gallery's year scale skips years.** With photos dated 2019 to 2025 and some today, the scale shows 2026, 2025, 2024, 2022 on one screen and 2026, 2024, 2023, 2022 on another; the heading reads "November 2026 – August 2019". Code: `native/crates/ox-app/src/window/gallery.rs`.

## 5. Worked well

- **Search**: `*.txt` 86 (all readable `.txt`), `name:item` 30, `size:>1MB` 1, `"meeting notes"` 1, `ext:.txt` 85, `kind:folder`, `date:today`; contents search finds `find-me.txt` three folders down; 49,920 files searched in about 1.5 s without an index; indexing Documents (50,334 names) then a file created in a terminal showed up in a search 0.6 s later ("1 result · Cached"); natural sort in results (7, 37, 72…).
- **Recycle Bin**: Delete without asking, as Windows 11 does; Japanese, Arabic, Hebrew, accented and invalid-UTF-8 names deleted and restored intact; Restore all items; Empty Recycle Bin asks "Are you sure you want to permanently delete these 2 items?"; emptying a trashed `self -> .` link removed only the link.
- **Properties**: Security > Edit (Others: Allow Write gave `-rw-rw-rw-` as asked), two-row tabs for Documents like Windows, folder size with "≥" for unreadable parts, Hidden refusing to overwrite an existing dot-name.
- **Folder Options**: Show hidden and Hide extensions apply to the open window on Apply; hidden items drawn dimmed.
- **Compress and extract**: Compress to ZIP, 7z and TAR; Additional options (zip/7z/tar, GZip/BZip2/XZ); a folder with an unreadable subfolder stops with "Compression stopped … No existing file was replaced." and leaves no partial file; extract into an existing folder asks "Replace or Skip Files", "Let me decide" lists the three files, Cancel leaves the old file untouched; extracted files are `0664` now (review 4's `0600` is fixed).
- **Send to**: Documents (in Documents it makes "Meeting notes - Copy.txt", as Windows does), Desktop (create shortcut), Compressed (zipped) folder.
- **Drag and drop to another app**: a real drag from the file list offers `GdkFileList`, `text/uri-list`, `application/x-kde4-urilist` and `text/plain`; dropping a file from inside a ZIP hands over an extracted copy.
- **Symbolic links and special files**: links, loops, `self`, FIFOs, a socket and a 4 GB sparse file list without stalls; copying `self -> .` copies the link (no runaway copy); Undo Copy after the copy was deleted elsewhere says "Item Not Found" and deletes nothing.
- **Permissions**: 000 and 111 folders say "Location is not available … Access is denied." (Windows' words); a 444 folder lists names.
- **Long paths**: browsing down 120 levels to 4,090 bytes, Skip past the too-long part of a copy.
- **Themes**: System follows `color-scheme` changes live, and an open Properties window follows too.
- **Dates and locales**: en_US 12-hour, en_GB and de_DE forms, Asia/Kolkata half-hour offset, the DST start (3/9/2025 3:00 AM), 2038 and 2100, Downloads' "Today"/"Yesterday" groups across midnight.
- **Names**: every Unicode and odd name listed, sorted and deleted correctly; Windows-forbidden characters shown as they are.

### One hour of mixed use

User `oxtest2`, its own display, scripted but real xdotool input: a seeded random mix about every 0.5 to 3 s of typing folders into the address bar, clicking the navigation pane, arrow keys, opening items (the user's default app for every type was a silent test viewer so no other window took the keyboard), Back/Forward/Up, Details and Preview panes, all eight views, search with typing and arrowing through results, new folder plus rename plus Delete plus Undo in `~/Work`, copy from `~/Reference` and paste into `~/Work` (answering the name-conflict window when it came up), new tabs, Properties, Select all, F5, zoom, Gallery with scrolling, the Recycle Bin, right-click menus and Compress. A script created and deleted a file in `~/Work` every 2 s throughout. A watchdog asked the app's main loop over D-Bus (`org.gtk.Actions.List`) every 0.5 s.

- 857 actions in 60 minutes (119 arrow-key runs, 102 address-bar trips, 64 searches, 51 copy/paste, 47 tabs, 45 new folders, 38 Properties, 29 Gallery visits, 7 compresses).
- 0 crashes, 0 panics, 0 main-loop replies slower than 1 s.
- `~/Reference` (62 files) unchanged by md5 at the end.
- Memory (RSS) 368 MB at the start (the app had already run 12 minutes), 555 MB at 10 minutes, 597 MB at 21, 613 MB at 32, 616 MB at 43, 622 MB at 54, 680 MB at the end; threads 24 to 39, open files 31 to 43. Most of the growth is in the first 20 minutes (Gallery and thumbnails), then it levels off. The soak job owns memory; listed for its numbers.
- stderr: 8 x `Gtk-WARNING: GtkText - did not receive a focus-out event` (after inline renames; review 5 saw it too), nothing else from the app.
- Felt off: nothing new beyond the items above. One observation for the New/rename owners: 3 of 52 folders made with Ctrl+Shift+N kept the name "New folder" when the new name was typed 0.8 s after the shortcut; the other 49 took it. Not investigated.

## 6. Counts

- Findings: 1 data item (silent omission when compressing), 3 broken everyday actions, 2 Microsoft-confirmed differences, 8 needs-VM differences, 8 looks. 13 worked-well areas plus the one-hour session.
- Runs: 13 app starts across two users (en_US, en_GB, de_DE, ja_JP; New York, Kolkata, a live switch to Tokyo), one 60-minute mixed session with 857 actions.
- Crashes 0, hangs 0 (no main-loop stall over 1 s), Rust panics 0, Gtk-CRITICAL 0.
- Warnings (app lines only): 8 `GtkText - did not receive a focus-out event` (inline rename, known); 2 `openxplorer-WARNING: Could not update the search cache: Only local paths, smb:// locations and connected devices are supported in this build.`, printed when the Recycle Bin was emptied or items were restored from it: the cache is handed `trash:` addresses it cannot take (`native/crates/ox-app/src/search/cache/operations.rs` line 234; it should skip `trash:` instead of warning). Environment lines left out: the UDisks2 volume monitor (13), "bubblewrap is not installed" (2), one dconf profile warning and one C-locale warning before the locales were generated.
- gdb: three runs stopped on `SIGPIPE` in the `ox-picture-feed` thread (`picture_guard::decoder::feed` writing to a decoder child that had exited). Rust ignores `SIGPIPE`, so outside gdb the write just fails with `EPIPE`; not a crash. Later runs passed `SIGPIPE` through.
- Data checks: `~/Reference` md5 unchanged after the hour; Hidden's rename refused to overwrite; Undo Copy with the copy gone deleted nothing; emptying a trashed `self -> .` link removed only the link; a cancelled extract left the old file (`one`) in place.
- Not tested: the Share success path (no email app), "Open as administrator…" with a real authentication agent, the update check (the container's proxy answered HTTP 403), printing (no Print command to test), Wayland, HiDPI, the packaged .deb, NFS or CIFS, translations.

## VM CHECK

On Windows 11 24H2 with File Explorer, in a test user's Documents:

1. **Empty text file (B1)**: right-click > New > Text document, Enter, Enter. Expected: Notepad opens.
2. **Share with no mail app (B2)**: select a `.txt`, Share. Note what the share window offers and what happens with Outlook/Mail not set up.
3. **Indexing (B3)**: a fresh user: Settings > Privacy & security > Searching Windows > note the default indexed locations; search Documents for a word inside a `.txt`.
4. **Create archive (D1, L6)**: make `C:\T\Links` with a file, `mklink link.txt file.txt` and `mklink /D linkdir C:\T\Other` (admin or Developer Mode). Right-click Links > Compress to > Additional options. Note the Method list for tar, whether Level is active for None, and, with "Retain symbolic links" unticked, whether the archive holds the targets' contents, links, or nothing; and whether Explorer says anything.
5. **Copy a file symlink (U3)**: with `mklink notes-link.txt ..\notes.txt` in a subfolder, copy `notes-link.txt` to the Desktop. Is the Desktop item a file with the text, or a link? Same for a directory link (`mklink /D`).
6. **Folder deleted while open (U4)**: open `Documents\Temp` in a tab, delete it from `cmd` (`rmdir Temp`), switch to the tab. Where does it go?
7. **Time-zone change (U5)**: Explorer open in Details view, change the time zone in Settings, look at Date modified without refreshing, then after F5.
8. **Print (U6)**: right-click a `.txt`, a `.pdf` and a `.jpg` > Show more options: is Print there, and in the modern menu?
9. **File contents (U7)**: Search options > Advanced options > File contents; then Folder Options > Search: is "Always search file names and contents" ticked? Close and reopen Explorer: is File contents still on?
10. **Japanese dates (U8)**: Region format Japanese; Date modified in Details view.
11. **Symlink types (U9)**: the Type column of a `mklink /D` folder link, a file link and a `.lnk`.
12. **RTL names (L3)**: files named `עברית.txt` and `مستند عربي.txt` in Details view: left next to the icon, or right?
