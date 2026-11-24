// SPDX-License-Identifier: AGPL-3.0-only
//! System-frame layout, tab gestures, and drop geometry on a private display.

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use ox_core::settings::{PreferencesUpdate, Settings};

use crate::test_support::harness::{wait_for_frames, wait_until, Fixture, TestWindow};

/// Saves the opt-in before a window is constructed, as on the next launch.
fn open_system_window(fixture: &Fixture) -> TestWindow {
    TestWindow::open_prepared(&fixture.uri(), |context| {
        context.update_preferences(
            PreferencesUpdate {
                system_window_decorations: Some(true),
                ..PreferencesUpdate::default()
            },
            |result| result.expect("fixture settings are writable"),
        );
        wait_until("system decorations to be read", || {
            context.settings_data().preferences.system_window_decorations
        });
    })
}

#[gtk::test]
fn system_window_decorations_keep_tabs_in_content_without_duplicate_buttons() {
    let fixture = Fixture::standard();
    let test = open_system_window(&fixture);
    let imp = test.window.imp();
    assert_eq!(imp.content_box.first_child(), Some(imp.tab_bar.get().upcast()));
    assert!(!imp.start_caption_buttons.is_visible());
    assert!(!imp.end_caption_buttons.is_visible());
    assert!(test.window.is_decorated());
    assert!(!imp.tab_bar.has_css_class("titlebar"));
    let controllers = imp.tab_bar.observe_controllers();
    assert!(controllers
        .iter::<gtk::glib::Object>()
        .filter_map(Result::ok)
        .any(|controller| controller.is::<gtk::DragSource>()));
    assert!(
        Settings::open(test.settings_directory())
            .data()
            .preferences
            .system_window_decorations
    );
    test.activate("new-tab", None);
    assert_eq!(test.window.tab_count(), 2);
    test.activate("settings", None);
    assert!(test.window.shows_settings());
    assert!(imp.tab_bar.is_visible());
}

#[gtk::test]
fn system_window_decorations_drop_geometry_tracks_the_content_tab_row() {
    let fixture = Fixture::standard();
    let test = open_system_window(&fixture);
    wait_for_frames(&test.window, 3);
    let bar = test
        .window
        .imp()
        .tab_bar
        .compute_bounds(&test.window)
        .expect("mapped tab row");
    let strip = test
        .window
        .tab_strip()
        .compute_bounds(&test.window)
        .expect("mapped strip");
    let x = strip.x() + strip.width() / 2.0;
    assert!(test
        .window
        .is_in_tab_row(gtk::graphene::Point::new(x, bar.y() + bar.height() / 2.0)));
    assert!(!test
        .window
        .is_in_tab_row(gtk::graphene::Point::new(x, bar.y() - 1.0)));
    assert!(!test
        .window
        .is_in_tab_row(gtk::graphene::Point::new(x, bar.y() + bar.height() + 1.0)));
}

#[gtk::test]
fn system_window_decorations_preserve_real_pointer_tab_reordering() {
    use crate::test_support::pointer::{screen_point, Button, Pointer};
    use crate::window::widget_tree::children;

    let fixture = Fixture::standard();
    let test = open_system_window(&fixture);
    test.window
        .add_tab(&fixture.uri_of("Documents"))
        .expect("a fixture folder");
    test.wait_for_listing("the second tab");
    wait_for_frames(&test.window, 3);
    let ids: Vec<_> = test
        .window
        .imp()
        .session
        .borrow()
        .tabs()
        .iter()
        .map(|tab| tab.id)
        .collect();
    let tabs: Vec<_> = children(&test.window.tab_strip().tab_list()).collect();
    let mut pointer = Pointer::connect();
    let from = screen_point(&mut pointer, &tabs[0], (60.0, 18.0), (8, 8));
    let to = screen_point(&mut pointer, &tabs[1], (150.0, 18.0), (8, 8));
    pointer.move_to(from.0, from.1);
    pointer.press(Button::Primary);
    crate::test_support::harness::wait_for(std::time::Duration::from_millis(200));
    pointer.glide(from, to, 12);
    crate::test_support::harness::wait_for(std::time::Duration::from_millis(200));
    pointer.release(Button::Primary);
    wait_until("the system-frame tabs to reorder", || {
        test.window.imp().session.borrow().tabs()[0].id == ids[1]
    });
    assert_eq!(test.window.imp().session.borrow().tabs()[1].id, ids[0]);
}

/// Saves the opt-in as Settings or the title bar's menu does while
/// `test`'s window is open.
fn choose_system_decorations(test: &TestWindow, on: bool) {
    test.context.update_preferences(
        PreferencesUpdate {
            system_window_decorations: Some(on),
            ..PreferencesUpdate::default()
        },
        |result| result.expect("fixture settings are writable"),
    );
    wait_until("the choice to be read", || {
        test.context.settings_data().preferences.system_window_decorations == on
    });
}

/// Turning system decorations on or off applies to the windows already
/// open, at once, as Chrome's "Use system title bar and borders" does:
/// the tab row moves under the desktop's title bar and the app's caption
/// buttons go, and back. The window stays shown with its tabs, and a
/// maximized window stays maximized.
///
/// parity: LOOK-032
#[gtk::test]
fn an_open_window_follows_the_system_decorations_choice_at_once() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    test.window
        .add_tab(&fixture.uri_of("Documents"))
        .expect("a fixture folder");
    test.wait_for_listing("the second tab");
    let imp = test.window.imp();
    let custom_at_start = imp.content_box.first_child() != Some(imp.tab_bar.get().upcast());

    choose_system_decorations(&test, true);
    wait_until("the window to take the desktop's frame", || {
        test.window.uses_system_decorations() && test.window.is_visible()
    });
    wait_for_frames(&test.window, 3);
    let in_content = imp.content_box.first_child() == Some(imp.tab_bar.get().upcast());
    let titlebar_gone = test.window.titlebar().is_none();
    let captions_hidden = !imp.start_caption_buttons.is_visible() && !imp.end_caption_buttons.is_visible();
    let tabs_kept = test.window.tab_count();

    choose_system_decorations(&test, false);
    wait_until("the window to take its own frame again", || {
        !test.window.uses_system_decorations() && test.window.is_visible()
    });
    wait_for_frames(&test.window, 3);
    let back = test.window.titlebar() == Some(imp.tab_bar.get().upcast());

    assert!(custom_at_start);
    assert!(in_content, "the tab row is the first row of the content");
    assert!(titlebar_gone, "the desktop draws the title bar");
    assert!(captions_hidden);
    assert_eq!(tabs_kept, 2);
    assert!(back, "the tab row is the title bar again");
    assert!(imp.end_caption_buttons.is_visible());
    assert!(test.window.is_mapped());
}

/// A real right-click on the empty part of the title bar opens Windows'
/// window menu with the system title bar choice under it; a right-click
/// on a tab still opens the tab's menu.
///
/// parity: LOOK-032
#[gtk::test]
fn a_right_click_on_the_empty_title_bar_opens_the_window_menu() {
    use crate::test_support::pointer::{screen_point, Button, Pointer};
    use crate::window::widget_tree::children;

    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    test.window.present();
    wait_for_frames(&test.window, 3);
    let mut pointer = Pointer::connect();
    let drag_area = test.window.imp().title_drag.get();
    let (x, y) = screen_point(&mut pointer, &drag_area, (20.0, 10.0), (8, 8));
    pointer.move_to(x, y);
    pointer.tap(Button::Secondary);
    wait_until("the window menu", || {
        test.window.title_bar_menu().is_some_and(|menu| menu.is_visible())
    });
    let labels: Vec<String> = test
        .window
        .title_bar_menu()
        .expect("the window menu")
        .items()
        .into_iter()
        .map(|item| item.label)
        .collect();
    test.window.title_bar_menu().expect("the window menu").popdown();

    let tab = children(&test.window.tab_strip().tab_list())
        .next()
        .expect("a tab");
    let (x, y) = screen_point(&mut pointer, &tab, (40.0, 18.0), (8, 8));
    pointer.move_to(x, y);
    pointer.tap(Button::Secondary);
    wait_until("the tab's menu", || test.window.tab_strip().menu().is_visible());
    let window_menu_on_tab = test.window.title_bar_menu().is_some_and(|menu| menu.is_visible());
    test.window.tab_strip().menu().popdown();
    pointer.move_to(720, 500);

    assert_eq!(
        labels,
        [
            "Restore",
            "Move",
            "Size",
            "Minimize",
            "Maximize",
            "Close",
            "Use system title bar and borders"
        ]
    );
    assert!(!window_menu_on_tab, "a tab keeps its own menu");
}

/// The menu's check item turns the choice on, and every open window
/// follows it.
///
/// parity: LOOK-032
#[gtk::test]
fn the_window_menus_check_item_turns_system_decorations_on() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    test.activate("system-title-bar", None);
    wait_until("the window to take the desktop's frame", || {
        test.window.uses_system_decorations() && test.window.is_visible()
    });
    assert!(
        Settings::open(test.settings_directory())
            .data()
            .preferences
            .system_window_decorations
    );
    test.activate("system-title-bar", None);
    wait_until("the window to take its own frame again", || {
        !test.window.uses_system_decorations() && test.window.is_visible()
    });
}
