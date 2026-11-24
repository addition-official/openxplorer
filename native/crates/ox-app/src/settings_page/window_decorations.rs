// SPDX-License-Identifier: AGPL-3.0-only
//! The opt-in desktop frame, which every open window takes at once
//! (LOOK-032), as the title bar's window menu also offers it.

use ox_core::settings::PreferencesUpdate;

use super::bindings::PreferenceBinding;
use super::row::{ControlName, SettingRow};
use super::search::RowText;
use super::SettingsPage;

const SYSTEM_WINDOW_DECORATIONS: RowText = RowText {
    title: "Use system window decorations",
    description: "Use your desktop's title bar and window buttons, with the tabs below them. Open \
                  windows change at once. Where the desktop draws no title bars, GTK draws a standard \
                  one.",
    keywords: "appearance layout frame border borders titlebar title bar desktop native system \
               decorations KDE KWin GNOME Wayland X11 CSD SSD minimize maximize close",
};

const HYPHEN_IN_WINDOW_TITLES: RowText = RowText {
    title: "Separate window titles with a hyphen (like Windows)",
    description: "Titles a window \"Documents - OpenXplorer\", as Windows writes \"Documents - File \
                  Explorer\", instead of \"Documents — OpenXplorer\".",
    keywords: "window title caption taskbar alt tab hyphen dash em dash separator windows explorer",
};

/// "Separate window titles with a hyphen (like Windows)", under the
/// system decorations switch: off by default, the owner's em dash; every
/// open window retitles at once.
pub(super) fn hyphen_row(page: &SettingsPage) -> SettingRow {
    let row = SettingRow::new(HYPHEN_IN_WINDOW_TITLES);
    let switch = page.preference_switch(PreferenceBinding {
        read: |preferences| preferences.hyphen_in_window_titles,
        write: |on| PreferencesUpdate {
            hyphen_in_window_titles: Some(on),
            ..PreferencesUpdate::default()
        },
    });
    row.add_control(&switch, ControlName::RowTitle);
    row
}

/// The searchable, keyboard-accessible Appearance > Layout switch.
pub(super) fn row(page: &SettingsPage) -> SettingRow {
    let row = SettingRow::new(SYSTEM_WINDOW_DECORATIONS);
    let switch = page.preference_switch(PreferenceBinding {
        read: |preferences| preferences.system_window_decorations,
        write: |on| PreferencesUpdate {
            system_window_decorations: Some(on),
            ..PreferencesUpdate::default()
        },
    });
    row.add_control(&switch, ControlName::RowTitle);
    row
}
