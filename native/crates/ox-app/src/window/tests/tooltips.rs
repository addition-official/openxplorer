// SPDX-License-Identifier: AGPL-3.0-only
//! Tooltips only where Windows 11 File Explorer shows them (compared side
//! by side with Windows 11 26200): a navigation pane row shows one only
//! while its name is cut short, with the full name; its chevrons and
//! glyphs show none.

use std::fs;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

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

/// The command bar's Details button shows Windows 11's tooltip, "Show or
/// hide the details pane." (confirmed in Windows 11 26200).
///
/// parity: PROP-001, ACC-001
#[gtk::test]
fn the_details_button_says_it_shows_or_hides_the_details_pane() {
    let test = TestWindow::open(&Fixture::standard().uri());
    let button = descendants::<gtk::ToggleButton>(test.window.command_bar())
        .into_iter()
        .find(|button| button.has_css_class("details-toggle"))
        .expect("the command bar has its Details button");

    assert_eq!(
        button.tooltip_text().as_deref(),
        Some("Show or hide the details pane.")
    );
}

/// The folder tree's rows carry no fixed tooltip (they used to show the
/// full path), as Windows' navigation pane shows one only while a name
/// is cut short.
///
/// parity: SIDE-028
#[gtk::test]
fn folder_tree_rows_carry_no_fixed_tooltip() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    test.window.imp().locations.borrow_mut().home = fixture.root().parent().map(Into::into);
    test.activate("folder-tree", None);
    test.activate("folder-tree-option", Some("expand-to-current"));
    let tree = test.window.folder_tree();
    wait_until("the tree", || tree.outline().len() > 1);
    wait_for_frames(&test.window, 3);

    let with_tooltips: Vec<String> = descendants::<gtk::Widget>(tree)
        .into_iter()
        .filter_map(|widget| widget.tooltip_text().map(|text| text.to_string()))
        .collect();

    assert!(with_tooltips.is_empty(), "{with_tooltips:?}");
}

/// The search box and the Settings page's Back button carry none of the
/// app's own explanations as tooltips: Windows' search box has none.
///
/// parity: SRCH-035, SET-001
#[gtk::test]
fn the_search_box_and_settings_back_button_have_no_tooltip() {
    let test = TestWindow::open(&Fixture::standard().uri());
    test.activate("settings", None);
    let back = descendants::<gtk::Button>(&test.window)
        .into_iter()
        .find(|button| button.has_css_class("settings-back"))
        .expect("the Settings page's Back button");

    assert_eq!(test.window.search_box().entry().tooltip_text(), None);
    assert_eq!(back.tooltip_text(), None);
}

/// A window closed, or hidden, while the real pointer rests on a row of
/// the navigation pane whose name is cut short (its tooltip about to
/// show) and the hidden chevrons fade in, logs no critical message: no
/// tooltip timer or pointer query is left for a window that is gone
/// (`Gdk-CRITICAL gdk_surface_get_device_position` killed the test
/// process once). A user closing a window from the keyboard with the
/// pointer over the pane does the same.
///
/// parity: SIDE-032, ACC-001
#[gtk::test]
fn closing_a_window_with_the_pointer_over_the_pane_logs_no_critical() {
    use ox_core::settings::{PreferencesUpdate, Settings};

    use super::right_drag::PROBE;
    use crate::test_support::harness::wait_for;
    use crate::test_support::pointer::{screen_point, Pointer};

    let recorder = crate::test_support::gtk_log::LogRecorder::start();
    let fixture = Fixture::standard();
    let long = "A folder with a very long name that goes on past the edge of the pane";
    fs::create_dir(fixture.path(long)).expect("fixture folder");
    let mut pointer = Pointer::connect();
    for hide in [false, true] {
        let test = TestWindow::open(&fixture.uri_of(long));
        Settings::open(test.settings_directory())
            .update_preferences(&PreferencesUpdate {
                hide_expand_arrows: Some(true),
                ..PreferencesUpdate::default()
            })
            .expect("the settings file takes the change");
        test.context.reload_settings();
        test.activate("pin-folder", None);
        wait_until("the pin", || {
            test.window.sidebar().labels().contains(&long.to_owned())
        });
        test.window.present();
        wait_for_frames(&test.window, 3);
        let sidebar = test.window.sidebar();
        let row = row_named(&test, long);
        let on_row = screen_point(
            &mut pointer,
            sidebar.list(),
            (120.0, sidebar.middle_of(long)),
            PROBE,
        );
        let on_list = screen_point(&mut pointer, test.window.folder_pane(), (300.0, 200.0), PROBE);
        pointer.glide(on_list, on_row, 10);
        // Within GTK's tooltip delay, while the chevrons fade in.
        wait_for(std::time::Duration::from_millis(100));
        assert!(shown_tooltip(&row).is_some(), "the row's name is cut short");
        if hide {
            test.window.set_visible(false);
        } else {
            test.window.close();
        }
        drop(test);
        // Longer than GTK's tooltip delay and the fade.
        wait_for(std::time::Duration::from_millis(1200));
    }
    // Off every window, past the double-click time.
    pointer.move_to(1430, 990);
    wait_for(std::time::Duration::from_millis(700));
    let criticals: Vec<String> = recorder
        .warnings()
        .into_iter()
        .filter(|warning| warning.contains("CRITICAL") || warning.contains("gdk_surface_get_device_position"))
        .collect();
    assert!(criticals.is_empty(), "{criticals:?}");
}

/// A menu opened and closed under the resting pointer, the navigation
/// pane turned back on and the window destroyed at once, as
/// `the_navigation_pane_hides_and_a_places_button_lists_its_places` does,
/// leave nothing for GTK's tooltip timer. Closing the Places menu used to
/// leave the window's keyboard focus on its first row, and with the
/// Places button then hidden GTK 4.14 kept the menu alive after its
/// window; GTK's tooltip, pointed at the menu by the pointer's crossing
/// into it, then asked for the pointer's position on a menu with no
/// surface (`Gdk-CRITICAL gdk_surface_get_device_position`, SIGTRAP in the
/// deb 61 and 62 test runs).
///
/// parity: SIDE-024
#[gtk::test]
fn a_window_destroyed_right_after_a_menu_closed_under_the_pointer_logs_no_critical() {
    use super::right_drag::PROBE;
    use crate::test_support::harness::wait_for;
    use crate::test_support::pointer::{screen_point, Pointer};

    let recorder = crate::test_support::gtk_log::LogRecorder::start();
    let fixture = Fixture::standard();
    let mut pointer = Pointer::connect();
    let mut freed = Vec::new();
    for round in 0..2 {
        let test = TestWindow::open(&fixture.uri());
        test.window.present();
        test.activate("sidebar", None);
        wait_for_frames(&test.window, 3);
        let places = test.window.places_button().clone();
        let menu = places.popover().expect("the places menu");
        if round == 0 {
            // Where the menu opens: the pointer rests there, and the
            // second round opens the menu right under it.
            places.popup();
            wait_for_frames(&menu, 3);
            let middle = (f64::from(menu.width()) / 2.0, f64::from(menu.height()) / 2.0);
            let over = screen_point(&mut pointer, &menu, middle, PROBE);
            pointer.move_to(over.0, over.1);
            wait_for(std::time::Duration::from_millis(50));
        } else {
            places.popup();
        }
        places.popdown();
        test.activate("sidebar", None);
        let left = menu.downgrade();
        drop(menu);
        drop(places);
        drop(test);
        // Longer than GTK's tooltip delay.
        wait_for(std::time::Duration::from_millis(1200));
        freed.push(left.upgrade().is_none());
    }
    // Off every window, past the double-click time.
    pointer.move_to(1430, 990);
    wait_for(std::time::Duration::from_millis(700));

    let criticals: Vec<String> = recorder
        .warnings()
        .into_iter()
        .filter(|warning| warning.contains("gdk_surface_get_device_position"))
        .collect();
    assert!(criticals.is_empty(), "{criticals:?}");
    assert_eq!(freed, [true, true], "the menu goes with its window");
}
