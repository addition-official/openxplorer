# Title

fix: Hide every known extension, one size per file, a wider Size column, odd names, trimmed names, See more and Alt alone

# Body

Model: claude-opus-5-5

## What changes

1. **Hidden extensions for every known type.** With "Hide extensions for known file types" on, `sunset.png`, `data.json`, `page.html` and `script.py` kept their extensions because GIO calls those types "uncertain" (several types share the extension). The app now treats any extension GIO names a type for as known, as the Open and Save dialog already did. Only the last extension is hidden (`Bundle.tar.gz` shows as `Bundle.tar`), and an extension of digits alone (`Notes v2.1`) stays part of the name. Shortcut detection is unchanged for those digits (`libz.so.1` links).
2. **One size per file outside the Size column.** The Details pane and the item tooltip now write a size with the same formatter as the status bar and the Tiles/Content lines (`4.38 KB` for 4,486 bytes everywhere). The Size column keeps whole kilobytes rounded up (`5 KB`). One byte reads `1 byte`, not `1 bytes`. The decimal format follows Microsoft's documented `StrFormatByteSize` output (https://learn.microsoft.com/en-us/windows/win32/api/shlwapi/nf-shlwapi-strformatbytesizew); that Explorer's pane and status bar use exactly this is not confirmed by Microsoft documentation.
3. **Size column width.** The default Size width is 130 px (was 90), enough for ten digits of kilobytes (`9,999,999,999 KB`) and a 6 GB file's `6,291,456 KB`, which was cut short. Extension keeps 90; the 70 px minimum is unchanged.
4. **Odd names.** A file whose name holds a line break or bytes that are not UTF-8 can now be renamed, copied and moved to the Recycle Bin. The "Encoded control characters" check now applies only to typed addresses; an existing item's URI keeps its exact bytes once its server, share or device part is checked and its path is absolute and normal.
5. **Trailing periods and spaces.** Renaming in place, the Rename dialog, New folder and the New menu's files trim periods and spaces at the end of a typed name ("Archive." and "Archive. " become "Archive"), as the Windows shell does. Microsoft: "Do not end a file or directory name with a space or a period" (https://learn.microsoft.com/en-us/windows/win32/fileio/naming-a-file). "Notes .txt" is unchanged; a name of only periods ("...") is kept as before.
6. **"…" is named See more.** The command bar's "…" tooltip and accessible name are "See more", as Microsoft names it (https://support.microsoft.com/en-us/windows/experience/fileexplorer/file-explorer-in-windows). The guide and ui-spec follow.
7. **Alt alone.** Pressing and releasing Alt with nothing between moves the keyboard to the command bar's first button with its focus ring; Escape gives the keyboard back to where it was. Alt with another key (Alt+Left, Alt+D, ...) is only that shortcut. Microsoft: "Alt moves focus to the app bar" (https://support.microsoft.com/en-us/windows/use-a-screen-reader-to-explore-and-navigate-file-explorer-in-windows-e7d3a548-87dd-459f-a991-9fde3f7ce927).
8. **About says Linux** (separate commits): Settings > About and About this build say "An independent Windows 11–inspired file manager for Linux." instead of "for Zorin.".
9. **One spelling of license** (separate commits): the About page's lead line and the manual say "license", matching the "License & source" heading. Settings search still finds "licence".

Not changed: an empty name in the Open/Save dialog's rename. The main window refuses an empty name the same way, and the parity note for VIEW-069 records that "an empty name is still refused", so this is left for the owner to decide.

Translations: the template is regenerated. zh_CN keeps its existing translations, moved to the new plural "{bytes} byte" entry and the edited About strings. "See more" has no Chinese translation yet (machine translations are not accepted).

## How it was tested

- Test first: each item has a test that failed on unchanged main and passes now (list in the status file).
- `python3 native/tools/check.py`, `python3 native/parity/check.py`, rustfmt and Clippy: results in the status file.
- Real input with xdotool on a private Xvfb display, at 1x and 2x, light and dark, with fictional files only: checklist in the status file.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
