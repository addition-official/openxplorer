// SPDX-License-Identifier: AGPL-3.0-only
//! Settings > Appearance > Layout > "Column header style" in a real
//! window (VIEW-014). The app's own style stays the default; Windows'
//! draws the sorted column as Windows 11 File Explorer does: only a small
//! gray chevron centered over the column, above its title, which keeps
//! its color and weight, with no line under it and no change of
//! background, while every other title draws nothing new.

use std::time::Duration;

use gtk::prelude::*;
use ox_core::settings::{ColumnHeaderStyle, PreferencesUpdate, Settings};

use super::geometry::laid_out;
use super::icons::{pixel_rows, Pixel};
use crate::folder_view::sorting::SortColumn;
use crate::test_support::harness::{wait_for, wait_for_frames, wait_until, Fixture, TestWindow};

/// Saves `style` as Settings or another window would, and lets the
/// window draw it.
fn save_style(test: &TestWindow, style: ColumnHeaderStyle) {
    let update = PreferencesUpdate {
        column_header_style: Some(style),
        ..PreferencesUpdate::default()
    };
    Settings::open(test.settings_directory())
        .update_preferences(&update)
        .expect("the settings file takes the change");
    test.context.reload_settings();
    wait_until("the window to follow the style", || {
        test.window.column_header_style() == style
    });
    settle_colors(test);
}

/// Lets the titles' 83ms color fades (base.css, M01) end and the window
/// redraw.
fn settle_colors(test: &TestWindow) {
    wait_for(Duration::from_millis(250));
    wait_for_frames(&test.window, 2);
}

/// Sorts the window by `column` in `direction` (`ascending` or
/// `descending`), as the Sort menu does, and lets it redraw.
fn sort_by(test: &TestWindow, column: &str, direction: &str) {
    test.activate("sort", Some(column));
    test.activate("direction", Some(direction));
    settle_colors(test);
}

/// The title of `column` in the window's details view.
fn title_of(test: &TestWindow, column: SortColumn) -> gtk::Widget {
    let pane = test.window.folder_pane();
    let titles = pane.details().titles_by_column();
    titles.title(column).expect("a title for every column").clone()
}

/// The pixels `title` draws, row by row, as the header draws it in
/// place.
fn drawn(title: &gtk::Widget) -> Vec<Vec<Pixel>> {
    let renderer = title
        .native()
        .and_then(|native| native.renderer())
        .expect("the title is on a drawn window");
    let header = title.parent().expect("a title is in the header");
    let place = title.compute_bounds(&header).expect("the title is laid out");
    let snapshot = gtk::Snapshot::new();
    header.snapshot_child(title, &snapshot);
    let node = snapshot.to_node().expect("the title draws something");
    pixel_rows(&renderer.render_texture(&node, Some(&place)))
}

/// Whether two pixels differ visibly.
fn differ(one: Pixel, other: Pixel) -> bool {
    one.iter()
        .zip(other)
        .any(|(channel, background)| channel.abs_diff(background) > 40)
}

/// The inked pixels of `rows`, those that differ from the top-left
/// corner's background, as (column, row).
fn ink(rows: &[Vec<Pixel>], limit: usize) -> Vec<(usize, usize)> {
    let background = rows[0][0];
    let mut inked = Vec::new();
    for (y, row) in rows.iter().take(limit).enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            if differ(*pixel, background) {
                inked.push((x, y));
            }
        }
    }
    inked
}

/// With Windows' style the chevron stands at the title's top edge, well
/// clear of the text under it, in a column as wide as a new window's and
/// in one narrower than its title, where the text runs under the middle.
///
/// parity: VIEW-014
#[gtk::test]
fn windows_style_keeps_the_chevron_clear_of_the_text_in_any_column_width() {
    let fixture = Fixture::standard();
    let test = laid_out(&fixture.uri());
    save_style(&test, ColumnHeaderStyle::Windows);
    let view = test.window.folder_pane().details();
    let modified = view.column(SortColumn::Modified).expect("a Date modified column");
    for width in [None, Some(70)] {
        if let Some(width) = width {
            modified.set_fixed_width(width);
        }
        for direction in ["ascending", "descending"] {
            sort_by(&test, "modified", direction);
            let title = title_of(&test, SortColumn::Modified);
            let rows = drawn(&title);
            // Every inked row, the line under the header left out.
            let inked_rows: Vec<usize> = (0..rows.len() - 1)
                .filter(|row| !ink(&rows[*row..=*row], 1).is_empty())
                .collect();
            let top = *inked_rows.first().expect("a chevron");
            assert!(top <= 2, "{width:?} {direction}: from row {top}");
            let chevron_end = inked_rows
                .iter()
                .zip(inked_rows.iter().skip(1))
                .find(|(row, next)| **next > **row + 1)
                .map(|(row, _)| *row)
                .expect("text under the chevron");
            let text_top = inked_rows
                .iter()
                .find(|row| **row > chevron_end)
                .expect("text under the chevron");
            assert!(
                text_top - chevron_end > 3,
                "{width:?} {direction}: the chevron ends at row {chevron_end}, the text starts at {text_top}"
            );
        }
    }
}
