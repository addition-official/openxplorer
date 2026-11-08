# Title

fix(compact-view): Space Compact view as Windows 11's Compact view

# Body

Model: not recorded here (this session's rules keep model names out of files pushed to the repository)

Compact view (View > Show > Compact view, and Settings > Appearance) now spaces the file list and the navigation pane the way Windows 11 File Explorer's Compact view does. Details rows, group headings, the column header, the navigation pane, Tiles and Content's headings and lines are all closer, as measured from Windows at 100% scaling. Everything still grows with the text size and doubles at 2x. Normal view is unchanged: its captures are pixel-identical to main's.

The Open and Save dialogs are browser windows, so they follow Compact view too; their spacing measures the same as the main window's. The List and icon views had no compact spacing and keep theirs.

One earlier decision changes: the UI spec kept 24 px as the lowest row, as the smallest pointer and touch target. Windows' 21 px pitch makes compact Details rows 19 px tall (hover, selection and focus boxes included). Normal view keeps its 30 px rows.

## Measured values (1x, logical pixels)

Distances are from the top of one line's capitals to the next, measured from captures of the real app under Xvfb, the way the Windows values were measured.

| What | Windows 11 Compact | Ours before (main) | Ours after |
|---|---|---|---|
| Details: row to row | 21 | 26 | 21 |
| Details: heading to its first row | 27 | 26 | 27 |
| Details: last row to the next heading | 26 | 34 | 26 |
| Column header, with its line | 26 | 32 | 26 |
| Column header to the first heading's text | about 14 | 17 | 14 |
| Navigation pane: row to row | 24 | 25 | 24 |
| Navigation pane: across a separator | 48 | 49 | 48 |
| Tiles: row to row | 57 (inferred from heading to heading) | 75 | 57 |
| Tiles: lines in a tile | 16 | 15 | 16 |
| Tiles: heading to heading over one row | 83 | 110 | 83 |
| Tiles: heading text to the icon's top | 26 | 36 | 28 (our icon art is 39 px tall to Windows' 42) |
| Tiles: icon's bottom to the next heading's text | 14 to 16 | 35 | 16 |
| Content: item to item | 48 | 48 | 48 |
| Content: name to second line | 18 | 15 | 18 |
| Content: heading to the first name | 31 to 32 | 30 | 32 |
| Content: last name to the next heading | 48 to 49 | 53 | 48 |

Main had no compact Tiles or Content, so their "before" values are main's usual spacing, measured from main's captures with Compact view on.

## How it was tested

- New tests, written first, fail on main and pass with the change:
  - `window/tests/compact_spacing.rs` (GTK): the distances above, measured between labels, and Page Up/Down counting the compact rows; a second test pins the normal view's spacing. Run at 1x and also at `GDK_SCALE=2`.
  - `tests/display_scale.rs`: the real app at `GDK_SCALE=1` and `2`, compact and normal Details rows (21 and 42 device pixels, 32 and 64).
  - `theme/fonts.rs`: the generated compact rules, and the normal rules unchanged.
- `python3 native/parity/check.py`: passes.
- `python3 native/tools/check.py`: rustfmt and clippy pass. In ox-app's suite 2106 tests pass and 2 fail, and check.py stops there; I ran the remaining test executables and the doctests the same way myself. The failures, all of which also fail on unchanged main in this container:
  - `location_fixture_drift::python_fixture_matches_desktop_core_py`: the container's Python 3.13 parses `smb://[V1.x]/` differently from the committed fixture. With Python 3.12 first in PATH it passes, on main and on this branch; the full run used that workaround.
  - `thumbnails::thumbnailers::tests::a_sandboxed_thumbnailer_runs_within_its_limits` and `a_sandboxed_thumbnailer_sees_only_its_item`: the container can't start bubblewrap's sandbox (run on main too).
  - 9 tests in ox-core's library (8 in `admin_copy`, which find the test folders writable by other users under this container's `/tmp`, and `clipboard::tests::external_fingerprint_matches_the_python_clipboard`). This branch changes nothing in ox-core, so main builds the same test binary.
- Item check boxes turned on: in compact Tiles and Details the box fits its row, nothing clipped.
- `python3 tools/audit-public-data.py`: no issues.
- Captures of the real app under Xvfb, driven with xdotool (hover, click, Down/Up, heading click, Left/Right), at 1x and 2x in light and dark, with fictional files. The checklist below was computed from those captures.

## Checklist (computed from the captures)

Each line is measured from the PNGs, in logical pixels, 0.5 px tolerance. "Off 8" lines compare normal view between this branch and main, pixel for pixel.

```
PASS 1x-dark Details 1 rows: row to row 21.0, 21.0, 21.0, 21.0 (target 21)
PASS 1x-dark Details 2 headings: heading to first row 27.0, 27.0; last row to heading 26.0 (targets 27, 26)
PASS 1x-dark Details 3 header: column header 26.0 with its line (target 26); first heading text 14.0 below it (Windows about 14)
PASS 1x-dark Details 4 hover: hover box (226.0, 245.0) (19.0 tall, target 19, ends before the next row's box)
PASS 1x-dark Details 5 click/keys: selected (226.0, 245.0), Down (247.0, 266.0), Up (226.0, 245.0) (19 tall, 21 apart, focus outline inside)
PASS 1x-dark Details 6 heading: hover (273.0, 292.0), click (273.0, 292.0) (19 tall); Left hides 3 rows, Right shows them again (11 lines)
PASS 1x-dark Tiles 1 rows: tile rows [57.0, 57.0] apart (target 57)
FAIL 1x-dark Tiles 2 headings: heading over one row to the next 83.0 (target 83); heading text to icon top 28.0 (Windows 26); icon bottom to heading text 16.0 (Windows 14-16)
PASS 1x-dark Tiles 4 hover: hover box (228.0, 284.0) (56.0 tall, 1 px gap to the next row)
PASS 1x-dark Tiles 5 click/keys: selected (228.0, 284.0), Down (285.0, 341.0)
PASS 1x-dark Tiles 6 heading: hover (343.0, 364.0), click (343.0, 364.0); Left folds (14 lines vs 15), Right unfolds (15)
PASS 1x-dark Content 1 rows: items 48.0 apart (target 48), name to second line 18.0 (Windows 18; 17.5 at 2x rounds to 17 or 18)
PASS 1x-dark Content 2 headings: heading to first name 32.0 (Windows 31-32), last name to heading 48.0 (Windows 48-49)
PASS 1x-dark Content 4 hover: hover fill and its separator lines (221.0, 270.0) (47 between the item's lines, one item of 48)
PASS 1x-dark Content 5 click/keys: selected (221.0, 270.0), Down (269.0, 318.0)
PASS 1x-dark Content 6 heading: hover (319.0, 346.0), click (319.0, 346.0) (27 tall, as the usual view's)
PASS 1x-dark Navigation 7: rows [24.0, 24.0, 24.0, 24.0] apart (target 24), across the separator 48.0 (target 48); hover (252.0, 275.0), click (252.0, 275.0), keyboard focus (252.0, 275.0), Down (276.0, 299.0) (23 tall, 24 apart)
PASS 1x-dark Off 8 details-base: identical
PASS 1x-dark Off 8 tiles-base: identical
PASS 1x-dark Off 8 content-base: identical
PASS 1x-dark Off 8 nav-hover: identical
PASS 1x-dark Off 8 nav-click: identical
PASS 1x-dark Off 8 nav-focus: identical
PASS 1x-dark Off 8 nav-down: identical
PASS 1x-dark Save 9: Save dialog spacing [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0] vs main window [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0]
PASS 1x-light Details 1 rows: row to row 21.0, 21.0, 21.0, 21.0 (target 21)
PASS 1x-light Details 2 headings: heading to first row 27.0, 27.0; last row to heading 26.0 (targets 27, 26)
PASS 1x-light Details 3 header: column header 26.0 with its line (target 26); first heading text 14.0 below it (Windows about 14)
PASS 1x-light Details 4 hover: hover box (226.0, 245.0) (19.0 tall, target 19, ends before the next row's box)
PASS 1x-light Details 5 click/keys: selected (226.0, 245.0), Down (247.0, 266.0), Up (226.0, 245.0) (19 tall, 21 apart, focus outline inside)
PASS 1x-light Details 6 heading: hover (273.0, 292.0), click (273.0, 292.0) (19 tall); Left hides 3 rows, Right shows them again (11 lines)
PASS 1x-light Tiles 1 rows: tile rows [57.0, 57.0] apart (target 57)
FAIL 1x-light Tiles 2 headings: heading over one row to the next 83.0 (target 83); heading text to icon top 28.0 (Windows 26); icon bottom to heading text 16.0 (Windows 14-16)
PASS 1x-light Tiles 4 hover: hover box (228.0, 284.0) (56.0 tall, 1 px gap to the next row)
PASS 1x-light Tiles 5 click/keys: selected (228.0, 284.0), Down (285.0, 341.0)
PASS 1x-light Tiles 6 heading: hover (343.0, 364.0), click (343.0, 364.0); Left folds (14 lines vs 15), Right unfolds (15)
PASS 1x-light Content 1 rows: items 48.0 apart (target 48), name to second line 18.0 (Windows 18; 17.5 at 2x rounds to 17 or 18)
PASS 1x-light Content 2 headings: heading to first name 32.0 (Windows 31-32), last name to heading 48.0 (Windows 48-49)
PASS 1x-light Content 4 hover: hover fill and its separator lines (221.0, 270.0) (47 between the item's lines, one item of 48)
PASS 1x-light Content 5 click/keys: selected (221.0, 270.0), Down (269.0, 318.0)
PASS 1x-light Content 6 heading: hover (319.0, 346.0), click (319.0, 346.0) (27 tall, as the usual view's)
PASS 1x-light Navigation 7: rows [24.0, 24.0, 24.0, 24.0] apart (target 24), across the separator 48.0 (target 48); hover (252.0, 275.0), click (252.0, 275.0), keyboard focus (252.0, 275.0), Down (276.0, 299.0) (23 tall, 24 apart)
PASS 1x-light Off 8 details-base: identical
PASS 1x-light Off 8 tiles-base: identical
PASS 1x-light Off 8 content-base: identical
PASS 1x-light Off 8 nav-hover: identical
PASS 1x-light Off 8 nav-click: identical
PASS 1x-light Off 8 nav-focus: identical
PASS 1x-light Off 8 nav-down: identical
PASS 1x-light Save 9: Save dialog spacing [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0] vs main window [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0]
PASS 2x-dark Details 1 rows: row to row 21.0, 21.0, 21.0, 21.0 (target 21)
PASS 2x-dark Details 2 headings: heading to first row 27.0, 27.0; last row to heading 26.0 (targets 27, 26)
PASS 2x-dark Details 3 header: column header 26.0 with its line (target 26); first heading text 14.5 below it (Windows about 14)
PASS 2x-dark Details 4 hover: hover box (226.0, 245.0) (19.0 tall, target 19, ends before the next row's box)
PASS 2x-dark Details 5 click/keys: selected (226.0, 245.0), Down (247.0, 266.0), Up (226.0, 245.0) (19 tall, 21 apart, focus outline inside)
PASS 2x-dark Details 6 heading: hover (273.0, 292.0), click (273.0, 292.0) (19 tall); Left hides 3 rows, Right shows them again (11 lines)
PASS 2x-dark Tiles 1 rows: tile rows [57.0, 57.0] apart (target 57)
FAIL 2x-dark Tiles 2 headings: heading over one row to the next 83.0 (target 83); heading text to icon top 27.0 (Windows 26); icon bottom to heading text 17.0 (Windows 14-16)
PASS 2x-dark Tiles 4 hover: hover box (228.0, 284.0) (56.0 tall, 1 px gap to the next row)
PASS 2x-dark Tiles 5 click/keys: selected (228.0, 284.0), Down (285.0, 341.0)
PASS 2x-dark Tiles 6 heading: hover (343.0, 364.0), click (343.0, 364.0); Left folds (14 lines vs 15), Right unfolds (15)
PASS 2x-dark Content 1 rows: items 48.0 apart (target 48), name to second line 17.0 (Windows 18; 17.5 at 2x rounds to 17 or 18)
PASS 2x-dark Content 2 headings: heading to first name 32.0 (Windows 31-32), last name to heading 48.5 (Windows 48-49)
PASS 2x-dark Content 4 hover: hover fill and its separator lines (221.0, 270.0) (47 between the item's lines, one item of 48)
PASS 2x-dark Content 5 click/keys: selected (221.0, 270.0), Down (269.0, 318.0)
PASS 2x-dark Content 6 heading: hover (319.0, 346.0), click (319.0, 346.0) (27 tall, as the usual view's)
PASS 2x-dark Navigation 7: rows [24.0, 24.0, 24.0, 24.0] apart (target 24), across the separator 48.0 (target 48); hover (252.0, 275.0), click (252.0, 275.0), keyboard focus (252.0, 275.0), Down (276.0, 299.0) (23 tall, 24 apart)
PASS 2x-dark Off 8 details-base: identical
PASS 2x-dark Off 8 tiles-base: identical
PASS 2x-dark Off 8 content-base: identical
PASS 2x-dark Off 8 nav-hover: identical
PASS 2x-dark Off 8 nav-click: identical
PASS 2x-dark Off 8 nav-focus: identical
PASS 2x-dark Off 8 nav-down: identical
PASS 2x-dark Save 9: Save dialog spacing [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0] vs main window [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0]
PASS 2x-light Details 1 rows: row to row 21.0, 21.0, 21.0, 21.0 (target 21)
PASS 2x-light Details 2 headings: heading to first row 27.0, 27.0; last row to heading 26.0 (targets 27, 26)
PASS 2x-light Details 3 header: column header 26.0 with its line (target 26); first heading text 14.5 below it (Windows about 14)
PASS 2x-light Details 4 hover: hover box (226.0, 245.0) (19.0 tall, target 19, ends before the next row's box)
PASS 2x-light Details 5 click/keys: selected (226.0, 245.0), Down (247.0, 266.0), Up (226.0, 245.0) (19 tall, 21 apart, focus outline inside)
PASS 2x-light Details 6 heading: hover (273.0, 292.0), click (273.0, 292.0) (19 tall); Left hides 3 rows, Right shows them again (11 lines)
PASS 2x-light Tiles 1 rows: tile rows [57.0, 57.0] apart (target 57)
FAIL 2x-light Tiles 2 headings: heading over one row to the next 83.0 (target 83); heading text to icon top 27.0 (Windows 26); icon bottom to heading text 17.0 (Windows 14-16)
PASS 2x-light Tiles 4 hover: hover box (228.0, 284.0) (56.0 tall, 1 px gap to the next row)
PASS 2x-light Tiles 5 click/keys: selected (228.0, 284.0), Down (285.0, 341.0)
PASS 2x-light Tiles 6 heading: hover (343.0, 364.0), click (343.0, 364.0); Left folds (14 lines vs 15), Right unfolds (15)
PASS 2x-light Content 1 rows: items 48.0 apart (target 48), name to second line 17.0 (Windows 18; 17.5 at 2x rounds to 17 or 18)
PASS 2x-light Content 2 headings: heading to first name 32.0 (Windows 31-32), last name to heading 48.5 (Windows 48-49)
PASS 2x-light Content 4 hover: hover fill and its separator lines (221.0, 270.0) (47 between the item's lines, one item of 48)
PASS 2x-light Content 5 click/keys: selected (221.0, 270.0), Down (269.0, 318.0)
PASS 2x-light Content 6 heading: hover (319.0, 346.0), click (319.0, 346.0) (27 tall, as the usual view's)
PASS 2x-light Navigation 7: rows [24.0, 24.0, 24.0, 24.0] apart (target 24), across the separator 48.0 (target 48); hover (252.0, 275.0), click (252.0, 275.0), keyboard focus (252.0, 275.0), Down (276.0, 299.0) (23 tall, 24 apart)
PASS 2x-light Off 8 details-base: identical
PASS 2x-light Off 8 tiles-base: identical
PASS 2x-light Off 8 content-base: identical
PASS 2x-light Off 8 nav-hover: identical
PASS 2x-light Off 8 nav-click: identical
PASS 2x-light Off 8 nav-focus: identical
PASS 2x-light Off 8 nav-down: identical
PASS 2x-light Save 9: Save dialog spacing [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0] vs main window [27.0, 21.0, 21.0, 26.0, 27.0, 21.0, 21.0, 26.0, 27.0, 21.0]
```

The four FAIL lines are one finding: in Tiles, the heading's text stands 28 px above the icon's top (27 at 2x) against Windows' 26, because our icon art is 39 px tall to Windows' 42. Moving the heading down would push the gap under the icons past Windows' 14 to 16 while keeping the 83 px heading to heading.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
