# Title

feat(details): Offer Windows' column header style as a setting

# Body

Model: claude-opus-5-5

## What this does

Adds **Settings > Appearance > Layout > Column header style** with two choices:

- **OpenXplorer (default)**: the current look from #374, unchanged. A chevron after the sorted title, which is bright; any sort other than Name ascending turns the title and chevron the accent with a 2 px line under it. This is the default and is not written to `settings.json`.
- **Windows**: Windows 11 File Explorer's look. Only a small gray chevron (^ ascending, v descending), 7 x 4 px at 100% scale and 14 x 7 px at 200%, centered over the sorted column on the title's top edge. The title keeps the same color and weight as the other titles, with no accent, no underline and no background change. Hover and pressed use the subtle grays (`@ox_hover`, `@ox_pressed`). Stored as `"columnHeaderStyle": "windows"`.

Every open window switches right away. A class on the window (`windows-column-headers`, `src/window/column_header_style.rs`) selects the CSS in `folder-views.css`, so nothing gets rebuilt.

It also merges `pass-7-views`, which changes some of the same files (the Details view's stylesheet, the window's preferences, the settings tests and the .pot).

## Tests

- `ox-core`: `the_column_header_style_is_the_apps_own_until_windows_is_chosen`. Saves, reloads, ignores unknown values, and doesn't store the default.
- Settings: `the_column_header_style_row_saves_the_choice` and `the_column_header_styles_offer_the_apps_own_first_as_the_default`.
- Window, by rendering the header's pixels:
  - `windows_style_draws_no_accent_or_line_on_the_sorted_title`: with a sort that isn't the default, the sorted title draws the same pixels as when it was unsorted, except for the chevron rows, and its label keeps the other titles' color.
  - `windows_style_shows_one_small_chevron_over_the_sorted_column`: only the sorted title has ink above its text. That ink is 6 to 8 x 3 to 5 px, at the top edge, centered over the column, and points up or down to match the direction. GTK's indicator takes no room.
  - `the_apps_own_column_titles_stay_the_default` and `the_apps_own_style_comes_back_with_its_accent`.
- The settings search now treats the new row as one of the arrow settings (`redesign.rs`), and the Appearance row list includes it.
- `python3 native/parity/check.py` passes. `native/tools/check.py` (Rust 1.92.0) passes fmt, clippy and the inventory checks. The test executables were run as root in a container, where permission-based tests fail for reasons unrelated to this change; every new test and all 121 settings tests pass.

## Notes

- I measured the chevron's size and position in app captures (Xvfb) at 1x and 2x. I did not compare against a live Windows 11 machine, so the Windows reference values are observed and unverified, as `ui-spec.md` already says.
- Docs: added a "Sort indicator, Windows style" row to `native/docs/ui-spec.md` and updated VIEW-014's native note in `native/parity/features.toml`. That note still described the chevron from before #374 and named a test that was renamed.

CLA: I agree
