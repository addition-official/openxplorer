# Title

feat(preview): Show Office and OpenDocument files in the Preview pane and as thumbnails

# Body

Model: claude-opus-5-5
CLA: I agree

## Problem and resulting behavior

Windows 11's Preview pane shows Word, Excel and PowerPoint files through Office's preview handlers, and Explorer shows their thumbnails. OpenXplorer said "No preview available." for every office document and kept their type icons.

Now, when LibreOffice is installed:

- **Preview pane.** A Word, Excel or PowerPoint file (docx, docm, doc, xlsx, xlsm, xls, pptx, pptm, ppt) or an OpenDocument text, spreadsheet or presentation (odt, ods, odp and their templates) shows its pages one under the other, scrollable, drawn the way the pane already draws a PDF's.
- **Thumbnails.** At Medium icons and larger, a document's first page is its thumbnail, framed by a light gray line so a mostly white page still reads as a page. It is written into the shared freedesktop cache through the app's existing writer (`Thumb::URI`, `Thumb::MTime`). Rows of Details, List and Small icons keep the type icon, as Windows draws them, even when the cache has the thumbnail. The Details pane shows it.
- **Without LibreOffice** (or without bubblewrap or poppler's `pdftoppm`), the pane says "No preview available." under the document's icon. Nothing fails loudly.

How it runs:

- LibreOffice converts the document to PDF inside the thumbnailers' existing bubblewrap sandbox: no network, no session bus, no other process, the system read-only, and the document alone of the user's files, read-only under a name of the sandbox's own.
- It runs within the thumbnailers' memory, file-size, processor and task limits, with its thread pool held to 4 (`MAX_CONCURRENCY`). It is stopped after 15 seconds, or as soon as the selection changes.
- Each run gets a new private profile (`-env:UserInstallation` in the sandbox's own folder). The profile is written beforehand with `DisableMacrosExecution`, `MacroSecurityLevel` 3, link updates set to Never (Writer 0, Calc 1), `BlockUntrustedRefererLinks` and no OpenCL.
- LibreOffice never sees the document's folder, so it can leave no `.~lock` or `~$` file beside it.
- One document converts at a time. The Preview pane goes first: it stops a thumbnail's conversion of another document, which is asked again later. At most four thumbnails wait.
- The PDF is cached in `~/.cache/winspace/document-previews`, keyed by the file's real path, size, modification time and inode (256 PDFs, 512 MB, least recently used first). The first pages of the last eight documents stay in memory, so selecting a document again shows it at once.
- Only files of at most 100 MB that start as a ZIP, a Compound File, RTF or markup (the HTML `.xls` exports many sites produce) are converted.
- A damaged document and one that needs a password are not tried again in the run. Neither is one that runs out of time twice. A cancelled conversion is tried again, and so is one the app could not prepare or keep (a full disk).
- If a sandbox cannot mount its own `/proc`, which LibreOffice needs, no conversion is started; the app logs that once.
- A document's pages are drawn once more when the first try gives nothing and the request still stands.
- A thumbnail conversion that the Preview pane stopped waits for its turn again, so arrowing through Large icons leaves no tile without its thumbnail.
- A document that takes the full 15 s holds the one thumbnail slot that long.

Not included: the Flatpak `org.libreoffice.LibreOffice`. Flatpak always mounts the app's own data folder read-write, and that is where the user's LibreOffice keeps its profile. A converted document could therefore change the profile the user opens next, which this sandbox otherwise rules out.

Fixed on the way, in `picture_guard/decoder.rs`: the picture decoders run in bubblewrap with `--die-with-parent`. That flag ends a decoder when the thread that started it exits, not when the app does. GLib's worker threads exit once idle, so decoders kept in the pool were ended while they waited. The next picture handed to one got no answer ("A picture's decoder ended without an answer"). On main, the first PDF selected after launch said "No preview available." in 3 of 3 runs, and thumbnails went missing now and then. The pool now drops an idle decoder that no longer runs. A test kills an idle decoder and fails without the fix. In the real app, Large icons runs went from 2 to 5 such failures each to none.

No new dependency. The test documents are written by the tests themselves as small hand-built ZIP containers (`test_support/office_documents.rs`, with a deflate bomb written through GIO). No third-party file is used.

## Validation

All runs used the unprivileged `tester` account with check.py's isolation: private Xvfb, private session bus, throwaway HOME and XDG folders. Rust 1.92.0 was used, as in CI.

- `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `python3 native/parity/check.py`, `python3 native/tools/i18n.py check`, `python3 tools/sync-docs.py --check`, check.py's inventory and driver tests: pass.
- `python3 native/tools/check.py`: stops at ox-core's `location_fixture_drift`, which fails on main too (Python 3.13). Every test binary was then run in check.py's own isolation, one by one.
  - 75 of 88 binaries pass, and the doctests pass.
  - The 12 failing ox-core binaries (Python-interop, `admin_copy` and versions suites) are untouched by this branch.
  - ox-app: 2,654 passed. Its failures are the 7 that also fail on main in the same isolation, plus one timing-sensitive test per run that passes alone.
- New converter tests (17) and window tests (5) run where LibreOffice, pdftoppm and a working bubblewrap are installed, and skip with a note elsewhere. Here, LibreOffice 24.2 was used:
  - each kind converts in 0.46–0.67 s;
  - a cache hit takes about 70 µs;
  - a slow workbook is stopped at 15.0 s;
  - a 256 MiB deflate bomb is stopped by the file-size limit in about 2 s;
  - damaged, password-required and noise files fail quietly;
  - cancelling stops LibreOffice at once;
  - a document changed to the same size 1 ms later is converted again and never shown from its old PDF;
  - a docx embedding a 20000×20000 picture converts in 13 s at a 664 MiB peak.
- Preview pane, cold, from selection to first page: small documents 0.58–0.88 s; a 600-paragraph docx 1.3 s, a 3,000-row xlsx 2.0–2.2 s, a 120-slide pptx 1.0–1.1 s. Selecting a document again takes 106–170 ms, including the pane's 90 ms settle.
- Real X input (XTEST) through 50 mixed documents with the Preview pane, then the Details pane, then in Large icons while every thumbnail was made: the longest main-loop step was 4.7–48 ms. No LibreOffice was left running, and nothing was written beside the documents.
- With a 30-second workbook's thumbnail converting, the selected document showed in 0.76 s. The test fails when preemption is disabled.
- A sandboxed conversion works beside the user's own running LibreOffice and leaves it alone.
- The real app, driven with xdotool on a private Xvfb, 1x in light and dark, through the same 50 documents: every keypress showed within 0.49 s (screenshot-based timing). The first document after launch shows its page, and no LibreOffice remains after quitting.
- LibreOffice's peak memory per conversion was 175–322 MiB.
- The real app in Large icons (Ctrl+Shift+2), with real keys moving the selection while thumbnails were made: every visible document got its framed first page (47 thumbnails in 66 s), the damaged and noise files kept their icons, and no decoder failures occurred.
- Not run:
  - a file encrypted by Office itself (an OpenDocument whose manifest asks for a password was tested);
  - a systemd user scope (the container has none, so the per-process limits were used);
  - Wayland;
  - a Next.js build (no website code changed).

## Compatibility and release impact

- New cache folder `~/.cache/winspace/document-previews` (folder 0700, PDFs 0600). No settings change, and existing `winspace` paths are untouched.
- LibreOffice and poppler-utils are optional; no package depends on them.
- The interface guide (`apps/web/lib/docs.json`, regenerated with `tools/sync-docs.py`) and the parity inventory (VIEW-070, VIEW-057) describe the new previews.
- All test documents and captures use fictional content only (docs/PRIVACY.md).
