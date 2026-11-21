# Title

feat(icons): Draw icons and glyphs from Windows files the user chooses

# Body

Model: claude-opus-5-5

Adds Windows icon files and the Windows symbol font the user already has as further sources for the icons of File icons (AKolenda/openxplorer#366, now in 2.0.21).

This change reads icon files and a symbol font that the user supplies. It ships no Windows icons or fonts: nothing from Microsoft is in the repository, the package, the docs, the tests or the pictures, and the font is read, never installed.

## What it does

- **Settings › Appearance › File icons** (from #366) ends with **Icons from Windows icon files**, off by default, **Windows icon files** with **Choose files…** (one or more `.dll`, `.mun` or `.exe`) and **Clear**, and **Windows symbol font** with **Choose font…** and **Clear**.
- The order of sources is: a choice made in File icons, then the Windows icon files, then the built-in icon. A picture of the user's always wins. A choice of one of the app's pictures is drawn as the files draw that kind, or as the app draws it when the files have no such picture (the document with a zipper). Only the paths are saved (`useIconFiles`, `iconFiles`, `symbolFont`); the row says how many icons the files gave.
- While it's on, the chosen files are read off the main thread. `imageres.dll.mun` and `shell32.dll.mun` (or the same names without `.mun`) draw:
  - the standard folders as Windows 11's flat tiles (Desktop 183, Downloads 184, Documents 112, Pictures 113, Music 108, Videos 189), wherever they appear: the navigation pane, Home's Quick access tiles, folder lists and the address bar;
  - the folder, Home, Gallery, This PC, Network, the Recycle Bin (empty and full) and the Settings tab;
  - fixed drives (Local Disk included, as the plain drive), removable and optical drives, network computers, shares and mapped drives;
  - ZIP and other archives, text files, pictures, audio, video, programs, scripts, shortcuts, and the blank page for files no app claims.
- Each icon uses the image that best fits its place and scale: 16/20 px in the navigation pane, 32/48 px in lists, 256 px in large views, PNG or bitmap entries.
- **Windows symbol font:** with the user's own `SegoeIcons.ttf` (Segoe Fluent Icons) or `segmdl2.ttf` chosen, the command bar, menus, navigation buttons, address bar chevrons, search, panes, tabs, caption buttons, Settings and the status bar's view buttons draw Windows' glyphs, by Microsoft's published codepoints, at the same size and in the theme's colors. Glyphs the font lacks keep the built-in Fluent icon.
- Anything missing, unreadable or broken keeps the built-in icon. Turning the setting off brings the built-in icons back at once. Command-bar and Details-pane glyphs aren't changed; Windows draws those from a font.

## How

- `ox_core::windows_icons` is a small PE resource reader. It reads `RT_GROUP_ICON` and `RT_ICON` with bounds-checked `pread`s and caps the file size, sections, directory entries, images per group and image size. It decodes bitmaps itself and checks a PNG's header before anything decodes it. There's no unsafe code and no new crate: `flate2` and `crc32fast` were already dependencies, so `cargo-sources.json` is unchanged.
- Decoded images are cached as PNG files only in `~/.cache/winspace/windows-icons`, keyed by each file's path, size, time and inode. Folders of files no longer chosen are removed.
- `ArtImage` asks one function, `icons::user_icons::texture_for`, before drawing its own icon, after #366's `file_icons::art_for` has applied the user's choice; This PC's drive tiles and the address bar use `user_icons::show_icon`. Reading or turning off the files counts a new File icons generation (`file_icons::count_new_generation`), so folder views redraw through #366's mechanism. #366's `kind_of` also covers the art this adds, so its choices keep applying.
- New `Art` values for a listed standard folder and for removable and optical drives, and new file kinds for PDFs, archives, programs, scripts and shortcuts, each draw the same built-in icon as before, so nothing changes while the setting is off.
- Each role records whether its group number is registry-verified or a best match from the icon sheet; the cache key includes the role table.
- `ox_core::windows_glyphs` is a small TrueType outline reader (cmap formats 4 and 12, simple and composite glyphs, bounds-checked and capped). It writes each glyph as a symbolic SVG into a hicolor theme directory in the cache, which joins GTK's icon theme search path; `icons::set_icon` then draws the glyph's name. A generated stylesheet repeats only the skin's `-gtk-icontheme` chevron declarations with the font's names.
- Parity: LOOK-030 (icon files) and LOOK-031 (symbol font); #366 has LOOK-029.

## Tests

Tests build tiny synthetic PE32/PE32+ files with synthetic test patterns inside the test. They cover parser bounds and refusals, every truncation, 3,000 seeded mutations, header bytes at their extremes, caps, name matching, the role table and its provenance, every role's art, the file kinds and drive media, the cache, fallback, the setting off by default and stored only when chosen, the best size per scale, the Settings rows driving the navigation pane, and a window test covering the navigation pane, a folder list, the address bar and Home's Quick access tiles, and, for the font, a synthetic TrueType font built in the test that drives the reader, the glyph swap, the fallback and the Settings row.

Run here with Rust 1.92.0: `python3 native/parity/check.py`, plus every stage of `python3 native/tools/check.py`. In this container `check.py` runs as root, so the app refuses to start and permission tests are bypassed. Those tests passed when rerun as an unprivileged user. Two Python-fixture tests fail here for reasons unrelated to this change: the container's Python 3.13 can't import the system `gi` module, and its `ipaddress` gives a different answer. GTK's file dialog itself isn't driven, and nothing was tried with real Windows files.
