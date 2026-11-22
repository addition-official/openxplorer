# Title

fix(details-pane): Make the Details pane and status bar read as Windows 11's and stop decoding every photo passed

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Pass 12 tested the Details pane (Alt+Shift+P) and the status bar against Windows 11 with real input: every common file type (mp3, mp4, jpg, png, gif, pdf, docx, odt, csv, zip, 7z), damaged files, a 3 GB file, odd names, folders, drives, This PC, Network, the Recycle Bin, archive members, several selected items, 100–200 % text, renames, deletions and changes made outside the app, search results and folder sizes. It fixes what was broken or unlike Windows:

- **Arrowing through photos decoded every one of them.** With the pane open, each photo the selection passed was decoded in full, and the decoding went on after the selection had moved on. Holding Down through 60 photos of 24 megapixels cost 6.4 seconds of processor time with the pane open against 1.2 without it. The pane now waits 90 ms for the selection to rest before it reads a picture, a document's thumbnail, a folder's picture or a recording's player, as the Preview pane's pages already did, and a new selection cancels the wait and the decoding. The same run now costs about what it costs with the pane closed.
- **Several selected items showed a Size row.** Windows 11's pane shows several items as their stacked pictures, "N items selected" and its hint to select a single file, and leaves their size to the status bar. The pane now does the same, also for several drives on This PC.
- **A drive selected on This PC had no Properties button**, though Alt+Enter opened its Properties. The button now shows for a drive or a connected share and opens its Properties.
- **Places were pictured as a plain yellow folder.** With nothing selected, the pane drew a folder for the Recycle Bin, Network, Recent files, Gallery, the home folder and the standard folders such as Downloads. It now pictures each as its tab does: the bin, Network's and Home's icons in their navigation pane colors (as This PC already was), and a standard folder's own picture.
- **Search results were counted as "41 results".** Windows 11 counts them as items, as it counts a folder's: they now read "41 items" or "1,204 items", through the folder count's translated message; the number was written without grouping and the English was not translatable.
- **A measured folder kept an empty Size.** After Calculate folder size, the folder's row kept an empty Size cell until the list was drawn again for another reason, such as dismissing the bar: GTK keeps a row's cells bound when its item is reported changed but is the same object. The Size cells on screen are now bound again as each size arrives.
- **Properties touched the hint box** when several items were selected; it now stands as clear of it as of a file's rows.

The status bar's trailing divider ("6 items | 1 item selected 19.0 KB |") was added independently by pass-7-views; this branch merges that and keeps its version.

## Tests

Each fix has a test that failed on the code before it (run and seen failing):

- `window::tests::details_preview::arrowing_through_photos_reads_only_the_one_the_selection_stops_on` (6 reads before, 1 after)
- `window::details_pane::content::tests::several_items_are_counted_stacked_and_hinted`, `a_drive_selected_on_this_pc_offers_properties`, and `window::tests::details_pane::several_items_show_their_count_and_windows_hint`
- `window::tests::landing_pages::the_panes_properties_button_follows_a_selected_drive` (also proves GTK logs no action/target mismatch while the button changes)
- `window::details_pane::content::tests::a_place_is_pictured_as_its_tab_is`
- `search::report::tests::the_status_bar_counts_results` and the search window tests
- `window::tests::item_dialogs::a_measured_folder_shows_its_size_in_its_row_at_once` (also checks that keyboard focus stays on its row)
- `window::tests::panes_layout::the_properties_button_stands_clear_of_the_hint`

Real input with xdotool under a private Xvfb, as an unprivileged user with a throwaway home: arrowing through 60 photos of 24 megapixels (processor time 641–669 ticks before, 87–132 after, 110 with the pane closed) and through 1,040 files (1,089 ticks with the pane open, 1,008 closed; the pane kept up with 1,000 key presses at 10 ms); a drive's Properties opened from the pane's button; renaming, replacing and deleting files from outside while selected; captures of every changed screen in the light and dark themes. No "Bail out", CRITICAL, panic or signal from the app.

`native/tools/check.py` in this container (Rust 1.97): rustfmt, the parity unit tests and parity check, the i18n check, the tools unit tests and the icon guard pass. Clippy `-D warnings` stops on three lints new in Rust 1.97 (`duration_suboptimal_units`, `map_unwrap_or`, `decimal_bitwise_operands`) in files this branch does not touch, as it does on unchanged main; with only those allowed it passes on both. Every test executable was run as an unprivileged user in check.py's isolation, with the retired Python app extracted from tag v2.0.0: main 5d21fd3e has 4,979 passed and 2 failed, both this container's Python (`location_fixture_drift::python_fixture_matches_desktop_core_py`, `network_python::fuse_export_paths_match_native_opening_py`); this branch, with the lower passes merged, has 5,131 passed and 4 failed: those 2, `accessibility::animations_stay_off_when_the_desktop_turns_them_off`, which pass-11-new-menu's own tick callback in `window/loading.rs` trips, and `context_menus::a_pinned_folder_menu_copies_compresses_and_creates_in_that_folder`, which timed out once under load and passes three times out of three alone. `native/parity/check.py` passes.

## Not covered

- Windows 11's exact rows for each file type, a recycled item, an archive member and a drive in the new Details pane were not confirmed from a Microsoft source; they are on the VM checklist and were not changed.
- Windows 11's pane is read-only (Microsoft Q&A, 2025); the app's has no editable fields either, so rating, tags and title editing were not added.
- Sharing from the pane needs the desktop's Email portal, which this container does not have; it was covered by the existing tests only.
- The branch merges pass-7-views, pass-9-scaling, pass-10-home-recent and pass-11-new-menu (and through them every lower pass and network-like-windows), as the jobs' rules ask; the merges are not part of this change.
