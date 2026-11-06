// SPDX-License-Identifier: AGPL-3.0-only
//! Click diagnostics for the file list, on only while `OX_DEBUG_CLICKS` is
//! set (test-combined-10.3). A click that selects nothing on one desktop
//! but works in the tests is followed here, on standard error: the GTK
//! version and pointer settings once, then for each press every gesture
//! in the list that takes part (begin, claim, deny, release, cancel, end),
//! the selection's changes, the list's model being replaced, and where
//! keyboard focus goes. Nothing changes what a click does.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use super::BrowserWindow;

/// The variable that turns the diagnostics on.
pub(super) const VARIABLE: &str = "OX_DEBUG_CLICKS";

/// Whether the diagnostics are on.
pub(super) fn enabled() -> bool {
    std::env::var_os(VARIABLE).is_some()
}

/// Writes one diagnostic line.
pub(super) fn log(text: &str) {
    let micros = glib::monotonic_time() / 1000;
    eprintln!("OX_DEBUG_CLICKS [{micros} ms] {text}");
}

/// A short name for `widget`: its type and its CSS name.
fn widget_name(widget: &gtk::Widget) -> String {
    format!("{}<{}>", widget.type_().name(), widget.css_name())
}

/// A short name for `controller` on `widget`.
fn controller_name(controller: &gtk::EventController, widget: &gtk::Widget) -> String {
    let name = controller
        .name()
        .map(|name| format!(" '{name}'"))
        .unwrap_or_default();
    format!(
        "{}{} {:?} on {}",
        controller.type_().name(),
        name,
        controller.propagation_phase(),
        widget_name(widget)
    )
}

impl BrowserWindow {
    /// Follows clicks in `view` while the diagnostics are on.
    pub(super) fn attach_click_debug(&self, view: &gtk::Widget) {
        if !enabled() {
            return;
        }
        let settings = gtk::Settings::default();
        log(&format!(
            "GTK {}.{}.{}, display {:?}, drag threshold {:?}, double-click time {:?} ms, distance {:?}",
            gtk::major_version(),
            gtk::minor_version(),
            gtk::micro_version(),
            gdk_display_name(),
            settings
                .as_ref()
                .map(|s| s.property::<i32>("gtk-dnd-drag-threshold")),
            settings
                .as_ref()
                .map(|s| s.property::<i32>("gtk-double-click-time")),
            settings
                .as_ref()
                .map(|s| s.property::<i32>("gtk-double-click-distance")),
        ));
        let seen: Rc<RefCell<HashSet<usize>>> = Rc::default();
        let observer = gtk::GestureClick::new();
        observer.set_button(0);
        observer.set_name(Some("ox-click-debug"));
        observer.set_propagation_phase(gtk::PropagationPhase::Capture);
        observer.connect_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[weak]
            view,
            #[strong]
            seen,
            move |gesture, presses, x, y| {
                let position = window.folder_pane().owners().position_at(&view, x, y);
                log(&format!(
                    "press button {} (#{presses}) at ({x:.0}, {y:.0}): position {position:?}, selected before {:?}, focus {:?}",
                    gesture.current_button(),
                    window.folder_pane().model().selected_positions(),
                    gtk::prelude::GtkWindowExt::focus(&window).map(|focus| widget_name(&focus)),
                ));
                window.log_selection_changes(&seen);
                instrument(&view, &seen);
            }
        ));
        observer.connect_released(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |_, presses, x, y| {
                log(&format!(
                    "release (#{presses}) at ({x:.0}, {y:.0}); selected now {:?}",
                    window.folder_pane().model().selected_positions()
                ));
                let window = window.downgrade();
                glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                    if let Some(window) = window.upgrade() {
                        log(&format!(
                            "300 ms after release: selected {:?}, focus {:?}",
                            window.folder_pane().model().selected_positions(),
                            gtk::prelude::GtkWindowExt::focus(&window).map(|focus| widget_name(&focus)),
                        ));
                    }
                });
            }
        ));
        observer.connect_cancel(|_, _| {
            log("the debug observer's sequence was cancelled: another gesture claimed it");
        });
        observer.connect_unpaired_release(|_, x, y, button, _| {
            log(&format!(
                "unpaired release of button {button} at ({x:.0}, {y:.0})"
            ));
        });
        view.add_controller(observer);
        view.connect_notify_local(Some("model"), |view, _| {
            log(&format!(
                "the list's model was replaced on {}",
                widget_name(view.upcast_ref())
            ));
        });
        self.connect_focus_widget_notify(|window| {
            log(&format!(
                "focus moved to {:?}",
                gtk::prelude::GtkWindowExt::focus(window).map(|focus| widget_name(&focus))
            ));
        });
    }

    /// Logs every change of the current list's selection, once per model.
    fn log_selection_changes(&self, seen: &Rc<RefCell<HashSet<usize>>>) {
        let model = self.folder_pane().model();
        let selection = model.selection().clone();
        let key = selection.as_ptr() as usize;
        if !seen.borrow_mut().insert(key) {
            return;
        }
        selection.connect_selection_changed(|selection, position, count| {
            let selected = selection.selection();
            log(&format!(
                "selection changed from {position} for {count}: {} selected, first {:?}",
                selected.size(),
                (selected.size() > 0).then(|| selected.nth(0))
            ));
        });
        selection.connect_items_changed(|_, position, removed, added| {
            log(&format!("list items changed at {position}: -{removed} +{added}"));
        });
    }
}

/// The name of the default display, such as `wayland-0`.
fn gdk_display_name() -> Option<String> {
    gtk::gdk::Display::default().map(|display| display.name().to_string())
}

/// Connects logging to every gesture under `root` not yet followed.
fn instrument(root: &gtk::Widget, seen: &Rc<RefCell<HashSet<usize>>>) {
    let mut stack = vec![root.clone()];
    while let Some(widget) = stack.pop() {
        let controllers = widget.observe_controllers();
        for controller in controllers.iter::<glib::Object>().filter_map(Result::ok) {
            let Ok(controller) = controller.downcast::<gtk::EventController>() else {
                continue;
            };
            if controller.name().as_deref() == Some("ox-click-debug") {
                continue;
            }
            if !seen.borrow_mut().insert(controller.as_ptr() as usize) {
                continue;
            }
            let Ok(gesture) = controller.clone().downcast::<gtk::Gesture>() else {
                continue;
            };
            let name = controller_name(&controller, &widget);
            let label = name.clone();
            gesture.connect_sequence_state_changed(move |_, _, state| {
                log(&format!("{label}: sequence {state:?}"));
            });
            let label = name.clone();
            gesture.connect_begin(move |_, _| log(&format!("{label}: begin")));
            let label = name.clone();
            gesture.connect_end(move |_, _| log(&format!("{label}: end")));
            let label = name.clone();
            gesture.connect_cancel(move |_, _| log(&format!("{label}: cancel")));
            if let Some(click) = gesture.downcast_ref::<gtk::GestureClick>() {
                let label = name.clone();
                click.connect_pressed(move |_, n, _, _| log(&format!("{label}: pressed #{n}")));
                let label = name.clone();
                click.connect_released(move |_, n, _, _| log(&format!("{label}: released #{n}")));
                let label = name.clone();
                click.connect_stopped(move |_| log(&format!("{label}: stopped")));
            }
            if let Some(drag) = gesture.downcast_ref::<gtk::GestureDrag>() {
                let label = name.clone();
                drag.connect_drag_begin(move |_, _, _| log(&format!("{label}: drag-begin")));
                let label = name.clone();
                drag.connect_drag_end(move |_, dx, dy| log(&format!("{label}: drag-end ({dx:.1}, {dy:.1})")));
            }
        }
        let mut child = widget.first_child();
        while let Some(current) = child {
            child = current.next_sibling();
            stack.push(current);
        }
    }
}
