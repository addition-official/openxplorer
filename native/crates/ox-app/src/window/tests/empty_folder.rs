// SPDX-License-Identifier: AGPL-3.0-only
//! An empty folder in a real window: its "This folder is empty" page takes
//! the folder pane's right-click menu, keys and drops, as an empty folder
//! does in Windows Explorer, so New and Paste work there.

use gtk::glib::translate::IntoGlib;
use gtk::prelude::*;
use gtk::{gdk, glib};
use ox_core::settings::ContextMenu;

use super::context_menus::choose_menu_style;
use super::file_ops_support::{press_shortcut, select_names};
use crate::test_support::harness::{descendants, wait_until, Fixture, TestWindow};
use crate::window::folder_pane::PanePage;
use crate::window::menu_popover::MenuPopover;

/// Makes the empty folder "Empty" in `fixture`.
fn make_empty_folder(fixture: &Fixture) -> String {
    std::fs::create_dir(fixture.path("Empty")).expect("the empty folder is made");
    fixture.uri_of("Empty")
}

/// Goes to `uri` in `test`'s tab and returns the empty page's area, the
/// whole of the pane, once it shows.
pub(in crate::window) fn go_to_empty_folder(test: &TestWindow, uri: &str) -> gtk::Widget {
    test.activate("go-to", Some(uri));
    wait_until("the empty page", || {
        !test.window.is_loading() && test.window.folder_pane().page() == Some(PanePage::Empty)
    });
    shown_empty_page(test)
}

/// The area of the empty page that shows now.
pub(in crate::window) fn shown_empty_page(test: &TestWindow) -> gtk::Widget {
    let state = descendants::<gtk::Box>(&test.window)
        .into_iter()
        .find(|page| page.has_css_class("empty-state") && page.is_mapped())
        .expect("the empty page shows");
    state.parent().expect("the empty page's area")
}

/// The context menu of `page`, if it has one.
fn menu_of(page: &gtk::Widget) -> Option<MenuPopover> {
    descendants::<MenuPopover>(page).into_iter().next()
}

/// The controllers of `widget` of type `T`.
fn controllers<T: IsA<glib::Object>>(widget: &gtk::Widget) -> Vec<T> {
    widget
        .observe_controllers()
        .iter::<glib::Object>()
        .filter_map(Result::ok)
        .filter_map(|controller| controller.downcast::<T>().ok())
        .collect()
}

/// A right-click on an empty folder opens the folder menu there, and
/// Paste in it pastes the copied file into the folder.
///
/// parity: CMD-011
#[gtk::test]
fn right_clicking_an_empty_folder_opens_its_menu_and_paste_works() {
    let fixture = Fixture::standard();
    let empty = make_empty_folder(&fixture);
    let test = TestWindow::open(&fixture.uri());
    choose_menu_style(&test, ContextMenu::Win10);
    select_names(&test, &["Notes 2.txt"]);
    test.activate("copy", None);
    let page = go_to_empty_folder(&test, &empty);

    let right_click = controllers::<gtk::GestureClick>(&page)
        .into_iter()
        .find(|gesture| gesture.button() == gdk::BUTTON_SECONDARY)
        .expect("the empty page takes a right-click");
    right_click.emit_by_name::<()>("pressed", &[&1_i32, &20.0_f64, &20.0_f64]);
    let menu = menu_of(&page).expect("the empty page has the folder menu");
    wait_until("the menu", || menu.is_visible());

    assert!(menu.row("New…").is_sensitive());
    assert!(menu.row("Paste").is_sensitive());
    menu.row("Paste").emit_activate();
    wait_until("the pasted copy", || fixture.path("Empty/Notes 2.txt").is_file());
}

/// The keyboard reaches an empty folder: focus follows from the list to
/// its page, the Menu key opens the folder menu there, and Backspace goes
/// back.
///
/// parity: CMD-011, NAV-004
#[gtk::test]
fn an_empty_folder_takes_the_folder_keys() {
    let fixture = Fixture::standard();
    let empty = make_empty_folder(&fixture);
    let test = TestWindow::open(&fixture.uri());
    test.window.folder_pane().focus_view();
    let page = go_to_empty_folder(&test, &empty);

    let focus = gtk::prelude::GtkWindowExt::focus(&test.window).expect("something has focus");
    assert!(
        focus == page || focus.is_ancestor(&page),
        "focus is on the empty page, not {focus:?}"
    );
    test.window.open_context_menu_from_keyboard();
    let menu = menu_of(&page).expect("the empty page has the folder menu");
    wait_until("the menu", || menu.is_visible());
    menu.popdown();

    let keys = controllers::<gtk::EventControllerKey>(&page)
        .into_iter()
        .next()
        .expect("the empty page takes the folder keys");
    let _: bool = keys.emit_by_name(
        "key-pressed",
        &[
            &gdk::Key::BackSpace.into_glib(),
            &0_u32,
            &gdk::ModifierType::empty(),
        ],
    );
    wait_until("Back", || {
        test.window.current_uri().as_deref() == Some(fixture.uri().as_str())
    });
}

/// Ctrl+V pastes into an empty folder.
///
/// parity: CMD-011
#[gtk::test]
fn ctrl_v_pastes_into_an_empty_folder() {
    let fixture = Fixture::standard();
    let empty = make_empty_folder(&fixture);
    let test = TestWindow::open(&fixture.uri());
    select_names(&test, &["Notes 2.txt"]);
    test.activate("copy", None);
    go_to_empty_folder(&test, &empty);

    press_shortcut(&test, gdk::Key::v, gdk::ModifierType::CONTROL_MASK);

    wait_until("the pasted copy", || fixture.path("Empty/Notes 2.txt").is_file());
}
