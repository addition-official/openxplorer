# Title

feat(new): Offer the OpenDocument types in New, and show a new item that sorts last in a long folder

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 11 tested the New menu and its templates against Windows 11: the command bar's New and right-click > New (items, order, icons), Ctrl+Shift+N, each document type opened in its application, the Templates folder, naming and rename in place, Escape, read-only folders, an SMB share (writable and read-only), inside a ZIP, the Recycle Bin, Home and Gallery, Undo of every item, and New in an empty folder and in a 10,000-file folder. It fixes what was broken or unlike Windows:

- **A new item that sorts last in a long folder stayed off screen.** In a folder of 400 or 10,000 files, New Text Document (after "file 9999.txt") or New folder with folders not first was selected but the list stopped one row short of it: the list scrolled before it had measured the new last row. Its name field stayed hidden without the keyboard, and the name typed next was lost. The view now scrolls again once the row is laid out and renames it in place only then; any input meanwhile still drops the rename.
- **The OpenDocument types were missing.** Windows lists a file type in New once a program registers it (Word, Excel and PowerPoint with Office; the OpenDocument types with LibreOffice). New now offers OpenDocument Presentation, OpenDocument Spreadsheet and OpenDocument Text while the desktop has an application for the type, with that application's icon, among the documents in the order of their extensions, as Windows sorts them (.bmp, .odp, .ods, .odt, .rtf, .txt, .zip). Each makes "New OpenDocument Text.odt" and so on at once and renames it in place. The files are empty OpenDocument packages (stored `mimetype` first, manifest, `content.xml`, `styles.xml`; a presentation has one blank 16:9 slide) that LibreOffice 24.2 opens with no repair prompt.
- **Right-click > New showed an old list of templates.** The folder menu built its New submenu when it opened and read the Templates folder again only for the next menu, so a template added meanwhile was missing (and a removed one still offered). An open folder menu, and the New menu opened in place, now get the list once it has been read, as the command bar's New already did.
- **A refused new file showed Rust's error number.** In a folder without write permission, New Text Document said "…/New Text Document.txt: Permission denied (os error 13)"; it now ends with the system's reason alone, as New folder's GIO message does. New stays on in such folders, as the owner decided (#138).
- **The delete question broke a short name inside its words.** Undo of New folder on a share, which has no Recycle Bin, asks to delete it permanently; with no detail line beside it, the name wrapped as "New / fold- / er". A name of up to 40 characters now keeps one line.

The user guide (docs JSON and the generated Markdown) and the in-app manual mention the OpenDocument types.

This branch merges origin/network-like-windows, origin/pass-3-rename and origin/pass-10-home-recent (which holds passes 4, 6 and 7, with network-like-windows' Network tree kept over pass-4's).

## Tests

Each fix has a test that failed on the code before it (run and seen failing):

- `window::tests::new_folder::a_new_item_that_sorts_last_in_a_long_folder_is_shown_with_the_keyboard` (the name field whole inside the list and holding the keyboard, for New Text Document and New folder in a 400-file folder)
- `window::tests::new_text_document::the_opendocument_types_are_offered_while_an_application_opens_them` (menu order, icons, names, content; a type no application opens is not made), `window::file_ops::opendocuments::tests::*`, `ox_core::ops::opendocument::tests::*`, and `ops_create::each_new_document_is_made_under_its_windows_name_with_its_content` (the archive reader lists each package)
- `window::tests::new_text_document::a_template_added_while_the_window_is_open_is_in_the_folder_menus_new_at_once` (compact and classic menus)
- `ox_core::ops::new_from_template::tests::a_refused_new_file_says_the_systems_reason_without_its_number`
- `window::file_ops::delete::tests::a_short_name_alone_stays_on_one_line`

Real input with xdotool under a private Xvfb with Openbox and xcompmgr, as an unprivileged user with a throwaway home holding mp3, mp4, pdf, jpg, docx, odt, zip, 7z, a 2 GB sparse file, broken jpg and mp4, and odd names: every New item from the command bar and the folder menu; Escape and typed names; numbered names; Templates with a subfolder, a hidden file, a link, a `.desktop` file and an executable script (the last three are skipped or copied without the execute bit); New in a 10,000-file folder; Undo of all seven document types in order and Redo; a Samba share on 127.0.0.1 (create, rename, Undo asks to delete permanently) and a read-only share ("Permission denied", nothing left); inside a ZIP, the Recycle Bin, Windows' Home page and Gallery (New off, Ctrl+Shift+N does nothing). The new files were opened in their applications: the bitmap with ImageMagick, the RTF with LibreOffice and AbiWord, the ZIP with the app, unzip and 7z, each OpenDocument file in LibreOffice Writer, Calc and Impress (by double-click from the app for the spreadsheet) and converted to PDF and to docx/xlsx/pptx. No "Bail out", CRITICAL, panic or signal from the app.

`native/tools/check.py` with Rust 1.92: every stage before the test executables passes at the merged head (parity tests, parity check, i18n check, tools tests, icon guard, rustfmt, Clippy `-D warnings`), and the doctests pass. As root in this container it then stops at `display_scale`, whose snapshots start the app, which refuses root; every test executable was run in check.py's own isolation as an unprivileged user instead: 87 executables, 5,096 passed, 34 failed, no crash. 32 of the failed names also fail on unchanged upstream main (the Python-app fixture tests); `empty_folder_look::a_long_note_wraps_inside_the_pane` fails on origin/pass-10-home-recent alone, and `search_history::a_tab_on_results_keeps_its_selection_and_scroll_position` failed once in the long run and passes alone. Before pass-10 was merged, the app's test executable crashed (SIGSEGV in the menu sweep) exactly as origin/network-like-windows alone does; pass-10 brings pass-6's fix for it. `native/parity/check.py` passes.

## Not covered

- The navigation pane's pinned-folder New (sidebar menu) still builds its template list when it opens; it is in the navigation pane's code, which pass-4 and network-like-windows both change.
- Windows' New menu details (order by extension with more registered types, its icons, Ctrl+Shift+N from the search box) are from memory, not a Microsoft source; the status file lists steps to confirm them in Windows 11.
