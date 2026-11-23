// SPDX-License-Identifier: AGPL-3.0-only
//! The navigation pane against Windows 11's: where a pin from a menu goes,
//! what a drag of files does over the pinned folders, and the other
//! fixes of the navigation-pane pass.

use std::fs;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

use ox_core::location::file_uri;
use ox_core::places::FolderLocations;

use crate::test_support::harness::{wait_for, wait_for_frames, wait_until, Fixture, TestWindow};

/// A home folder with the standard folders Quick access lists, and a
/// project folder to pin, in a temporary directory.
struct PinHome {
    _root: tempfile::TempDir,
    home: PathBuf,
    config: PathBuf,
}

impl PinHome {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("the test home has room");
        let base = fs::canonicalize(root.path()).expect("the folder resolves");
        let home = base.join("home");
        let config = base.join("config");
        for folder in [
            "Desktop",
            "Documents",
            "Downloads",
            "Music",
            "Pictures",
            "Videos",
            "Projects",
        ] {
            fs::create_dir_all(home.join(folder)).expect("a folder");
        }
        fs::create_dir_all(&config).expect("a folder");
        Self {
            _root: root,
            home,
            config,
        }
    }

    fn locations(&self) -> FolderLocations {
        FolderLocations::new(self.home.clone(), &self.config)
    }

    fn uri_of(&self, name: &str) -> String {
        file_uri(&self.home.join(name))
    }
}

/// Moves the pointer off the window and waits past the double-click
/// time before a real-input test ends. A window destroyed within the
/// double-click time of a click on it leaves GTK state behind that
/// corrupts memory later, which crashed a later test (seen on unchanged
/// main too).
fn let_go(pointer: &mut crate::test_support::pointer::Pointer) {
    pointer.move_to(1430, 990);
    wait_for(std::time::Duration::from_millis(700));
}

/// The labels of the Quick access rows, in order.
fn pinned_labels(test: &TestWindow) -> Vec<String> {
    test.window
        .places()
        .quick_access
        .iter()
        .map(|place| place.label.clone())
        .collect()
}

/// As in Windows 11, a folder pinned from its menu or the command bar goes
/// at the end of the pinned folders, and a standard folder unpinned and
/// pinned again goes there too. It used to go first, above Desktop, since
/// the pin was saved without the order the pane showed.
///
/// parity: SIDE-005, SIDE-007
#[gtk::test]
fn a_folder_pinned_from_a_menu_goes_at_the_end_of_the_pins() {
    let home = PinHome::new();
    let test = TestWindow::open_with_standard_folders(&file_uri(&home.home), home.locations(), |_| {});
    test.wait_for_listing("the home folder");
    let before = pinned_labels(&test);

    test.activate("pin-location", Some(&home.uri_of("Projects")));
    wait_until("the pin", || {
        pinned_labels(&test).contains(&"Projects".to_owned())
    });
    let after_pin = pinned_labels(&test);
    test.window.unpin(&home.uri_of("Documents"));
    wait_until("the unpin", || {
        !pinned_labels(&test).contains(&"Documents".to_owned())
    });
    test.activate("pin-location", Some(&home.uri_of("Documents")));
    wait_until("the pin again", || {
        pinned_labels(&test).contains(&"Documents".to_owned())
    });
    let after_repin = pinned_labels(&test);

    assert!(before.len() >= 6, "the standard folders are pinned: {before:?}");
    assert_eq!(
        after_pin.last().map(String::as_str),
        Some("Projects"),
        "{after_pin:?}"
    );
    assert_eq!(
        after_pin[..before.len()],
        before[..],
        "the others keep their order"
    );
    assert_eq!(
        after_repin.last().map(String::as_str),
        Some("Documents"),
        "{after_repin:?}"
    );
}

/// A real drag of a file over the top edge of a pinned folder, where a
/// folder would be pinned, goes into that folder: a file cannot be
/// pinned, and the drop used to show the pin line and then refuse with
/// "Could not pin", leaving the file where it was.
///
/// parity: DND-011, DND-014
#[gtk::test]
fn a_real_drag_of_a_file_onto_the_edge_of_a_pin_goes_into_the_pinned_folder() {
    use super::right_drag::{item_on_screen, HOLD, PROBE};
    use crate::test_support::pointer::{screen_point, Button, Pointer};

    let fixture = Fixture::standard();
    fs::create_dir(fixture.path("Drop shelf")).expect("fixture subfolder");
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&fixture.uri_of("Drop shelf"));
    test.activate("pin-folder", None);
    let sidebar = test.window.sidebar();
    wait_until("the pin", || sidebar.labels().contains(&"Drop shelf".to_owned()));
    test.show(&fixture.uri());
    wait_for_frames(&test.window, 3);
    let pins_before = pinned_labels(&test);

    let from = item_on_screen(&mut pointer, &test, "Notes 2.txt");
    let edge = (24.0, sidebar.top_edge_of("Drop shelf") + 1.0);
    let onto = screen_point(&mut pointer, sidebar.list(), edge, PROBE);
    pointer.move_to(from.0, from.1);
    pointer.press(Button::Primary);
    pointer.glide(from, (from.0 + 12, from.1 + 12), 4);
    pointer.glide((from.0 + 12, from.1 + 12), onto, 12);
    wait_for(HOLD);
    let line = sidebar.drop_highlight_of("Drop shelf");
    pointer.release(Button::Primary);

    wait_until("the move into the pinned folder", || {
        fixture.path("Drop shelf/Notes 2.txt").is_file()
    });
    assert_eq!(line, Some("file-drop-active"), "the whole row is the folder");
    assert!(
        !fixture.path("Notes 2.txt").exists(),
        "a plain drag within a drive moves"
    );
    assert_eq!(pinned_labels(&test), pins_before, "nothing is pinned");
    let_go(&mut pointer);
}

/// The screen point of the middle of the navigation pane's resize handle.
fn handle_on_screen(pointer: &mut crate::test_support::pointer::Pointer, test: &TestWindow) -> (i32, i32) {
    use super::right_drag::PROBE;
    let handle = crate::window::sidebar_resizer::paned_handle(test.window.workspace())
        .expect("the workspace has a handle");
    let bounds = handle.compute_bounds(&handle).expect("a shown handle has bounds");
    let middle = (f64::from(bounds.width() / 2.0), f64::from(bounds.height() / 2.0));
    crate::test_support::pointer::screen_point(pointer, &handle, middle, PROBE)
}

/// Dragging the navigation pane's edge with a real pointer saves the new
/// width, which a new window starts with, as Windows Explorer remembers
/// it. `GtkPaned` takes the press, which used to end the gesture that saves
/// it before the edge moved, so no drag was ever saved.
///
/// parity: SIDE-023
#[gtk::test]
fn a_real_drag_of_the_pane_edge_saves_its_width() {
    use super::right_drag::HOLD;
    use crate::test_support::pointer::{Button, Pointer};

    let fixture = Fixture::standard();
    let mut pointer = Pointer::connect();
    let test = TestWindow::open(&fixture.uri());
    wait_for_frames(&test.window, 3);
    let workspace = test.window.workspace();
    let start = workspace.position();

    let from = handle_on_screen(&mut pointer, &test);
    pointer.move_to(from.0, from.1);
    pointer.press(Button::Primary);
    pointer.glide(from, (from.0 + 60, from.1), 12);
    wait_for(HOLD);
    pointer.release(Button::Primary);
    let moved = workspace.position();

    assert!(
        moved > start + 40,
        "the edge follows the pointer: {start} to {moved}"
    );
    wait_until("the width to be saved", || {
        test.context.settings_data().preferences.sidebar_width == u32::try_from(moved).ok()
    });
    let_go(&mut pointer);
}

/// A drive that cannot be ejected, because a program still uses it or
/// it is not a removable mount at all, leaves the tab on the folder it
/// showed, as Windows leaves its window there with "This device is
/// currently in use". The tab used to stay at Home, where it was moved so
/// that the window's own listing would not keep the drive busy.
///
/// parity: DEV-007, DEV-009
#[gtk::test]
fn a_drive_that_cannot_be_ejected_leaves_the_tab_where_it_was() {
    let fixture = Fixture::standard();
    let documents = fixture.uri();
    let test = TestWindow::open(&documents);

    test.window
        .remove_drive(&documents, crate::devices::Removal::Eject);
    let moved_away = test.window.current_uri();
    wait_until("the tab to be back on its folder", || {
        test.window.current_uri().as_deref() == Some(documents.as_str())
    });
    test.wait_for_listing("the folder again");

    assert_ne!(
        moved_away.as_deref(),
        Some(documents.as_str()),
        "the tab left the drive first"
    );
    assert_eq!(
        test.names(),
        ["Documents", "Notes 2.txt", "Notes 10.txt", "Résumé.txt"]
    );
}

/// The labels of `entries`, with "-" for a divider, and whether each item
/// is checked.
fn menu_labels(entries: &[crate::window::menu_popover::MenuEntry]) -> Vec<(String, bool)> {
    use crate::window::menu_popover::{ItemCheck, MenuEntry};
    entries
        .iter()
        .map(|entry| match entry {
            MenuEntry::Item(item) => (item.label.clone(), item.check == ItemCheck::Fixed(true)),
            MenuEntry::Divider => ("-".to_owned(), false),
        })
        .collect()
}

/// The item of `entries` labelled `label`.
fn menu_item(
    entries: &[crate::window::menu_popover::MenuEntry],
    label: &str,
) -> crate::window::menu_popover::MenuItem {
    use crate::window::menu_popover::MenuEntry;
    entries
        .iter()
        .find_map(|entry| match entry {
            MenuEntry::Item(item) if item.label == label => Some(item.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the menu has {label}"))
}

/// As in Windows 11, where "Expand to current folder" is off at first,
/// the folder tree does not open its folders down to the folder shown;
/// once the option is on, it does.
///
/// parity: SIDE-028
#[gtk::test]
fn the_tree_expands_to_the_current_folder_only_with_the_option_on() {
    let fixture = Fixture::standard();
    let reports = fixture.path("Documents").join("Reports");
    fs::create_dir(&reports).expect("fixture subfolder");
    let test = TestWindow::open(&file_uri(&reports));
    test.window.imp().locations.borrow_mut().home = fixture.root().parent().map(Into::into);
    test.activate("folder-tree", None);
    let tree = test.window.folder_tree();
    wait_until("the tree", || !tree.outline().is_empty());
    wait_for(std::time::Duration::from_millis(500));
    let closed = tree.outline();
    let selected_closed = tree.selected_uri();

    test.activate("folder-tree-option", Some("expand-to-current"));
    let shown = file_uri(&reports);
    wait_until("the tree opens down to Reports", || {
        tree.selected_uri().is_some_and(|uri| uri == shown)
    });

    assert_eq!(closed, ["Home"], "no folder opened on the way");
    assert_eq!(selected_closed, None, "Reports is not in the tree yet");
    assert_eq!(
        tree.outline(),
        ["Home", "  Example projects", "    Documents", "      Reports"]
    );
}

/// A window whose Network discovery finds one computer, "DESKTOP-FICTIONAL",
/// at `uri`, without sending anything onto a network.
fn with_a_computer_at(uri: &str, shown: &str) -> TestWindow {
    use ox_core::network::{DiscoveredServer, Discovery};
    let server = DiscoveredServer {
        uri: uri.to_owned(),
        label: "DESKTOP-FICTIONAL".to_owned(),
        host: "desktop-fictional".to_owned(),
    };
    TestWindow::open_prepared(shown, move |context| {
        context
            .network()
            .set_discoverer(crate::network::Discoverer::finding(Discovery {
                servers: vec![server],
                warnings: Vec::new(),
            }));
    })
}

/// The chevron of the sidebar row labelled `label`.
fn row_chevron(test: &TestWindow, label: &str) -> Option<gtk::Button> {
    use crate::test_support::harness::descendants;
    let sidebar = test.window.sidebar();
    let index = sidebar.labels().iter().rposition(|shown| shown == label)?;
    let row = sidebar.list().row_at_index(i32::try_from(index).ok()?)?;
    descendants::<gtk::Button>(&row)
        .into_iter()
        .find(|button| button.has_css_class("side-expander"))
}

/// As in Windows 11 (checked in 26200), Network in the navigation pane
/// lists the computers discovery found, under it and after the network
/// locations, each with a chevron of its own; going to Network discovers
/// them.
///
/// parity: SIDE-019, HOME-007
#[gtk::test]
fn network_lists_the_discovered_computers_each_with_a_chevron() {
    let fixture = Fixture::standard();
    let test = with_a_computer_at("smb://desktop-fictional/", &fixture.uri());
    let sidebar = test.window.sidebar();
    let before = sidebar.labels();

    test.activate("go-to", Some(ox_core::location::NETWORK_URI));
    wait_until("the computer under Network", || {
        sidebar.labels().contains(&"DESKTOP-FICTIONAL".to_owned())
    });
    let labels = sidebar.labels();
    let network = labels
        .iter()
        .position(|label| label == "Network")
        .expect("Network");
    let computer = labels
        .iter()
        .position(|label| label == "DESKTOP-FICTIONAL")
        .expect("the computer");
    let chevron = row_chevron(&test, "DESKTOP-FICTIONAL");

    assert!(
        !before.contains(&"DESKTOP-FICTIONAL".to_owned()),
        "nothing is discovered at start"
    );
    assert!(computer > network, "{labels:?}");
    assert!(
        labels[network + 1..computer]
            .iter()
            .all(|label| label != "This PC"),
        "inside Network: {labels:?}"
    );
    let chevron = chevron.expect("the computer has a chevron");
    assert_eq!(
        chevron.tooltip_text().as_deref(),
        Some("Expand DESKTOP-FICTIONAL")
    );
}

/// Expanding Network in the navigation pane discovers the computers on
/// the network, as Windows' does, without going to the Network page.
///
/// parity: SIDE-019, SIDE-033
#[gtk::test]
fn expanding_network_in_the_pane_discovers_the_computers() {
    let fixture = Fixture::standard();
    let test = with_a_computer_at("smb://desktop-fictional/", &fixture.uri());
    let sidebar = test.window.sidebar();
    sidebar.toggle_section("network");
    wait_for(std::time::Duration::from_millis(200));
    let collapsed = sidebar.labels().contains(&"DESKTOP-FICTIONAL".to_owned());

    sidebar.toggle_section("network");
    wait_until("the computer under Network", || {
        sidebar.labels_shown().contains(&"DESKTOP-FICTIONAL".to_owned())
    });

    assert!(!collapsed);
    assert_eq!(test.window.current_uri(), Some(fixture.uri()), "the window stays");
}

/// A real click on a computer's chevron lists its shared folders under
/// it, and another hides them; Right and Left on its row do the same.
/// The test's "computer" is a local folder, so no network is reached.
///
/// parity: SIDE-019, SIDE-033
#[gtk::test]
fn a_computer_s_chevron_lists_its_shares_under_it() {
    use super::right_drag::PROBE;
    use crate::test_support::pointer::{screen_point, Button, Pointer};

    let fixture = Fixture::standard();
    fs::create_dir(fixture.path("Projects")).expect("fixture subfolder");
    let mut pointer = Pointer::connect();
    let test = with_a_computer_at(&fixture.uri(), &fixture.uri_of("Documents"));
    let sidebar = test.window.sidebar();
    test.activate("go-to", Some(ox_core::location::NETWORK_URI));
    wait_until("the computer", || {
        row_chevron(&test, "DESKTOP-FICTIONAL").is_some()
    });
    test.activate("go-to", Some(&fixture.uri_of("Documents")));
    test.wait_for_listing("Documents");
    wait_for_frames(&test.window, 3);
    let click_chevron = |pointer: &mut Pointer| {
        let chevron = row_chevron(&test, "DESKTOP-FICTIONAL").expect("the chevron");
        let bounds = chevron.compute_bounds(&chevron).expect("a shown chevron");
        let middle = (f64::from(bounds.width() / 2.0), f64::from(bounds.height() / 2.0));
        let on = screen_point(pointer, &chevron, middle, PROBE);
        pointer.move_to(on.0, on.1);
        pointer.press(Button::Primary);
        pointer.release(Button::Primary);
    };
    let under_computer = || {
        let labels = sidebar.labels_shown();
        let at = labels
            .iter()
            .position(|label| label == "DESKTOP-FICTIONAL")
            .expect("the computer");
        labels[at + 1..].iter().take(2).cloned().collect::<Vec<_>>()
    };

    click_chevron(&mut pointer);
    wait_until("the shares", || {
        under_computer().first().map(String::as_str) == Some("Documents")
    });
    let opened = under_computer();
    let collapse_tooltip = row_chevron(&test, "DESKTOP-FICTIONAL").and_then(|chevron| chevron.tooltip_text());
    wait_for(std::time::Duration::from_millis(700));
    click_chevron(&mut pointer);
    wait_until("the shares to go", || {
        !sidebar.labels().contains(&"Projects".to_owned())
    });

    assert_eq!(opened, ["Documents", "Projects"]);
    assert_eq!(collapse_tooltip.as_deref(), Some("Collapse DESKTOP-FICTIONAL"));
    assert_eq!(
        test.window.current_uri(),
        Some(fixture.uri_of("Documents")),
        "the chevron opens nothing"
    );
    let_go(&mut pointer);
}

/// Right on a computer's row lists its shared folders, Right again moves
/// to the first of them, Left moves back to the computer and Left again
/// hides them, as in Windows' navigation pane.
///
/// parity: SIDE-019, SIDE-033
#[gtk::test]
fn right_and_left_open_and_close_a_computer() {
    use crate::test_support::pointer::Pointer;

    let fixture = Fixture::standard();
    fs::create_dir(fixture.path("Agency share")).expect("fixture subfolder");
    let mut pointer = Pointer::connect();
    let test = with_a_computer_at(&fixture.uri(), &fixture.uri_of("Documents"));
    let sidebar = test.window.sidebar();
    test.activate("go-to", Some(ox_core::location::NETWORK_URI));
    wait_until("the computer", || {
        row_chevron(&test, "DESKTOP-FICTIONAL").is_some()
    });
    test.activate("go-to", Some(&fixture.uri_of("Documents")));
    test.wait_for_listing("Documents");
    sidebar.focus_location(&fixture.uri());
    wait_for_frames(&test.window, 2);
    let focused = || {
        let row = sidebar.list().focus_child()?;
        let index = usize::try_from(row.downcast::<gtk::ListBoxRow>().ok()?.index()).ok()?;
        sidebar.labels().get(index).cloned()
    };
    let shares_shown = || sidebar.labels().contains(&"Agency share".to_owned());

    pointer.tap_key(gtk::gdk::Key::Right);
    wait_until("the shares", shares_shown);
    let after_right = focused();
    pointer.tap_key(gtk::gdk::Key::Right);
    wait_until("the first share", || focused().as_deref() == Some("Agency share"));
    pointer.tap_key(gtk::gdk::Key::Left);
    wait_until("back on the computer", || {
        focused().as_deref() == Some("DESKTOP-FICTIONAL")
    });
    pointer.tap_key(gtk::gdk::Key::Left);
    wait_until("the shares to go", || !shares_shown());

    assert_eq!(
        after_right.as_deref(),
        Some("DESKTOP-FICTIONAL"),
        "the keyboard stays on it"
    );
    assert_eq!(focused().as_deref(), Some("DESKTOP-FICTIONAL"));
    assert_eq!(test.window.current_uri(), Some(fixture.uri_of("Documents")));
    let_go(&mut pointer);
}

/// A real double-click on a computer under Network opens it and lists
/// its shared folders under it, as a double-click on a node of Windows'
/// navigation pane does; its menu opens it in a new tab or window.
///
/// parity: SIDE-019, SIDE-033
#[gtk::test]
fn a_real_double_click_on_a_computer_lists_its_shares() {
    use super::right_drag::PROBE;
    use crate::test_support::pointer::{screen_point, Button, Pointer};

    let fixture = Fixture::standard();
    fs::create_dir(fixture.path("Agency share")).expect("fixture subfolder");
    let mut pointer = Pointer::connect();
    let test = with_a_computer_at(&fixture.uri(), &fixture.uri_of("Documents"));
    let sidebar = test.window.sidebar();
    test.activate("go-to", Some(ox_core::location::NETWORK_URI));
    wait_until("the computer", || {
        row_chevron(&test, "DESKTOP-FICTIONAL").is_some()
    });
    wait_for_frames(&test.window, 3);
    let menu = menu_labels(
        &sidebar
            .menu_entries_at(sidebar.middle_of("DESKTOP-FICTIONAL"))
            .expect("the computer has a menu"),
    );
    let on = screen_point(
        &mut pointer,
        sidebar.list(),
        (90.0, sidebar.middle_of("DESKTOP-FICTIONAL")),
        PROBE,
    );
    wait_for(std::time::Duration::from_millis(700));
    pointer.move_to(on.0, on.1);
    for _ in 0..2 {
        pointer.press(Button::Primary);
        pointer.release(Button::Primary);
    }

    wait_until("the shares", || {
        sidebar.labels().contains(&"Agency share".to_owned())
    });
    wait_until("the computer opened", || {
        test.window.current_uri() == Some(fixture.uri())
    });
    assert!(
        menu.starts_with(&[
            ("Open in new tab".to_owned(), false),
            ("Open in new window".to_owned(), false)
        ]),
        "{menu:?}"
    );
    let_go(&mut pointer);
}
