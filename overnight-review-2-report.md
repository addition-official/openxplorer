# Overnight review 2: OpenXplorer main 0406518

Reviewed upstream main `04065186446251157309247ec9ee00c421c0f9d2` (after #327). This is a report only: no code changes, no branches, no PRs.

**Rank** is how soon someone switching from Windows would notice it: 1 = in the first minutes, 5 = rarely.

**Reproduced** says how each finding was checked:
- "Yes (X11)" or "Yes (Wayland)": driven with real xdotool input on the release build.
- "Code reading only": not run.

Items already open as PR #325 or #326 are left out.

## How it was tested

- **Build.** A release build of 0406518, run as `tester` with Rust 1.92.0.
- **Isolation.**
  - X11: a private Xvfb display with a private D-Bus session and a throwaway HOME, under `/home/tester/runs/<n>`.
  - Wayland: Weston (x11 backend) nested inside a private Xvfb display.
  - Never on a live desktop.
- **Fictional files.** Documents holds Budget 2026.txt, Notes 1-3.md, plan.pdf, Archive.zip, Bundle.tar.gz and Projects/Sub. Downloads holds files of several ages.
  - Big folders: 12,000 files in Big, 10,000 files in Downloads, and 3,000 pictures in Pictures.
  - Recent files: a recently-used.xbel naming those fictional files.
- **Combinations.**
  - 1x and 2x (`GDK_SCALE=2`), light and dark, normal and Compact view.
  - Classic-menu setting on and off.
  - Tab menu and navigation pane settings on and off.
  - Home "Open Quick access on Home" on and off.
  - Split view, and the zh_CN catalog.
- **Network.** A local Samba share on 127.0.0.1 with fictional pictures, opened through GVfs (`smb://`). Also run:
  - through a throttling proxy at 30 KB/s;
  - with the server stopped while the folder was open.
- **Open/Save dialog.** A script took the portal's bus name on the private bus and called the app's `org.freedesktop.impl.portal.FileChooser.SaveFile`, then the dialog was driven with xdotool.
- **Tests.**
  - `native/tools/check.py`: the parity inventory, i18n and driver checks ran, but the driver's own self-test `test_a_hung_test_leaves_no_display_bus_or_temporary_files` failed in this container (the timed-out child "died with SIGTERM"). That stops check.py, so the remaining stages were run with check.py's own functions: no drawn icons, rustfmt and Clippy (passed), then every test executable isolated (results under "Tests" below).
  - `native/parity/check.py`: not run.
- **Installed for this run:** xdotool, samba, smbclient, gettext and the en_US/zh_CN locales.
- **Environment limits.**
  - Xvfb has no window manager, so dialogs open at the top-left and menus have black corners. On X11, rows level with an open dialog also lost their Name text; that did not happen under Weston, so it is left out as an artifact.
  - Update checks reached GitHub only through a proxy that answered 403.
  - No kernel CIFS mount, phones, USB drives or real Windows machine.

## Tests

Every test executable ran isolated with check.py's own runner (81 executables plus doctests): **4,542 passed and 11 failed**. All 11 look environmental:
- `location_fixture_drift` (1 test) and `network_python` (1 test): the Python fixture comparisons can't import `gi` under the container's Python 3.13, the same environment problem as in the first review.
- `ox_core` (9 tests, all in `admin_copy::tests`): the checks refused the tests' temporary folders under this container's /tmp, which the tests expect them to accept. They fail on the cautious side. The details go to the manager separately.

No failure points at an app regression.

## A. Bugs and edge cases

| Rank | What happens | Exact steps | Suggested fix | Reproduced |
|---|---|---|---|---|
| 1 | **Ctrl+A while renaming selects every item in the folder.** Esc then leaves all of them selected, so a following Delete recycles the whole folder. This was in the first review and is still there. | Documents, click a1b, F2, Ctrl+A. The status bar says "13 items selected". Esc: still 13 selected. | The window-level Ctrl+A in `window/file_ops/shortcuts.rs:52`, `:104-119` doesn't see the rename box as a text field. Check for an in-place rename (or `gtk::Text` focus inside the rename popover), not just the window's focus widget. Add a test that types Ctrl+A in the rename box. | Yes (X11) |
| 2 | **Tooltips stay on top of menus, the rename box and the next folder.** This was in the first review and is still there, on Wayland too. | (1) Hover a file, then right-click or Shift+F10 it: the file's tooltip covers the menu. (2) Right-click a column title: "Drag to resize · double-click to fit loaded items (up to 2,000)" covers "Size Column to Fit". (3) Compress to > TAR File: the hovered row's tooltip covers the new rename box. (4) Double-click a folder: its "Date created" tooltip stays over the new folder's list. | Hide tooltips whenever a popover, menu or rename box opens, and on navigation. `quiet_tooltips.rs` only does this for drags. | Yes (X11 and Wayland) |
| 2 | **The tab keeps the old search's title.** | Documents, search `a?b`, Enter. Then search `a?b.txt`, then `Notes`. The tab still reads "a?b - Search Results in Documents" until something else redraws it. | `record_search_step` (`window/search_steps.rs:83-91`) only calls `render_location()` when `History::set_current_search` (`history.rs:227-231`) reports a change of kind. Since #310 (69e9d749) added the query to the title (`window/location_view.rs:83-93`), a new query must redraw too. Redraw the tab and window title whenever the query changes. | Yes (X11) |
| 2 | **Keyboard focus is lost after a Home menu closes.** | With "Open Quick access on Home" on, click a Quick access tile, press Right, then Shift+F10, then Esc. Left and Enter now do nothing. | `window/home_page.rs:204-231` and `:925-950` (#316, 24064bab) only unparent the popover. Return focus to the tile or row on close, as #321's `menu_opener` does for the navigation pane. Add a focus test to `window/tests/home_page.rs`. | Yes (X11) |
| 2 | **Pictures on an `smb://` share never get thumbnails**, so #327 has no effect for shares opened through GVfs. | Open `smb://127.0.0.1/Studio/Small` (3 small PNGs) in Large icons: type icons only. The same PNGs in ~/Pictures get thumbnails. `gio info` on the share reports only `standard::fast-content-type` (image/png) and no `standard::content-type`. | `ox-core/src/entry/info.rs:37` reads only `standard::content-type`. With none, `PreviewRequest::for_entry` (`thumbnails.rs:187`) returns `None`. Fall back to `fast-content-type`, or guess from the name, for remote entries. Decide this together with the default question in C. | Yes (X11; gvfs on Ubuntu 24.04) |
| 3 | **An offline share shows a raw system error, pre-selected.** | Open the share, stop the server, press F5. The page reads "This location is unavailable / Software caused connection abort", with that line highlighted as if selected. | Say what Windows says (the location is not available, check the network) and keep the raw error under details. Don't select the label's text on show (`window/loading/disconnected.rs`). | Yes (X11) |
| 3 | **Home's Recent and Favorites show file extensions** while every other view hides known ones (the default). | With Quick access on Home and the default settings, Recent lists "sunset.png", "plan.pdf" and "Budget 2026.txt", while Documents lists "plan" and "Budget 2026". | `file_row` in `window/home_page.rs:577-585` (#316) uses the raw name. Use the same shown-name helper as the file list. | Yes (X11) |
| 3 | **Browsing a TAR hides a safe relative link** and says "1 unsafe names or links are hidden.", which also has a plural error. | Open Bundle.tar.gz, which has `bundle/lib/libx.so -> libx.so.1`. Only libx.so.1 shows, and the toast appears. #308 extracts that same link. | `ox-core/src/archive/browse.rs:196` (`is_listable`) still treats every link as unsafe. Apply the extract rule (`archive/extract/plan.rs:303`) when listing. Fix the singular message (`archive_view/browser.rs:430`). | Yes (Wayland) |
| 3 | **About a quarter of the Chinese UI is English.** | `LANGUAGE=zh_CN`: Home shows "Favorites" and "Files you open recently show here."; the file menu shows "Add to Favorites" and "Compress to"; the status bar says "1 item selected". 630 of the 2,475 msgids in `po/openxplorer.pot` are missing from `po/zh_CN.po`, including #316's Home strings. No string was cut off in the views checked. | Update `po/zh_CN.po` from the .pot, or hide the language until it's complete. | Yes (X11) |
| 3 | **One file shows two sizes** (from the first review, still there). | Select Notes 1.tar (2,048 bytes). The list says "2 KB" and the status bar "2.00 KB". | Use one size formatter. | Yes (Wayland) |
| 4 | **The update error reads oddly.** | Settings > About with no access to GitHub: "Installed: 2.0.17 Could not check for updates. GitHub could not check for updates (HTTP 403). Try again later." The sentences run together, and GitHub didn't "check" anything. | Put a separator after the version (`update/state.rs:198`) and reword `ox-core/src/update/error.rs:65` ("Couldn't reach GitHub (HTTP 403)"). | Yes (environment's 403) |
| 4 | **"Connect as guest" looks like plain text** under the keyring note, with no spacing. | Open a guest share that asks for credentials. | Give `guest_button` (`resources/ui/network-sign-in.ui:276`) a link or button style and a top margin. | Yes (X11) |
| 4 | **Changing the view loses the focused item.** | Big (12,000 files): press End (file 11999 is focused), then Ctrl+Shift+2. The view jumps to the top and file 11999 is out of sight. | Scroll the focused item into view after a view change. | Yes (X11) |
| 4 | **An archive is named two ways, and search is off inside it.** | Open Bundle.tar.gz. The tab says "Bundle.tar.gz", while the address bar and file list say "Bundle.tar". The search box is grayed out. | Use the shown name for the tab. Search inside archives is a separate gap (`location_view.rs:261`). | Yes (Wayland) |
| 4 | **Split view at 1x cuts off columns.** | Start in split view at 1x. The right pane shows Name, Date modified and a clipped Type, with no Size and no horizontal scroll bar. The left pane has a scroll bar. | Fit the columns to the pane, or show a scroll bar in both panes. | Yes (X11) |
| 4 | **Compact Home looks cramped and uneven.** | Compact view with Quick access on Home: tile rows touch (the pin icon sits on the next tile's name), and there's an empty band about one row tall between Recent's column titles and the first row. | Check the `.home-*` compact spacing (#316, `compact_density.rs:46`). | Yes (X11, 1x and 2x) |
| 5 | **The share's name changes case.** | Type `smb://127.0.0.2/Studio/Photos`. The address bar shows "Studio", the navigation pane "studio on 127.0.0.2". | Use one source for the share's display name. | Yes (X11) |
| 5 | **TAR members with time 0 show "1/1/1980 12:00 AM".** | Open a TAR whose members have mtime 0. 1980 is the ZIP/DOS epoch; a TAR uses Unix time. | Leave Date modified empty for a zero TAR time, or show 1970. | Yes (Wayland) |

## B. Regressions in our merged work

**Method.**
- Every feature in the task's list was exercised in the app, with real input, on 0406518.
- A separate read of the diffs compared #297-#321 with everything merged after each one, including the owner's #322 and #327.

**Result.**
- No feature was undone or broken by a later merge or by the owner's own commits.
- Two of our merged PRs have their own defects, listed in A: #310's stale search tab title, and #316's lost focus after a Home menu and extensions shown in Recent.
- One gap appears where #316 meets an older rule, below.

| Rank | Feature | Status on main | Breaking commit | Reproduced |
|---|---|---|---|---|
| 1 | Home page search | **Broken.** With "Open Quick access on Home" on, the search box is grayed out: clicking it, Ctrl+E and typing do nothing. With the setting off, Home is the home folder and search works. Microsoft: "To find relevant files from your PC and the cloud, search from File Explorer Home." (https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows). | #316 (24064bab) made Home a landing page. `location_view.rs:261` (ea6807b87, earlier) disables search on every page. Exempt Home and search what Home lists (Recent, Favorites, Quick access folders). | Yes (X11) |
| - | Home: Quick access, Recent/Favorites, Compact | Works: tiles, the Recent list from recently-used.xbel, the Recent/Favorites switcher, Enter opens a tile, Compact redraws. See A for focus, extensions and compact spacing. | none | Yes (X11, 1x/2x, light/dark) |
| - | Shift+F10 and the classic-menu setting | Works: Win11 style gives the compact menu for Shift+F10, the Menu key and right-click. `contextMenu: win10` gives the classic menu for both. | none | Yes (X11) |
| - | Ctrl+T/Ctrl+W in the address bar and search box | Works: Alt+D then Ctrl+T opens a tab; Ctrl+E, typing, then Ctrl+W closes the tab; Ctrl+E then Ctrl+T opens a tab. | none | Yes (X11) |
| - | Esc on Replace or Skip, and during drags | Works: Esc on "The destination already has a file named..." skips, and the file stays. Esc in mid-drag on Wayland drops nothing. | none | Yes (X11, Wayland) |
| - | Rename extension question, Confirm Save As | Works: "If you change a file name extension..." with Yes/No; Esc returns to the box with the text selected; Yes renames. Saving over Budget 2026.txt asks "already exists. Do you want to replace it?" with No as default. | none | Yes (X11, Wayland portal call) |
| - | Save dialog name with hidden extensions | Works: SaveFile with `Report.txt` shows "Report", filter "Text". | none | Yes (Wayland portal call) |
| - | Column titles menu and Choose Details | Works: the right-click menu has Size Column to Fit, Size All Columns to Fit, the columns and More...; Choose Details opens with Name checked and grayed, and Esc cancels. The header tooltip covers the menu (A). | none | Yes (X11 2x dark compact, Wayland) |
| - | Recycle Bin typed in the address bar | Works: opens the Recycle Bin with its own icon. | none | Yes (X11) |
| - | `?` search wildcard | Works as #317 pinned it: `a?b.txt` finds a1b.txt and acb.txt. `a?b` finds nothing (see Not sure). | none | Yes (X11) |
| - | Tab menu and tab icons, both settings | Works: default gives Move tab to new window, Move tab to window..., Duplicate tab, Open windows..., Close tab, Close other tabs, Close tabs to the right; `moreTabMenuItems: false` gives Close tab (Ctrl+W), Close other tabs, Close tabs to the right, Duplicate tab. This PC and Recycle Bin tabs have their own icons. | none | Yes (X11) |
| - | Navigation pane, both settings | Works: Recent files, Recent locations, Recycle Bin and the capacity bar only with the setting on. | none | Yes (X11) |
| - | Properties dates | Works: "Saturday, November 14, 2026, 4:12:10 AM" for Created and Modified; Accessed "Yesterday, ...". | none | Yes (X11) |
| - | Grouping: Downloads by date, split panes | Works: 10,000 files grouped Today / Yesterday / Earlier this week / Last week / Last month, fully listed within about 3 s. In split view the inactive pane keeps its date groups. | none | Yes (X11) |
| - | Update button setting | The switch "Show the Update button in the command bar" is in Settings > About, on by default. The button itself wasn't seen (no update was reachable). | none | Partly (switch seen); button by code reading only |
| - | Archives: TAR browsing, Compress to | Works: Bundle.tar.gz browses like a folder; Compress to offers ZIP File, TAR File and Additional options; TAR File made "Notes 1.tar" and opened its rename box. See A for the hidden link. | none | Yes (Wayland) |
| - | Compact view spacing | Folder lists look as #309 intended at 1x and 2x. Home in compact is in A. | none | Yes (X11) |
| - | Search layout | Works: Folder path column, Search options, Close search, results within about 1 s in 12,000 files. | none | Yes (X11) |
| - | Owner's #322 (resize from inside edges) | Wayland shows resize cursors in the outer band. A click on the close button's top-right pixel still closes. A click 2 px below the top edge of an inactive tab doesn't select it (see Not sure). | none | Partly (X11 has no window manager, so a resize itself couldn't be tried) |
| - | Owner's #327 (network thumbnails on by default) | Has no effect on GVfs `smb://` shares (A). Microsoft's policy page describes the opposite default (C). A possible concern from reading the code, not tested: thumbnails share 2 slots, and a hung kernel CIFS mount could hold both and stall local thumbnails too (`thumbnails.rs:52-54`, `slots.rs`). | 04065186 | Yes (no thumbnails); the stall by code reading only |

## C. Differences from Windows 11 that Microsoft confirms

| Rank | What OpenXplorer does | Steps | What Windows does (source) | Suggested fix | Reproduced |
|---|---|---|---|---|---|
| 1 | A new window opens to "Home", which by default is a plain list of the home folder's subfolders. Quick access, Recent and Favorites only appear after turning on Settings > General > "Open Quick access on Home". | Start with default settings. | "By default, File Explorer opens to Quick access, renamed to File Explorer Home in Windows 11, Version 22H2." (https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows). "The focus will automatically be on the Home pane, which includes sections like Recommended, Favorites, and Recent." (https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-explore-and-navigate-file-explorer-in-windows) | Turn "Open Quick access on Home" on by default, once Home search (B) and the focus bug (A) are fixed. That's an owner decision, since #316 shipped it off. | Yes (X11) |
| 1 | Search is disabled on Home with Quick access on. | See B. | "To find relevant files from your PC and the cloud, search from File Explorer Home." (FE page above) | See B. | Yes (X11) |
| 3 | Backspace in the Open/Save dialog goes Back, never up. With no history it does nothing (from the first review, still there). | SaveFile in Documents, click the file list, press Backspace: still Documents. | "Backspace: Open a folder one level up if a folder is selected in the Save As or Open dialog box." (https://support.microsoft.com/en-us/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec, dialog box section) | In `window/input.rs:546-553`, go Up instead of Back while `is_picking()`. | Yes (Wayland) |
| 3 | The navigation pane has no Gallery. | Look at the navigation pane. | "This update introduces Gallery in File Explorer. Gallery makes it easy to access your photo collection." (https://support.microsoft.com/en-us/topic/september-26-2023-windows-configuration-update-542780c2-594c-46cb-979d-11116fe164ba) | Add a Gallery page of Pictures (and chosen folders) sorted by date taken. Larger feature. | Yes (X11) |
| 3 | Pictures in network folders are meant to get thumbnails by default (#327; see A for why they don't on GVfs). | Settings > Files & folders: network previews on. | "File Explorer displays only icons and never displays thumbnail images on network folders by default." (https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-thumbnails, DisableThumbnailsOnNetworkFolders). This is a policy page; please confirm on a stock Windows 11 PC before changing anything. | Owner decision: either revert the default in `ox-core/src/settings/view_options.rs:60` or keep it with this note. | Yes (setting state); Windows behavior from the source only |
| 4 | Compress to offers ZIP File, TAR File and Additional options, but no 7z. | Right-click a file > Compress to. | "Support for creating 7-zip and TAR archives" (https://learn.microsoft.com/en-us/windows/whats-new/whats-new-windows-11-version-24h2) | Add "7z File" once there's a 7z writer (backlog). | Yes (Wayland) |

## Not sure (needs a Windows check)

| Rank | What OpenXplorer does | What to look at on Windows 11 |
|---|---|---|
| 2 | Right-clicking a Home Quick access tile gives a plain list (Open, Open in new tab, Open in new window, Open in Terminal, Open folder with..., Cache this folder for search, Unpin from Quick access, Previous versions, Properties). There's no Cut/Copy/Rename strip and no "Show more options". | Right-click a pinned folder in Home's Quick access: is it the modern menu with icons at the top and Show more options? |
| 3 | Search `a?b` in a folder with a1b.txt and acb.txt finds nothing; `a?b.txt` finds both. The wildcard must match the whole name. | Search `a?b` in such a folder. Microsoft's AQS page (https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-advancedquerysyntax) explains `?` and `*` but doesn't say whether a pattern must match the whole name. |
| 3 | The Downloads tab shows a yellow folder, while the navigation pane shows the Downloads glyph. Home with Quick access off has a yellow folder in its tab too. | Open Downloads in a tab: which icon does the tab show? |
| 3 | On a desktop whose `gtk-decoration-layout` has only "close" (Weston and GNOME's default), the window has no minimize or maximize buttons. This is deliberate (`window/caption_buttons.rs`). | Windows always has all three. Decide whether OpenXplorer should force minimize, maximize and close. |
| 4 | After Enter in the search box, keyboard focus moves to the results. | Press Enter in the search box: does focus stay in the box? |
| 4 | "Budget 2026 (2)" sorts before "Budget 2026". | Sort by Name with both files. Microsoft confirms numeric sort (https://learn.microsoft.com/en-us/windows/client-management/mdm/policy-csp-admx-windowsexplorer) but not this case. |
| 4 | Column titles show a tooltip "Drag to resize · double-click to fit loaded items (up to 2,000)". | Does Windows show any tooltip on Details column titles? |
| 4 | Type-ahead shows "Jump to: no" and the matching name (Notes 1.md) in the status bar. | Windows shows nothing there while you type. |
| 4 | Properties: Created and Modified use the weekday ("Friday, ..."), Accessed uses "Yesterday, ..." for the same date. | Which wording does each date use on the General tab? |
| 5 | A click within the top 4 px of a tab (or of the caption buttons) of a floating window starts a resize instead of selecting the tab (#322, `resize_edges.rs:44`, `:371`). | How tall Explorer's inside top resize band is over its tabs. |

## Worked well

- **Big folders.**
  - 10,000 files in grouped Downloads listed progressively and finished within about 3 s.
  - 12,000 files: Ctrl+A, End and switching to Large icons were each about 1 s or less.
  - Search for "file 1199" there answered in under 1 s.
  - 3,000 local pictures got thumbnails while scrolling, and End showed the last ones within 3 s, at about 420 MB RSS.
- **Slow and offline network.**
  - A 30 KB/s share listed 400 items within 3 s with no frozen UI.
  - Stopping the server while the folder was open gave the unavailable page, not a hang, and the rest of the app kept working.
- **Wayland (Weston).** Menus, the column menu, Choose Details (centered), the Save dialog, Compress to, TAR browsing, Esc during a drag and resize cursors all behaved.
- **Everyday keys.** These matched Windows:
  - Ctrl+Shift+N with typing a name, type-ahead, Delete to the Recycle Bin without a prompt, and Ctrl+Z restoring.
  - Ctrl+1 to switch tabs, which the first review found broken.
  - Ctrl+Shift+2 and 6 to change the view, which the first review found broken.
- **No crashes** in any session.
- **Rename** with hidden extensions keeps the real extension (a1b becomes a1b.md.txt), as Windows does.
