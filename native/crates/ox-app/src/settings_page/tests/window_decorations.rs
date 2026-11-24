// SPDX-License-Identifier: AGPL-3.0-only
//! The desktop frame switch persists and applies to open and new windows.

use super::*;

#[gtk::test]
fn system_window_decorations_are_searchable_and_apply_to_open_and_new_windows() {
    let settings = SettingsTest::open();
    settings
        .page
        .open(Some(SettingsView::Category(Category::Appearance)));
    assert!(settings.shown_rows().contains(&"Use system window decorations"));
    settings.page.search("KWin");
    assert!(settings.shown_rows().contains(&"Use system window decorations"));
    let switch = switch_of(&settings.row("Use system window decorations"));
    assert!(!switch.is_active());
    let original_bar = settings
        .test
        .window
        .titlebar()
        .expect("custom title bar by default");

    switch.set_active(true);
    wait_until("system decorations to be saved", || {
        settings.saved_preferences().system_window_decorations
    });
    assert!(switch.is_active());
    // The open window takes the desktop's frame at once (LOOK-032).
    wait_until("the open window to follow", || {
        settings.test.window.titlebar().is_none() && settings.test.window.is_visible()
    });
    let system = settings.test.open_beside(&settings.fixture.uri());
    assert!(system.window.has_css_class("system-window-decorations"));
    assert!(!system
        .window
        .titlebar()
        .is_some_and(|bar| bar.is::<gtk::WindowHandle>()));

    switch.set_active(false);
    wait_until("custom decorations to be saved", || {
        !settings.saved_preferences().system_window_decorations
    });
    assert!(!switch.is_active());
    wait_until("both open windows to take their own frame again", || {
        !system.window.has_css_class("system-window-decorations")
            && settings.test.window.titlebar() == Some(original_bar.clone())
    });
    let custom = settings.test.open_beside(&settings.fixture.uri());
    assert!(custom
        .window
        .titlebar()
        .is_some_and(|bar| bar.is::<gtk::WindowHandle>()));
    assert_eq!(settings.test.window.titlebar(), Some(original_bar));
    assert!(settings.test.window.is_visible());
}
