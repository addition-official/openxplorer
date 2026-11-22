// SPDX-License-Identifier: AGPL-3.0-only
//! Opening shortcuts as Windows Explorer opens them: a shortcut goes to
//! its target, under the target's own address and name, and a broken one
//! asks "Problem with Shortcut" and offers to delete it.

use std::fs;
use std::os::unix::fs::symlink;

use gtk::prelude::*;

use crate::test_support::harness::descendants;
use crate::test_support::harness::{wait_for, wait_until, Fixture, TestWindow};
use crate::window::tests::file_ops_support::{
    dialog_over, open_dialog, require_private_trash, wait_for_no_dialog,
};

/// Opening a shortcut to a folder goes to the folder itself, as Windows
/// does, not into the shortcut's own path: the address bar and Up then
/// lead from where the folder really is.
///
/// parity: OPEN-027
#[gtk::test]
fn a_folder_shortcut_opens_the_folder_at_its_own_address() {
    let fixture = Fixture::standard();
    fs::create_dir(fixture.path("Desktop")).expect("fixture folder");
    symlink(
        fixture.path("Documents"),
        fixture.path("Desktop/Documents - Shortcut"),
    )
    .expect("link");
    let test = TestWindow::open(&fixture.uri_of("Desktop"));

    test.window
        .activate_item(test.position_of("Documents - Shortcut"));

    wait_until("the folder the shortcut points to", || {
        test.window.current_uri() == Some(fixture.uri_of("Documents")) && !test.window.is_loading()
    });
}

/// Opening a shortcut to a file hands the application the file itself,
/// whose name tells its type, not the shortcut's path.
///
/// parity: OPEN-027
#[gtk::test]
fn a_file_shortcut_opens_the_file_itself() {
    let fixture = Fixture::standard();
    symlink("../Notes 2.txt", fixture.path("Documents/Notes 2.txt - Shortcut")).expect("link");
    let test = TestWindow::open(&fixture.uri_of("Documents"));
    test.context.record_runs();

    test.window
        .activate_item(test.position_of("Notes 2.txt - Shortcut"));

    wait_until("the file to open", || {
        !test.context.recorded_launches().is_empty()
    });
    assert_eq!(test.context.recorded_launches(), [fixture.uri_of("Notes 2.txt")]);
}

/// A shortcut whose item was deleted asks "Problem with Shortcut", in
/// Windows' words, and offers to delete it: No keeps it, Yes moves it to
/// the Recycle Bin. A loop of links is broken the same way.
///
/// parity: OPEN-028
#[gtk::test]
fn a_broken_shortcut_offers_to_delete_it() {
    require_private_trash();
    let fixture = Fixture::standard();
    symlink(
        fixture.path("Old plan.txt"),
        fixture.path("Old plan.txt - Shortcut"),
    )
    .expect("link");
    symlink("self loop", fixture.path("self loop")).expect("loop");
    let test = TestWindow::open(&fixture.uri());
    test.context.record_runs();

    test.window
        .activate_item(test.position_of("Old plan.txt - Shortcut"));
    let dialog = open_dialog(&test);
    assert_eq!(dialog.title_text(), "Problem with Shortcut");
    assert_eq!(
        dialog.message_text(),
        "The item ‘Old plan.txt’ that this shortcut refers to has been changed or moved, so this \
         shortcut will no longer work properly.\n\nDo you want to delete this shortcut?"
    );
    assert_eq!(dialog.button_labels(), ["Yes", "No"]);
    dialog.press("No");
    wait_for(std::time::Duration::from_millis(200));
    assert!(
        fs::symlink_metadata(fixture.path("Old plan.txt - Shortcut")).is_ok(),
        "No keeps the shortcut"
    );

    test.window
        .activate_item(test.position_of("Old plan.txt - Shortcut"));
    open_dialog(&test).press("Yes");
    wait_until("the shortcut to go to the Recycle Bin", || {
        fs::symlink_metadata(fixture.path("Old plan.txt - Shortcut")).is_err()
    });
    wait_until("the shortcut to leave the list", || {
        !test.names().contains(&"Old plan.txt - Shortcut".to_owned())
    });

    test.window.activate_item(test.position_of("self loop"));
    let looped = open_dialog(&test);
    assert_eq!(looped.title_text(), "Problem with Shortcut");
    looped.press("No");
    assert!(test.context.recorded_launches().is_empty(), "nothing opened");
}

/// A shortcut onto a drive that is not plugged in says that the drive is
/// unavailable, as Windows says it, and offers nothing to delete: the
/// shortcut works again once the drive is back.
///
/// parity: OPEN-028
#[gtk::test]
fn a_shortcut_onto_a_drive_that_is_not_connected_says_so() {
    let fixture = Fixture::standard();
    symlink(
        "/media/fictional-user/BACKUP-USB/Photos",
        fixture.path("Photos - Shortcut"),
    )
    .expect("link");
    let test = TestWindow::open(&fixture.uri());

    test.window.activate_item(test.position_of("Photos - Shortcut"));
    let dialog = open_dialog(&test);

    assert_eq!(dialog.title_text(), "Problem with Shortcut");
    assert_eq!(
        dialog.message_text(),
        "The drive or network connection that the shortcut ‘Photos - Shortcut’ refers to is \
         unavailable. Make sure that the disk is properly inserted or the network resource is \
         available, and then try again."
    );
    assert_eq!(dialog.button_labels(), ["OK"]);
    dialog.press("OK");
    assert!(fs::symlink_metadata(fixture.path("Photos - Shortcut")).is_ok());
}

/// A `.desktop` link to a folder that was deleted is a broken shortcut
/// too, and Yes deletes the link file.
///
/// parity: OPEN-028
#[gtk::test]
fn a_desktop_link_to_a_deleted_folder_offers_to_delete_it() {
    require_private_trash();
    let fixture = Fixture::standard();
    let link = format!(
        "[Desktop Entry]\nType=Link\nName=Archive\nURL={}\n",
        fixture.uri_of("Archive 2025")
    );
    fs::write(fixture.path("Archive.desktop"), link).expect("the fixture is writable");
    let test = TestWindow::open(&fixture.uri());
    test.context.record_runs();

    test.window.activate_item(test.position_of("Archive.desktop"));
    let dialog = open_dialog(&test);
    assert_eq!(dialog.title_text(), "Problem with Shortcut");
    assert!(
        dialog
            .message_text()
            .starts_with("The item ‘Archive 2025’ that this shortcut"),
        "{}",
        dialog.message_text()
    );
    dialog.press("Yes");

    wait_until("the link file to go to the Recycle Bin", || {
        !fixture.path("Archive.desktop").exists()
    });
    assert_eq!(test.window.current_uri(), Some(fixture.uri()));
}

/// The next page of the wizard: a dialog over the window other than
/// `page`.
fn next_page(test: &TestWindow, page: &crate::dialog::Dialog) -> crate::dialog::Dialog {
    wait_until("the next page", || {
        dialog_over(test).is_some_and(|dialog| &dialog != page)
    });
    dialog_over(test).expect("the next page shows")
}

/// New ▸ Shortcut is Windows' Create Shortcut wizard: the location, which
/// must exist, then the name, which Windows proposes and which must be
/// free; Back keeps what was typed; Finish makes the shortcut and selects
/// it.
///
/// parity: OPS-004, OPS-051
#[gtk::test]
fn new_shortcut_is_the_create_shortcut_wizard() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());

    test.activate("new-link", None);
    let location = open_dialog(&test);
    assert_eq!(location.title_text(), "Create Shortcut");
    assert_eq!(
        location.message_text(),
        "What item would you like to create a shortcut for?\nThis wizard helps you to create shortcuts \
         to local or network programs, files, folders, computers, or Internet addresses."
    );
    assert_eq!(location.button_labels(), ["Cancel", "Next"]);
    let browse = descendants::<gtk::Button>(&location)
        .into_iter()
        .any(|button| button.label().as_deref() == Some("Browse…"));
    assert!(browse, "a Browse… button beside the location");
    let field = descendants::<gtk::Entry>(&location)[0].clone();
    field.set_text("/no/such/place");
    location.press("Next");
    wait_until("the refusal", || location.error_text().is_some());

    let notes = fixture.path("Notes 2.txt");
    field.set_text(&format!("\"{}\"", notes.display()));
    location.press("Next");
    let name = next_page(&test, &location);
    assert_eq!(name.title_text(), "Create Shortcut");
    assert!(
        name.message_text()
            .starts_with("What would you like to name the shortcut?"),
        "{}",
        name.message_text()
    );
    assert_eq!(name.button_labels(), ["Back", "Cancel", "Finish"]);
    let name_field = descendants::<gtk::Entry>(&name)[0].clone();
    assert_eq!(
        name_field.text(),
        "Notes 2.txt",
        "Windows proposes the item's name"
    );

    name.press("Back");
    let again = next_page(&test, &name);
    assert_eq!(
        descendants::<gtk::Entry>(&again)[0].text(),
        format!("\"{}\"", notes.display()),
        "Back keeps the location"
    );
    again.press("Next");
    let name = next_page(&test, &again);
    name.press("Finish");
    wait_until("the refusal of a taken name", || name.error_text().is_some());
    descendants::<gtk::Entry>(&name)[0].set_text("Notes shortcut");
    name.press("Finish");
    wait_for_no_dialog(&test);

    wait_until("the shortcut to be selected", || {
        test.selected_names() == ["Notes shortcut"]
    });
    assert_eq!(
        fs::read_link(fixture.path("Notes shortcut")).expect("a link"),
        notes
    );
    assert_eq!(
        fs::read(&notes).expect("the file"),
        fs::read(fixture.path("Notes shortcut")).expect("followed")
    );
}

/// A shortcut to an Internet address is a `.desktop` link, named "New
/// Internet Shortcut" as Windows proposes, which opens the address.
///
/// parity: OPS-051
#[gtk::test]
fn new_shortcut_to_an_internet_address_is_a_desktop_link() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());

    test.activate("new-link", None);
    let location = open_dialog(&test);
    descendants::<gtk::Entry>(&location)[0].set_text("https://example.org/news");
    location.press("Next");
    let name = next_page(&test, &location);
    assert_eq!(
        descendants::<gtk::Entry>(&name)[0].text(),
        "New Internet Shortcut"
    );
    name.press("Finish");
    wait_for_no_dialog(&test);

    let link = fixture.path("New Internet Shortcut.desktop");
    wait_until("the link file", || link.is_file());
    let text = fs::read_to_string(&link).expect("a desktop entry");
    assert!(text.contains("Type=Link\n"), "{text}");
    assert!(text.contains("URL=https://example.org/news\n"), "{text}");
    wait_until("the shortcut to be selected", || {
        test.selected_names() == ["New Internet Shortcut.desktop"]
    });
}
