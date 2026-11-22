// SPDX-License-Identifier: AGPL-3.0-only
//! Opening shortcuts as Windows Explorer opens them: a shortcut goes to
//! its target, under the target's own address and name, and a broken one
//! asks "Problem with Shortcut" and offers to delete it.

use std::fs;
use std::os::unix::fs::symlink;

use crate::test_support::harness::{wait_for, wait_until, Fixture, TestWindow};
use crate::window::tests::file_ops_support::{open_dialog, require_private_trash};

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
