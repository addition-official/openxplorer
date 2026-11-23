// SPDX-License-Identifier: AGPL-3.0-only
//! "Behave like Windows Explorer", first on the General page: one switch
//! that sets every setting of [`ox_core::settings::WINDOWS_SETTINGS`] to the value that works
//! as Windows 11 File Explorer does, and turned off puts the user's
//! earlier values back ([`ox_core::settings::windows_mode`]).
//!
//! It changes no behavior by itself; each of its settings keeps its own
//! row, which shows the change at once. A setting changed on its own row
//! while the switch is on leaves the switch on, marked "Custom", as
//! Windows marks a partly ticked box, and the line under the title says
//! how many settings differ.

use gtk::glib;
use gtk::prelude::*;
use ox_core::i18n::{gettext, ngettext};
use ox_core::settings::{differing, turn_off, turn_on, windows_mode, Preferences, WindowsMode};

use super::group::SettingsGroup;
use super::parts;
use super::row::{ControlName, SettingRow};
use super::search::RowText;
use super::SettingsPage;

/// The row's text. Its description is the ⓘ's; the line under the title
/// is [`explanation`].
pub(super) const BEHAVE_LIKE_WINDOWS: RowText = RowText {
    title: "Behave like Windows Explorer",
    description: "Sets every setting that has a Windows 11 File Explorer behavior to it, all at once. \
                  Each setting still has its own switch. Turning this off puts back the settings you \
                  had before you turned it on.",
    keywords: "windows explorer mode like windows behave all settings at once master switch file \
               explorer 11 defaults compatibility",
};

/// The line under the title while the switch is off or fully on.
const EXPLANATION: &str =
    crate::i18n::message_id("Uses Windows 11 File Explorer's behavior wherever OpenXplorer offers a choice.");

/// The CSS class of the switch while it reads Custom.
const MIXED: &str = "mixed";

/// The line under the title for `preferences`: the plain explanation, or
/// how many settings differ while the switch reads Custom.
pub(super) fn explanation(preferences: &Preferences) -> String {
    match windows_mode(preferences) {
        WindowsMode::Off | WindowsMode::On => gettext(EXPLANATION),
        WindowsMode::Custom => {
            let count = differing(preferences);
            ngettext(
                "Custom: {count} setting was changed since this was turned on.",
                "Custom: {count} settings were changed since this was turned on.",
                count as u64,
            )
            .replace("{count}", &count.to_string())
        }
    }
}

/// The group with the one row, put first on the General page.
pub(super) fn group(page: &SettingsPage) -> SettingsGroup {
    let group = SettingsGroup::new(&gettext("Windows Explorer"));
    group.add_row(&row(page));
    group
}

/// The row: its switch, "Custom" before it while a setting differs, and
/// the line under its title.
fn row(page: &SettingsPage) -> SettingRow {
    let row = SettingRow::new(BEHAVE_LIKE_WINDOWS);
    let details = gettext(BEHAVE_LIKE_WINDOWS.description);
    let custom = parts::value_label(&gettext("Custom"));
    custom.set_visible(false);
    row.add_control(&custom, ControlName::OwnLabel);
    let switch = parts::switch();
    row.add_control(&switch, ControlName::RowTitle);
    page.follow_preferences(glib::clone!(
        #[weak]
        row,
        #[weak]
        switch,
        #[weak]
        custom,
        #[strong]
        details,
        move |preferences| {
            let mode = windows_mode(preferences);
            switch.set_active(mode != WindowsMode::Off);
            let mixed = mode == WindowsMode::Custom;
            custom.set_visible(mixed);
            if mixed {
                switch.add_css_class(MIXED);
            } else {
                switch.remove_css_class(MIXED);
            }
            let line = explanation(preferences);
            row.set_description(&line);
            let description = format!("{line} {details}");
            switch.update_property(&[gtk::accessible::Property::Description(&description)]);
        }
    ));
    switch.connect_active_notify(glib::clone!(
        #[weak]
        page,
        move |switch| {
            if !page.is_user_change() {
                return;
            }
            let preferences = page.context().settings_data().preferences;
            let update = if switch.is_active() {
                turn_on(&preferences)
            } else {
                turn_off(&preferences)
            };
            page.save_preferences(update);
        }
    ));
    row
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ox_core::settings::WINDOWS_SETTINGS;

    /// The markers of a setting titled as Windows' behavior.
    const MARKERS: [&str; 2] = ["(like Windows)", "(Windows)"];

    /// Every string literal of the Rust files under `folder` that has one
    /// of [`MARKERS`], with the file it is in.
    fn marked_literals(folder: &Path, found: &mut Vec<(String, String)>) {
        let mut entries: Vec<_> = std::fs::read_dir(folder)
            .expect("the sources")
            .map(|entry| entry.expect("a source entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                marked_literals(&path, found);
                continue;
            }
            // This file names the markers themselves.
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs")
                || path.ends_with(file!().trim_start_matches("crates/ox-app/"))
            {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a source file");
            // Titles are one-line literals; comments name them in passing.
            let code = text.lines().filter(|line| !line.trim_start().starts_with("//"));
            for line in code {
                for marker in MARKERS {
                    for (at, _) in line.match_indices(marker) {
                        let start = line[..at].rfind('"').map_or(0, |quote| quote + 1);
                        let end = line[at..].find('"').map_or(line.len(), |quote| at + quote);
                        found.push((line[start..end].to_owned(), path.display().to_string()));
                    }
                }
            }
        }
    }

    /// The guard: a setting titled "(like Windows)" or "(Windows)"
    /// anywhere in the app must have its line in [`WINDOWS_SETTINGS`], so
    /// "Behave like Windows Explorer" sets it too.
    ///
    /// parity: SET-020
    #[test]
    fn every_setting_titled_like_windows_is_in_the_registry() {
        let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut found = Vec::new();
        marked_literals(&sources, &mut found);
        assert!(
            !found.is_empty(),
            "the search finds the titles that are there, such as the removable drives' delete"
        );
        let missing: Vec<_> = found
            .iter()
            .filter(|(title, _)| WINDOWS_SETTINGS.iter().all(|setting| setting.title != title))
            .collect();
        assert!(
            missing.is_empty(),
            "add a line to WINDOWS_SETTINGS (ox-core settings/windows_mode.rs) for: {missing:?}"
        );
    }
}
