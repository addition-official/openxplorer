// SPDX-License-Identifier: AGPL-3.0-only
//! Glyphs from the Windows symbol font the user chose (LOOK-031).
//!
//! Windows 11 File Explorer draws its command bar, menus, navigation
//! buttons and chevrons with the Segoe Fluent Icons font. When Windows
//! icon files are on and the user chose `SegoeIcons.ttf` (or Segoe MDL2
//! Assets, `segmdl2.ttf`), [`follow_preferences`] has the glyphs the app
//! uses read off the main thread ([`ox_core::windows_glyphs`]) and written
//! as symbolic icons into the user's cache, one per bundled icon
//! ([`glyph_of`] names its codepoint, from Microsoft's published table of
//! Segoe Fluent Icons). That folder joins the icon theme's search path,
//! and [`drawn_name`] makes every [`super::set_icon`] show the glyph's
//! icon instead of the bundled one, so GTK draws it at the same size and
//! in the same CSS colour, following the theme. The chevrons the skin
//! draws in CSS follow through an override stylesheet. A glyph the font
//! lacks keeps the bundled icon, and turning the font off brings every
//! bundled icon back at once.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{gdk, glib};
use ox_core::settings::Preferences;
use ox_core::windows_glyphs::LoadedGlyphs;

use super::icon::ALL_ICONS;
use super::Icon;

/// A glyph of Segoe Fluent Icons: its codepoint and the name Microsoft's
/// table gives it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowsGlyph {
    /// Its codepoint, in the Private Use Area.
    pub(crate) codepoint: u32,
    /// Its name in Microsoft's table.
    pub(crate) name: &'static str,
}

/// Where a bundled icon's Windows counterpart comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowsSource {
    /// A glyph of the symbol font.
    Glyph(WindowsGlyph),
    /// Not a glyph: a colour picture the Windows icon files draw by role
    /// (LOOK-030), or that only the app draws.
    Picture,
    /// No glyph of Windows matches; the reason.
    None(&'static str),
}

/// A glyph named `name` at `codepoint`.
const fn glyph(codepoint: u32, name: &'static str) -> WindowsSource {
    WindowsSource::Glyph(WindowsGlyph { codepoint, name })
}

/// Why an icon kept from an earlier design has no glyph.
const UNUSED: &str = "kept from an earlier design; not drawn";

/// Where `icon`'s Windows counterpart comes from: the Segoe Fluent Icons
/// glyph Windows 11 draws in the same place, by Microsoft's published
/// codepoints. An exhaustive table, hence its length.
#[expect(clippy::too_many_lines, reason = "one line per bundled icon")]
pub(crate) const fn windows_source(icon: Icon) -> WindowsSource {
    match icon {
        Icon::Add => glyph(0xE710, "Add"),
        Icon::Apps => glyph(0xE71D, "AllApps"),
        Icon::AppsListDetail => glyph(0xECA5, "Tiles"),
        Icon::ArrowClockwise => glyph(0xE72C, "Refresh"),
        Icon::ArrowCounterclockwise => glyph(0xE777, "UpdateRestore"),
        Icon::ArrowDown => glyph(0xE74B, "Down"),
        Icon::ArrowEject => glyph(0xF847, "Eject"),
        Icon::ArrowDownload => glyph(0xE896, "Download"),
        Icon::ArrowLeft => glyph(0xE72B, "Back"),
        Icon::ArrowRedo => glyph(0xE7A6, "Redo"),
        Icon::ArrowRight => glyph(0xE72A, "Forward"),
        Icon::ArrowSort => glyph(0xE8CB, "Sort"),
        Icon::ArrowUndo => glyph(0xE7A7, "Undo"),
        Icon::ArrowUp => glyph(0xE74A, "Up"),
        Icon::Bluetooth => glyph(0xE702, "Bluetooth"),
        Icon::CheckboxChecked => glyph(0xE73A, "CheckboxComposite"),
        Icon::Checkmark => glyph(0xE73E, "CheckMark"),
        Icon::ChevronDown => glyph(0xE70D, "ChevronDown"),
        Icon::ChevronDown16 => glyph(0xE96E, "ChevronDownSmall"),
        Icon::ChevronLeft16 => glyph(0xE96F, "ChevronLeftSmall"),
        Icon::ChevronRight16 => glyph(0xE970, "ChevronRightSmall"),
        Icon::ChevronUp16 => glyph(0xE96D, "ChevronUpSmall"),
        Icon::CircleSmallFilled => glyph(0xE915, "RadioBullet"),
        Icon::ClipboardPaste => glyph(0xE77F, "Paste"),
        Icon::Clock => glyph(0xE917, "Clock"),
        Icon::Code => glyph(0xE943, "Code"),
        Icon::Copy => glyph(0xE8C8, "Copy"),
        Icon::Cut => glyph(0xE8C6, "Cut"),
        Icon::Delete => glyph(0xE74D, "Delete"),
        Icon::Desktop => glyph(0xE7F4, "TVMonitor"),
        Icon::Dismiss => glyph(0xE8BB, "ChromeClose"),
        Icon::Dismiss16 => glyph(0xE711, "Cancel"),
        Icon::Document => glyph(0xE8A5, "Document"),
        Icon::Eye => glyph(0xE890, "View"),
        Icon::Folder => glyph(0xE8B7, "Folder"),
        Icon::FolderAdd => glyph(0xE8F4, "NewFolder"),
        Icon::FolderZip => glyph(0xF012, "ZipFolder"),
        Icon::Grid => glyph(0xE8A9, "ViewAll"),
        Icon::HardDrive => glyph(0xEDA2, "HardDrive"),
        Icon::History => glyph(0xE81C, "History"),
        Icon::Home => glyph(0xE80F, "Home"),
        Icon::Image => glyph(0xE8B9, "Picture"),
        Icon::Info => glyph(0xE946, "Info"),
        Icon::Laptop => glyph(0xE7F8, "DeviceLaptopNoPic"),
        Icon::Link => glyph(0xE71B, "Link"),
        Icon::MapNetworkDrive24 => glyph(0xE8CE, "MapDrive"),
        Icon::Maximize => glyph(0xE922, "ChromeMaximize"),
        Icon::MoreHorizontal => glyph(0xE712, "More"),
        Icon::MusicNote => glyph(0xE90B, "MusicInfo"),
        Icon::Open => glyph(0xE8E5, "OpenFile"),
        Icon::Organization => glyph(0xE968, "Network"),
        Icon::PaintBrush => glyph(0xE771, "Personalize"),
        Icon::PanelLeft => glyph(0xE90C, "DockLeft"),
        Icon::PanelRight => glyph(0xE90D, "DockRight"),
        Icon::PanelRightGallery => glyph(0xE8A1, "PreviewLink"),
        Icon::Pause16 => glyph(0xE769, "Pause"),
        Icon::Phone => glyph(0xE8EA, "CellPhone"),
        Icon::Play16 => glyph(0xE768, "Play"),
        Icon::Pin => glyph(0xE718, "Pin"),
        Icon::Rename => glyph(0xE8AC, "Rename"),
        Icon::Search => glyph(0xE721, "Search"),
        Icon::SelectAll24 => glyph(0xE8B3, "SelectAll"),
        Icon::SelectNone24 => glyph(0xE8E6, "ClearSelection"),
        Icon::Settings => glyph(0xE713, "Settings"),
        Icon::Share => glyph(0xE72D, "Share"),
        Icon::ShieldLock => glyph(0xEA18, "Shield"),
        Icon::SquareMultiple => glyph(0xE923, "ChromeRestore"),
        Icon::Star => glyph(0xE734, "FavoriteStar"),
        Icon::StarFilled => glyph(0xE735, "FavoriteStarFill"),
        Icon::Subtract => glyph(0xE921, "ChromeMinimize"),
        Icon::TextBulletList => glyph(0xE8FD, "BulletedList"),
        Icon::TextColumnTwo => glyph(0xEA37, "List"),
        Icon::Video => glyph(0xE714, "Video"),
        Icon::WeatherMoon => glyph(0xE708, "QuietHours"),
        Icon::WeatherSunny => glyph(0xE706, "Brightness"),
        Icon::WindowConsole => glyph(0xE756, "CommandPrompt"),
        Icon::WindowMultiple => glyph(0xE7C4, "TaskView"),
        Icon::WindowNew => glyph(0xE78B, "NewWindow"),
        Icon::Wrench => glyph(0xE90F, "Repair"),
        Icon::AddNetworkLocation24
        | Icon::DisconnectNetworkDrive24
        | Icon::InvertSelection24
        | Icon::Options24 => WindowsSource::None(
            "an original glyph matching Windows 11's; Microsoft's table names none for it",
        ),
        Icon::ArrowAutofitHeight | Icon::ContentView | Icon::RectangleLandscape | Icon::SearchSettings => {
            WindowsSource::None("Microsoft's table names no glyph for this command")
        }
        Icon::ArrowReset
        | Icon::DeleteDismiss
        | Icon::Server
        | Icon::ChevronDoubleLeft16
        | Icon::ChevronDoubleRight16 => WindowsSource::None(
            "no glyph of this name in Microsoft's table (Reset, EmptyRecycleBin, Server, double chevrons)",
        ),
        Icon::DocumentAdd | Icon::DocumentCopy | Icon::DocumentText => {
            WindowsSource::None("Windows shows the file type's picture here, not a glyph")
        }
        Icon::DismissCircleFilled => WindowsSource::None("a coloured status badge, not a command glyph"),
        Icon::ArrowSwap
        | Icon::Braces
        | Icon::Markdown
        | Icon::SelectAllOff
        | Icon::SelectAllOn
        | Icon::Table => WindowsSource::None(UNUSED),
        Icon::CodeColor20
        | Icon::CodeColor24
        | Icon::DismissCircleColor32
        | Icon::DocumentColor20
        | Icon::DocumentColor32
        | Icon::DocumentColor48
        | Icon::DocumentTextColor20
        | Icon::DocumentTextColor32
        | Icon::DocumentTextColor48
        | Icon::FileFolder
        | Icon::HeadphonesColor20
        | Icon::HeadphonesColor32
        | Icon::HeadphonesColor48
        | Icon::ImageColor20
        | Icon::ImageColor32
        | Icon::ImageColor48
        | Icon::MailColor20
        | Icon::TableColor20
        | Icon::TableColor32
        | Icon::TableColor48
        | Icon::VideoColor20
        | Icon::VideoColor32
        | Icon::VideoColor48
        | Icon::WarningColor32 => WindowsSource::Picture,
    }
}

/// The glyph of `icon`, when Windows draws one in its place.
pub(crate) const fn glyph_of(icon: Icon) -> Option<WindowsGlyph> {
    match windows_source(icon) {
        WindowsSource::Glyph(glyph) => Some(glyph),
        WindowsSource::Picture | WindowsSource::None(_) => None,
    }
}

/// The name of the icon the cache draws `icon`'s glyph as:
/// `ox-win-add-20-symbolic` for `ox-add-20-symbolic`.
fn glyph_icon_name(icon: Icon) -> String {
    let name = icon.name();
    format!("ox-win-{}", name.strip_prefix("ox-").unwrap_or(name))
}

/// The bundled icon a drawn icon name stands for: itself, or the icon a
/// glyph's name was made from.
fn icon_named(name: &str) -> Option<Icon> {
    let bundled = name.strip_prefix("ox-win-").map(|rest| format!("ox-{rest}"));
    let name = bundled.as_deref().unwrap_or(name);
    ALL_ICONS.into_iter().find(|icon| icon.name() == name)
}

/// Where the glyphs of the chosen font stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Off, or no font chosen: the app's own glyphs.
    Off,
    /// The chosen font is being read.
    Reading,
    /// The font was read and draws `found` of `wanted` glyphs.
    Ready {
        /// How many glyphs it draws.
        found: usize,
        /// How many the app asked for.
        wanted: usize,
    },
}

/// The glyphs of the chosen font, on the main thread.
struct State {
    /// The font to read while the setting is on; `None` while off.
    wanted: Option<PathBuf>,
    /// Counts every change of [`Self::wanted`].
    generation: u64,
    /// The glyphs the font gave.
    loaded: Option<LoadedGlyphs>,
    /// The cache folder added to the icon theme's search path.
    searched: Option<PathBuf>,
    /// The stylesheet that draws the skin's chevrons from the font.
    stylesheet: Option<gtk::CssProvider>,
    /// The glyph icon names handed out, made once per icon.
    names: HashMap<&'static str, &'static str>,
    status: Status,
    /// Called with the status whenever it changes, until they return false.
    followers: Vec<Box<dyn Fn(Status) -> bool>>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        wanted: None,
        generation: 0,
        loaded: None,
        searched: None,
        stylesheet: None,
        names: HashMap::new(),
        status: Status::Off,
        followers: Vec::new(),
    });
}

/// The CSS priority of the chevrons' stylesheet: above every layer of
/// the skin.
const STYLESHEET_PRIORITY: u32 = gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 5;

/// The icon name `icon` is drawn by: its glyph from the chosen font when
/// the font has one, else its own.
pub(crate) fn drawn_name(icon: Icon) -> &'static str {
    let own = icon.name();
    if glyph_of(icon).is_none() {
        return own;
    }
    STATE.with_borrow_mut(|state| {
        let has_glyph = state
            .loaded
            .as_ref()
            .is_some_and(|loaded| loaded.contains(&glyph_icon_name(icon)));
        if !has_glyph {
            return own;
        }
        // At most one name per bundled icon is ever made, so keeping them
        // for the life of the process is bounded.
        state
            .names
            .entry(own)
            .or_insert_with(|| Box::leak(glyph_icon_name(icon).into_boxed_str()))
    })
}

/// Reads the chosen font when Windows icon files are on and a font is
/// chosen, or goes back to the app's own glyphs. Called at startup and
/// whenever the settings change; nothing happens when the choice is the
/// same.
pub(crate) fn follow_preferences(preferences: &Preferences) {
    let wanted = preferences
        .use_icon_files
        .then(|| preferences.symbol_font.as_ref().map(PathBuf::from))
        .flatten();
    let generation = STATE.with_borrow_mut(|state| {
        if state.wanted == wanted {
            return None;
        }
        state.wanted.clone_from(&wanted);
        state.generation += 1;
        state.loaded = None;
        state.status = if wanted.is_some() {
            Status::Reading
        } else {
            Status::Off
        };
        Some(state.generation)
    });
    let Some(generation) = generation else {
        return;
    };
    apply();
    let Some(font) = wanted else {
        return;
    };
    let glyphs: Vec<(String, u32)> = ALL_ICONS
        .into_iter()
        .filter_map(|icon| Some((glyph_icon_name(icon), glyph_of(icon)?.codepoint)))
        .collect();
    let cache = ox_core::windows_glyphs::glyph_cache_folder();
    glib::spawn_future_local(async move {
        let read =
            gio::spawn_blocking(move || ox_core::windows_glyphs::load_glyphs(&font, &glyphs, &cache)).await;
        let loaded = read.unwrap_or_default();
        let current = STATE.with_borrow_mut(|state| {
            if state.generation != generation {
                return false;
            }
            state.status = Status::Ready {
                found: loaded.found(),
                wanted: glyph_count(),
            };
            state.loaded = Some(loaded);
            true
        });
        if current {
            apply();
        }
    });
}

/// How many bundled icons have a glyph of the font.
pub(crate) fn glyph_count() -> usize {
    ALL_ICONS
        .into_iter()
        .filter(|icon| glyph_of(*icon).is_some())
        .count()
}

/// Where the glyphs of the chosen font stand now.
pub(crate) fn status() -> Status {
    STATE.with_borrow(|state| state.status)
}

/// Calls `follower` with the status now and whenever it changes, until it
/// returns false.
pub(crate) fn follow_status(follower: impl Fn(Status) -> bool + 'static) {
    if follower(status()) {
        STATE.with_borrow_mut(|state| state.followers.push(Box::new(follower)));
    }
}

/// Tells the status followers, dropping those that are gone.
fn tell_followers() {
    let (followers, status) =
        STATE.with_borrow_mut(|state| (std::mem::take(&mut state.followers), state.status));
    let kept: Vec<Box<dyn Fn(Status) -> bool>> = followers
        .into_iter()
        .filter(|follower| follower(status))
        .collect();
    STATE.with_borrow_mut(|state| {
        let added = std::mem::replace(&mut state.followers, kept);
        state.followers.extend(added);
    });
}

/// Makes the display draw what the state holds: the icon theme searches
/// the glyphs' folder, the chevrons' stylesheet draws theirs, and every
/// image on screen shows its icon again.
fn apply() {
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let theme = gtk::IconTheme::for_display(&display);
    let (old, new, stylesheet) = STATE.with_borrow_mut(|state| {
        let new = state
            .loaded
            .as_ref()
            .and_then(LoadedGlyphs::folder)
            .map(std::path::Path::to_path_buf);
        (state.searched.take(), new, state.stylesheet.take())
    });
    let mut paths = theme.search_path();
    if let Some(old) = &old {
        paths.retain(|path| path != old);
    }
    if let Some(new) = &new {
        paths.push(new.clone());
    }
    if old != new {
        theme.set_search_path(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    }
    if let Some(stylesheet) = stylesheet {
        gtk::style_context_remove_provider_for_display(&display, &stylesheet);
    }
    let css = chevron_stylesheet(crate::theme::skin_rules(), drawn_name);
    let stylesheet = (!css.is_empty()).then(|| {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(&css);
        gtk::style_context_add_provider_for_display(&display, &provider, STYLESHEET_PRIORITY);
        provider
    });
    STATE.with_borrow_mut(|state| {
        state.searched = new;
        state.stylesheet = stylesheet;
    });
    redraw_images();
    tell_followers();
}

/// The rules of `rules` that draw a bundled icon through
/// `-gtk-icontheme`, with only those declarations, the icon names
/// replaced as `drawn` draws them; empty when `drawn` replaces none.
pub(crate) fn chevron_stylesheet(rules: &str, drawn: impl Fn(Icon) -> &'static str) -> String {
    let mut text = String::with_capacity(rules.len());
    let mut rest = rules;
    // Comments out, so a brace inside one cannot split a rule.
    while let Some(start) = rest.find("/*") {
        text.push_str(&rest[..start]);
        rest = rest[start..]
            .find("*/")
            .map_or("", |end| &rest[start + end + 2..]);
    }
    text.push_str(rest);
    let mut stylesheet = String::new();
    for rule in text.split('}') {
        let Some((selector, body)) = rule.split_once('{') else {
            continue;
        };
        let mut replaced = false;
        let declarations: Vec<String> = body
            .split(';')
            .filter(|declaration| declaration.contains("-gtk-icontheme(\"ox-"))
            .map(|declaration| {
                let mut declaration = declaration.trim().to_owned();
                for icon in ALL_ICONS {
                    let quoted = format!("\"{}\"", icon.name());
                    let drawn_as = drawn(icon);
                    if drawn_as != icon.name() && declaration.contains(&quoted) {
                        declaration = declaration.replace(&quoted, &format!("\"{drawn_as}\""));
                        replaced = true;
                    }
                }
                declaration
            })
            .collect();
        if replaced {
            stylesheet.push_str(selector.trim());
            stylesheet.push_str(" { ");
            stylesheet.push_str(&declarations.join("; "));
            stylesheet.push_str("; }\n");
        }
    }
    stylesheet
}

/// Shows every bundled glyph on screen again as [`drawn_name`] draws it
/// now: each image of every window whose icon is a bundled icon or a
/// glyph's.
fn redraw_images() {
    for window in gtk::Window::list_toplevels() {
        let mut pending = vec![window];
        while let Some(widget) = pending.pop() {
            let mut child = widget.first_child();
            while let Some(current) = child {
                child = current.next_sibling();
                pending.push(current);
            }
            let Some(image) = widget.downcast_ref::<gtk::Image>() else {
                continue;
            };
            let Some(icon) = image.icon_name().as_deref().and_then(icon_named) else {
                continue;
            };
            if glyph_of(icon).is_some() {
                super::set_icon(image, icon, image.pixel_size());
            }
        }
    }
}

/// Waits until the chosen font was read, for tests.
#[cfg(test)]
pub(crate) fn wait_until_read() {
    crate::test_support::harness::wait_until("the symbol font to be read", || {
        !matches!(status(), Status::Reading)
    });
}

#[cfg(test)]
mod tests;
