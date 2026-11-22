// SPDX-License-Identifier: AGPL-3.0-only
//! The menus made for one opening and unparented once they closed (a
//! place's, a Home item's, the address bar's, a crumb's, the Details
//! pane's options, This PC's group heading's, the status bar's and Back's
//! and Forward's history) crashed the app: when a pointer event reached a closed
//! menu's hidden popup before the menu was unparented, GTK 4.14 kept the
//! pointer's focus on a row of it, and the window's next layout asked the
//! unrealized popup for the pointer (`gdk_surface_request_motion` on
//! NULL, `gtk_window_native_layout`, a SIGSEGV in a full test run). Each
//! of these menus is now kept by its owner and shown again, as the file
//! list's is, so its popup lives as long as the owner.

use gtk::prelude::*;
use ox_core::location::HOME_URI;
use ox_core::settings::ContextMenu;

use super::command_bar_switch::Session;
use super::context_menus::choose_menu_style;
use super::menu_sweep::{close, open_menu};
use crate::test_support::harness::{descendants, wait_for, wait_for_frames, wait_until, Fixture};
use crate::test_support::pointer::Button;
use crate::window::menu_popover::MenuPopover;

/// Right-clicks the middle of `widget` with the real pointer and returns
/// the menu that opens.
fn right_click(session: &mut Session, widget: &impl IsA<gtk::Widget>, what: &str) -> MenuPopover {
    let at = session.middle_of(widget);
    let window = session.test.window.clone();
    for _ in 0..2 {
        session.pointer.move_to(at.0, at.1);
        session.pointer.press(Button::Secondary);
        session.pointer.release(Button::Secondary);
        if let Some(menu) = open_menu(&window) {
            return menu;
        }
    }
    panic!("a right-click on {what} opens a menu");
}

/// Closes `menu` with a real Escape, with the pointer resting on its
/// first row, as a person leaves it, and checks that the menu's popup
/// is kept: still in the window and realized, once the closing has run.
fn close_keeps_the_popup(session: &mut Session, menu: &MenuPopover, what: &str) {
    if let Some(row) = menu.rows().into_iter().next() {
        let on_row = session.middle_of(&row);
        session.pointer.move_to(on_row.0, on_row.1);
    }
    close(session, menu);
    // The closing's idle work, and a layout of the window under the
    // pointer that rested on the menu.
    session.test.window.queue_resize();
    wait_for_frames(&session.test.window, 3);
    wait_for(std::time::Duration::from_millis(300));
    assert!(menu.parent().is_some(), "{what}: the closed menu is kept");
    assert!(menu.is_realized(), "{what}: its popup is kept");
}

/// Each menu that was made for one opening is kept by its owner after it
/// closes, and the next opening shows the same menu again.
///
/// parity: CMD-008, HOME-013, NAV-031, PROP-010
#[gtk::test]
fn closed_menus_keep_their_popup_and_open_again() {
    let fixture = Fixture::real_world();
    let report = fixture.uri_of("Quarterly report.pdf");
    let root = fixture.uri();
    let documents = fixture.uri_of("Documents");
    let mut session = Session::open_on(fixture, None);
    super::home_page::open_quick_access_on_home(&session.test, true);
    session.test.activate("add-to-favorites", Some(&report));
    choose_menu_style(&session.test, ContextMenu::Win11);
    session.test.activate("go-to", Some(HOME_URI));
    wait_until("Home", || {
        session.test.window.current_uri().as_deref() == Some(HOME_URI)
    });
    wait_for_frames(&session.test.window, 6);
    let pane = session.test.window.folder_pane().clone();

    // A Quick access tile's place menu.
    let tile = descendants::<gtk::FlowBoxChild>(&pane)
        .into_iter()
        .find(WidgetExt::is_drawable)
        .expect("a Quick access tile");
    let menu = right_click(&mut session, &tile, "a Quick access tile");
    close_keeps_the_popup(&mut session, &menu, "a place menu");
    let again = right_click(&mut session, &tile, "a Quick access tile");
    assert_eq!(again, menu, "the place menu is shown again");
    close(&mut session, &again);

    // A Recent or Favorites row's menu.
    let body = pane.landing().clone();
    wait_until("the favorite saved", || {
        !session
            .test
            .context
            .settings_data()
            .preferences
            .favorites
            .is_empty()
    });
    session.test.window.render_landing();
    crate::window::home_page::choose_tab(&body, "Favorites");
    wait_for_frames(&session.test.window, 4);
    let row = crate::window::home_page::row_cell_widgets(&body, "Quarterly report")
        .into_iter()
        .next()
        .expect("the Favorites row");
    let menu = right_click(&mut session, &row, "a Favorites row");
    close_keeps_the_popup(&mut session, &menu, "a Home item's menu");
    let again = right_click(&mut session, &row, "a Favorites row");
    assert_eq!(again, menu, "the Home item's menu is shown again");
    close(&mut session, &again);

    // The address bar's menu.
    let address = session.test.window.address_bar().clone();
    let menu = right_click(&mut session, &address, "the address bar");
    close_keeps_the_popup(&mut session, &menu, "the address bar's menu");
    let again = right_click(&mut session, &address, "the address bar");
    assert_eq!(again, menu, "the address bar's menu is shown again");
    close(&mut session, &again);

    // The Details pane's options.
    session.test.open_details_pane();
    let details = session.test.window.details_pane().clone();
    let menu = details.show_options_menu(1.0, 1.0);
    wait_until("the options", || menu.is_mapped());
    close_keeps_the_popup(&mut session, &menu, "the Details pane's options");
    assert_eq!(
        details.show_options_menu(1.0, 1.0),
        menu,
        "the options are shown again"
    );
    menu.popdown();
    wait_until("the options to close", || !menu.is_mapped());

    // This PC's group heading menu.
    session.test.activate("go-to", Some(ox_core::location::PC_URI));
    wait_until("This PC", || {
        session.test.window.current_uri().as_deref() == Some(ox_core::location::PC_URI)
    });
    wait_for_frames(&session.test.window, 6);
    let header = crate::window::landing::group_headers(pane.landing())
        .into_iter()
        .next()
        .expect("This PC heads its groups");
    let menu = right_click(&mut session, &header, "This PC's group heading");
    close_keeps_the_popup(&mut session, &menu, "a group heading's menu");
    let again = right_click(&mut session, &header, "This PC's group heading");
    assert_eq!(again, menu, "the heading's menu is shown again");
    close(&mut session, &again);

    // The status bar's menu.
    session.test.activate("go-to", Some(&root));
    session.test.wait_for_listing("the folder");
    if let Some(menu) = session.test.window.show_status_bar_menu(4.0, 4.0) {
        wait_until("the status bar's menu", || menu.is_mapped());
        close_keeps_the_popup(&mut session, &menu, "the status bar's menu");
        assert_eq!(
            session.test.window.show_status_bar_menu(4.0, 4.0).as_ref(),
            Some(&menu),
            "the status bar's menu is shown again"
        );
        menu.popdown();
    }

    // Back's history menu.
    session.test.window.navigate(&documents).expect("a folder");
    session.test.wait_for_listing("Documents");
    let back = super::history::navigation_button(&session.test, "back");
    let menu = session
        .test
        .window
        .popup_history_menu(back.upcast_ref(), crate::window::session::Direction::Backward)
        .expect("Back has a menu");
    wait_until("the history menu", || menu.is_mapped());
    close_keeps_the_popup(&mut session, &menu, "Back's history menu");
    let again = session
        .test
        .window
        .popup_history_menu(back.upcast_ref(), crate::window::session::Direction::Backward)
        .expect("Back has a menu");
    assert_eq!(again, menu, "Back's menu is shown again");
    again.popdown();
}
