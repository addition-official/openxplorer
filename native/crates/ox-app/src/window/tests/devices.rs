// SPDX-License-Identifier: AGPL-3.0-only
//! Drives, volumes and phones coming and going: every volume monitor
//! signal redraws every window, as the `mounts` events of
//! `v2.0.0:desktop/winspace.py` refresh the environment in `v2.0.0:desktop/ui/app.js`.
//!
//! A test cannot plug in a drive, so each signal is emitted on the
//! desktop's volume monitor without a drive, volume or mount; the window
//! reads the monitor itself and does not look at what the signal names.

use gtk::prelude::*;
use gtk::{gio, glib};
use ox_core::settings::{BookmarkRequest, Settings};

use crate::test_support::harness::{wait_until, Fixture, TestWindow};
use crate::window::environment::VOLUME_MONITOR_SIGNALS;

/// No drive, volume or mount, as the value `signal` carries.
fn no_object_for(signal: &str) -> glib::Value {
    if signal.starts_with("mount-") {
        None::<gio::Mount>.to_value()
    } else if signal.starts_with("volume-") {
        None::<gio::Volume>.to_value()
    } else {
        None::<gio::Drive>.to_value()
    }
}

/// Every mount, volume and drive signal refreshes the sidebar of every
/// window: a pin another process saved meanwhile appears after each one,
/// as the environment is read again with the devices.
///
/// parity: DEV-001, DEV-002
#[gtk::test]
fn every_volume_monitor_signal_refreshes_every_window() {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    let beside = test.open_beside(&fixture.uri());
    let mut python_app = Settings::open(test.settings_directory());
    let monitor = gio::VolumeMonitor::get();

    for signal in VOLUME_MONITOR_SIGNALS {
        let label = format!("Pinned before {signal}");
        std::fs::create_dir(fixture.path(signal)).expect("a new folder");
        let pin = BookmarkRequest::new(fixture.uri_of(signal), &label);
        python_app
            .pin_many(&[pin], None, None)
            .expect("the settings file takes a pin");

        monitor.emit_by_name_with_values(signal, &[no_object_for(signal)]);

        for shown in [&test, &beside] {
            wait_until(&label, || shown.window.sidebar().labels().contains(&label));
        }
    }
}

/// Emits GIO's `show-processes` on `operation`, as `GVfs` does while a
/// program keeps the drive busy, offering `choices`.
fn emit_show_processes(operation: &gio::MountOperation, choices: &[&str]) {
    let no_processes = glib::Value::from_type(glib::Type::from_name("GArray").expect("GArray is a type"));
    let choices: Vec<String> = choices.iter().map(|choice| (*choice).to_owned()).collect();
    operation.emit_by_name_with_values(
        "show-processes",
        &[
            "Volume is busy\nOne or more applications are keeping the volume busy.".to_value(),
            no_processes,
            choices.to_value(),
        ],
    );
}

/// Eject, Safely remove or Disconnect of a drive a program still uses
/// asks as Windows 11 does (checked in the VM on 2026-11-22): a message
/// box with the warning icon saying "'STICK' is currently in use.", that
/// open files should be saved and closed, and that Continue closes them,
/// which might lose data; Cancel has the keyboard. Cancel stops, Try
/// Again stops this try and asks for another, and Continue takes GIO's
/// first choice ("Unmount Anyway"). GIO asking again while the box shows
/// opens no second box.
///
/// parity: DEV-007, DEV-008
#[gtk::test]
fn a_drive_still_in_use_asks_in_windows_words() {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use crate::dialog::Dialog;

    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    let message = "'STICK' is currently in use.\n\nSave any open files on this disc, and then close the \
                   files or programs using the files before trying again.\n\nIf you choose to continue, \
                   the files will be closed, which might cause data to be lost.";
    let boxes = || -> Vec<Dialog> {
        gtk::Window::list_toplevels()
            .into_iter()
            .filter_map(|window| window.downcast::<Dialog>().ok())
            .filter(|dialog| dialog.is_visible() && dialog.title().as_deref() == Some("OpenXplorer"))
            .collect()
    };
    for (button, handled, retry_asked) in [
        ("Cancel", false, false),
        ("Try Again", false, true),
        ("Continue", true, false),
    ] {
        let retry = Rc::new(Cell::new(false));
        let operation = test.window.removal_operation("STICK", Rc::clone(&retry));
        let replies = Rc::new(RefCell::new(Vec::new()));
        let heard = Rc::clone(&replies);
        operation.connect_reply(move |operation, result| {
            heard.borrow_mut().push((result, operation.choice()));
        });

        emit_show_processes(operation.upcast_ref(), &["Unmount Anyway", "Cancel"]);
        wait_until("the box", || !boxes().is_empty());
        emit_show_processes(operation.upcast_ref(), &["Unmount Anyway", "Cancel"]);
        crate::test_support::harness::settle();
        let shown = boxes();
        assert_eq!(shown.len(), 1, "one box, however often GIO asks");
        let dialog = shown.into_iter().next().expect("the box");
        assert_eq!(dialog.message_text(), message);
        assert_eq!(dialog.button_labels(), ["Cancel", "Try Again", "Continue"]);
        let warning = crate::test_support::harness::descendants::<gtk::Image>(&dialog)
            .into_iter()
            .any(|image| image.icon_name().as_deref() == Some("ox-warning-32-color") && image.is_visible());
        assert!(warning, "the warning icon");
        let focus = gtk::prelude::GtkWindowExt::focus(&dialog).and_downcast::<gtk::Button>();
        assert_eq!(focus.and_then(|button| button.label()).as_deref(), Some("Cancel"));
        dialog.press(button);
        drop(dialog);
        wait_until("the answer", || !replies.borrow().is_empty());

        let (result, choice) = replies.borrow()[0];
        if handled {
            assert_eq!(
                (result, choice),
                (gio::MountOperationResult::Handled, 0),
                "{button}"
            );
        } else {
            assert_eq!(result, gio::MountOperationResult::Aborted, "{button}");
        }
        assert_eq!(retry.get(), retry_asked, "{button}");
        assert!(boxes().is_empty());
    }
}
