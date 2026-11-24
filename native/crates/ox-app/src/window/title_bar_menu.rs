// SPDX-License-Identifier: AGPL-3.0-only
//! The title bar's window menu (LOOK-032).
//!
//! A right-click on the empty part of the title bar, beside the tabs,
//! opens the window menu Windows 11 shows there: Restore, Move, Size,
//! Minimize, Maximize, then Close (Alt+F4), with Restore enabled only on a
//! maximized window and Move, Size and Maximize only on a window that is
//! not. Under them, as Chrome offers "Use system title bar and borders"
//! there, a check item turns the desktop's own title bar and borders on or
//! off for every open window ([`super::title_bar`]). A right-click on a
//! tab, the "+" or a caption button keeps its own menu or none.
//!
//! With the desktop's frame the tab row is the first row of the window's
//! content; it is still a [`gtk::WindowHandle`], and its empty part has
//! the same menu.

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use ox_core::settings::PreferencesUpdate;

use crate::icons::Icon;

use super::actions::plain_action;
use super::menu_popover::{ItemCheck, MenuEntry, MenuItem, MenuPopover};
use super::window_action::WindowAction;
use super::BrowserWindow;

/// The widget name of the kept window menu on the tab row.
const MENU_NAME: &str = "title-bar-menu";

/// The window menu's Move or Size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameDrag {
    /// Move: the whole window follows the pointer.
    Move,
    /// Size: the window's lower right corner follows the pointer.
    Size,
}

impl FrameDrag {
    /// The pointer shown while it waits for a press on Wayland.
    const fn cursor(self) -> &'static str {
        match self {
            FrameDrag::Move => "move",
            FrameDrag::Size => "se-resize",
        }
    }
}

/// What the window menu depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WindowMenuFacts {
    /// The window is maximized (or full screen): Restore is enabled, and
    /// Move, Size and Maximize are not.
    pub maximized: bool,
    /// The desktop draws the title bar and borders.
    pub system_title_bar: bool,
}

/// The window menu, in Windows 11's order, then the system title bar
/// choice.
pub(super) fn window_menu(facts: WindowMenuFacts) -> Vec<MenuEntry> {
    let restore = MenuItem::new(
        &ox_core::i18n::gettext("Restore"),
        Icon::SquareMultiple,
        WindowAction::RestoreWindow,
    )
    .disabled_when(!facts.maximized);
    let moving = MenuItem::new(
        &ox_core::i18n::gettext("Move"),
        Icon::Dismiss,
        WindowAction::MoveWindow,
    )
    .without_glyph()
    .disabled_when(facts.maximized);
    let size = MenuItem::new(
        &ox_core::i18n::gettext("Size"),
        Icon::Dismiss,
        WindowAction::SizeWindow,
    )
    .without_glyph()
    .disabled_when(facts.maximized);
    let minimize = MenuItem::new(
        &ox_core::i18n::gettext("Minimize"),
        Icon::Subtract,
        WindowAction::MinimizeWindow,
    );
    let maximize = MenuItem::new(
        &ox_core::i18n::gettext("Maximize"),
        Icon::Maximize,
        WindowAction::MaximizeWindow,
    )
    .disabled_when(facts.maximized);
    let close = MenuItem::new(
        &ox_core::i18n::gettext("Close"),
        Icon::Dismiss,
        WindowAction::CloseWindow,
    )
    .with_shortcut("Alt+F4");
    let mut system = MenuItem::new(
        &ox_core::i18n::gettext("Use system title bar and borders"),
        Icon::Dismiss,
        WindowAction::SystemTitleBar,
    )
    .without_glyph();
    system.check = ItemCheck::Fixed(facts.system_title_bar);
    vec![
        restore.into(),
        moving.into(),
        size.into(),
        minimize.into(),
        maximize.into(),
        MenuEntry::Divider,
        close.into(),
        MenuEntry::Divider,
        system.into(),
    ]
}

impl BrowserWindow {
    /// Adds the window menu's actions and opens the menu on a right-click
    /// on the empty part of the tab row.
    pub(super) fn install_title_bar_menu(&self) {
        self.add_action_entries([
            plain_action(WindowAction::RestoreWindow, GtkWindowExt::unmaximize),
            plain_action(WindowAction::MoveWindow, |window| {
                window.begin_frame_drag(FrameDrag::Move);
            }),
            plain_action(WindowAction::SizeWindow, |window| {
                window.begin_frame_drag(FrameDrag::Size);
            }),
            plain_action(WindowAction::MinimizeWindow, GtkWindowExt::minimize),
            plain_action(WindowAction::MaximizeWindow, GtkWindowExt::maximize),
            plain_action(
                WindowAction::SystemTitleBar,
                BrowserWindow::toggle_system_title_bar,
            ),
        ]);
        let press = gtk::GestureClick::new();
        press.set_button(gdk::BUTTON_SECONDARY);
        // Before the tab row's own handling, which would show the
        // desktop's window menu instead.
        press.set_propagation_phase(gtk::PropagationPhase::Capture);
        press.connect_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |gesture, _, x, y| {
                if window.is_on_empty_title_bar(x, y) {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    window.show_window_menu_at(x, y);
                }
            }
        ));
        self.imp().tab_bar.add_controller(press);
        self.install_armed_frame_drag();
    }

    /// Whether (`x`, `y`) of the tab row is on its empty part: not on a
    /// tab, a button (the "+", a scroll arrow, a caption) or anything else
    /// that takes clicks of its own.
    fn is_on_empty_title_bar(&self, x: f64, y: f64) -> bool {
        let bar = self.imp().tab_bar.get();
        let Some(target) = bar.pick(x, y, gtk::PickFlags::DEFAULT) else {
            return false;
        };
        let bar: gtk::Widget = bar.upcast();
        std::iter::successors(Some(target), gtk::Widget::parent)
            .take_while(|widget| *widget != bar)
            .all(|widget| !widget.is::<gtk::Button>() && !widget.has_css_class("tab"))
    }

    /// Opens the window menu at (`x`, `y`) of the tab row.
    fn show_window_menu_at(&self, x: f64, y: f64) {
        let menu = MenuPopover::kept_by(&self.imp().tab_bar.get(), MENU_NAME);
        menu.set_entries(window_menu(WindowMenuFacts {
            maximized: self.is_maximized() || self.is_fullscreen(),
            system_title_bar: self
                .context()
                .settings_data()
                .preferences
                .system_window_decorations,
        }));
        #[expect(clippy::cast_possible_truncation, reason = "pointer positions are small")]
        let target = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
        menu.set_pointing_to(Some(&target));
        menu.popup();
    }

    /// Unparents the window menu, if it was opened, as the window goes.
    pub(super) fn drop_title_bar_menu(&self) {
        let bar: gtk::Widget = self.imp().tab_bar.get().upcast();
        let menus: Vec<MenuPopover> = super::widget_tree::children(&bar)
            .filter_map(|child| child.downcast::<MenuPopover>().ok())
            .filter(|menu| menu.widget_name() == MENU_NAME)
            .collect();
        for menu in menus {
            menu.unparent();
        }
    }

    /// The window menu, once it was opened, for tests.
    #[cfg(test)]
    pub(super) fn title_bar_menu(&self) -> Option<MenuPopover> {
        super::widget_tree::children(self.imp().tab_bar.upcast_ref::<gtk::Widget>())
            .filter_map(|child| child.downcast::<MenuPopover>().ok())
            .find(|menu| menu.widget_name() == MENU_NAME)
    }

    /// Move or Size, as `drag` says: the window, or its lower right
    /// corner, follows the pointer until a click. On X11, button 0 asks the window
    /// manager for its keyboard move or size, as Windows' Move and Size
    /// follow the pointer and the arrow keys at once. A Wayland compositor
    /// starts a move or a size only from a button held down, so there the
    /// next press in the window starts it, under the move or resize
    /// pointer, and the window follows that drag; Escape gives up.
    fn begin_frame_drag(&self, drag: FrameDrag) {
        let display = WidgetExt::display(self);
        if display.type_().name() == "GdkWaylandDisplay" {
            self.imp().armed_frame_drag.set(Some(drag));
            self.set_cursor_from_name(Some(drag.cursor()));
            return;
        }
        let Some(pointer) = display.default_seat().and_then(|seat| seat.pointer()) else {
            return;
        };
        let Some(toplevel) = self.surface().and_downcast::<gdk::Toplevel>() else {
            return;
        };
        let surface: gdk::Surface = toplevel.clone().upcast();
        let (x, y) = surface
            .device_position(&pointer)
            .map_or((0.0, 0.0), |(x, y, _)| (x, y));
        start_frame_drag(&toplevel, drag, &pointer, 0, (x, y), gdk::CURRENT_TIME);
    }

    /// Hands the press of `gesture` at (`x`, `y`) of the window to the
    /// compositor as the Move or Size the window menu armed, if it did.
    fn start_armed_frame_drag(&self, gesture: &gtk::GestureClick, x: f64, y: f64) -> bool {
        let Some(drag) = self.imp().armed_frame_drag.take() else {
            return false;
        };
        self.set_cursor(None::<&gdk::Cursor>);
        let (Some(toplevel), Some(device)) = (
            self.surface().and_downcast::<gdk::Toplevel>(),
            gesture.current_event_device(),
        ) else {
            return true;
        };
        let (offset_x, offset_y) = NativeExt::surface_transform(self);
        let button = i32::try_from(gesture.current_button()).unwrap_or(1);
        start_frame_drag(
            &toplevel,
            drag,
            &device,
            button,
            (x + offset_x, y + offset_y),
            gesture.current_event_time(),
        );
        true
    }

    /// Lets a press start the Move or Size the window menu armed on
    /// Wayland, before anything in the window sees it, and Escape give it
    /// up.
    fn install_armed_frame_drag(&self) {
        let press = gtk::GestureClick::new();
        press.set_button(0);
        press.set_propagation_phase(gtk::PropagationPhase::Capture);
        press.connect_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            move |gesture, _, x, y| {
                if window.start_armed_frame_drag(gesture, x, y) {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                }
            }
        ));
        self.add_controller(press);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed(glib::clone!(
            #[weak(rename_to = window)]
            self,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, key, _, _| {
                if key == gdk::Key::Escape && window.imp().armed_frame_drag.take().is_some() {
                    window.set_cursor(None::<&gdk::Cursor>);
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            }
        ));
        self.add_controller(keys);
    }

    /// "Use system title bar and borders": saves the opposite choice; every
    /// open window follows it at once ([`Self::follow_window_decorations`]).
    fn toggle_system_title_bar(&self) {
        let on = !self
            .context()
            .settings_data()
            .preferences
            .system_window_decorations;
        self.context().update_preferences(
            PreferencesUpdate {
                system_window_decorations: Some(on),
                ..PreferencesUpdate::default()
            },
            self.preference_failure_reply(),
        );
    }
}

/// Asks the compositor or window manager to move `toplevel`, or to size
/// it from its lower right corner, as `drag` says, with `device` from
/// `at` (surface coordinates).
fn start_frame_drag(
    toplevel: &gdk::Toplevel,
    drag: FrameDrag,
    device: &gdk::Device,
    button: i32,
    at: (f64, f64),
    time: u32,
) {
    match drag {
        FrameDrag::Move => toplevel.begin_move(device, button, at.0, at.1, time),
        FrameDrag::Size => {
            toplevel.begin_resize(
                gdk::SurfaceEdge::SouthEast,
                Some(device),
                button,
                at.0,
                at.1,
                time,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::menu_popover::ItemAvailability;
    use super::*;

    fn labels(entries: &[MenuEntry]) -> Vec<String> {
        entries
            .iter()
            .map(|entry| match entry {
                MenuEntry::Item(item) => item.label.clone(),
                MenuEntry::Divider => "-".to_owned(),
            })
            .collect()
    }

    fn disabled(entries: &[MenuEntry]) -> Vec<String> {
        entries
            .iter()
            .filter_map(|entry| match entry {
                MenuEntry::Item(item) if item.availability == ItemAvailability::Disabled => {
                    Some(item.label.clone())
                }
                _ => None,
            })
            .collect()
    }

    /// The window menu is Windows 11's title bar menu, in its order, with
    /// a separator before Close (Alt+F4), then the system title bar
    /// choice, checked while it is on. On a window that is not maximized
    /// Restore is greyed out, and on a maximized one Move, Size and
    /// Maximize are.
    ///
    /// parity: LOOK-032
    #[test]
    fn the_window_menu_is_windows_with_the_system_title_bar_choice() {
        let normal = window_menu(WindowMenuFacts {
            maximized: false,
            system_title_bar: false,
        });
        let maximized = window_menu(WindowMenuFacts {
            maximized: true,
            system_title_bar: true,
        });

        assert_eq!(
            labels(&normal),
            [
                "Restore",
                "Move",
                "Size",
                "Minimize",
                "Maximize",
                "-",
                "Close",
                "-",
                "Use system title bar and borders"
            ]
        );
        assert_eq!(disabled(&normal), ["Restore"]);
        assert_eq!(disabled(&maximized), ["Move", "Size", "Maximize"]);
        let item = |entries: &[MenuEntry], label: &str| -> MenuItem {
            entries
                .iter()
                .find_map(|entry| match entry {
                    MenuEntry::Item(item) if item.label == label => Some(item.clone()),
                    _ => None,
                })
                .expect("the menu has the item")
        };
        assert_eq!(item(&normal, "Close").shortcut, Some("Alt+F4"));
        assert!(
            !item(&normal, "Use system title bar and borders").shows_glyph,
            "only its check mark"
        );
        assert_eq!(
            item(&normal, "Use system title bar and borders").check,
            ItemCheck::Fixed(false)
        );
        assert_eq!(
            item(&maximized, "Use system title bar and borders").check,
            ItemCheck::Fixed(true)
        );
    }
}
