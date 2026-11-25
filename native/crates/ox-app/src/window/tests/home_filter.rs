// SPDX-License-Identifier: AGPL-3.0-only
//! Home's Filter, as Windows 11's command bar has it on Home (reference
//! ref-24): a button between View and "…", only on Home, whose menu
//! chooses one kind of file for Recent and Favorites to list. The kinds
//! are named for what they are rather than for Microsoft's apps (the
//! owner's decision): All File Types, Documents, Spreadsheets,
//! Presentations, Images, Videos and PDF. Clicks are real (XTEST,
//! `crate::test_support::pointer`). Every name below is fictional.

use gtk::prelude::*;

use crate::test_support::harness::{descendants, wait_until, TestWindow};
use crate::test_support::pointer::{Button, Pointer};
use crate::window::home_page;

use super::home_recent::{body, click, recent_home};

/// The command bar's menu button with `class`.
fn command(test: &TestWindow, class: &str) -> gtk::MenuButton {
    descendants::<gtk::MenuButton>(test.window.command_bar())
        .into_iter()
        .find(|button| button.has_css_class(class))
        .unwrap_or_else(|| panic!("the command bar has .{class}"))
}

/// The command bar's Filter button.
fn filter_button(test: &TestWindow) -> gtk::MenuButton {
    descendants::<gtk::MenuButton>(test.window.command_bar())
        .into_iter()
        .find(|button| button.has_css_class("filter-command"))
        .expect("the command bar has Filter")
}

/// One file of each kind, and two that are of none.
const FILES: [&str; 11] = [
    "report.docx",
    "notes.odt",
    "budget.xlsx",
    "data.csv",
    "slides.pptx",
    "photo.jpg",
    "clip.mp4",
    "invoice.pdf",
    "plain.txt",
    "letter.rtf",
    "deck.odp",
];

/// Filter shows on Home alone, between View and "…", and its menu is
/// Windows 11's radio list with the kinds named for what they are, All
/// File Types chosen at first. Sort is grayed on Home, as Windows 11
/// grays it there (ref-24, ref-38), and usable again in a folder.
///
/// parity: HOME-038
#[gtk::test]
fn filter_shows_on_home_alone_with_its_kinds() {
    let (fixture, test) = recent_home(&["plan.txt"]);
    let filter = filter_button(&test);
    wait_until("Filter on Home", || filter.is_visible());
    let sort = command(&test, "sort-command");
    assert!(!sort.is_sensitive(), "Sort is grayed on Home");
    // Windows 11's funnel before the label, as its Filter button draws it.
    let glyphs: Vec<String> = descendants::<gtk::Image>(&filter)
        .iter()
        .filter_map(|image| image.icon_name().map(|name| name.to_string()))
        .collect();
    assert_eq!(
        glyphs.first().map(String::as_str),
        Some(crate::icons::Icon::Filter.name()),
        "the funnel comes first: {glyphs:?}"
    );
    let mut pointer = Pointer::connect();
    click(&mut pointer, &test, &filter, None, Button::Primary);
    let menu = filter
        .popover()
        .and_downcast::<crate::window::menu_popover::MenuPopover>()
        .expect("Filter's menu");
    wait_until("the menu", || menu.is_mapped());
    assert_eq!(
        menu.row_labels(),
        [
            "All File Types",
            "Documents",
            "Spreadsheets",
            "Presentations",
            "Images",
            "Videos",
            "PDF"
        ]
    );
    assert_eq!(menu.checked_labels(), ["All File Types"]);
    menu.popdown();

    test.window
        .navigate(&fixture.uri_of("Documents"))
        .expect("Documents");
    test.wait_for_listing("Documents");
    wait_until("no Filter in a folder", || !filter.is_visible());
    assert!(sort.is_sensitive(), "Sort is usable in a folder");
    test.activate("home", None);
    wait_until("Sort grayed on Home again", || !sort.is_sensitive());
}

/// Each kind lists only its files, in Recent and in Favorites: Documents
/// .doc, .docx, .odt and .rtf; Spreadsheets .xls, .xlsx, .ods and .csv;
/// Presentations .ppt, .pptx and .odp; Images and Videos by their type;
/// PDF .pdf. All File Types lists them all again.
///
/// parity: HOME-038
#[gtk::test]
fn each_kind_lists_only_its_files() {
    let (fixture, test) = recent_home(&FILES);
    let shown = |kind: &str, count: usize| {
        test.activate("home-filter", Some(kind));
        wait_until("Recent filtered", || {
            home_page::list_names(body(&test)).len() == count
        });
        let mut names = home_page::list_names(body(&test));
        names.sort();
        names
    };
    assert_eq!(shown("documents", 3), ["letter", "notes", "report"]);
    assert_eq!(shown("spreadsheets", 2), ["budget", "data"]);
    assert_eq!(shown("presentations", 2), ["deck", "slides"]);
    assert_eq!(shown("images", 1), ["photo"]);
    assert_eq!(shown("videos", 1), ["clip"]);
    assert_eq!(shown("pdf", 1), ["invoice"]);
    assert_eq!(shown("all", FILES.len()).len(), FILES.len());

    for favorite in ["Documents/budget.xlsx", "Documents/photo.jpg"] {
        test.activate("add-to-favorites", Some(&fixture.uri_of(favorite)));
    }
    wait_until("two favorites", || {
        test.context.settings_data().preferences.favorites.len() == 2
    });
    test.activate("home-filter", Some("images"));
    home_page::choose_tab(body(&test), "Favorites");
    wait_until("Favorites filtered", || {
        home_page::list_names(body(&test)) == ["photo"]
    });
}

/// A kind with no file says so, and how to list every file again.
///
/// parity: HOME-038
#[gtk::test]
fn a_kind_with_no_file_says_so() {
    let (_fixture, test) = recent_home(&["plan.txt"]);
    test.activate("home-filter", Some("videos"));
    let empty = || {
        crate::test_support::harness::descendants::<gtk::Label>(body(&test))
            .into_iter()
            .filter(|label| label.has_css_class("home-empty") && label.is_drawable())
            .map(|label| label.label().to_string())
            .collect::<Vec<_>>()
    };
    wait_until("the empty text", || {
        empty() == ["No files of this kind. Choose All File Types in Filter to list every file."]
    });
    test.activate("home-filter", Some("all"));
    wait_until("the file again", || {
        home_page::list_names(body(&test)) == ["plan"]
    });
}
