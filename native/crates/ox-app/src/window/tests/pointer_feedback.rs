// SPDX-License-Identifier: AGPL-3.0-only
//! What a real pointer on check.py's private display sees while it drags:
//! the label on the drag icon that says what a release would do (DND-028),
//! and the rubber band drawn from the empty part of a details row
//! (SEL-012). GTK's own drag source, drop targets and gestures run, so
//! these catch what calling the handlers directly cannot, such as which
//! gesture takes a press on a row.

use gtk::gdk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::right_drag::{blank_on_screen, item_on_screen, HOLD, PROBE};
use crate::test_support::harness::{wait_for, wait_for_frames, wait_until, Fixture, TestWindow};
use crate::test_support::pointer::{screen_point, Button, Pointer};
use crate::window::file_drag::{shown_drag_glyph, shown_drag_label};

/// A folder whose rows fill the details view.
const FULL_FOLDER: usize = 120;

/// A real drag of a file over a folder of its drive says "Move to" that
/// folder, "Copy to" it while Ctrl is held, and nothing over blank space of
/// its own folder, where a release does nothing, as in Windows 11.
///
/// parity: DND-028
#[gtk::test]
fn a_real_drag_says_what_a_release_would_do_and_follows_the_keys() {
    let fixture = Fixture::standard();
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&fixture.uri());
    wait_for_frames(&test.window, 3);
    let from = item_on_screen(&mut pointer, &test, "Notes 2.txt");
    let onto = item_on_screen(&mut pointer, &test, "Documents");
    let blank = blank_on_screen(&mut pointer, &test);

    pointer.move_to(from.0, from.1);
    pointer.press(Button::Primary);
    pointer.glide(from, (from.0 + 12, from.1 + 12), 4);
    pointer.glide((from.0 + 12, from.1 + 12), onto, 12);
    wait_until("the move label", || {
        shown_drag_label().as_deref() == Some("Move to Documents")
    });
    let move_glyph = shown_drag_glyph();
    pointer.hold_key(gdk::Key::Control_L);
    pointer.glide(onto, (onto.0 + 4, onto.1), 2);
    wait_until("the copy label", || {
        shown_drag_label().as_deref() == Some("Copy to Documents")
    });
    let copy_glyph = shown_drag_glyph();
    pointer.let_go_key(gdk::Key::Control_L);
    pointer.glide((onto.0 + 4, onto.1), blank, 12);
    wait_for(HOLD);
    let over_own_folder = shown_drag_label();
    pointer.release(Button::Primary);
    wait_until("the drag to end", || {
        test.window.imp().outgoing_drag.borrow().is_none()
    });

    assert_eq!(move_glyph.as_deref(), Some("ox-arrow-right-20-symbolic"));
    assert_eq!(copy_glyph.as_deref(), Some("ox-add-20-symbolic"));
    assert_eq!(over_own_folder, None, "a release in its own folder does nothing");
    assert!(fixture.path("Notes 2.txt").is_file(), "nothing moved");
}

/// The screen point of the Size column of the row at `position`, past the
/// end of its name.
fn beside_name_on_screen(pointer: &mut Pointer, test: &TestWindow, position: u32) -> (i32, i32) {
    let view = test.window.folder_pane().input_widget();
    let cell = test
        .window
        .folder_pane()
        .owners()
        .file_cell_at(position, &view)
        .expect("the row is shown");
    let bounds = cell.compute_bounds(&view).expect("the row is laid out");
    // The empty end of the Name cell, the only part of a row that is empty
    // space in Windows 11 (test-combined-10.1).
    let column = bounds;
    let point = (
        f64::from(column.x() + column.width()) - 8.0,
        f64::from(bounds.y() + bounds.height() / 2.0),
    );
    screen_point(pointer, &view, point, PROBE)
}

/// In a folder whose rows fill the details view, a real click beside an
/// unselected row's name selects that row, as Windows 11's full-row select
/// does, and draws no rubber band (test-combined-10.3; upstream #226 took
/// the press for a band).
///
/// parity: SEL-002, SEL-012
#[gtk::test]
fn a_real_click_beside_a_details_name_selects_its_row() {
    let fixture = Fixture::with_files(FULL_FOLDER);
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&fixture.uri());
    wait_for_frames(&test.window, 3);
    let model = test.window.folder_model();
    model.select_only(0);
    let click_at = beside_name_on_screen(&mut pointer, &test, 3);

    pointer.move_to(click_at.0, click_at.1);
    pointer.press(Button::Primary);
    pointer.release(Button::Primary);
    wait_until("the clicked row to be selected", || {
        model.selected_positions() == [3]
    });
    wait_for(HOLD);
    assert!(
        !test.window.folder_pane().rubber_band_shown(),
        "no band is drawn for a click on a row"
    );
}
