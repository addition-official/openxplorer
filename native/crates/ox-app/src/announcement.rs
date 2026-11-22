// SPDX-License-Identifier: AGPL-3.0-only
//! Messages read to screen readers without moving focus.
//!
//! GTK 4.14 gives a widget no accessibility context at all when
//! `GTK_A11Y` is `none`, and its test context when the accessibility bus
//! cannot be reached or `GTK_A11Y` is `test`. Neither reaches a screen
//! reader, and both crash an announcement: the binding wraps the missing
//! context as if it were there, so asking its type reads a null pointer,
//! and the test context has no announce handler, so
//! `gtk_accessible_announce` calls a null function. [`announce`] therefore
//! never asks for a context when `GTK_A11Y` is `none`, and skips a widget
//! whose context is the test one.
//!
//! Every message the app reads goes through [`announce`]: a drag that
//! starts or ends, a refused character while renaming and the
//! type-to-select hint in the status bar.

use gtk::prelude::*;

/// The type name of GTK's accessibility context that reaches no screen
/// reader.
const TEST_CONTEXT: &str = "GtkTestATContext";

/// The variable GTK reads to choose its accessibility backend.
const BACKEND_VARIABLE: &str = "GTK_A11Y";

/// Whether GTK was told to give widgets no accessibility context, as
/// `GTK_A11Y=none` does (GTK compares it without regard to case).
fn accessibility_is_off(backend: Option<&str>) -> bool {
    backend.is_some_and(|backend| backend.eq_ignore_ascii_case("none"))
}

/// Reads `message` to screen readers from `widget`, at `priority`, when an
/// accessibility bus can carry it.
pub(crate) fn announce(
    widget: &impl IsA<gtk::Accessible>,
    message: &str,
    priority: gtk::AccessibleAnnouncementPriority,
) {
    let backend = std::env::var(BACKEND_VARIABLE).ok();
    if accessibility_is_off(backend.as_deref()) {
        return;
    }
    if widget.at_context().type_().name() != TEST_CONTEXT {
        widget.announce(message, priority);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name of [`announcing_with_accessibility_off`], as the test
    /// program takes it.
    const CHILD_TEST: &str = "announcement::tests::announcing_with_accessibility_off";

    /// Announcing from a widget whose accessibility context reaches no
    /// screen reader does nothing instead of crashing.
    #[gtk::test]
    fn a_widget_without_an_accessibility_bus_announces_nothing() {
        let label = gtk::Label::new(Some("Dragging 2 items"));
        announce(
            &label,
            "Dragging 2 items",
            gtk::AccessibleAnnouncementPriority::Medium,
        );
    }

    /// `GTK_A11Y=none`, in any case, turns announcements off; other
    /// backends and an unset variable leave them to the context.
    #[test]
    fn only_gtk_a11y_none_turns_announcements_off() {
        assert!(accessibility_is_off(Some("none")));
        assert!(accessibility_is_off(Some("NONE")));
        assert!(!accessibility_is_off(None));
        assert!(!accessibility_is_off(Some("atspi")));
        assert!(!accessibility_is_off(Some("test")));
        assert!(!accessibility_is_off(Some("")));
    }

    /// Run only by [`announcing_never_crashes_with_gtk_a11y_none`], in a
    /// test program started with `GTK_A11Y=none`: GTK then gives these
    /// widgets no accessibility context, which crashed every announcement.
    /// The widgets are the kinds the app announces from: a window (a drag),
    /// a text field (renaming) and a label (the status bar hint).
    #[gtk::test]
    #[ignore = "started by announcing_never_crashes_with_gtk_a11y_none with GTK_A11Y=none"]
    fn announcing_with_accessibility_off() {
        assert_eq!(std::env::var(BACKEND_VARIABLE).as_deref(), Ok("none"));
        let window = gtk::Window::new();
        let editor = gtk::Text::new();
        let hint = gtk::Label::new(Some("Report"));
        let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
        column.append(&editor);
        column.append(&hint);
        window.set_child(Some(&column));
        window.present();
        let high = gtk::AccessibleAnnouncementPriority::High;
        let medium = gtk::AccessibleAnnouncementPriority::Medium;
        announce(&window, "Dragging Report 2026.odt", medium);
        announce(
            &editor,
            "A file name can't contain any of the following characters:",
            high,
        );
        announce(&hint, "Report", medium);
        announce(&gtk::Label::new(None), "Drag ended", medium);
        window.destroy();
    }

    /// With `GTK_A11Y=none`, announcing from a drag, a rename or the
    /// status bar does not crash the app: the test program runs
    /// [`announcing_with_accessibility_off`] in a process of its own with
    /// that setting, and it must pass rather than die on a signal.
    #[test]
    fn announcing_never_crashes_with_gtk_a11y_none() {
        let program = std::env::current_exe().expect("the test program");
        let output = std::process::Command::new(program)
            .args([CHILD_TEST, "--exact", "--ignored", "--test-threads=1"])
            .env(BACKEND_VARIABLE, "none")
            .output()
            .expect("the test program starts");
        let report = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "the announcements crashed or failed with GTK_A11Y=none ({:?}):\n{report}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(report.contains("1 passed"), "the announcing test ran:\n{report}");
    }
}
