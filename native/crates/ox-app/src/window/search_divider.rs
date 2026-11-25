// SPDX-License-Identifier: AGPL-3.0-only
//! The divider between the address bar and the search box (LOOK-012).
//!
//! As in Windows 11 File Explorer, the search box starts 499 pixels wide
//! (measured on the reference shots ref-01, ref-24 and ref-29 of a 1920
//! pixel window: x 1410 to 1908) and 8 pixels after the address bar,
//! which takes the rest of the row. Dragging the divider between them
//! changes the search box's width, which every window then starts with,
//! as Windows remembers it. When the window grows or shrinks, the address
//! bar takes the difference and the search box keeps its width, until the
//! address bar is at its least; the search box then shrinks, down to its
//! least width. How Windows shares the row in narrow windows is not
//! measured yet.
//!
//! The paned tells of each new layout through its position, which GTK
//! moves as the row's width changes (the address bar takes the
//! difference, as its resize flags in `window.ui` ask); the divider is
//! then put back where the search box gets its wanted width, as far as
//! the address bar's least width allows. A move of the divider while the
//! row keeps its width is the user's drag, whose width is saved.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::preferences::Preference;
use super::BrowserWindow;

/// Windows 11's default search box width.
pub(super) const DEFAULT_SEARCH_WIDTH: i32 = 499;

/// How long after the divider last moved its width is saved, so a drag
/// saves once.
const SAVE_DELAY: Duration = Duration::from_millis(300);

/// The divider's state: the search box's wanted width, the row width it
/// last saw, so a move that comes with a new width is the window's, not a
/// drag, and a pending save.
#[derive(Default)]
struct DividerState {
    wanted: Cell<i32>,
    row_width: Cell<i32>,
    save: Cell<Option<glib::SourceId>>,
}

/// The width the divider's handle takes, 8 pixels as Windows' gap
/// (`paned.address-search > separator` in navigation.css); before its
/// first layout the handle has no width yet to read.
fn handle_width(paned: &gtk::Paned) -> i32 {
    super::sidebar_resizer::paned_handle(paned)
        .map_or(HANDLE_WIDTH, |handle| handle.width().max(HANDLE_WIDTH))
}

/// The handle's width in the stylesheet.
const HANDLE_WIDTH: i32 = 8;

impl BrowserWindow {
    /// Gives the search box the saved or default width and saves a width
    /// the user drags the divider to.
    pub(super) fn install_search_divider(&self) {
        let saved = self.context().settings_data().preferences.search_box_width;
        let wanted = saved
            .and_then(|width| i32::try_from(width).ok())
            .unwrap_or(DEFAULT_SEARCH_WIDTH);
        let paned = self.imp().address_search.get();
        let state = Rc::new(DividerState::default());
        state.wanted.set(wanted);
        paned.connect_notify_local(
            Some("position"),
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[strong]
                state,
                move |paned, _| window.follow_search_divider(paned, &state)
            ),
        );
    }

    /// Saves the search box's width after the user moved the divider: a
    /// move while the row kept its width.
    fn follow_search_divider(&self, paned: &gtk::Paned, state: &Rc<DividerState>) {
        let total = paned.width();
        if total <= 0 {
            return;
        }
        // A new row width, the first one too: the divider goes where the
        // search box gets its wanted width, as far as the address bar's
        // least width allows, so the box comes back to it when the window
        // grows again.
        let resized = state.row_width.replace(total) != total;
        if resized || !paned.is_position_set() {
            let wanted = total - handle_width(paned) - state.wanted.get();
            let position = wanted.clamp(
                paned.min_position(),
                paned.max_position().max(paned.min_position()),
            );
            if position != paned.position() || !paned.is_position_set() {
                paned.set_position(position);
            }
            return;
        }
        let width = total - handle_width(paned) - paned.position();
        if let Some(pending) = state.save.take() {
            pending.remove();
        }
        let source = glib::timeout_add_local_once(
            SAVE_DELAY,
            glib::clone!(
                #[weak(rename_to = window)]
                self,
                #[strong]
                state,
                move || {
                    state.save.set(None);
                    state.wanted.set(width);
                    window.save_preference(Preference::SearchBoxWidth(width));
                }
            ),
        );
        state.save.set(Some(source));
    }
}
