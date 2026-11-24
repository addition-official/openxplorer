# Title

feat(new): Offer the OpenDocument types in New, and show a new item that sorts last in a long folder

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/11-pass-10-home-recent)

## What changes for the user
- **A new item that sorts last in a long folder is shown with the keyboard.** In a folder of 400 or 10,000 files, a new item that sorts last (New Text Document after "file 9999.txt", or New folder when folders are not first) now scrolls into view once its row is laid out and is renamed in place only then; any input meanwhile still drops the rename.
- **New offers the OpenDocument types.** New lists OpenDocument Presentation, OpenDocument Spreadsheet and OpenDocument Text while the desktop has an application for the type, with that application's icon, among the documents in the order of their extensions, as Windows sorts them (.bmp, .odp, .ods, .odt, .rtf, .txt, .zip). Each makes "New OpenDocument Text.odt" and so on at once and renames it in place. The files are empty OpenDocument packages (stored `mimetype` first, manifest, `content.xml`, `styles.xml`; a presentation has one blank 16:9 slide) that LibreOffice 24.2 opens with no repair prompt.
- **Right-click > New shows the current templates.** An open folder menu, and the New menu opened in place, now get the template list once it has been read, as the command bar's New already did.
- **A refused new file says the system's reason** without Rust's error number. New stays on in folders without write permission, as the owner decided (#138); this is unchanged.
- **The delete question keeps a short name on one line.** A name of up to 40 characters no longer wraps as "New / fold- / er".
- **New and the folder's right-click menu match a Windows 11 VM check.** New is a plain list in the command bar and in right-click > New alike, with no separators and no Ctrl+Shift+N beside Folder, and Rich Text Document is offered, like the OpenDocument types, only while an application opens RTF files. A folder's blank space in the compact menu is View, Sort by, Group by | Undo <step> (while there is one), New, Properties | Open in Terminal | Show more options, with Paste alone in the icon row and no Refresh (F5 still refreshes); the app's own Open as administrator... stays where it applies. Network's blank-space menu also has no Refresh. New stays visible and grayed on This PC, Gallery, the Recycle Bin and Home, and Undo after New and a rename gives back the name, then moves the file to the Recycle Bin without a question.
- New Bitmap image deliberately differs from Windows: Windows makes an empty, 0-byte `.bmp`; Linux image programs refuse an empty file, so the app writes a valid 58-byte 1 x 1 white bitmap.
- The user guide (docs JSON and the generated Markdown) and the in-app manual mention the OpenDocument types, Rich Text Document's condition and the bitmap.
- For developers: the PR's own "Drop redundant tests" commits remove some of the tests it added, with the helpers only they used.

## Why
- The list scrolled before it had measured the new last row, so the new item's name field stayed hidden without the keyboard and the name typed next was lost.
- Windows lists a file type in New once a program registers it; the OpenDocument types were missing with LibreOffice installed.
- The folder menu built its New submenu when it opened, so a template added meanwhile was missing and a removed one was still offered.
- A refused new file said, for example, "…/New Text Document.txt: Permission denied (os error 13)".
- With no detail line beside it, a short name in the delete question wrapped inside its words.
- New had separators, a shortcut and an unconditional Rich Text Document, and the folder menu's rows differed from Windows 11's.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "New OpenDocument Presentation"
- "New OpenDocument Spreadsheet"
- "New OpenDocument Text"
- "Creating a document…"
- The first line of the translatable help text about file operations changes (the lines after it in the same entry are unchanged): "- New in the command bar lists what Windows 11 lists: Folder, Shortcut, Bitmap image, Rich Text Document, Text Document and Compressed (zipped) Folder, then the files in your Templates folder. A new item is made at once as, for example, "New Text Document.txt" (or "New Text Document (2).txt" when that name is taken), then renamed in place: type its name and press Enter." → "- New in the command bar lists what Windows 11 lists: Folder, Shortcut, Bitmap image, Text Document and Compressed (zipped) Folder, with Rich Text Document while an app opens RTF files and OpenDocument Presentation, Spreadsheet and Text while an office suite such as LibreOffice is installed, then the files in your Templates folder. A new item is made at once as, for example, "New Text Document.txt" (or "New Text Document (2).txt" when that name is taken), then renamed in place: type its name and press Enter."

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix with a test seen failing before it: `window::tests::new_folder::a_new_item_that_sorts_last_in_a_long_folder_is_shown_with_the_keyboard`, `window::tests::new_text_document::the_opendocument_types_are_offered_while_an_application_opens_them`, `window::file_ops::opendocuments::tests::*`, `ox_core::ops::opendocument::tests::*`, `ops_create::each_new_document_is_made_under_its_windows_name_with_its_content`, `new_text_document::a_template_added_while_the_window_is_open_is_in_the_folder_menus_new_at_once`, `ox_core::ops::new_from_template::tests::a_refused_new_file_says_the_systems_reason_without_its_number`, `window::file_ops::delete::tests::a_short_name_alone_stays_on_one_line`, `context_menus::the_compact_folder_menu_has_windows_11s_rows_and_paste_in_its_icon_row`, `context_menu::entries::tests::the_compact_background_menu_ends_with_show_more_options`, `new_text_document::the_new_menu_shows_no_shortcut_and_rich_text_only_with_an_application`, the New rows of `command_bar::the_menus_list_the_current_items_between_the_same_dividers` and `context_menus::right_clicking_blank_space_opens_the_folder_menu_and_new_opens_beside_it`, and the guards `new_text_document::new_is_shown_grayed_where_nothing_can_be_made` and `new_text_document::undo_after_new_and_rename_gives_back_the_name_then_the_file_goes_without_a_question` (some of these were later removed as redundant and are not in this branch); real input with xdotool under a private Xvfb with Openbox and xcompmgr as an unprivileged user with a throwaway home: every New item from the command bar and the folder menu, Templates with odd entries, New in a 10,000-file folder, Undo and Redo of all seven document types, a local Samba share (writable and read-only), inside a ZIP, the Recycle Bin, Home and Gallery, and each new file opened in its application (the OpenDocument files in LibreOffice Writer, Calc and Impress, and converted to PDF and to docx/xlsx/pptx); `native/tools/check.py` with Rust 1.92 through the stages before the test executables, and the doctests; every test executable in check.py's isolation as an unprivileged user; `native/parity/check.py`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
