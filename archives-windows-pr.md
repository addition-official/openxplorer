# Title
feat(archives): Browse TAR archives like folders, add Compress to TAR File and a Create archive window, and extract relative TAR links

# Body
Model: claude-opus-5-5

This brings OpenXplorer's archive handling closer to Windows 11 File Explorer. Each change is covered by tests that fail on the previous main.

**TAR archives open like folders.** Windows 11 reads .tar, .tar.gz, .tgz, .tar.bz2, .tar.xz and .tar.zst itself ([Windows Insider blog, build 22631.2199](https://blogs.windows.com/windows-insider/2023/08/18/announcing-windows-11-insider-preview-build-22621-2199-and-22631-2199-beta-channel/); [Zip and unzip files](https://support.microsoft.com/en-us/windows/zip-and-unzip-files-8d28fa72-f2f9-712f-67df-f80cf89fd4e5)). TAR archives now follow the same setting as ZIP, Settings › ZIP & archives › Double-clicking a ZIP. With "Like a folder", the default, double-clicking a TAR opens it in the tab, with the address bar, Back and Up, folders inside, Copy, and Extract all in the command bar. "In a pop-up window" still opens it in the pop-up window. That window and the messages shown inside an archive now say "archive" instead of "ZIP". 7z and RAR are not included because the app has no decoder for them.

**Compress to › TAR File.** Windows 11 creates TAR archives as well as ZIP ([Windows Insider blog, build 25992](https://blogs.windows.com/windows-insider/2023/11/08/announcing-windows-11-insider-preview-build-25992-canary-channel/)). The Compress to submenu now lists ZIP File, TAR File and Additional options. TAR File writes a plain .tar beside the selection right away, named after the first item (Photos.tar, or Photos (2).tar when that name is taken), with the same progress, Cancel and never-replace rules as ZIP File. After ZIP File or TAR File, the new archive is selected and renamed in place, and Undo moves it to the Recycle Bin, the way New folder works. 7z is not included because the app has no 7z encoder.

**Additional options opens a Create archive window (ARC-023).** It replaces the earlier Compress dialog, which asked for a name and offered "ZIP (.zip)" or "TAR.XZ (.tar.xz)". The new window is laid out like Windows 11's: the archive's full path with Browse…, the archive format (zip or tar), the compression method (Deflate for zip; None, GZip, BZip2 or XZ for tar), a compression level from Faster to Smaller, and Create and Cancel. The path's ending follows the chosen format and method. A name that is already taken is refused with the reason, and nothing is replaced. Undo moves the new archive to the Recycle Bin. The symbolic-link and hard-link check boxes are grayed and cleared because links are never kept. .tar.zst, 7z and compressing a single file without an archive are not offered, because the app has no suitable encoder for them. gzip and bzip2 output come from the flate2 and bzip2 crates the app already uses; no dependency is added.

**Relative links in TAR archives.** Extract all and Extract here used to refuse a TAR that held any symbolic link, such as a library's `libx.so -> libx.so.1`. Such a link is now extracted as a link when its target is relative and stays inside the extracted folder. Any other link stops the extraction before anything is written, with a message that names it. Hard links, devices and FIFOs are still refused, links in ZIP archives are still refused, and the archive browser still hides links. The existing path and size checks are unchanged.

**Extracting onto existing files.** When Extract all goes into a folder that already has some of the files, the Replace or Skip question used to say "Moving 1 item from .openxplorer-extract-… to Bundle", naming the hidden folder the extraction is built in. It now says "Extracting 1 item from Bundle.zip to Bundle".

The parity notes (ARC-012, ARC-022, ARC-023, ARC-024, ARC-026), the user guide and the translation template are updated. Changed messages need their Chinese translation redone.

## How it was tested
- After merging the latest main: rustfmt, Clippy with the workspace lints (no warnings), `python3 native/parity/check.py`, the translation template check and the docs sync check pass, and every ox-core archive test passes. The app tests for archives, browsing in the tab, context menus, the Create archive window, New folder, pinned-folder commands, the progress window and restore conflicts: 144 passed and 4 failed. The same 4 also fail on the previous main on the test machine; 3 wait for a desktop application to launch and 1 for a menu's placement.
- Earlier on this branch: `python3 native/tools/check.py` stopped at `location_fixture_drift`, which also fails on the previous main on the test machine (its Python 3.13 parses one URL differently from the committed fixture). The whole suite, run with no fail-fast in the same isolated setup (Xvfb, a private D-Bus and a throwaway home folder), had 4,210 passed and 13 failed, and the same 13 fail on the previous main on the test machine (administrator mount tests, sandboxed thumbnailer tests and two Python comparison tests).
- New tests: `tar_archives_open_in_the_tab_like_a_folder` (all six TAR types), `extracting_onto_existing_files_names_the_archive`, `compress_to_tar_puts_the_selection_into_a_new_tar_beside_it`, `a_new_archive_is_renamed_in_place_and_undo_takes_it_away`, `additional_options_lay_out_the_create_archive_dialog`, `create_writes_the_chosen_format_and_undo_takes_it_away`, and `native/crates/ox-core/tests/archive_tar_links.rs`.
- The real app under Xvfb with mouse and keyboard input and fictional files:
  - every TAR type opened by double-click, browsed, Back and Up, a file opened from inside, Extract all from inside, and the three views, in light and dark;
  - extracting twice with Replace and Skip, and the link, FIFO, path and size refusals;
  - Compress to › TAR File on one file, two files and a folder, with a taken name, renaming in place and Undo;
  - the Create archive window with every format and method, the level slider, Browse…, Cancel, a taken name and Undo.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
