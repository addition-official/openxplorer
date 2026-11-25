// SPDX-License-Identifier: AGPL-3.0-only
//! Tooltips only where Windows 11 File Explorer shows them (compared side
//! by side with Windows 11 26200): a navigation pane row shows one only
//! while its name is cut short, with the full name; its chevrons and
//! glyphs show none.

use std::fs;

use gtk::prelude::*;

use crate::test_support::harness::{descendants, wait_for_frames, wait_until, Fixture, TestWindow};
use crate::window::sidebar::row::shown_tooltip;

/// The sidebar row labelled `label`.
fn row_named(test: &TestWindow, label: &str) -> gtk::ListBoxRow {
    let index = test
        .window
        .sidebar()
        .labels()
        .iter()
        .position(|shown| shown == label)
        .unwrap_or_else(|| panic!("the sidebar shows {label}"));
    test.window
        .sidebar()
        .list()
        .row_at_index(i32::try_from(index).expect("a short sidebar"))
        .expect("a row for every label")
}

/// A navigation pane row shows no tooltip while its whole name shows, as
/// Gallery or Home in Windows 11, and the full name while it is cut
/// short with an ellipsis; no chevron or glyph of the pane has one.
///
/// parity: SIDE-001, ACC-001
#[gtk::test]
fn navigation_pane_rows_show_a_tooltip_only_when_their_name_is_cut_short() {
    let fixture = Fixture::standard();
    let long = "A folder with a very long name that goes on past the edge of the pane";
    fs::create_dir(fixture.path(long)).expect("fixture folder");
    let test = TestWindow::open(&fixture.uri_of(long));
    test.activate("pin-folder", None);
    wait_until("the pin", || {
        test.window.sidebar().labels().contains(&long.to_owned())
    });
    wait_for_frames(&test.window, 3);

    let gallery = shown_tooltip(&row_named(&test, "Gallery"));
    let home = shown_tooltip(&row_named(&test, "Home"));
    let cut = shown_tooltip(&row_named(&test, long));
    let with_static_tooltips: Vec<String> = descendants::<gtk::Widget>(test.window.sidebar().list())
        .into_iter()
        .filter_map(|widget| {
            widget
                .tooltip_text()
                .map(|text| format!("{}: {text}", widget.css_name()))
        })
        .collect();

    assert_eq!(gallery, None);
    assert_eq!(home, None);
    assert_eq!(cut.as_deref(), Some(long));
    assert!(
        with_static_tooltips.is_empty(),
        "no row, chevron or glyph carries a fixed tooltip: {with_static_tooltips:?}"
    );
}
