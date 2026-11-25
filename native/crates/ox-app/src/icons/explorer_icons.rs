// SPDX-License-Identifier: AGPL-3.0-only
//! Windows 11 File Explorer's own command icons, from the folder the user
//! chose (LOOK-034).
//!
//! Windows 11 File Explorer draws its command bar, the View menu and
//! button, its menus' rows, the pane buttons and the address bar's root
//! with small coloured SVG pictures kept beside it, in
//! `FileExplorerExtensions\Assets\images`, one per command and one copy
//! per look: `contrast-standard/theme-dark`, `contrast-standard/theme-light`,
//! and `contrast-black` and `contrast-white` for high contrast. When
//! Windows icon files are on and the user chose that folder (or a copy of
//! it), [`follow_preferences`] lists its files off the main thread, and
//! every image drawn as a command ([`super::command_image`]) or as the
//! address bar's root shows the picture of its command ([`file_of`]),
//! read from the user's folder, in its own colours (Windows' two-tone
//! accent included), scaled from its 16-pixel square. Nothing is bundled
//! or copied: the pictures are read where they are, and a command without
//! a picture there, or a picture that cannot be read, keeps the app's own
//! icon. Turning the folder off brings every icon back at once.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gtk::{gdk, glib};
use ox_core::settings::Preferences;

use super::Icon;

/// Where an image stands, which decides the picture it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Spot {
    /// Not a command: the app's own icon always.
    #[default]
    Plain,
    /// A command of the command bar, a menu row, the View button or a
    /// pane button.
    Command,
    /// The icon at the left end of the address bar.
    AddressRoot,
}

/// The look whose copy of the pictures is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Look {
    /// `contrast-standard/theme-light`.
    Light,
    /// `contrast-standard/theme-dark`.
    Dark,
    /// `contrast-white`: high contrast on light surfaces.
    HighContrastLight,
    /// `contrast-black`: high contrast on dark surfaces.
    HighContrastDark,
}

impl Look {
    /// The folder of this look's pictures, under the chosen folder.
    pub(crate) const fn folder(self) -> &'static str {
        match self {
            Self::Light => "contrast-standard/theme-light",
            Self::Dark => "contrast-standard/theme-dark",
            Self::HighContrastLight => "contrast-white",
            Self::HighContrastDark => "contrast-black",
        }
    }

    /// Every look, in the order the folders are listed.
    const ALL: [Self; 4] = [
        Self::Light,
        Self::Dark,
        Self::HighContrastLight,
        Self::HighContrastDark,
    ];

    /// The look for a dark or light appearance, in high contrast or not.
    const fn of(dark: bool, high_contrast: bool) -> Self {
        match (dark, high_contrast) {
            (false, false) => Self::Light,
            (true, false) => Self::Dark,
            (false, true) => Self::HighContrastLight,
            (true, true) => Self::HighContrastDark,
        }
    }
}

/// The picture Windows 11 File Explorer draws for the command `icon`
/// stands for in a command spot: the file's name without `.svg`, as in
/// its `Assets\images` folder. An exhaustive table, hence its length.
pub(crate) const fn command_file(icon: Icon) -> Option<&'static str> {
    Some(match icon {
        // The command bar.
        Icon::Add => "windows.newitem",
        Icon::Cut => "windows.cut",
        Icon::Copy => "windows.copy",
        Icon::ClipboardPaste => "windows.paste",
        Icon::Rename => "windows.rename",
        Icon::Share => "Windows.ModernShare",
        Icon::Delete => "windows.ribbondelete",
        Icon::ArrowSort => "SortBy",
        Icon::Grid => "View",
        Icon::FolderAdd => "newfolder",
        Icon::Link => "link",
        Icon::FolderZip => "Windows.Zip.Action",
        Icon::RotateLeft => "windows.rotate270",
        Icon::RotateRight => "windows.rotate90",
        Icon::SetAsBackground => "windows.setdesktopwallpaper",
        Icon::ArrowEject => "windows.eject",
        Icon::MapNetworkDrive24 => "windows.mapnetworkdrive",
        Icon::DisconnectNetworkDrive24 => "windows.disconnectnetworkdrive",
        Icon::AddNetworkLocation24 => "windows.addnetworklocation",
        // The View menu and the View button.
        Icon::TextDescription => "windows.iconsize.details",
        Icon::ViewExtraLarge => "windows.iconsize.extralarge",
        Icon::ViewLarge => "windows.iconsize.large",
        Icon::ViewMedium => "windows.iconsize.medium",
        Icon::ViewSmall => "windows.iconsize.smallicon",
        Icon::TextColumnTwo => "windows.iconsize.list",
        Icon::AppsListDetail => "windows.iconsize.tile",
        Icon::ContentView => "windows.iconsize.content",
        Icon::ArrowAutofitHeight => "windows.compactmode",
        Icon::PanelLeft => "windows.navpane",
        Icon::PanelRight => "windows.readingpane",
        Icon::PanelRightGallery => "windows.previewpane",
        Icon::CheckboxChecked => "windows.selectioncheckboxes",
        Icon::Eye => "windows.showhiddenfiles",
        // The "…" menu.
        Icon::ArrowUndo => "windows.undo",
        Icon::ArrowRedo => "windows.redo",
        Icon::SelectAll24 => "windows.selectall",
        Icon::SelectNone24 => "windows.selectnone",
        Icon::InvertSelection24 => "windows.invertselection",
        Icon::Options24 => "windows.folderoptions",
        // The menus' rows.
        Icon::OpenInNewTab => "windows.opennewtab",
        Icon::OpenInNewWindow => "windows.opennewwindow",
        Icon::OpenWith => "windows.openwith",
        Icon::CopyAsPath => "windows.copyaspath",
        Icon::Wrench | Icon::Info => "windows.properties",
        Icon::WindowConsole => "windows.powershell.run",
        Icon::Pin => "windows.pintohome",
        Icon::Unpin => "windows.unpinfromhome",
        Icon::Star => "windows.pintohomefile",
        Icon::ShowMoreOptions => "expandtoclassic",
        Icon::ArrowClockwise => "windows.refresh",
        Icon::Filter => "SearchFilter",
        Icon::ExtractAll => "windows.compressedfile.extract",
        Icon::TextBulletList => "windows.groupbycolumn",
        Icon::ArrowCounterclockwise => "windows.recyclebin.restoreitems",
        Icon::DeleteDismiss => "windows.recyclebin.empty",
        Icon::Dismiss => "windows.close",
        Icon::Dismiss16 => "windows.searchclosetab",
        Icon::SearchSettings => "SearchAdvancedOptions",
        _ => return None,
    })
}

/// The picture Windows 11 File Explorer draws at the left end of the
/// address bar for the root `icon` stands for: Home, This PC, Gallery and
/// Network.
pub(crate) const fn address_root_file(icon: Icon) -> Option<&'static str> {
    match icon {
        Icon::Home => Some("addressbar.home"),
        Icon::Desktop => Some("addressbar.desktop"),
        Icon::Image => Some("addressbar.gallery"),
        Icon::GlobeDesktop => Some("addressbar.network"),
        _ => None,
    }
}

/// The picture of `icon` in `spot`, by file name without `.svg`.
pub(crate) const fn file_of(icon: Icon, spot: Spot) -> Option<&'static str> {
    match spot {
        Spot::Plain => None,
        Spot::Command => command_file(icon),
        Spot::AddressRoot => address_root_file(icon),
    }
}

/// Where the pictures stand now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Status {
    /// Off, or no folder chosen: the app's own icons.
    Off,
    /// The chosen folder is being listed.
    Reading,
    /// The folder has none of Windows Explorer's look folders.
    NotExplorerIcons,
    /// The folder was listed and has pictures for `found` of the `wanted`
    /// commands in the current look.
    Ready {
        /// How many commands have a picture.
        found: usize,
        /// How many commands the app draws from it.
        wanted: usize,
    },
}

/// The files of each look's folder, by their name in lower case (Windows'
/// names ignore case), to their path.
type Listing = HashMap<&'static str, HashMap<String, PathBuf>>;

/// The state of this thread's pictures.
struct State {
    /// The folder wanted now.
    wanted: Option<PathBuf>,
    /// Bumped on every change, so a late listing is dropped.
    generation: u64,
    /// The files of the folder wanted, once listed.
    listing: Option<Listing>,
    /// The look drawn now.
    look: Look,
    /// Each picture's bytes, read once; `None` when it cannot be read.
    read: HashMap<PathBuf, Option<glib::Bytes>>,
    /// Where things stand.
    status: Status,
    /// Told whenever the status changes, until one returns false.
    followers: Vec<Box<dyn Fn(Status) -> bool>>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        wanted: None,
        generation: 0,
        listing: None,
        look: Look::Light,
        read: HashMap::new(),
        status: Status::Off,
        followers: Vec::new(),
    });
}

/// Lists the chosen folder when Windows icon files are on and a folder is
/// chosen, or goes back to the app's own icons. Called at startup and
/// whenever the settings change; nothing happens when the choice is the
/// same.
pub(crate) fn follow_preferences(preferences: &Preferences) {
    let wanted = preferences
        .use_icon_files
        .then(|| preferences.explorer_icons.as_ref().map(PathBuf::from))
        .flatten();
    let generation = STATE.with_borrow_mut(|state| {
        if state.wanted == wanted {
            return None;
        }
        state.wanted.clone_from(&wanted);
        state.generation += 1;
        state.listing = None;
        state.read.clear();
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
    let Some(folder) = wanted else {
        return;
    };
    glib::spawn_future_local(async move {
        let listing = gio::spawn_blocking(move || list_folder(&folder))
            .await
            .unwrap_or_default();
        let current = STATE.with_borrow_mut(|state| {
            if state.generation != generation {
                return false;
            }
            state.listing = Some(listing);
            state.status = status_of(state);
            true
        });
        if current {
            apply();
        }
    });
}

/// The files of every look's folder under `folder`, or of `folder`'s
/// `images` folder when the user chose the `Assets` folder above it.
fn list_folder(folder: &Path) -> Listing {
    let has_looks = |folder: &Path| Look::ALL.iter().any(|look| folder.join(look.folder()).is_dir());
    let root = if has_looks(folder) {
        folder.to_path_buf()
    } else {
        folder.join("images")
    };
    let mut listing = Listing::new();
    for look in Look::ALL {
        let Ok(entries) = std::fs::read_dir(root.join(look.folder())) else {
            continue;
        };
        let files = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let stem = name.strip_suffix(".svg").or_else(|| name.strip_suffix(".SVG"))?;
                Some((stem.to_lowercase(), entry.path()))
            })
            .collect();
        listing.insert(look.folder(), files);
    }
    listing
}

/// The status `state`'s listing gives in its look.
fn status_of(state: &State) -> Status {
    let Some(listing) = &state.listing else {
        return if state.wanted.is_some() {
            Status::Reading
        } else {
            Status::Off
        };
    };
    if listing.is_empty() {
        return Status::NotExplorerIcons;
    }
    let files = listing.get(state.look.folder());
    let names = mapped_files();
    let found = names
        .iter()
        .filter(|name| files.is_some_and(|files| files.contains_key(&name.to_lowercase())))
        .count();
    Status::Ready {
        found,
        wanted: names.len(),
    }
}

/// Every picture the app draws from the folder, each once.
pub(crate) fn mapped_files() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = super::icon::ALL_ICONS
        .iter()
        .chain(super::icon::COMMAND_ROLES.iter())
        .flat_map(|icon| [command_file(*icon), address_root_file(*icon)])
        .flatten()
        .collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// Draws the look for a `dark` appearance, in `high_contrast` or not, and
/// shows every command picture again when that changed it.
pub(crate) fn set_look(dark: bool, high_contrast: bool) {
    let look = Look::of(dark, high_contrast);
    let changed = STATE.with_borrow_mut(|state| {
        if state.look == look {
            return false;
        }
        state.look = look;
        state.status = status_of(state);
        state.listing.is_some()
    });
    if changed {
        apply();
    }
}

/// The look drawn now.
#[cfg(test)]
pub(crate) fn look() -> Look {
    STATE.with_borrow(|state| state.look)
}

/// The path of `icon`'s picture in `spot` in the look drawn now, when the
/// chosen folder has one.
pub(crate) fn path_of(icon: Icon, spot: Spot) -> Option<PathBuf> {
    let name = file_of(icon, spot)?;
    STATE.with_borrow(|state| {
        state
            .listing
            .as_ref()?
            .get(state.look.folder())?
            .get(&name.to_lowercase())
            .cloned()
    })
}

/// Whether `icon` in `spot` is drawn from the chosen folder now.
pub(crate) fn draws(icon: Icon, spot: Spot) -> bool {
    path_of(icon, spot).is_some_and(|path| bytes_of(&path).is_some())
}

/// The bytes of the picture at `path`, read once.
fn bytes_of(path: &Path) -> Option<glib::Bytes> {
    // A picture is a few kilobytes; one larger than this is not one.
    const LARGEST: u64 = 256 * 1024;
    if let Some(read) = STATE.with_borrow(|state| state.read.get(path).cloned()) {
        return read;
    }
    let read = std::fs::metadata(path)
        .ok()
        .filter(|meta| meta.is_file() && meta.len() <= LARGEST)
        .and_then(|_| std::fs::read(path).ok())
        .map(glib::Bytes::from_owned);
    STATE.with_borrow_mut(|state| state.read.insert(path.to_path_buf(), read.clone()));
    read
}

/// The picture of `icon` in `spot`, `pixels` logical pixels square at
/// `scale`, as a texture: the picture's 16-pixel square drawn at `em`
/// pixels of a `square`-pixel square, centred in it. `None` when the
/// folder has no picture for it, or it cannot be read or drawn.
pub(crate) fn texture_of(
    icon: Icon,
    spot: Spot,
    square: i32,
    em: i32,
    pixels: i32,
    scale: i32,
) -> Option<gdk::Texture> {
    let path = path_of(icon, spot)?;
    let bytes = bytes_of(&path)?;
    let svg = padded_svg(
        std::str::from_utf8(&bytes).ok()?,
        square,
        em,
        pixels * scale.max(1),
    )?;
    gdk::Texture::from_bytes(&glib::Bytes::from_owned(svg.into_bytes())).ok()
}

/// `svg`, a picture drawn in a 16-unit square, nested in a picture
/// `device` pixels square that shows it `em` pixels wide, centred in a
/// `square`-pixel square. `None` when `svg` is not a single `<svg>`
/// element.
pub(crate) fn padded_svg(svg: &str, square: i32, em: i32, device: i32) -> Option<String> {
    let body = svg.trim_start_matches('\u{feff}').trim();
    let body = match body.strip_prefix("<?xml") {
        Some(rest) => rest.split_once("?>")?.1.trim_start(),
        None => body,
    };
    let open = body.strip_prefix("<svg")?;
    if !body.ends_with("</svg>") {
        return None;
    }
    let tag_end = open.find('>')?;
    let attributes = &open[..tag_end];
    // The picture's own size is replaced by the 16-unit square it is drawn
    // in; its viewBox stays.
    let kept = ["width", "height", "x", "y"]
        .iter()
        .fold(attributes.to_owned(), |kept, name| strip_attribute(&kept, name));
    let inner = format!(
        "<svg x=\"0\" y=\"0\" width=\"16\" height=\"16\"{kept}{}",
        &open[tag_end..]
    );
    let em = f64::from(em.max(1));
    let side = 16.0 * f64::from(square.max(1)) / em;
    let inset = (side - 16.0) / 2.0;
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{device}\" height=\"{device}\" \
         viewBox=\"{} {} {side} {side}\">{inner}</svg>",
        -inset, -inset
    ))
}

/// `attributes` without the attribute `name` (`name="…"` or `name='…'`).
fn strip_attribute(attributes: &str, name: &str) -> String {
    let mut kept = String::with_capacity(attributes.len());
    let mut rest = attributes;
    while let Some(at) = find_attribute(rest, name) {
        kept.push_str(&rest[..at]);
        let after = &rest[at + name.len()..];
        let value = after.trim_start().strip_prefix('=').map(str::trim_start);
        let Some(value) = value else {
            kept.push_str(&rest[at..at + name.len()]);
            rest = after;
            continue;
        };
        let quote = value
            .chars()
            .next()
            .filter(|quote| *quote == '"' || *quote == '\'');
        let Some(quote) = quote else {
            rest = "";
            break;
        };
        rest = value[1..].split_once(quote).map_or("", |(_, tail)| tail);
    }
    kept.push_str(rest);
    kept
}

/// Where the attribute `name` starts in `attributes`: after white space,
/// followed by `=` or white space.
fn find_attribute(attributes: &str, name: &str) -> Option<usize> {
    let mut from = 0;
    while let Some(found) = attributes[from..].find(name) {
        let at = from + found;
        let before = attributes[..at].chars().next_back();
        let after = attributes[at + name.len()..].chars().next();
        if before.is_some_and(char::is_whitespace) && after.is_some_and(|c| c == '=' || c.is_whitespace()) {
            return Some(at);
        }
        from = at + name.len();
    }
    None
}

/// Where the pictures stand now.
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

/// Shows every image again and tells the status followers.
fn apply() {
    super::redraw_scaled_images();
    super::user_icons::redraw_art_images();
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

/// Waits until the chosen folder was listed, for tests.
#[cfg(test)]
pub(crate) fn wait_until_read() {
    crate::test_support::harness::wait_until("the Explorer icons folder to be listed", || {
        !matches!(status(), Status::Reading)
    });
}

#[cfg(test)]
mod tests;
