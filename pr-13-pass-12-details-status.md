# Title

fix(details-pane): Make the Details pane and status bar read as Windows 11's and stop decoding every photo passed

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/12-pass-11-new-menu)

## What changes for the user
- **Arrowing through photos no longer decodes each one.** The Details pane now waits 90 ms for the selection to rest before it reads a picture, a document's thumbnail, a folder's picture or a recording's player, as the Preview pane's pages already did, and a new selection cancels the wait and the decoding.
- **Several selected items show Windows' hint, not a Size row.** The pane shows their stacked pictures, "N items selected" and the hint to select a single file, and leaves their size to the status bar; also for several drives on This PC. Properties stands as clear of the hint as of a file's rows.
- **A drive selected on This PC has a Properties button**, which opens its Properties, as Alt+Enter does; also for a connected share.
- **Places are pictured as their tabs are.** With nothing selected, the pane shows the Recycle Bin, Network's and Home's icons in their navigation pane colors, and a standard folder's own picture, instead of a plain yellow folder.
- **Search results are counted as items**: "41 items" or "1,204 items", through the folder count's translated message, with "(first 500)" when the list stopped there and "Cached" for the search cache's results.
- **A measured folder's size shows in its row at once** after Calculate folder size.
- **The status bar's dividers** match Windows 11: the count ends with a divider even with nothing selected ("3 items |"), and the selection with another ("3 items | 1 item selected |").
- **The hint** keeps "Select a single file to get more information."; Windows adds "and share your cloud content", and the app has no cloud.
- **No Date created.** A file lists Type, Size, File location and Date modified; a folder the same without Size.
- **A drive's rows.** A drive selected on This PC lists Type, Space used (a percentage), Space free, Total size and File system, read on a worker thread and worded as the drive's Properties words them ("Local Disk", "NTFS", or Linux's own name such as "ext4").
- **A recycled file** is headed by the full path it was deleted from, located in the trash folder that holds it (the file GVfs names in `standard::target-uri`, kept as `EntryMeta::trash_file`), with no Original location or Date deleted rows, and Share attaches that file.

## Why
- With the pane open, every photo the selection passed was decoded in full, and the decoding went on after the selection moved on: holding Down through 60 photos of 24 megapixels cost 6.4 seconds of processor time against 1.2 with the pane closed. The same run now costs about what it costs with the pane closed.
- Windows 11's pane shows several items without a Size row, offers Properties for a drive, and pictures places by their own icons.
- Search results read "41 results", without grouping and in English that could not be translated.
- GTK keeps a row's cells bound when its item is reported changed but is the same object, so a measured folder kept an empty Size cell until the list was drawn again.
- A Windows 11 VM check showed the status bar's dividers, the file and folder rows without Date created, a drive's rows and a recycled file's heading.

## CHANGED OWNER DECISIONS
None.

## New and changed visible text
- "{shown} results" is removed; search results are counted with the existing item count message.
- "(first 500)"
- "Cached"
- "Space used"
- "Space free"
- "Total size"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: each fix with a test seen failing before it: `window::tests::details_preview::arrowing_through_photos_reads_only_the_one_the_selection_stops_on` (6 reads before, 1 after), `window::details_pane::content::tests::several_items_are_counted_stacked_and_hinted`, `window::tests::details_pane::several_items_show_their_count_and_windows_hint`, `window::tests::landing_pages::the_panes_properties_button_follows_a_selected_drive`, `search::report::tests::the_status_bar_counts_results` and the search window tests, `window::tests::item_dialogs::a_measured_folder_shows_its_size_in_its_row_at_once`, and from the VM check `window::tests::environment::the_status_bar_counts_this_pc_and_a_folder`, `this_pc_opens_with_the_first_tile_focused_and_nothing_selected`, `window::tests::details_pane::nothing_selected_shows_windows_hint_alone`, `window::tests::landing_pages::this_pc_counts_its_tiles_in_the_status_bar_and_the_details_pane`, `window::details_pane::content::tests::a_file_shows_its_type_size_location_and_date_modified`, ox-core `entry::info::tests::a_recycled_item_keeps_its_file_in_the_trash` and the recycled cases of `window::share::tests::share_needs_files_and_none_in_zips` (the branch's own test pruning removed five further tests); real input with xdotool under a private Xvfb as an unprivileged user with a throwaway home (60 photos of 24 megapixels: 641-669 ticks of processor time before, 87-132 after, 110 with the pane closed; 1,040 files; a drive's Properties from the pane's button; files renamed, replaced and deleted from outside while selected; light and dark captures); `native/tools/check.py`'s rustfmt, parity, i18n, tools and icon stages, with Clippy `-D warnings` passing once three lints new in Rust 1.97 were allowed, as on unchanged main; every test executable in check.py's isolation as an unprivileged user (5,131 passed and 4 failed: two Python fixture tests that also fail on unchanged main, an animations test that the New menu work's later commit fixes, and one test that timed out under load and passed alone); `native/parity/check.py`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
