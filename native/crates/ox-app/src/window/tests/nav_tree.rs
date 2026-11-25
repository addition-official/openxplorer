// SPDX-License-Identifier: AGPL-3.0-only
//! The navigation pane's drives and folders as a tree, as Windows 11
//! draws it (compared side by side with Windows 11 26200): a drive, a
//! share or a folder with folders has a chevron that lists them under
//! it, read off the main loop; the chevron, Right and Left, and a
//! double-click open and close it; with "Hide expand arrows" on, the
//! chevrons show only while the pointer is over the pane.

use std::fs;
use std::time::Duration;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use ox_core::location::file_uri;
use ox_core::settings::{PreferencesUpdate, Settings};

use super::right_drag::PROBE;
use crate::test_support::harness::{descendants, wait_for, wait_for_frames, wait_until, Fixture, TestWindow};
use crate::test_support::pointer::{screen_point, Button, Pointer};
use crate::volumes::{MountControls, VolumeKind, VolumeRow, VolumeState};
use crate::window::sidebar::entries::RowLevel;

/// A USB stick mounted at `uri`, as the volume monitor would list it,
/// and the places drawn again.
fn stick_at(test: &TestWindow, uri: &str) {
    test.window.imp().volumes.replace(vec![VolumeRow {
        label: "STICK".to_owned(),
        kind: VolumeKind::Drive,
        state: VolumeState::Mounted {
            uri: uri.to_owned(),
            controls: MountControls {
                can_unmount: true,
                can_eject: true,
                can_stop: true,
                can_open_in_disks: false,
                can_format: false,
                is_optical: false,
            },
        },
    }]);
    test.window.render_places();
}

/// A stick with the folders Alpha (holding Inner), Beta (empty) and a
/// hidden one, and two files, as the fixture's "Example projects".
fn stick_fixture() -> Fixture {
    let fixture = Fixture::empty();
    fs::create_dir_all(fixture.path("Alpha/Inner")).expect("fixture folders");
    fs::create_dir(fixture.path("Beta")).expect("fixture folder");
    fs::create_dir(fixture.path(".cache")).expect("hidden fixture folder");
    fixture.write("notes.txt");
    fixture.write("song.mp3");
    fixture
}

/// The labels of the rows shown after `parent`, each with its level,
/// down to the next row at `parent`'s level or above.
fn rows_under(test: &TestWindow, parent: &str) -> Vec<(String, RowLevel, bool)> {
    let sidebar = test.window.sidebar();
    let entries = sidebar.entries_for_tests();
    let Some(at) = entries.iter().position(|entry| entry.label == parent) else {
        return Vec::new();
    };
    let depth = entries[at].level.depth();
    entries[at + 1..]
        .iter()
        .take_while(|entry| entry.level.depth() > depth)
        .map(|entry| (entry.label.clone(), entry.level, entry.expander.is_some()))
        .collect()
}

/// The chevron of the row labelled `label`.
fn chevron_of(test: &TestWindow, label: &str) -> Option<gtk::Button> {
    let sidebar = test.window.sidebar();
    let index = sidebar.labels().iter().position(|shown| shown == label)?;
    let row = sidebar.list().row_at_index(i32::try_from(index).ok()?)?;
    descendants::<gtk::Button>(&row)
        .into_iter()
        .find(|button| button.has_css_class("side-expander"))
}

/// Clicks the middle of `widget` with the real pointer.
fn click(pointer: &mut Pointer, widget: &impl IsA<gtk::Widget>) {
    let bounds = widget.compute_bounds(widget).expect("a shown widget");
    let middle = (f64::from(bounds.width() / 2.0), f64::from(bounds.height() / 2.0));
    let on = screen_point(pointer, widget, middle, PROBE);
    pointer.move_to(on.0, on.1);
    pointer.press(Button::Primary);
    pointer.release(Button::Primary);
}

/// Moves the pointer off the window and waits past the double-click time
/// before the test's window is destroyed (see `nav_pane.rs`'s `let_go`).
fn let_go(pointer: &mut Pointer) {
    pointer.move_to(1430, 990);
    wait_for(Duration::from_millis(700));
}

/// A real click on a drive's chevron lists its folders under it, one
/// level in, without files or hidden folders, each with a chevron only
/// when it holds folders; a click on a folder's chevron lists its own a
/// level further in, and a click on the drive's chevron hides them all.
/// No click opens a folder.
///
/// parity: SIDE-033
#[gtk::test]
fn a_real_click_on_a_drive_s_chevron_lists_its_folder_tree() {
    let stick = stick_fixture();
    let shown = Fixture::standard();
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&shown.uri());
    stick_at(&test, &stick.uri());
    wait_for_frames(&test.window, 3);
    let collapsed = rows_under(&test, "STICK");

    click(
        &mut pointer,
        &chevron_of(&test, "STICK").expect("the drive has a chevron"),
    );
    wait_until("the folders with their chevrons", || {
        rows_under(&test, "STICK")
            == [
                ("Alpha".to_owned(), RowLevel::Grandchild, true),
                ("Beta".to_owned(), RowLevel::Grandchild, false),
            ]
    });
    wait_for_frames(&test.window, 2);
    click(
        &mut pointer,
        &chevron_of(&test, "Alpha").expect("Alpha has a chevron"),
    );
    wait_until("Alpha's folder", || {
        rows_under(&test, "Alpha") == [("Inner".to_owned(), RowLevel::Nested(3), false)]
    });
    let indent = |label: &str| {
        let sidebar = test.window.sidebar();
        let index = sidebar
            .labels()
            .iter()
            .position(|shown| shown == label)
            .expect("a row");
        let row = sidebar
            .list()
            .row_at_index(i32::try_from(index).expect("small"))
            .expect("a row");
        let icon = descendants::<crate::icons::ArtImage>(&row)
            .into_iter()
            .next()
            .expect("an icon");
        icon.compute_bounds(sidebar.list()).expect("shown").x()
    };
    wait_for_frames(&test.window, 2);
    let steps = (
        indent("Alpha") - indent("STICK"),
        indent("Inner") - indent("Alpha"),
    );
    let still_shown = test.window.current_uri();
    wait_for(Duration::from_millis(700));
    click(
        &mut pointer,
        &chevron_of(&test, "STICK").expect("the drive's chevron"),
    );
    wait_until("the tree to close", || rows_under(&test, "STICK").is_empty());

    assert!(collapsed.is_empty(), "closed at first: {collapsed:?}");
    assert_eq!(steps, (16.0, 16.0), "each level 16 pixels further in");
    assert_eq!(still_shown, Some(shown.uri()), "a chevron opens nothing");
    let_go(&mut pointer);
}

/// Right on a drive's row lists its folders, Right again moves to the
/// first one, Left moves back to the drive and Left again closes it, as
/// in Windows' navigation pane.
///
/// parity: SIDE-033
#[gtk::test]
fn right_and_left_open_and_close_a_drive() {
    let stick = stick_fixture();
    let shown = Fixture::standard();
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&shown.uri());
    stick_at(&test, &stick.uri());
    let sidebar = test.window.sidebar();
    sidebar.focus_row_for_tests("STICK");
    wait_for_frames(&test.window, 2);
    let focused = || {
        let row = sidebar.list().focus_child()?.downcast::<gtk::ListBoxRow>().ok()?;
        sidebar.labels().get(usize::try_from(row.index()).ok()?).cloned()
    };

    pointer.tap_key(gtk::gdk::Key::Right);
    wait_until("the folders", || !rows_under(&test, "STICK").is_empty());
    let after_right = focused();
    pointer.tap_key(gtk::gdk::Key::Right);
    wait_until("the first folder", || focused().as_deref() == Some("Alpha"));
    pointer.tap_key(gtk::gdk::Key::Left);
    wait_until("back on the drive", || focused().as_deref() == Some("STICK"));
    pointer.tap_key(gtk::gdk::Key::Left);
    wait_until("the tree to close", || rows_under(&test, "STICK").is_empty());

    assert_eq!(
        after_right.as_deref(),
        Some("STICK"),
        "the keyboard stays on the drive"
    );
    assert_eq!(test.window.current_uri(), Some(shown.uri()));
    let_go(&mut pointer);
}

/// A real double-click on a drive's row opens it and lists its folders,
/// as in Windows' navigation pane.
///
/// parity: SIDE-033
#[gtk::test]
fn a_real_double_click_on_a_drive_opens_it_and_lists_its_folders() {
    let stick = stick_fixture();
    let shown = Fixture::standard();
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&shown.uri());
    stick_at(&test, &stick.uri());
    wait_for_frames(&test.window, 3);
    let sidebar = test.window.sidebar();
    let on = screen_point(
        &mut pointer,
        sidebar.list(),
        (110.0, sidebar.middle_of("STICK")),
        PROBE,
    );
    pointer.move_to(on.0, on.1);
    for _ in 0..2 {
        pointer.press(Button::Primary);
        pointer.release(Button::Primary);
    }

    wait_until("the folders", || !rows_under(&test, "STICK").is_empty());
    wait_until("the drive opened", || {
        test.window.current_uri() == Some(stick.uri())
    });
    let_go(&mut pointer);
}

/// A drive with thousands of folders lists them without holding up the
/// window: the first 1,000, then a row that opens the drive to show the
/// rest.
///
/// parity: SIDE-033, PERF-001
#[gtk::test]
fn a_drive_with_thousands_of_folders_lists_the_first_thousand() {
    let stick = Fixture::empty();
    for number in 0..1500 {
        fs::create_dir(stick.path(&format!("Folder {number:04}"))).expect("fixture folder");
    }
    let shown = Fixture::standard();
    let test = TestWindow::open(&shown.uri());
    stick_at(&test, &stick.uri());

    test.activate("toggle-folder", Some(&stick.uri()));
    wait_until("the folders", || rows_under(&test, "STICK").len() > 1);
    let rows = rows_under(&test, "STICK");

    assert_eq!(rows.len(), 1001, "1,000 folders and the row for the rest");
    assert_eq!(rows[0].0, "Folder 0000");
    assert_eq!(rows[1000].0, "More folders in STICK…");
}

/// With "Expand to current folder" on, the tree opens down to the folder
/// the tab shows and highlights it; off (the default), it stays closed.
///
/// parity: SIDE-028, SIDE-033
#[gtk::test]
fn the_tree_opens_to_the_current_folder_only_with_the_option_on() {
    let stick = stick_fixture();
    let test = TestWindow::open(&Fixture::standard().uri());
    stick_at(&test, &stick.uri());
    let inner = file_uri(&stick.path("Alpha/Inner"));

    test.activate("go-to", Some(&inner));
    test.wait_for_listing("Inner");
    wait_for(Duration::from_millis(500));
    let closed = rows_under(&test, "STICK");
    test.activate("go-to", Some(&Fixture::standard().uri()));
    test.activate("folder-tree-option", Some("expand-to-current"));
    test.activate("go-to", Some(&inner));
    wait_until("the tree down to Inner", || {
        rows_under(&test, "Alpha")
            .iter()
            .any(|(label, _, _)| label == "Inner")
    });
    wait_for_frames(&test.window, 2);
    let selected = test.window.sidebar().list().selected_row().map(|row| row.index());
    let inner_row = test
        .window
        .sidebar()
        .labels()
        .iter()
        .position(|label| label == "Inner")
        .and_then(|index| i32::try_from(index).ok());

    assert!(closed.is_empty(), "off by default: {closed:?}");
    assert_eq!(selected, inner_row, "the folder shown is highlighted");
}

/// With "Hide expand arrows" on, the drives' and folders' chevrons draw
/// nothing while the pointer is outside the pane, show once the real
/// pointer is over it, and still expand a drive when clicked.
///
/// parity: SIDE-032, SIDE-033
#[gtk::test]
fn hidden_chevrons_show_while_the_real_pointer_is_over_the_pane() {
    let stick = stick_fixture();
    let shown = Fixture::standard();
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&shown.uri());
    stick_at(&test, &stick.uri());
    Settings::open(test.settings_directory())
        .update_preferences(&PreferencesUpdate {
            hide_expand_arrows: Some(true),
            ..PreferencesUpdate::default()
        })
        .expect("the settings file takes the change");
    test.context.reload_settings();
    test.window.folder_pane().grab_focus();
    let draws = |widget: &gtk::Button| {
        let picture = gtk::WidgetPaintable::new(Some(widget));
        let snapshot = gtk::Snapshot::new();
        picture.snapshot(&snapshot, f64::from(widget.width()), f64::from(widget.height()));
        snapshot.to_node().is_some()
    };
    let chevron = || chevron_of(&test, "STICK").expect("the drive's chevron");
    let off_pane = screen_point(&mut pointer, test.window.folder_pane(), (400.0, 300.0), PROBE);
    pointer.move_to(off_pane.0, off_pane.1);
    wait_until("the chevron to hide", || !draws(&chevron()));

    let over = screen_point(&mut pointer, test.window.sidebar().list(), (110.0, 20.0), PROBE);
    pointer.move_to(over.0, over.1);
    wait_until("the chevron to show under the pointer", || draws(&chevron()));
    click(&mut pointer, &chevron());
    wait_until("the folders", || !rows_under(&test, "STICK").is_empty());
    let_go(&mut pointer);
}
