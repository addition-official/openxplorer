// SPDX-License-Identifier: AGPL-3.0-only
//! Keeps GTK's tooltip timer off popovers that have closed.
//!
//! GTK 4.14 points its one tooltip at the surface the pointer last moved
//! or entered on, a popover's included, and starts its timer. When a
//! window is destroyed GTK lets go of the tooltip only if it points at the
//! window itself. A popover that closed with the pointer over it, and
//! whose window was destroyed before GTK heard of the pointer back over
//! the window, kept the tooltip; once the popover had lost its surface
//! with the window, the timer asked for the pointer's position on no
//! surface at all: `Gdk-CRITICAL gdk_surface_get_device_position:
//! assertion 'GDK_IS_SURFACE (surface)' failed`, which ended the test
//! process. GTK keeps such a popover alive, for one, when its button is
//! hidden after a row in it had the keyboard. Later GTK versions check
//! the surface; with 4.14 a menu item that closes its own window can meet
//! the same timer.
//!
//! So as a popover closes in a window that stays shown, GTK is asked to
//! look again at where the pointer is ([`gtk::Widget::trigger_tooltip_query`]),
//! which points its tooltip at the window under the pointer: the window's
//! own destruction then lets go of it. A popover that closes because its
//! window is being hidden or destroyed is left alone, as asking then would
//! point the tooltip back at the window GTK has just let go of.

use gtk::prelude::*;

/// Points GTK's tooltip back at `popover`'s window whenever `popover`
/// closes while the window stays shown.
pub(crate) fn release_tooltip_on_close(popover: &impl IsA<gtk::Popover>) {
    popover.as_ref().connect_closed(|popover| {
        let Some(window) = popover.root().and_downcast::<gtk::Window>() else {
            return;
        };
        if window.is_visible() {
            window.trigger_tooltip_query();
        }
    });
}
