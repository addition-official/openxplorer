// SPDX-License-Identifier: AGPL-3.0-only
//! Keeps GTK's tooltip timer off popovers that have closed.
//!
//! GTK 4.14 keeps one tooltip per display and points it at the surface
//! the pointer last moved over or entered, a popover's included, starting
//! its 500 ms timer each time. It lets go of that surface only when its
//! widget is freed, or, for a window, when the window is destroyed. A
//! popover that GTK keeps alive after its window, unrealized, so stays the
//! tooltip's target, and the timer then asks for the pointer's position on
//! no surface at all: `Gdk-CRITICAL gdk_surface_get_device_position:
//! assertion 'GDK_IS_SURFACE (surface)' failed`, which ended the test
//! process (deb 61 and 62, in
//! `the_navigation_pane_hides_and_a_places_button_lists_its_places`).
//!
//! GTK kept the Places menu alive that way: closing a menu leaves the
//! window's keyboard focus on the menu's row, and once the menu's button
//! was hidden (turning the navigation pane back on hides the Places
//! button) the menu outlived its window. So as a popover closes:
//!
//! - a keyboard focus still inside it, once its own handlers have run,
//!   goes back to the nearest widget around it that takes it (a menu
//!   button's button, usually), or nowhere, and the popover is freed with
//!   its window (a submenu leaves the keyboard to its menu);
//! - GTK is asked to look again at where the pointer is
//!   ([`gtk::Widget::trigger_tooltip_query`]), which points its tooltip at
//!   the window under the pointer, if the pointer is over it.
//!
//! A popover that closes because its window is being hidden or destroyed
//! is left alone, as asking then would point the tooltip back at the
//! window GTK has just let go of.

use gtk::prelude::*;

/// Moves the keyboard out of `popover` and points GTK's tooltip back at
/// its window whenever `popover` closes while the window stays shown.
/// Runs after the popover's other "closed" handlers, which may send the
/// keyboard back to whatever opened the popover; a submenu, a popover
/// inside another, leaves the keyboard to the menu it belongs to.
pub(crate) fn release_tooltip_on_close(popover: &impl IsA<gtk::Popover>) {
    popover.as_ref().connect_local("closed", true, |values| {
        let popover = values.first()?.get::<gtk::Popover>().ok()?;
        let window = popover.root().and_downcast::<gtk::Window>()?;
        if !window.is_visible() {
            return None;
        }
        let nested = popover
            .parent()
            .is_some_and(|parent| parent.ancestor(gtk::Popover::static_type()).is_some());
        let focus_inside = GtkWindowExt::focus(&window).is_some_and(|focus| focus.is_ancestor(&popover));
        if focus_inside && !nested {
            let around = std::iter::successors(popover.parent(), WidgetExt::parent)
                .find(|widget| widget.is_drawable() && widget.grab_focus());
            if around.is_none() {
                GtkWindowExt::set_focus(&window, None::<&gtk::Widget>);
            }
        }
        window.trigger_tooltip_query();
        None
    });
}
