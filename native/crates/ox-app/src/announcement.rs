// SPDX-License-Identifier: AGPL-3.0-only
//! Messages read to screen readers without moving focus.
//!
//! GTK 4.14 gives a widget its test accessibility context when the
//! accessibility bus cannot be reached, or when `GTK_A11Y` is `test`.
//! That context has no announce handler, so `gtk_accessible_announce`
//! calls a null function and the app crashes. With `GTK_A11Y` set to
//! `none` a widget has no context at all, which gtk-rs's `at_context`
//! does not allow for. Nothing listens to such a widget, so [`announce`]
//! skips it.

use std::sync::OnceLock;

use gtk::prelude::*;

/// The type name of GTK's accessibility context that reaches no screen
/// reader.
const TEST_CONTEXT: &str = "GtkTestATContext";

/// Reads `message` to screen readers from `widget`, at `priority`, when an
/// accessibility bus can carry it.
pub(crate) fn announce(
    widget: &impl IsA<gtk::Accessible>,
    message: &str,
    priority: gtk::AccessibleAnnouncementPriority,
) {
    if has_no_accessibility() {
        return;
    }
    if widget.at_context().type_().name() != TEST_CONTEXT {
        widget.announce(message, priority);
    }
}

/// Whether GTK was told to give widgets no accessibility context
/// (`GTK_A11Y=none`, read as GTK reads it, without regard to case).
fn has_no_accessibility() -> bool {
    static NONE: OnceLock<bool> = OnceLock::new();
    *NONE.get_or_init(|| std::env::var("GTK_A11Y").is_ok_and(|value| value.eq_ignore_ascii_case("none")))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// With `GTK_A11Y=none`, which some users set to quiet the warning
    /// about a missing accessibility bus, GTK gives widgets no
    /// accessibility context at all. Announcing, as type-to-select does
    /// for every key, does nothing instead of stopping the app. GTK reads
    /// the variable once, so the test runs itself again with it set.
    #[gtk::test]
    fn a_widget_without_any_accessibility_context_announces_nothing() {
        if std::env::var_os("GTK_A11Y").as_deref() != Some(std::ffi::OsStr::new("none")) {
            let status = std::process::Command::new(std::env::current_exe().expect("the test binary"))
                .args([
                    "announcement::tests::a_widget_without_any_accessibility_context_announces_nothing",
                    "--exact",
                    "--test-threads=1",
                ])
                .env("GTK_A11Y", "none")
                .status()
                .expect("the test binary runs again");
            assert!(status.success(), "announcing with GTK_A11Y=none failed: {status}");
            return;
        }
        let window = gtk::Window::new();
        let label = gtk::Label::new(Some("Notes"));
        window.set_child(Some(&label));
        announce(&label, "Notes", gtk::AccessibleAnnouncementPriority::Medium);
        window.destroy();
    }
}
