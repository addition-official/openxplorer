# Title

fix(transfers): Keep pasting when one item is unreadable or missing, and refuse a folder pasted into itself plainly

# Body

Model: claude-opus-5-5

## Summary

Three paste problems:

- **An unreadable subfolder stopped the whole copy.** Copying a folder that held a readable file and a subfolder only its owner could open showed "Operation stopped: Error opening directory ... Permission denied" with only OK, and nothing was copied. Now everything readable is copied and the unreadable subfolder is asked about by itself in the progress window: "Folder Access Denied", "You need permission to perform this action", Try Again, Skip and Cancel. The same goes for one nested deeper.
- **One missing item stopped the whole paste.** Cutting two files, deleting one from outside the app and pasting showed "Operation stopped: Error when getting information for file ..." and moved neither. Now the other file moves and the missing one is asked about in the progress window: "Item Not Found", "Could not find this item", Try Again, Skip and Cancel. Several missing items are asked about one by one.
- **A folder pasted into its own subfolder was shown as an unexpected error with Try Again.** It is now refused in the progress window in plain words: "Interrupted Action", "You can't copy a folder into a folder inside it." ("move"), with Skip and Cancel only. Nothing is changed. This also applies when the folder is dropped onto its subfolder.

Cause of the first two: before any item ran, the check for protected (snapshot) locations walked every selected item, and any read error there failed the whole paste (`preflight` in `ops/resolved_transfer.rs`, through the walk in `transfer/guard.rs`). That check still refuses a protected location before anything changes. It now leaves an item that is gone or cannot be read to the engine, which asks about that item when its turn comes and checks it again first. A copy no longer walks into a folder it cannot list; the copy asks about that folder by itself.

**Deliberate difference from Windows:** a moved folder with an unreadable subfolder is asked about as one item, "Folder Access Denied" with Try Again, Skip and Cancel, and is kept where it is. Windows would move it. The app's safety rule for protected snapshot folders (XFER-020) never moves a tree it could not check, so this stays as it is, with no setting. Other selected items move normally, and Try Again moves the folder once it can be read.

The protected-location rule still covers what could not be checked first:
- A copy checks every entry's final location before writing it. A protected folder that becomes readable after Try Again therefore stops the whole item, and nothing of that item is published.
- A selected folder merged into a folder of the same name is planned entry by entry, so it must be read whole before anything runs. If part of it cannot be read, the paste stops before anything changes, as it did before.

An item that is already gone (or cannot be read) when Paste checks for name conflicts, while the destination has its name, no longer stops that check. It goes in as a free item, never replacing the existing one, and is asked about in its turn. When a copy meets its own staging inside the source (the same share reached under two host names), it is refused the same way as a folder pasted into itself, with Skip and Cancel.

Undo after a partial copy or move reverses only what was done.

## Windows' behavior

- Microsoft documents no wording for a folder pasted into its own subfolder. The only Microsoft text is the old `SHFileOperation` code `DE_DESTSUBTREE`, "The destination is a subtree of the source." ([learn.microsoft.com](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shfileoperationa)). The dialog sentence "The destination folder is a subfolder of the source folder." appears only in user posts on Microsoft Q&A. So the app uses its own plain wording, with no "unexpected error" and no Try Again.

## Testing

- New `native/crates/ox-core/tests/ops_partial_failures.rs`, run as an unprivileged user. On unchanged upstream main, all five item tests fail with the reported errors. With this change all six pass:
  - an unreadable subfolder (copy);
  - one nested two levels down;
  - a moved folder with an unreadable subfolder beside another item;
  - missing items in a move;
  - a folder pasted into its subfolder (move and copy);
  - a protected snapshot that still stops a move with a missing item;
  - a merged folder that can no longer be listed, which moves nothing;
  - a folder made readable and tried again that holds a snapshot, which publishes nothing;
  - a folder named with `#` and `?` copied under the app's real previous-versions protection;
  - an item deleted before Paste whose name the destination already has.
- New engine test: the late staging-alias refusal is asked about as a folder pasted into itself.
- New `failure_dialog.rs` test for the refusal's wording.
- rustfmt and pedantic Clippy (Rust 1.92.0) are clean. `parity/check.py`, `i18n.py check` and `sync-docs.py --check`, the public-data audit and `pnpm check` pass. Every `native/tools/check.py` step passes except two test executables that also fail on unchanged main in the test container: `location_fixture_drift` (its Python 3.13 `ipaddress` now accepts `[V1.x]`) and `network_python` (`gi` fails to import).
- Real app under Xvfb, run as an unprivileged user on fictional files, with real xdotool keys and pointer, light and dark at 1x. The checks covered:
  - copy and move with an unreadable folder: Skip, Try Again after making the folder readable again, and Cancel;
  - a nested unreadable folder;
  - cut and copy with a missing item: Skip, Try Again after the file came back, and Cancel;
  - several missing items;
  - folder into its subfolder by cut, by copy, and by dragging it onto the subfolder;
  - Undo after each partial operation;
  - nothing left behind after Cancel.
- Pictures of the three questions were taken and reviewed at 1x and 2x, light and dark.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
