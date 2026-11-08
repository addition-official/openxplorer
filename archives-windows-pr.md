# Title
feat(archives): Browse TAR archives like folders, add Compress to › TAR File, extract relative TAR links and name the archive when extracting onto existing files

# Body
Model: claude-opus-5-5

Four archive gaps against Windows 11 File Explorer, each covered by a test that fails on main.

**Browse TAR archives in the window.** Windows 11 reads .tar, .tar.gz, .tgz, .tar.bz2, .tar.xz and .tar.zst itself ([Windows Insider blog, build 22631.2199](https://blogs.windows.com/windows-insider/2023/08/18/announcing-windows-11-insider-preview-build-22621-2199-and-22631-2199-beta-channel/); [Zip and unzip files](https://support.microsoft.com/en-us/windows/zip-and-unzip-files-8d28fa72-f2f9-712f-67df-f80cf89fd4e5)). Double-clicking a TAR archive now opens it in the tab the way a ZIP opens (address bar, Back and Up, folders inside, Copy, Extract all in the bar), following Settings › ZIP & archives › Double-clicking a ZIP. The pop-up window and the messages inside an archive now say "archive" instead of "ZIP". 7z and RAR are not included: the app has no decoder for them.

**Compress to › TAR File.** Windows 11 creates TAR archives as well as ZIP ([Windows Insider blog, build 25992](https://blogs.windows.com/windows-insider/2023/11/08/announcing-windows-11-insider-preview-build-25992-canary-channel/)). The Compress to submenu now has ZIP File, TAR File and Additional options. TAR File writes a plain .tar beside the selection, named after the first item (Photos.tar, Photos (2).tar, ...), with the same progress, Cancel and never-replace rules as ZIP File. 7z is not included: no encoder is linked.

**Relative links in TAR archives.** Extract all and Extract here used to refuse a whole TAR that held any symbolic link, such as a library's `libx.so -> libx.so.1`. Now a TAR's symbolic link is extracted as a link when its target is relative and stays inside the extracted folder. Any other link stops the extraction before anything is written, with a message that names it. Hard links, devices and FIFOs are still refused, ZIP links are still refused, and the archive browser still hides links. The existing path and size checks are unchanged.

**Extracting onto existing files.** Extract all into a folder that already has some of the files asks Replace or Skip. That question used to say "Moving 1 item from .openxplorer-extract-… to Bundle", naming the private folder the extraction is built in. It now says "Extracting 1 item from Bundle.zip to Bundle".

The parity notes (ARC-012, ARC-022, ARC-023, ARC-024, ARC-026), the user guide and the translation template are updated. Changed messages lose their existing Chinese translation until it is redone.

## How it was tested
- rustfmt, Clippy with the workspace lints (no warnings), `python3 native/parity/check.py`, the translation template check, the docs sync check and the public-data audit all pass.
- `python3 native/tools/check.py` passes its earlier steps but stops at `location_fixture_drift`, which also fails on unchanged main here: the container's Python 3.13 parses one URL differently from the committed fixture. The whole suite was then run with no fail-fast in the same isolated setup (Xvfb, private session bus, throwaway home): 4,210 passed, 13 failed, and the same 13 fail on unchanged main (administrator mount tests, sandboxed thumbnailer tests and two Python comparison tests that depend on the container).
- New tests, each failing on main first: `tar_archives_open_in_the_tab_like_a_folder` (all six TAR types), `extracting_onto_existing_files_names_the_archive`, `compress_to_tar_puts_the_selection_into_a_new_tar_beside_it` (one file, two files, a folder) and `native/crates/ox-core/tests/archive_tar_links.rs`.
- The real app under Xvfb with mouse input and fictional files, in the light and dark themes: every TAR type opened by double-click, browsed, Back and Up; a file opened from inside; Extract all from inside; the three views; extracting twice with Replace and Skip; the link, FIFO, path and size refusals; and Compress to › TAR File on one file, two files and a folder, each then browsed in the app.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
