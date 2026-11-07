# feat(preview): Show file contents in the Preview pane, as Windows 11 does

Model: claude-opus-5-5

## What changes

View > Preview pane (Alt+P) now shows the selected file's contents instead of only its icon or thumbnail:

- **Text and source files** (txt, md, log, csv, json, xml, ini, conf, rs, py, js, ts, c, cpp, h, sh, html, css and any type GIO calls text): the first 1 MB as read-only, selectable plain text in the desktop's monospace font at the app's text size, wrapping at the pane's edge. No syntax highlighting.
- **PDFs**: the pages one under the other, scrollable, drawn by poppler's `pdftoppm` inside the existing bubblewrap thumbnailer sandbox (no network, no session bus, the PDF alone of the user's files), four pages per run, up to 50 pages.
- **Pictures**: as large as the pane allows, aspect ratio kept, never stretched beyond their own size, never widening the pane.
- **Audio and video**: unchanged, the pane's existing player.
- **A folder, several items, a binary file, an unknown type, or a network file with network previews off**: "No preview available." centered in the pane. Nothing selected: "Select a file to preview.".

## Windows 11 behavior and sources

- The Windows TXT Previewer always wraps; wrapping cannot be turned off ([voidtools forum](https://www.voidtools.com/forum/viewtopic.php?t=14313)).
- A selected folder shows "No preview available." ([Microsoft Q&A](https://learn.microsoft.com/en-us/answers/questions/2615842/show-files-and-folders-in-the-preview-pane)); a type without a preview handler shows the same.
- With nothing selected the pane says "Select a file to preview." ([MakeUseOf](https://www.makeuseof.com/windows-10-preview-pane-not-working-fixes/)).
- Not found in sources, chosen as the most Windows-like and safest option: the text font (fixed-width, as the task asked), the message for several selected items ("No preview available."), and no note after a truncated text file.

## Safety and speed

- Nothing is read on the UI thread. A selection must settle for 90 ms before its file is read, a new selection cancels the previous read, and late results are dropped.
- Text: at most 1 MB is read, the file is opened without following links or blocking on pipes, and a NUL byte or more than 1 in 50 bytes of invalid UTF-8 or control characters counts as binary. UTF-16 with a BOM is decoded. Lines over 4,096 characters are broken so layout stays quick.
- Pictures: refused over 64 megapixels from their header, before decoding (the same cap as the thumbnail work in #285, which is not on main yet), then decoded to the pane's size.
- PDFs are only drawn in the sandbox; with no bubblewrap or no `pdftoppm` the pane says "No preview available.". Nothing is executed outside it.
- Network shares, phones and cameras follow the existing network-previews setting, as thumbnails do.
- No new link dependencies. Packages may want to recommend `poppler-utils` and `bubblewrap`.

## Testing

- Real input with xdotool on Xvfb, the app running as an unprivileged user with fictional files, at 1x and 2x in light and dark: text, Markdown, a 50 MB log (stays responsive), Rust source, a 20-page PDF (scrolled), a 6000 x 4000 JPEG, a PNG, an MP3 and an MP4 (same as main), a ZIP, an exe-like binary, a folder, two files, a file on a local Samba share with network previews off, and arrowing through 30 files. All pass.
- Unit and window tests: `preview_page` (4 new), `preview_pane` (3 new, 2 existing), plus `details_pane`, `details_preview` and `environment`, all pass.
- rustfmt clean; clippy shows no warnings in changed code; `parity/check.py` passes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
