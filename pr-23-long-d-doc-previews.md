# Title

feat(preview): Show Office and OpenDocument files in the Preview pane and as thumbnails

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/22-long-a-fuzz)

## What changes for the user
When LibreOffice is installed (with bubblewrap and poppler's `pdftoppm`):
- **Preview pane.** A Word, Excel or PowerPoint file (docx, docm, doc, xlsx, xlsm, xls, pptx, pptm, ppt) or an OpenDocument text, spreadsheet or presentation (odt, ods, odp and their templates) shows its pages one under the other, scrollable, drawn the way the pane already draws a PDF's.
- **Thumbnails.** At Medium icons and larger, a document's first page is its thumbnail, framed by a light gray line so a mostly white page still reads as a page. It is written into the shared freedesktop cache through the app's existing writer. Rows of Details, List and Small icons keep the type icon, as Windows draws them, even when the cache has the thumbnail. The Details pane shows it.
- **Without LibreOffice**, bubblewrap or `pdftoppm`, the pane says "No preview available." under the document's icon, and nothing fails loudly.

How it runs:
- LibreOffice converts the document to PDF inside the thumbnailers' existing bubblewrap sandbox: no network, no message bus, no other process, the system read-only, and the document alone of the user's files, read-only under a name of the sandbox's own. LibreOffice never sees the document's folder, so it leaves no `.~lock` or `~$` file beside it.
- It runs within the thumbnailers' memory, file-size, processor and task limits, with its thread pool held to 4, and is stopped after 15 seconds or as soon as the selection changes.
- Each run gets a new private profile, written beforehand with macros disabled, macro security level 3, link updates set to Never, untrusted referer links blocked and no OpenCL.
- One document converts at a time. The Preview pane goes first: it stops a thumbnail's conversion of another document, which waits for its turn again, so arrowing through Large icons leaves no tile without its thumbnail. At most four thumbnails wait.
- The PDF is cached in `~/.cache/winspace/document-previews` (folder 0700, PDFs 0600), keyed by the file's real path, size, modification time and inode (256 PDFs, 512 MB, least recently used first). The first pages of the last eight documents stay in memory, so selecting a document again shows it at once.
- Only files of at most 100 MB that start as a ZIP, a Compound File, RTF or markup (the HTML `.xls` exports many sites produce) are converted. A damaged document, one that needs a password and one that runs out of time twice are not tried again while the app runs; a cancelled conversion, and one the app could not prepare or keep (a full disk), are.
- If a sandbox cannot mount its own `/proc`, which LibreOffice needs, no conversion is started and the app logs that once. A document's pages are drawn once more when the first try gives nothing and the request still stands.
- The Flatpak `org.libreoffice.LibreOffice` is not used: Flatpak always mounts the app's own data folder read-write, where the user's LibreOffice keeps its profile, so a converted document could change the profile the user opens next.

Fixed on the way, in `picture_guard/decoder.rs`: picture decoders kept in the pool were ended while they waited, because bubblewrap's `--die-with-parent` ends a decoder when the thread that started it exits, and GLib's worker threads exit once idle. The next picture handed to one got no answer, so the first PDF selected after launch could say "No preview available." and thumbnails went missing now and then. The pool now drops an idle decoder that no longer runs. (PR 24 then starts the decoders from a thread that lasts as long as the app.)

The parity inventory (VIEW-070, VIEW-057) describes the new previews. No new dependency; LibreOffice and poppler-utils are optional. The test documents are small hand-built ZIP containers written by the tests themselves (`test_support/office_documents.rs`, with a deflate bomb written through GIO); no third-party file is used.

## Why
Windows 11's Preview pane shows Word, Excel and PowerPoint files through Office's preview handlers, and Explorer shows their thumbnails. OpenXplorer said "No preview available." for every office document and kept their type icons.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
None.

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: all as an unprivileged account in `check.py`'s isolation (private Xvfb, private message bus, throwaway home and XDG folders), Rust 1.92.0. `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`; `python3 native/parity/check.py`, `python3 native/tools/i18n.py check`, `python3 tools/sync-docs.py --check`, `check.py`'s inventory and driver tests. `python3 native/tools/check.py` stopped at ox-core's `location_fixture_drift`, which fails on main too, so every test binary was run one by one in its isolation: 75 of 88 binaries and the doctests passed, the 12 failing ox-core binaries (Python-interop, `admin_copy` and versions suites) are untouched by the branch, and ox-app had 2,654 passed with the same 7 failures as main plus one timing-sensitive test per run that passes alone. 17 new converter tests and 5 new window tests (they skip where LibreOffice, `pdftoppm` or a working bubblewrap is missing) with LibreOffice 24.2: each kind converts in 0.46 to 0.67 s, a slow workbook is stopped at 15.0 s, a 256 MiB deflate bomb is stopped by the file-size limit, damaged, password-required and noise files fail quietly, cancelling stops LibreOffice at once, a document changed to the same size 1 ms later is converted again and never shown from its old PDF, the selection goes before a slow thumbnail in Large icons, a decoder that ended while idle is replaced, and closing a window leaves no LibreOffice running. Real X input (XTEST and xdotool) through 50 mixed documents with the Preview pane, the Details pane and Large icons while every thumbnail was made, in light and dark: the longest main-loop step was 4.7 to 48 ms, every keypress showed within 0.49 s, every visible document got its framed first page, no decoder failures occurred, no LibreOffice was left running and nothing was written beside the documents. Not run: a file encrypted by Office itself, a per-user systemd scope, Wayland, and a Next.js build.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
