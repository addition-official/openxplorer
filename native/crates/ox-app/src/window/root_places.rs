// SPDX-License-Identifier: AGPL-3.0-only
//! The list the chevron after the address bar's first icon opens, as
//! Windows 11 File Explorer draws it (seen on build 26200 on 2026-11-25):
//! a plain text list, no icons, 214 pixels wide with rows of about 32,
//! ten rows tall and scrolling past them with a thin scroll bar. Its rows,
//! in Windows' order: Home, Gallery, Desktop, Documents, Downloads, Music,
//! Pictures, Videos, the user's own folder by its folder name, This PC,
//! Libraries, its removable and optical drives, Network, Control Panel,
//! the Recycle Bin, then the Desktop's own folders (seen whole on build
//! 26200 on 2026-11-25). Libraries and Control Panel, which the app has no
//! page for, are left out; a drive is listed while it is mounted, as one
//! without a disc has no folder to open. Choosing a row goes there.

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use ox_core::location::{file_uri, GALLERY_URI, NETWORK_URI, PC_URI, TRASH_URI};
use ox_core::places::{KnownFolder, Place};

use super::window_action::WindowAction;
use super::BrowserWindow;

/// The list's width, as measured on Windows 11.
const LIST_WIDTH: i32 = 214;

/// A row's height.
const ROW_HEIGHT: i32 = 32;

/// How many rows show before the list scrolls.
const SHOWN_ROWS: i32 = 10;

/// The CSS class of the list, which `menus.css` draws.
const LIST_CLASS: &str = "root-places";

/// The standard folders the list names, in Windows' order, which differs
/// from Quick access's.
const STANDARD_ORDER: [KnownFolder; 6] = [
    KnownFolder::Desktop,
    KnownFolder::Documents,
    KnownFolder::Downloads,
    KnownFolder::Music,
    KnownFolder::Pictures,
    KnownFolder::Videos,
];

/// The rows: each label and where it leads. `home` is where Home leads,
/// `known_folders` the navigation pane's standard folders and `user` the
/// user's own folder, named by its folder name.
pub(super) fn root_places(
    home: &str,
    known_folders: &[Place],
    user: &std::path::Path,
    drives: &[(String, String)],
    desktop_folders: &[(String, String)],
) -> Vec<(String, String)> {
    let mut rows = vec![
        (ox_core::i18n::gettext("Home"), home.to_owned()),
        (ox_core::i18n::gettext("Gallery"), GALLERY_URI.to_owned()),
    ];
    for folder in STANDARD_ORDER {
        if let Some(place) = known_folders
            .iter()
            .find(|place| place.known_folder == Some(folder))
        {
            rows.push((place.label.clone(), place.uri.clone()));
        }
    }
    if let Some(name) = user.file_name() {
        rows.push((name.to_string_lossy().into_owned(), file_uri(user)));
    }
    rows.push((ox_core::i18n::gettext("This PC"), PC_URI.to_owned()));
    // Windows lists Libraries here, which the app has no page for.
    rows.extend(drives.iter().cloned());
    rows.push((ox_core::i18n::gettext("Network"), NETWORK_URI.to_owned()));
    // Windows lists Control Panel here, which the app has no page for.
    rows.push((ox_core::i18n::gettext("Recycle Bin"), TRASH_URI.to_owned()));
    rows.extend(desktop_folders.iter().cloned());
    rows
}

/// A text row.
fn row(label: &str) -> gtk::ListBoxRow {
    let text = gtk::Label::builder()
        .label(label)
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        // A long folder name is cut short rather than widening the list.
        .max_width_chars(1)
        .hexpand(true)
        .build();
    gtk::ListBoxRow::builder()
        .child(&text)
        .height_request(ROW_HEIGHT)
        .build()
}

impl BrowserWindow {
    /// Opens the list under the chevron `anchor`, once the Desktop's
    /// folders are listed (off the main thread), its first row selected
    /// for the keyboard.
    pub(super) fn show_places_menu(&self, anchor: &gtk::Widget) {
        let home = self.home_location();
        let known_folders = self.context().known_folders();
        let desktop = known_folders
            .iter()
            .find(|place| place.known_folder == Some(KnownFolder::Desktop))
            .map(|place| place.uri.clone())
            .filter(|uri| !ox_core::location::same_location(uri, &file_uri(&glib::home_dir())));
        let drives = self.removable_drives();
        let show_hidden = self.shows_hidden_files();
        glib::spawn_future_local(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[weak]
            anchor,
            async move {
                let desktop_folders = match desktop {
                    Some(uri) => super::crumb_menus::list_subfolders(&uri, show_hidden).await,
                    None => Vec::new(),
                };
                let desktop_folders: Vec<(String, String)> = desktop_folders
                    .into_iter()
                    .map(|folder| (folder.name, folder.uri))
                    .collect();
                let places = root_places(
                    &home,
                    &known_folders,
                    &glib::home_dir(),
                    &drives,
                    &desktop_folders,
                );
                window.popup_places_list(&anchor, places);
            }
        ));
    }

    /// The mounted removable and optical drives, by name, which Windows'
    /// list shows after This PC (its CD drive, seen on build 26200).
    fn removable_drives(&self) -> Vec<(String, String)> {
        self.imp()
            .volumes
            .borrow()
            .iter()
            .filter(|row| matches!(row.art(), crate::icons::Art::RemovableDrive(_)))
            .filter_map(|row| Some((row.label.clone(), row.uri()?.to_owned())))
            .collect()
    }

    /// Opens `places` as the text list under `anchor`.
    fn popup_places_list(&self, anchor: &gtk::Widget, places: Vec<(String, String)>) {
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Browse)
            .activate_on_single_click(true)
            .build();
        for (label, _) in &places {
            list.append(&row(label));
        }
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .propagate_natural_height(true)
            .max_content_height(ROW_HEIGHT * SHOWN_ROWS)
            .width_request(LIST_WIDTH)
            .child(&list)
            .build();
        let popover = gtk::Popover::builder()
            .has_arrow(false)
            .position(gtk::PositionType::Bottom)
            .halign(gtk::Align::Start)
            .css_classes(["ox-menu", LIST_CLASS])
            .child(&scroller)
            .build();
        crate::tooltip_guard::release_tooltip_on_close(&popover);
        let bar = self.address_bar();
        popover.set_parent(bar);
        if let Some(bounds) = anchor.compute_bounds(bar) {
            #[expect(clippy::cast_possible_truncation, reason = "widget bounds are small")]
            let pointing = gdk::Rectangle::new(
                bounds.x() as i32,
                bounds.y() as i32,
                bounds.width() as i32,
                bounds.height() as i32,
            );
            popover.set_pointing_to(Some(&pointing));
        }
        let targets: Vec<String> = places.into_iter().map(|(_, uri)| uri).collect();
        list.connect_row_activated(glib::clone!(
            #[weak]
            popover,
            #[weak(rename_to = window)]
            self,
            move |_, row| {
                let Some(uri) = usize::try_from(row.index())
                    .ok()
                    .and_then(|index| targets.get(index))
                else {
                    return;
                };
                popover.popdown();
                WindowAction::GoTo.activate_from(&window, Some(&uri.to_variant()));
            }
        ));
        popover.connect_closed(|popover| {
            let closed = popover.clone();
            glib::idle_add_local_once(move || closed.unparent());
        });
        popover.popup();
        if let Some(first) = list.row_at_index(0) {
            list.select_row(Some(&first));
            first.grab_focus();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    /// parity: NAV-023
    #[test]
    fn the_root_list_names_windows_places_in_windows_order() {
        let place = |folder: KnownFolder, name: &str| Place {
            label: folder.label().to_owned(),
            uri: format!("file:///home/demo/{name}"),
            known_folder: Some(folder),
            is_shared: false,
        };
        // Quick access's order, which the list does not follow.
        let known = [
            place(KnownFolder::Desktop, "Desktop"),
            place(KnownFolder::Downloads, "Downloads"),
            place(KnownFolder::Documents, "Documents"),
            place(KnownFolder::Pictures, "Pictures"),
            place(KnownFolder::Music, "Music"),
            place(KnownFolder::Videos, "Videos"),
        ];

        let drives = [("CD Drive".to_owned(), "file:///media/demo/CD".to_owned())];
        let desktop = [("Zeta".to_owned(), "file:///home/demo/Desktop/Zeta".to_owned())];

        let rows = root_places(
            "file:///home/demo",
            &known,
            Path::new("/home/demo"),
            &drives,
            &desktop,
        );

        let labels: Vec<&str> = rows.iter().map(|(label, _)| label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Home",
                "Gallery",
                "Desktop",
                "Documents",
                "Downloads",
                "Music",
                "Pictures",
                "Videos",
                "demo",
                "This PC",
                "CD Drive",
                "Network",
                "Recycle Bin",
                "Zeta"
            ],
            "Windows' order (seen on build 26200): Libraries after This PC and Control Panel \
             before the Recycle Bin are left out"
        );
        assert_eq!(rows[3].1, "file:///home/demo/Documents");
        assert_eq!(rows[8].1, "file:///home/demo");
    }
}
