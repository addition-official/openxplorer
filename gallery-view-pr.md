# Title

feat(gallery): Add a Gallery page under Home with the photos of Pictures by month

# Body

Model: claude-opus-5-5

## Summary

Adds Gallery, as Windows 11 File Explorer has it: a navigation pane row right under Home that shows the photos in the Pictures folder and its subfolders as thumbnails, newest first, grouped by month.

- The row is listed in both states of "Show more items in the navigation pane". Typing "Gallery" in the address bar opens it too, and Back, Forward and new tabs work as for any location. The tab and the address bar show the photo glyph.
- A photo is dated by its EXIF date taken, or else by its date modified. Each month gets a heading such as "September 2026".
- Gallery is listed like a folder, so selecting, opening, the right-click menu, Shift+F10, Open file location, Delete to the Recycle Bin, Rename, Properties, check boxes, Ctrl+mouse wheel and the panes all behave as they do elsewhere. New and Paste are off, because Gallery is not a folder.
- Folders are read on a worker thread and photos arrive in batches, so a library of 10,000 photos never freezes the window. Leaving Gallery stops the reading.
- One inotify thread watches the whole Pictures tree, so photos added or deleted while Gallery is open appear or disappear within about a second.
- The Collection menu in the command bar shows every folder or just one, and can add or remove folders. Added folders are saved as `galleryFolders` in settings.json, and only once one has been added.
- Hidden photos follow Show hidden items. An empty or missing Pictures folder shows "No photos to show" and no error.
- No new dependencies. Nothing is sent over the network. Settings.json is the only file written.

Other folders are unchanged. Month headings in the icon sizes are turned on only for Gallery.

## Behaviors and sources

| Behavior | Source |
|---|---|
| Gallery is a node in the navigation pane | [Build 23435](https://blogs.windows.com/windows-insider/2023/04/14/announcing-windows-11-insider-preview-build-23435/) |
| The most recently taken photos appear at the top | [Windows configuration update, September 26, 2023](https://support.microsoft.com/en-us/topic/september-26-2023-windows-configuration-update-542780c2-594c-46cb-979d-11116fe164ba) |
| The Collection dropdown chooses which folders are shown, and subfolders can be added | [Build 23435](https://blogs.windows.com/windows-insider/2023/04/14/announcing-windows-11-insider-preview-build-23435/), [Build 23481](https://blogs.windows.com/windows-insider/2023/06/14/announcing-windows-11-insider-preview-build-23481/) |
| Check boxes on photos while Item check boxes is on | [Build 23451](https://blogs.windows.com/windows-insider/2023/05/04/announcing-windows-11-insider-preview-build-23451/) |
| Ctrl+mouse wheel resizes the thumbnails | [Build 23440](https://blogs.windows.com/windows-insider/2023/04/19/announcing-windows-11-insider-preview-build-23440/) |
| Alt+P and Shift+Alt+P work with a photo focused | [Build 22631.2191](https://blogs.windows.com/windows-insider/2023/08/10/announcing-windows-11-insider-preview-build-22621-2191-and-22631-2191/) |
| A tooltip on hover | [Build 23471](https://blogs.windows.com/windows-insider/2023/06/01/announcing-windows-11-insider-preview-build-23471/) |
| A loading state, not an empty one, while photos are read | [Build 23565](https://blogs.windows.com/windows-insider/2023/10/11/announcing-windows-11-insider-preview-build-23565-dev-channel/) |
| F6 and Shift+F6 cycle the tabs, content and navigation pane | [Use a screen reader to explore and navigate File Explorer](https://support.microsoft.com/en-us/accessibility/windows/use-a-screen-reader-to-explore-and-navigate-file-explorer-in-windows) |

## Not confirmed

Microsoft's documentation does not cover the following. The current behavior is in brackets.

- Gallery's exact position in the navigation pane, and its icon [right under Home, Fluent image glyph]
- The default folders, and whether subfolders are included [Pictures, with all subfolders]
- Videos, and the list of image formats [images only]
- Which date orders the photos, and the fallback [EXIF date taken, else date modified]
- Month grouping, and the heading format [one group per month, "September 2026"]
- The default thumbnail size, and whether it is remembered [Large icons on each visit, never saved]
- What a single click and a double-click do, and the items in the context menu [the same as for a file in a folder]
- The items of the Collection menu, the Manage Collection dialog, and whether Pictures can be removed ["All folders", one item per folder, "Add folder…", "Remove folder"; Pictures cannot be removed]
- The empty-state wording, and a date scrubber [the app's own wording; no scrubber]
- Sort and Group by inside Gallery, and arrow keys across months [the app's existing behavior]

Windows-only features are left out: OneDrive Camera Roll backup, Add Phone Photos, Gallery in the file picker, and iCloud.

## How it was tested

- Tests were written first: eight window tests in `window/tests/gallery.rs`, all of which fail on main. Unit tests cover EXIF dates, month groups, the saved folder list and the tree watch. All of them pass.
- Every native test binary and the doctests were run with `native/tools/check.py`'s isolation and compared by name with unchanged main. 4,450 tests ran and 4,437 passed. The 13 failures are the 11 that fail on main by name, plus two thumbnailer-sandbox tests that also fail on main where bubblewrap is installed.
- rustfmt, Clippy (the workspace lints, and again with the pedantic flags), `native/parity/check.py` and the i18n check all pass.
- The real app was driven with xdotool under Xvfb, using generated pictures only, at 1x and 2x, in light and dark, and in normal and Compact view:
  - the row and the address bar
  - keyboard access with F6
  - selection, opening with a mock default app, the menus, Delete, Properties and Open file location
  - photos added and removed while Gallery is open
  - hidden files
  - an empty and a missing Pictures folder
  - a 670 px and a maximized window
  - 10,000 photos
  - a slow FUSE mount
- No GTK criticals appeared in any run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
