// SPDX-License-Identifier: AGPL-3.0-only
//! What higher contrast draws: what a user who asked the desktop for it
//! must be able to see.

use gtk::prelude::*;
use ox_core::settings::Theme;

use super::icons::{assert_same_colour, painted_colour, ContrastGuard, TRANSITION_TIME};
use crate::test_support::harness::{
    descendants, skin, wait_for, wait_for_frames, Fixture, TestWindow, ThemeGuard,
};

/// The drawn row of the Details view that is selected.
fn selected_row(test: &TestWindow) -> gtk::Widget {
    let details = test.window.folder_pane().details().column_view().clone();
    descendants::<gtk::Widget>(&details)
        .into_iter()
        .filter(|widget| {
            widget.css_name() == "row"
                && widget
                    .parent()
                    .is_some_and(|parent| parent.css_name() == "listview")
        })
        .find(|row| row.state_flags().contains(gtk::StateFlags::SELECTED))
        .expect("a selected row is drawn")
}

/// With higher contrast, a selected item is ringed in the text color, in
/// the light and the dark appearance, apart from the keyboard's ring:
/// Windows 11's selection fill alone is #3d3d3d on #202020 in dark (1.5 to
/// 1) and #cce8ff on white in light (1.2 to 1), and the item under the
/// pointer is a shade of the same, so the selection could not be told
/// from the background or from the item under the pointer.
///
/// parity: ACC-011
#[gtk::test]
fn high_contrast_rings_the_selection_in_the_text_color() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    test.activate("view", Some("details"));
    let _theme = ThemeGuard::keep();
    test.select_named("Notes 2.txt");
    // The keyboard elsewhere, so no focus ring is drawn on the row.
    test.window.search_box().grab_focus();
    let _contrast = ContrastGuard::raise();
    for theme in [Theme::Light, Theme::Dark] {
        skin().set_theme(theme);
        wait_for(TRANSITION_TIME);
        wait_for_frames(&test.window, 2);
        let row = selected_row(&test);
        let text = descendants::<gtk::Label>(&row)
            .into_iter()
            .find(|label| label.text() == "Notes 2")
            .expect("the row shows its name")
            .color();
        let middle = row.width() / 2;
        assert_same_colour(
            painted_colour(&row, middle, 0),
            text,
            &format!("{theme:?}: the top edge"),
        );
        let bottom = row.height() - 1;
        assert_same_colour(
            painted_colour(&row, middle, bottom),
            text,
            &format!("{theme:?}: the bottom edge"),
        );
    }
}
