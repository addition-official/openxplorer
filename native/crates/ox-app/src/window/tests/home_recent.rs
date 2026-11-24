// SPDX-License-Identifier: AGPL-3.0-only
//! Home's Recent and Favorites rows act on the files they name, as the
//! rows of Windows 11's Home do: a Recent row offers Copy and Share, and
//! no Cut, Rename or Delete, by its menu, its keys or the command bar;
//! Remove from list takes a file off the desktop's recently used list
//! and leaves the file; a
//! renamed file stays in Recent and Favorites under its new name, and one
//! found gone leaves Recent. A click on blank space selects nothing, and
//! the Quick access tiles are dragged into a new order. Clicks, keys and drags are
//! real (XTEST, `crate::test_support::pointer`). Every name below is
//! fictional.

use std::fs;

use gtk::prelude::*;

use crate::app_context::add_to_desktop_history;
use crate::test_support::harness::{descendants, wait_for, wait_for_frames, wait_until, Fixture, TestWindow};
use crate::test_support::pointer::{screen_point, Button, Pointer};
use crate::window::home_page;
use crate::window::menu_popover::MenuPopover;

use super::file_ops_support::{
    dialog_over, open_dialog, require_private_trash, text_field, wait_for_no_dialog,
};
use super::home_page::open_quick_access_on_home;

/// The Home page's address.
const HOME_PAGE: &str = "ox:home";

/// A window showing Home, with "Open Quick access on Home" on, whose
/// Recent lists `Documents/<name>` of a new fixture for each of `names`,
/// each just added to the desktop's history.
fn recent_home(names: &[&str]) -> (Fixture, TestWindow) {
    require_private_trash();
    let schemes = gtk::gio::Vfs::default().supported_uri_schemes();
    assert!(
        schemes.iter().any(|scheme| scheme == "recent"),
        "this test needs GVfs with its Recent backend; found {schemes:?}"
    );
    let fixture = Fixture::standard();
    for name in names {
        let relative = format!("Documents/{name}");
        fs::write(fixture.path(&relative), b"Synthetic notes\n").expect("fixture file");
        add_to_desktop_history(&fixture.uri_of(&relative), "text/plain");
    }
    let test = TestWindow::open(&fixture.uri());
    open_quick_access_on_home(&test, true);
    test.activate("home", None);
    wait_until("the Home page", || {
        test.window.current_uri().as_deref() == Some(HOME_PAGE)
    });
    for name in names {
        let shown = stem(name);
        wait_until("the recent file", || {
            home_page::list_names(body(&test)).contains(&shown)
        });
    }
    // The desktop's list may still announce the additions, which list
    // Recent again: let that happen before the test acts.
    wait_for(std::time::Duration::from_millis(1200));
    (fixture, test)
}

/// `name` as Home shows it while extensions are hidden.
fn stem(name: &str) -> String {
    name.rsplit_once('.').map_or(name, |(stem, _)| stem).to_owned()
}

/// The Home page's body in the active pane.
fn body(test: &TestWindow) -> &gtk::Box {
    test.window.folder_pane().landing()
}

/// The names Recent lists.
fn recent_names(test: &TestWindow) -> Vec<String> {
    home_page::list_names(body(test))
}

/// Moves the real pointer to the middle of `widget`, or to `at` in it,
/// and clicks `button`.
fn click(
    pointer: &mut Pointer,
    test: &TestWindow,
    widget: &impl IsA<gtk::Widget>,
    at: Option<(f64, f64)>,
    button: Button,
) {
    wait_for_frames(&test.window, 2);
    let at = at.unwrap_or((f64::from(widget.width()) / 2.0, f64::from(widget.height()) / 2.0));
    let (x, y) = screen_point(pointer, widget, at, (8, 8));
    pointer.move_to(x, y);
    pointer.press(button);
    pointer.release(button);
}

/// The menu open in the folder pane.
fn open_menu(test: &TestWindow) -> MenuPopover {
    let open = || {
        descendants::<MenuPopover>(test.window.folder_pane())
            .into_iter()
            .find(gtk::prelude::WidgetExt::is_mapped)
    };
    wait_until("a menu", || open().is_some());
    let menu = open().expect("the menu is open");
    wait_for_frames(&test.window, 2);
    menu
}

/// Selects the row named `name` with a real click.
fn click_row(pointer: &mut Pointer, test: &TestWindow, name: &str) {
    wait_until("the row", || {
        home_page::list_names(body(test)).contains(&name.to_owned())
    });
    let row = home_page::item_named(body(test), name);
    click(pointer, test, &row, None, Button::Primary);
    wait_until("the row to be selected", || row.is_selected() && row.has_focus());
}

/// Remove from list takes the file off the desktop's recently used list
/// and off Home's Recent; the file itself stays.
///
/// parity: HOME-036
#[gtk::test]
fn remove_from_recent_drops_the_row_and_keeps_the_file() {
    let (fixture, test) = recent_home(&["plan.txt", "budget.txt"]);
    let plan = fixture.uri_of("Documents/plan.txt");
    let mut pointer = Pointer::connect();
    let row = home_page::item_named(body(&test), "plan");
    click(&mut pointer, &test, &row, None, Button::Secondary);
    let menu = open_menu(&test);

    menu.click_row("Remove from list");

    wait_until("the row to leave Recent", || {
        !recent_names(&test).contains(&"plan".to_owned())
    });
    assert!(recent_names(&test).contains(&"budget".to_owned()));
    assert!(fixture.path("Documents/plan.txt").is_file(), "the file stays");
    assert!(!gtk::RecentManager::default().has_item(&plan));
}

/// Delete, Shift+Delete, F2 and Ctrl+X do nothing on a Recent row, as
/// in Windows 11's Home (checked 2026-11-22): no file goes to the Recycle
/// Bin, no dialog opens, the clipboard keeps what it had, and the row
/// stays.
///
/// parity: HOME-036
#[gtk::test]
fn delete_f2_and_ctrl_x_do_nothing_on_a_recent_row() {
    use super::recent_rows::copied_uris;

    let (fixture, test) = recent_home(&["plan.txt", "budget.txt"]);
    test.window.clipboard().set_text("Synthetic text");
    let mut pointer = Pointer::connect();
    click_row(&mut pointer, &test, "plan");

    pointer.tap_key(gtk::gdk::Key::Delete);
    pointer.hold_key(gtk::gdk::Key::Shift_L);
    pointer.tap_key(gtk::gdk::Key::Delete);
    pointer.let_go_key(gtk::gdk::Key::Shift_L);
    pointer.tap_key(gtk::gdk::Key::F2);
    pointer.hold_key(gtk::gdk::Key::Control_L);
    pointer.tap_key(gtk::gdk::Key::x);
    pointer.let_go_key(gtk::gdk::Key::Control_L);
    wait_for(std::time::Duration::from_millis(800));

    assert!(dialog_over(&test).is_none(), "no dialog");
    assert!(fixture.path("Documents/plan.txt").is_file(), "the file stays");
    assert_eq!(copied_uris(&test), None, "nothing cut");
    assert!(recent_names(&test).contains(&"plan".to_owned()));
    assert_eq!(test.window.current_uri().as_deref(), Some(HOME_PAGE));
}

/// A rename anywhere in the app keeps the file in Recent and Favorites
/// under its new name, as the desktop's list follows the rename.
///
/// parity: HOME-036
#[gtk::test]
fn a_renamed_file_stays_in_recent_and_favorites() {
    let (fixture, test) = recent_home(&["plan.txt"]);
    let plan = fixture.uri_of("Documents/plan.txt");
    test.activate("add-to-favorites", Some(&plan));
    wait_until("the favorite", || {
        test.context.settings_data().preferences.favorites.len() == 1
    });
    wait_for(std::time::Duration::from_millis(600));

    test.activate("rename-folder", Some(&plan));
    let dialog = open_dialog(&test);
    assert_eq!(dialog.title_text(), "Rename");
    text_field(&dialog).set_text("final plan.txt");
    dialog.press("Save");
    wait_for_no_dialog(&test);

    wait_until("the file to be renamed", || {
        fixture.path("Documents/final plan.txt").is_file()
    });
    assert!(!fixture.path("Documents/plan.txt").exists());
    wait_until("Recent to list the new name", || {
        recent_names(&test) == ["final plan"]
    });
    let renamed = fixture.uri_of("Documents/final plan.txt");
    assert!(test.window.is_favorite(&renamed), "Favorites follows the rename");
    assert!(!test.window.is_favorite(&plan));
}

/// Ctrl+C on a Recent row puts the file on the clipboard, as its menu's
/// Copy does.
///
/// parity: HOME-036
#[gtk::test]
fn ctrl_c_on_a_recent_row_copies_the_file() {
    use super::recent_rows::copied_uris;

    let (fixture, test) = recent_home(&["plan.txt"]);
    test.window.clipboard().set_text("Synthetic text");
    let mut pointer = Pointer::connect();
    click_row(&mut pointer, &test, "plan");

    pointer.hold_key(gtk::gdk::Key::Control_L);
    pointer.tap_key(gtk::gdk::Key::c);
    pointer.let_go_key(gtk::gdk::Key::Control_L);

    let plan = fixture.uri_of("Documents/plan.txt");
    wait_until("the copied file", || {
        copied_uris(&test).as_deref() == Some(plan.as_str())
    });
}

/// Opening a Recent file that is gone says "Item Not Found", as Windows
/// does, and the row then leaves Recent.
///
/// parity: HOME-015, HOME-036
#[gtk::test]
fn a_recent_file_found_gone_leaves_recent() {
    let (fixture, test) = recent_home(&["plan.txt", "budget.txt"]);
    fs::remove_file(fixture.path("Documents/plan.txt")).expect("the file is removable");

    home_page::open_item(body(&test), "plan");
    let dialog = open_dialog(&test);
    assert_eq!(dialog.title_text(), "Item Not Found");
    dialog.press("OK");
    wait_for_no_dialog(&test);

    wait_until("the gone file to leave Recent", || {
        !recent_names(&test).contains(&"plan".to_owned())
    });
    assert!(recent_names(&test).contains(&"budget".to_owned()));
}

/// Clear File Explorer history (Folder Options > General > Privacy)
/// empties Recent on Home at once; Favorites and the pinned folders stay,
/// as Microsoft's answer says they do.
///
/// parity: HOME-015, HOME-036
#[gtk::test]
fn clearing_history_empties_recent_and_keeps_favorites() {
    let (fixture, test) = recent_home(&["plan.txt"]);
    test.activate("add-to-favorites", Some(&fixture.uri_of("Documents/plan.txt")));
    wait_until("the favorite", || {
        test.context.settings_data().preferences.favorites.len() == 1
    });

    test.window.clear_explorer_history();

    wait_until("Recent to empty", || {
        home_page::empty_text(body(&test)).as_deref() == Some("Files you open recently show here.")
    });
    assert_eq!(test.context.settings_data().preferences.favorites.len(), 1);
    home_page::choose_tab(body(&test), "Favorites");
    wait_until("the favorite still listed", || {
        home_page::list_names(body(&test)) == ["plan"]
    });
}

/// A Quick access tile dragged onto the right half of another goes after
/// it, and onto the left half before it, as an insertion mark shows in
/// Windows; the navigation pane takes the same order, which is saved.
///
/// parity: HOME-037
#[gtk::test]
fn a_tile_dragged_onto_another_moves_there() {
    let fixture = Fixture::standard();
    for name in ["Alpha", "Beta", "Gamma"] {
        fs::create_dir(fixture.path(name)).expect("fixture folder");
    }
    let test = TestWindow::open(&fixture.uri());
    open_quick_access_on_home(&test, true);
    for name in ["Alpha", "Beta", "Gamma"] {
        test.activate("pin-location", Some(&fixture.uri_of(name)));
        wait_until("the pin", || home_tiles(&test).contains(&name.to_owned()));
    }
    test.activate("home", None);
    wait_until("the Home page", || {
        test.window.current_uri().as_deref() == Some(HOME_PAGE)
    });
    wait_until("the tiles", || {
        pins_in(&home_tiles(&test)) == ["Alpha", "Beta", "Gamma"]
    });

    let mut pointer = Pointer::connect();
    drag_tile(&mut pointer, &test, "Alpha", "Gamma", 0.8);
    wait_until("Alpha after Gamma", || {
        pins_in(&home_tiles(&test)) == ["Beta", "Gamma", "Alpha"]
    });
    drag_tile(&mut pointer, &test, "Alpha", "Beta", 0.2);
    wait_until("Alpha before Beta", || {
        pins_in(&home_tiles(&test)) == ["Alpha", "Beta", "Gamma"]
    });
    let order = test.context.settings_data().quick_order;
    let saved: Vec<String> = ["Alpha", "Beta", "Gamma"]
        .iter()
        .map(|name| fixture.uri_of(name))
        .collect();
    let positions: Vec<usize> = saved
        .iter()
        .map(|uri| {
            order
                .iter()
                .position(|kept| kept == uri)
                .expect("saved in the order")
        })
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]), "{order:?}");
}

/// The Quick access tiles Home shows, in order.
fn home_tiles(test: &TestWindow) -> Vec<String> {
    if test.window.current_uri().as_deref() != Some(HOME_PAGE) {
        test.activate("home", None);
        wait_until("the Home page", || {
            test.window.current_uri().as_deref() == Some(HOME_PAGE)
        });
    }
    home_page::tile_names(body(test))
}

/// The fixture's pins among `tiles`, in order.
fn pins_in(tiles: &[String]) -> Vec<String> {
    tiles
        .iter()
        .filter(|name| ["Alpha", "Beta", "Gamma"].contains(&name.as_str()))
        .cloned()
        .collect()
}

/// Drags the tile named `from` with the real pointer and drops it at
/// `across` (0 to 1) of the width of the tile named `onto`.
fn drag_tile(pointer: &mut Pointer, test: &TestWindow, from: &str, onto: &str, across: f64) {
    wait_for_frames(&test.window, 2);
    let source = home_page::tile_named(body(test), from);
    let target = home_page::tile_named(body(test), onto);
    let start = screen_point(pointer, &source, (40.0, f64::from(source.height()) / 2.0), (8, 8));
    let end = screen_point(
        pointer,
        &target,
        (
            f64::from(target.width()) * across,
            f64::from(target.height()) / 2.0,
        ),
        (8, 8),
    );
    pointer.move_to(start.0, start.1);
    pointer.press(Button::Primary);
    pointer.glide(start, (start.0 + 12, start.1 + 4), 4);
    pointer.glide((start.0 + 12, start.1 + 4), end, 12);
    wait_for_frames(&test.window, 3);
    pointer.release(Button::Primary);
    wait_for_frames(&test.window, 3);
}

/// A text editor installed in the private data folder the tests run with
/// (native/tools/check.py) until it is dropped, listed as one for plain
/// text, as the desktop's text editor is; it runs `true`.
struct TextEditor(std::path::PathBuf);

impl TextEditor {
    const ID: &'static str = "io.example.HomeTextEditor.desktop";
    const NAME: &'static str = "Sample Editor";

    fn install() -> Self {
        let data = gtk::glib::user_data_dir();
        assert!(
            data.starts_with(std::env::temp_dir()),
            "tests add applications only to a private data folder"
        );
        let folder = data.join("applications");
        fs::create_dir_all(&folder).expect("the data folder is writable");
        let path = folder.join(Self::ID);
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={}\nExec=true %U\nMimeType=text/plain;\n\
             Categories=Utility;TextEditor;\n",
            Self::NAME
        );
        fs::write(&path, entry).expect("the data folder is writable");
        wait_until("GIO to list the editor", || {
            crate::integration::installed_application(Self::ID).is_some()
        });
        Self(path)
    }
}

impl Drop for TextEditor {
    fn drop(&mut self) {
        if fs::remove_file(&self.0).is_ok() {
            wait_until("GIO to forget the editor", || {
                crate::integration::installed_application(Self::ID).is_none()
            });
        }
    }
}

/// "Edit in" and the name of the text editor the desktop offers for plain
/// text: the fixture's, or one installed on the computer that comes first.
fn edit_in_label() -> String {
    let editor = crate::integration::text_editor_for(Some("text/plain")).expect("a text editor");
    format!("Edit in {}", editor.name)
}

/// The rows of `menu` with their dividers ("-"), leaving out Share and
/// Share with, which come only where the desktop can share.
fn rows_without_share(menu: &MenuPopover) -> Vec<String> {
    menu.row_labels()
        .into_iter()
        .filter(|label| label != "Share" && label != "Share with")
        .collect()
}

/// The shortcut text shown on the row labelled `label`.
fn shortcut_of(menu: &MenuPopover, label: &str) -> Option<String> {
    descendants::<gtk::Label>(&menu.row(label))
        .into_iter()
        .find(|text| text.has_css_class("shortcut"))
        .map(|text| text.text().to_string())
}

/// Right-clicks the Recent row named `name` with the real pointer and
/// returns its menu.
fn recent_row_menu(pointer: &mut Pointer, test: &TestWindow, name: &str) -> MenuPopover {
    wait_until("the row", || recent_names(test).contains(&name.to_owned()));
    let row = home_page::item_named(body(test), name);
    click(pointer, test, &row, None, Button::Secondary);
    open_menu(test)
}

/// A Recent text file's menu is Windows 11's (checked on Windows 11, Home
/// > Recent, 2026-11-24): Copy (and Share) in the strip, then Open
/// (Enter), Open with, Share with, Open file location, Remove from list,
/// Add to Favorites, Compress to, Copy as path, Properties, the text
/// editor's own "Edit in" row after a divider, and "Show more options".
///
/// parity: HOME-036
#[gtk::test]
fn a_recent_text_files_menu_is_windows_11s() {
    let _editor = TextEditor::install();
    let (_fixture, test) = recent_home(&["extra_2.txt"]);
    let mut pointer = Pointer::connect();
    let menu = recent_row_menu(&mut pointer, &test, "extra_2");

    let strip = menu.strip_labels();
    assert_eq!(strip.first().map(String::as_str), Some("Copy"), "{strip:?}");
    assert!(
        strip
            .iter()
            .all(|label| ["Copy", "Share"].contains(&label.as_str())),
        "{strip:?}"
    );
    let edit_in = edit_in_label();
    assert_eq!(
        rows_without_share(&menu),
        [
            "Open",
            "Open with",
            "Open file location",
            "Remove from list",
            "Add to Favorites",
            "Compress to",
            "Copy as path",
            "Properties",
            "-",
            edit_in.as_str(),
            "-",
            "Show more options"
        ]
    );
    assert_eq!(shortcut_of(&menu, "Open").as_deref(), Some("Enter"));
    assert!(menu.row("Open with").is_sensitive());
    assert!(menu.row(&edit_in).is_sensitive());
}

/// A Recent file without a type, such as README, opens with "Open with"
/// (Enter), as Windows lists it, and has no other Open row.
///
/// parity: HOME-036
#[gtk::test]
fn a_recent_file_without_a_type_starts_with_open_with() {
    let (_fixture, test) = recent_home(&["README"]);
    let mut pointer = Pointer::connect();
    let menu = recent_row_menu(&mut pointer, &test, "README");

    let rows = rows_without_share(&menu);
    assert_eq!(rows.first().map(String::as_str), Some("Open with"), "{rows:?}");
    assert_eq!(shortcut_of(&menu, "Open with").as_deref(), Some("Enter"));
    assert!(!rows.contains(&"Open".to_owned()), "{rows:?}");
}

/// "Show more options" on a Recent text file opens Windows 11's classic
/// menu (checked 2026-11-24) in its groups: Open, the text editor's "Edit
/// in", Add to Favorites, Remove from list, Open with…; Copy as path,
/// Share, Restore previous versions; Send to; Copy; Create shortcut; Open
/// file location; Properties. No Cut, Delete or Rename; Windows' own
/// Edit, Print, Give access to and Defender scan are left out.
///
/// parity: HOME-036
#[gtk::test]
fn a_recent_text_files_classic_menu_is_windows_11s() {
    let _editor = TextEditor::install();
    let (_fixture, test) = recent_home(&["extra_2.txt"]);
    let mut pointer = Pointer::connect();
    let menu = recent_row_menu(&mut pointer, &test, "extra_2");
    menu.click_row("Show more options");
    let classic = open_menu(&test);

    let edit_in = edit_in_label();
    // Windows' classic menu has "Open with…" for a file with one
    // application and the submenu where there are several.
    let open_with =
        if crate::integration::menu_applications("file:///x.txt", Some("text/plain"), false, 3).len() < 2 {
            "Open with…"
        } else {
            "Open with"
        };
    assert_eq!(
        rows_without_share(&classic),
        [
            "Open",
            edit_in.as_str(),
            "Add to Favorites",
            "Remove from list",
            open_with,
            "-",
            "Copy as path",
            "Restore previous versions",
            "-",
            "Send to",
            "-",
            "Copy",
            "-",
            "Create shortcut",
            "-",
            "Open file location",
            "-",
            "Properties"
        ]
    );
    for label in ["Open", "Send to", "Copy", "Create shortcut", "Properties"] {
        assert!(classic.row(label).is_sensitive(), "{label} can be chosen");
    }
    classic.popdown();
}

/// With a Recent row selected, Send to, Share with and Create shortcut can
/// run on its file, as in Windows 11, where Share with is enabled: Send to
/// ▸ Compressed (zipped) folder makes the ZIP beside the file.
///
/// parity: HOME-036
#[gtk::test]
fn send_to_and_share_with_act_on_a_recent_row() {
    use crate::window::window_action::WindowAction;

    let (fixture, test) = recent_home(&["plan.txt", "budget.txt"]);
    let enabled = |action: WindowAction| {
        test.window
            .lookup_action(action.name())
            .is_some_and(|action| action.is_enabled())
    };
    let mut pointer = Pointer::connect();
    click_row(&mut pointer, &test, "plan");
    for action in [
        WindowAction::SendTo,
        WindowAction::SendToDesktop,
        WindowAction::CreateShortcut,
        WindowAction::CompressToZip,
    ] {
        wait_until("the action to follow the row", || enabled(action));
    }

    test.activate("compress-to-zip", None);
    wait_until("the ZIP beside the file", || {
        fixture.path("Documents/plan.zip").is_file()
    });
    assert!(fixture.path("Documents/plan.txt").is_file(), "the file stays");
}

/// The icon names the images of `menu` draw, strip and rows, leaving out
/// application icons (which are the applications' own).
fn menu_icon_names(menu: &MenuPopover) -> Vec<String> {
    descendants::<gtk::Image>(menu)
        .into_iter()
        .filter(gtk::prelude::WidgetExt::is_drawable)
        .filter_map(|image| image.icon_name().map(|name| name.to_string()))
        .collect()
}

/// Each strip button's label and the icon it draws.
fn menu_strip_icons(menu: &MenuPopover) -> Vec<(String, String)> {
    descendants::<gtk::Button>(menu)
        .into_iter()
        .filter(|button| button.ancestor(gtk::ListBox::static_type()).is_none())
        .filter_map(|button| {
            let label = descendants::<gtk::Label>(&button).first()?.text().to_string();
            let icon = descendants::<gtk::Image>(&button)
                .first()?
                .icon_name()?
                .to_string();
            Some((label, icon))
        })
        .collect()
}

/// Each row's label and the icon it draws.
fn menu_row_icons(menu: &MenuPopover) -> Vec<(String, String)> {
    menu.rows()
        .iter()
        .filter_map(|row| {
            let label = crate::window::menu_popover::row_label(row)?;
            let icon = descendants::<gtk::Image>(row)
                .first()
                .and_then(gtk::Image::icon_name)
                .map_or_else(|| "(app or none)".to_owned(), |name| name.to_string());
            Some((label, icon))
        })
        .collect()
}

/// With Windows icon files on and a symbol font chosen, a Recent row's
/// menu draws its strip's and rows' glyphs from the font, as the file
/// list's menu does. The font is synthetic, built in the test from the
/// codepoints Microsoft's table gives those glyphs.
///
/// parity: HOME-036, LOOK-031
#[gtk::test]
fn a_recent_rows_menu_draws_the_chosen_fonts_glyphs() {
    use ox_core::settings::PreferencesUpdate;
    use ox_core::windows_glyphs::synthetic::SyntheticFont;

    use crate::icons::{user_glyphs, Icon};

    let (fixture, test) = recent_home(&["extra_2.txt"]);
    let icons = [
        Icon::Copy,
        Icon::Share,
        Icon::Open,
        Icon::Apps,
        Icon::Folder,
        Icon::FolderZip,
        Icon::Dismiss,
        Icon::Star,
        Icon::StarFilled,
        Icon::Info,
        Icon::Link,
        Icon::Wrench,
        Icon::Document,
        Icon::MoreHorizontal,
        Icon::ChevronRight16,
        Icon::Cut,
        Icon::Rename,
        Icon::Delete,
    ];
    let codepoints: Vec<u32> = icons
        .iter()
        .filter_map(|icon| user_glyphs::glyph_of(*icon))
        .map(|glyph| glyph.codepoint)
        .collect();
    let font = fixture.path("SegoeIcons.ttf");
    std::fs::write(&font, SyntheticFont::with_codepoints(&codepoints).build()).expect("written");
    test.context.update_preferences(
        PreferencesUpdate {
            use_icon_files: Some(true),
            symbol_font: Some(font.display().to_string()),
            ..PreferencesUpdate::default()
        },
        |result| result.expect("saved"),
    );
    wait_until("the font to be read", || {
        matches!(user_glyphs::status(), user_glyphs::Status::Ready { .. })
    });

    let mut pointer = Pointer::connect();
    let menu = recent_row_menu(&mut pointer, &test, "extra_2");
    let names = menu_icon_names(&menu);
    let bundled: Vec<&String> = names.iter().filter(|name| !name.starts_with("ox-win-")).collect();
    let recent_rows = menu_row_icons(&menu);
    let recent_strip = menu_strip_icons(&menu);
    menu.popdown();
    // The file list's menu of the same file, for comparison.
    test.window
        .navigate(&fixture.uri_of("Documents"))
        .expect("Documents");
    test.wait_for_listing("Documents");
    super::file_ops_support::select_names(&test, &["extra_2.txt"]);
    test.window
        .right_click(test.window.folder_model().selected_positions().first().copied());
    let file_menu = open_menu(&test);
    let file_rows = menu_row_icons(&file_menu);
    let file_strip = menu_strip_icons(&file_menu);
    file_menu.popdown();
    assert!(
        bundled.is_empty(),
        "Recent's menu keeps bundled icons {bundled:?}"
    );
    // A command both menus have draws the same glyph in both; Recent's
    // Open shows the default application's icon, as Windows' does.
    for (label, icon) in recent_rows
        .iter()
        .chain(&recent_strip)
        .filter(|(label, _)| label != "Open")
    {
        if let Some((_, theirs)) = file_rows
            .iter()
            .chain(&file_strip)
            .find(|(their, _)| their == label)
        {
            assert_eq!(icon, theirs, "{label} draws the file menu's glyph");
        }
    }
}

/// The strip's buttons of `menu`, left to right, with their bounds in the
/// strip, and the strip's width.
fn strip_layout(menu: &MenuPopover) -> (Vec<gtk::graphene::Rect>, f32) {
    let strip = descendants::<gtk::Box>(menu)
        .into_iter()
        .find(|widget| widget.has_css_class("context-strip"))
        .expect("the menu's strip");
    let buttons: Vec<gtk::graphene::Rect> = descendants::<gtk::Button>(&strip)
        .iter()
        .filter_map(|button| button.compute_bounds(&strip))
        .collect();
    #[expect(clippy::cast_precision_loss, reason = "a strip is a few hundred pixels wide")]
    let width = strip.width() as f32;
    (buttons, width)
}

/// The compact menu's strip packs its buttons from the left at one fixed
/// width, whatever their number, with the rest of the strip empty, as
/// Windows 11 draws it (measured on Windows 11 at 1x, from the menu's
/// outer edge: Copy centred at 32, a 1-pixel divider at 65, Share
/// centred at 98, nothing after the last button). A Recent row's two
/// buttons and a file's five are as wide.
///
/// parity: HOME-036, CMD-010
#[gtk::test]
fn the_strip_packs_fixed_width_buttons_from_the_left() {
    let (fixture, test) = recent_home(&["extra_2.txt"]);
    let mut pointer = Pointer::connect();
    let menu = recent_row_menu(&mut pointer, &test, "extra_2");
    let (recent, strip_width) = strip_layout(&menu);
    menu.popdown();
    test.window
        .navigate(&fixture.uri_of("Documents"))
        .expect("Documents");
    test.wait_for_listing("Documents");
    super::file_ops_support::select_names(&test, &["extra_2.txt"]);
    test.window
        .right_click(test.window.folder_model().selected_positions().first().copied());
    let file_menu = open_menu(&test);
    let (file, _) = strip_layout(&file_menu);
    file_menu.popdown();

    assert_eq!(recent.len(), 2, "Copy and Share: {recent:?}");
    assert!(file.len() >= 4, "the file menu's strip: {file:?}");
    let width = recent[0].width();
    for button in recent.iter().chain(&file) {
        assert!(
            (button.width() - width).abs() <= 1.0,
            "one fixed width: {button:?} against {width}"
        );
    }
    // The first button fills the strip's start, inside the menu's 1-pixel
    // border: 64 pixels, centred at 32 from the menu's edge; each next one
    // starts with its 1-pixel divider, 65 pixels on.
    assert!((63.0..=65.0).contains(&width), "Windows' width: {width}");
    assert!(recent[0].x() <= 0.5, "packed from the left: {:?}", recent[0]);
    let pitch = recent[1].x() - recent[0].x();
    assert!(
        (64.0..=66.0).contains(&pitch),
        "65 pixels from button to button: {pitch}"
    );
    let used = recent[1].x() + recent[1].width();
    assert!(
        strip_width - used > 100.0,
        "the rest of the strip stays empty: {used} of {strip_width}"
    );
}
