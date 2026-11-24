# Title

feat(icons): Draw icons and glyphs from Windows files the user chooses

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/24-long-b-soak)

## What changes for the user
- **New rows at the end of Settings > Appearance > File icons:** "Icons from Windows icon files" (off by default), "Windows icon files" with "Choose files…" (one or more `.dll`, `.mun` or `.exe` files) and Clear, and "Windows symbol font" with "Choose font…" and Clear. Only the chosen paths are saved (`useIconFiles`, `iconFiles`, `symbolFont`), and the row says how many icons the files gave ("{found} of {total} icons come from the chosen files").
- **Nothing from Microsoft ships.** The app reads files the user already has, while it runs. Decoded pictures are kept only as PNG files in `~/.cache/winspace/windows-icons`, keyed by each file's path, size, time and inode; folders of files no longer chosen are removed. The font is read, never installed.
- **Order of sources:** a choice made in File icons comes first, then the Windows icon files, then the built-in icon. A picture of the user's own always wins. A picture chosen in File icons now also shows in the Details pane and in Properties.
- **With `imageres.dll.mun` and `shell32.dll.mun` chosen** (or the same names without `.mun`), the files are read off the main thread and draw:
  - the standard folders as Windows 11's flat tiles (Desktop 183, Downloads 184, Documents 112, Pictures 113, Music 108, Videos 189) in the navigation pane, Home's Quick access tiles, folder lists and the address bar;
  - the folder, Home, Gallery, This PC, Network, the Recycle Bin (empty and full) and the Settings tab;
  - fixed drives (Local Disk as the plain drive, never with Windows' logo), removable and optical drives, network computers (with the computer icon), shares and mapped drives;
  - ZIP folders as imageres group 174 (the zipper folder), other archives, text files, pictures, audio and video (imageres' file icons), programs, scripts, shortcuts, and the blank page for files no app claims.
- Each icon uses the image in the file that best fits its place and scale, resampled to the exact size shown.
- **With the user's own `SegoeIcons.ttf` (Segoe Fluent Icons) or `segmdl2.ttf` (Segoe MDL2 Assets) chosen,** the command bar, menus, navigation buttons, address bar chevrons, search, panes, tabs, caption buttons, Settings and the status bar's view buttons draw Windows' glyphs by Microsoft's published codepoints, in the theme's colors. Glyphs are drawn on whole pixels at the ems Windows uses, each em picked from the real font's ink: chevrons at WinUI's 8-pixel em, the navigation buttons' arrows at 12 pixels, New as AddTo, and the menus' X as Cancel. Glyphs the font lacks keep the built-in Fluent icon.
- **The built-in icons are sized to Windows 11's measured sizes**, for example the command bar's icon-only and labeled commands in an 18-pixel square (16 and 17 before), so the app's own glyphs ink 14 to 16 pixels, as Windows 11's command bar does.
- Anything missing, unreadable or broken keeps the built-in icon. Turning the setting off brings the built-in icons back at once.
- **Merge fixes** that reconcile this work with the network and tab work earlier in the series: the address bar's icon gets the merged tab places, a Network computer gets the computer icon, a test's USB drive is no longer taken for a disc (and the removable-delete test's `MountControls` gets `is_optical`), and the Windows glyph for the Code icon is dropped, since the merged address bar no longer has that icon.
- For developers: `ox_core::windows_icons` is a small PE resource reader (`RT_GROUP_ICON` and `RT_ICON`, bounds-checked reads, caps on file size, sections, directory entries, images per group and image size; bitmaps decoded in the app; a PNG's header checked before anything decodes it). `ox_core::windows_glyphs` is a small TrueType outline reader (cmap formats 4 and 12, simple and composite glyphs, bounds-checked and capped) that writes each glyph as a symbolic SVG into a hicolor theme folder in the cache. No unsafe code and no new crate. Parity LOOK-030 (icon files) and LOOK-031 (symbol font). The PR's "Drop redundant tests" commit also removes four Details pane tests (`content.rs`).

## Why
- The owner asked for the real Windows icons, read at run time from files the user already has and never shipped (2026-11-18), and for the command bar, menus and navigation glyphs to be drawn from the user's own Segoe Fluent Icons font (2026-11-20).
- The first version drew some places with older or wrong groups; the later fixes use Windows 11's special-folder tiles, imageres' file icons for pictures, audio and video, the zipper folder for ZIP files, and the drive and network groups checked against a real imageres file.
- Glyphs filling their square and icons scaled from the nearest stored size looked larger or blurrier than Windows' own.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "Icons from Windows icon files"
- "Shows folders, the standard folders, drives, Home, Gallery, This PC, Network, the Recycle Bin, Settings and common file types with the icons of Windows icon files you choose, for example imageres.dll.mun and shell32.dll.mun from a Windows installation. OpenXplorer includes no Windows icons: it reads the files you choose while it runs and keeps their pictures only in your cache folder. Anything the files do not have keeps OpenXplorer's own icon."
- "Off: OpenXplorer's own icons"
- "On, but no file is chosen yet"
- "Windows icon files"
- "The .dll, .mun or .exe files the icons are read from. Only where they are is saved, never the icons. Choosing files again replaces the list."
- "Choose files…"
- "Choose Windows icon files"
- "No files chosen"
- "Reading the chosen files…"
- "reading…"
- "{found} of {total} icons come from the chosen files"
- "The chosen files have none of these icons, so OpenXplorer's own are shown"
- "used while Icons from Windows icon files is on"
- "Windows symbol font"
- "The Segoe Fluent Icons font (SegoeIcons.ttf) or Segoe MDL2 Assets (segmdl2.ttf) from a Windows installation, whose glyphs then draw the command bar, menus, navigation buttons and address bar, in your theme's colors. Only where the font is is saved; it is read, never installed. Glyphs it lacks keep OpenXplorer's own."
- "Choose font…"
- "Choose the Windows symbol font"
- "Fonts"
- "No font chosen"
- "{found} of {wanted} glyphs"
- "it has none of these glyphs, so OpenXplorer's own are shown"
- "{name} · {state}"
- Search keywords for the rows: "windows icons icon files imageres shell32 dll mun exe resources folder drive recycle bin this pc home gallery look classic original", "windows icons icon files choose browse imageres shell32 dll mun exe clear remove", "windows symbol font glyphs segoe fluent icons mdl2 assets segoeicons segmdl2 ttf command bar menu buttons"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: tests that build tiny synthetic PE32/PE32+ files and a synthetic TrueType font inside the test, covering parser bounds and refusals, every truncation, 3,000 seeded mutations, header bytes at their extremes, caps, name matching, the role table and its provenance, every role's art, the file kinds and drive media, the cache, fallback, the setting off by default and stored only when chosen, the best size per scale, the Settings rows driving the navigation pane, the glyph swap and its fallback, and a window test (`window::tests::windows_icon_files`) covering the navigation pane, a folder list, the address bar and Home's Quick access tiles; `python3 native/parity/check.py` and every stage of `python3 native/tools/check.py` with Rust 1.92.0 (the permission tests passed when rerun as an unprivileged user; two Python-fixture tests failed for reasons unrelated to this change: Python 3.13 could not import the system `gi` module, and its `ipaddress` gave a different answer). GTK's file dialog itself was not driven, and nothing was tried with real Windows files.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
