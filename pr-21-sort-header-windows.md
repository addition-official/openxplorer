# Title

feat(details): Offer Windows' column header style as a setting

# Body

Model: claude-opus-5-5

CLA: I agree

Builds on: the previous PR in this series (pr/20-pass-19-a11y)

## What changes for the user
- New setting, Settings > Appearance > Layout > Column header style, with two choices:
  - **OpenXplorer (default)**: the current look, unchanged. A chevron after the sorted title, which is bright; any sort other than Name ascending turns the title and chevron the accent with a 2 px line under it. This is the default and is not written to `settings.json`.
  - **Windows**: Windows 11 File Explorer's look. Only a thin gray chevron, up while ascending and down while descending, 8 by 5 px at 100% scale, on the title's first row and centered over the sorted column. Every title's text sits 2 px higher than in the app's own style while the header keeps its height, so 3 px stay clear under the chevron. The sorted title keeps the other titles' color and weight, with no accent, no line and no background change; hover and press use the subtle grays. Stored as `"columnHeaderStyle": "windows"`.
- Every open window switches at once. A class on the window (`windows-column-headers`, `src/window/column_header_style.rs`) selects the CSS in `folder-views.css`, so nothing is rebuilt.
- `native/docs/ui-spec.md` gains a "Sort indicator, Windows style" row, and VIEW-014's parity note describes both styles and names the current tests.

## Why
The app's sort header (a chevron after a bright title, with the accent for a sort other than Name ascending) is the owner's chosen default, but people who want the Details view to look like Windows 11 had no way to get Windows' small gray chevron over the column. The chevron's size and position were measured on a Windows 11 reference capture (1920x1080, 100%, dark): 8 by 5 px on the header's first row, centered over the column, with the title's caps 8 px down and 3 px clear of it. An earlier draft placed it lower and closer to long titles such as "Date modified", where it read as part of the title.

## CHANGED OWNER DECISIONS
None. The owner's header look (owner decision 2026-11-21) stays the default; Windows' look is an opt-in choice.

## New and changed visible text
New:
- "Column header style"
- "How the Details view marks the sorted column. OpenXplorer puts an arrow after its title and colors a sort other than Name ascending; Windows shows only a small gray arrow above the title, as Windows 11 does."
- "OpenXplorer (default)"
- "Windows"
- Settings search words for the new setting: "column header title sort sorted arrow chevron caret accent underline details view windows 11 style look"

## Tests
Run on this branch: `cargo check --workspace --all-targets` (no errors or warnings), `cargo fmt --all -- --check` and `python3 native/parity/check.py`, all passing. The test suites were not run on this branch.

Run earlier on the feature's own branch before it was split into this series, not rerun here: settings `the_column_header_style_row_saves_the_choice`; window tests on the header's drawn pixels, `windows_style_draws_no_accent_or_line_on_the_sorted_title`, `windows_style_shows_one_small_chevron_over_the_sorted_column`, `windows_style_keeps_the_chevron_clear_of_the_text_in_any_column_width`, `the_apps_own_column_titles_stay_the_default` and `the_apps_own_style_comes_back_with_its_accent`; the settings search and Appearance row list tests in `redesign.rs`, which count the new row; and two tests this PR does not carry, ox-core `the_column_header_style_is_the_apps_own_until_windows_is_chosen` and settings `the_column_header_styles_offer_the_apps_own_first_as_the_default`. `python3 native/parity/check.py`; `native/tools/check.py` (Rust 1.92.0) passed fmt, Clippy and the inventory checks; the test executables ran as root in a container, where permission-based tests fail for unrelated reasons, and every new test and all 121 settings tests passed.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
