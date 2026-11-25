// SPDX-License-Identifier: AGPL-3.0-only
//! This PC in every view Windows 11 offers there (VIEW-071, DEV-001):
//! Extra large, Large, Medium and Small icons, List, Details, Tiles and
//! Content; sorted by Name, Type (the default), Total Size, Free Space,
//! File System, Percent Full or Network Location; and grouped by Name,
//! Type ("Devices and drives", "Network locations", the default), Total
//! Size, Free Space, File System, Percent Full or not at all, as Windows
//! 11's This PC offers them where the app has the data.
//!
//! This PC keeps a style of its own, as Network does
//! (`ox_core::settings::own_style_place`): it starts in Tiles grouped and
//! sorted by type, and a view, sort or grouping chosen there is saved for
//! it alone.
//! The window keeps the chosen style in the folder pane's own state, as
//! for any folder, and [`super::landing`] draws the tiles from it
//! ([`ThisPcLayout`]).
//!
//! Each view only changes how a tile is laid out ([`Shape`]); the tiles
//! keep their keys, menus, selection and drags, whatever the view.
//! Windows' Name sort orders the drives by their letter, not by their
//! label, so This PC's own order (Local Disk, then the drives as they are
//! listed) is its Name order; Descending turns it round.
//!
//! Grouped by size or fullness, a drive not measured yet is under
//! "Unknown"; once GIO measures it, This PC is drawn again with it in its
//! group, and remembers the measure for the next time it is drawn.

use std::cmp::Ordering;

use gtk::prelude::*;

use super::folder_pane::FolderView;
use super::landing::Capacity;
use super::window_action::WindowAction;
use crate::folder_view::grid::IconSize;
use crate::folder_view::icon_size::{
    cell_size, compact_density_details_row, content_row, tiles_cell, CellSize,
};
use crate::folder_view::sort_roles::{SortBy, SortRole};
use crate::folder_view::sorting::{SortColumn, SortDirection};
use crate::icons::{self, Icon};
use crate::text_size::{self, TextSize};
use ox_core::grouping::GroupBy;
use ox_core::i18n::message_id;

/// What This PC's tiles are sorted by, as Windows 11 sorts its drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ThisPcSort {
    /// The drive's place in This PC: Windows' order by drive letter.
    Name,
    /// "Local Disk", "Removable Disk", "Network Drive" and the rest.
    Type,
    /// The drive's size.
    TotalSize,
    /// The room left on the drive.
    FreeSpace,
    /// The drive's file system, such as NTFS or ext4.
    FileSystem,
    /// How full the drive is.
    PercentFull,
    /// A saved network location's address.
    NetworkLocation,
}

impl ThisPcSort {
    /// Every column of This PC's Details, in Windows' order.
    pub(super) const ALL: [ThisPcSort; 4] = [
        ThisPcSort::Name,
        ThisPcSort::Type,
        ThisPcSort::TotalSize,
        ThisPcSort::FreeSpace,
    ];

    /// The Sort menu's first keys, as Windows 11's This PC lists them.
    pub(super) const MENU: [ThisPcSort; 3] = [ThisPcSort::Name, ThisPcSort::Type, ThisPcSort::TotalSize];

    /// The keys under the Sort menu's More, as Windows 11's This PC lists
    /// those the app has the data for.
    pub(super) const MORE: [ThisPcSort; 4] = [
        ThisPcSort::FreeSpace,
        ThisPcSort::FileSystem,
        ThisPcSort::PercentFull,
        ThisPcSort::NetworkLocation,
    ];

    /// This PC's sort for the window's sort key `by`: Size is its Total
    /// Size; a key This PC has no column for sorts by type, This PC's
    /// default.
    pub(super) fn of(by: SortBy) -> Self {
        match by {
            SortBy::Column(SortColumn::Name) => ThisPcSort::Name,
            SortBy::Column(SortColumn::Size) => ThisPcSort::TotalSize,
            SortBy::Role(SortRole::FreeSpace) => ThisPcSort::FreeSpace,
            SortBy::Role(SortRole::FileSystem) => ThisPcSort::FileSystem,
            SortBy::Role(SortRole::PercentFull) => ThisPcSort::PercentFull,
            SortBy::Role(SortRole::NetworkLocation) => ThisPcSort::NetworkLocation,
            _ => ThisPcSort::Type,
        }
    }

    /// The window's sort key for this column, which a click on its title
    /// sorts by.
    pub(super) fn key(self) -> &'static str {
        match self {
            ThisPcSort::Name => SortBy::Column(SortColumn::Name).as_str(),
            ThisPcSort::Type => SortBy::Column(SortColumn::Type).as_str(),
            ThisPcSort::TotalSize => SortBy::Column(SortColumn::Size).as_str(),
            ThisPcSort::FreeSpace => SortBy::Role(SortRole::FreeSpace).as_str(),
            ThisPcSort::FileSystem => SortBy::Role(SortRole::FileSystem).as_str(),
            ThisPcSort::PercentFull => SortBy::Role(SortRole::PercentFull).as_str(),
            ThisPcSort::NetworkLocation => SortBy::Role(SortRole::NetworkLocation).as_str(),
        }
    }

    /// The column's title, as Windows 11's This PC heads it.
    pub(super) fn title(self) -> &'static str {
        match self {
            ThisPcSort::Name => ox_core::i18n::gettext_static("Name"),
            ThisPcSort::Type => ox_core::i18n::gettext_static("Type"),
            ThisPcSort::TotalSize => ox_core::i18n::gettext_static("Total Size"),
            ThisPcSort::FreeSpace => ox_core::i18n::gettext_static("Free Space"),
            ThisPcSort::FileSystem => ox_core::i18n::gettext_static("File System"),
            ThisPcSort::PercentFull => ox_core::i18n::gettext_static("Percent Full"),
            ThisPcSort::NetworkLocation => ox_core::i18n::gettext_static("Network Location"),
        }
    }

    /// The column's width at 100% text, Name's included.
    fn width(self) -> i32 {
        match self {
            ThisPcSort::Name => 256,
            ThisPcSort::Type => 144,
            ThisPcSort::TotalSize
            | ThisPcSort::FreeSpace
            | ThisPcSort::FileSystem
            | ThisPcSort::PercentFull
            | ThisPcSort::NetworkLocation => 112,
        }
    }
}

/// How This PC is laid out: the view, sort and grouping it keeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ThisPcLayout {
    pub(super) view: FolderView,
    pub(super) sort: ThisPcSort,
    pub(super) descending: bool,
    /// What the tiles are grouped by, or `None` for one run.
    pub(super) group: Option<ThisPcGroup>,
    /// Windows' Compact view, whose Details rows stand closer.
    pub(super) compact: bool,
}

impl Default for ThisPcLayout {
    /// Windows 11's This PC: Tiles grouped and sorted by type.
    fn default() -> Self {
        Self {
            view: FolderView::Tiles,
            sort: ThisPcSort::Type,
            descending: false,
            group: Some(ThisPcGroup::Type),
            compact: false,
        }
    }
}

impl ThisPcLayout {
    /// The layout for the window's `view`, sort key `by` in `direction`,
    /// grouped by `group_by`.
    pub(super) fn new(
        view: FolderView,
        by: SortBy,
        direction: SortDirection,
        group_by: GroupBy,
        compact: bool,
    ) -> Self {
        Self {
            view,
            sort: ThisPcSort::of(by),
            descending: direction == SortDirection::Descending,
            group: ThisPcGroup::of(group_by),
            compact,
        }
    }

    /// The shape of a tile in this layout at `text_size`.
    pub(super) fn shape(self, text_size: TextSize) -> Shape {
        match self.view {
            FolderView::Tiles => Shape::Tile(tiles_cell(text_size)),
            FolderView::Content => Shape::Content(content_row(text_size)),
            FolderView::Details => {
                Shape::Details(self.details_row(text_size), text_size::metrics(text_size).scale)
            }
            FolderView::Compact => Shape::Line(cell_size(IconSize::SMALL, text_size)),
            FolderView::Icons(size) if size.pixels() <= IconSize::SMALL.pixels() => {
                Shape::Line(cell_size(IconSize::SMALL, text_size))
            }
            FolderView::Icons(size) => Shape::Icon(size.pixels(), cell_size(size, text_size)),
        }
    }

    /// A Details row's height: Windows' 21 pixels apart in Compact view,
    /// else the app's details rows.
    fn details_row(self, text_size: TextSize) -> i32 {
        if self.compact {
            compact_density_details_row(text_size)
        } else {
            text_size::metrics(text_size).detail_row
        }
    }

    /// The CSS class of the tile boxes in this layout.
    pub(super) fn css_class(self) -> &'static str {
        match self.view {
            FolderView::Tiles => "pc-tiles",
            FolderView::Content => "pc-content",
            FolderView::Details => "pc-details",
            FolderView::Compact => "pc-list",
            FolderView::Icons(size) if size.pixels() <= IconSize::SMALL.pixels() => "pc-list",
            FolderView::Icons(_) => "pc-icons",
        }
    }

    /// Whether a row holds one tile across the page.
    pub(super) fn one_per_row(self) -> bool {
        matches!(self.view, FolderView::Details | FolderView::Content)
    }

    /// The order of two tiles' `a` and `b` facts in this layout.
    pub(super) fn compare(self, a: &DriveFacts, b: &DriveFacts) -> Ordering {
        let by_place = a.order.cmp(&b.order);
        let text = |text: &str| crate::folder_view::sorting::SortKey::new(text);
        let ordering = match self.sort {
            ThisPcSort::Name => by_place,
            ThisPcSort::Type => text(&a.kind).natural_cmp(&text(&b.kind)).then(by_place),
            ThisPcSort::FileSystem => text(&a.filesystem)
                .natural_cmp(&text(&b.filesystem))
                .then(by_place),
            ThisPcSort::NetworkLocation => text(&a.location).natural_cmp(&text(&b.location)).then(by_place),
            ThisPcSort::PercentFull => a.percent_full().cmp(&b.percent_full()).then(by_place),
            ThisPcSort::TotalSize => size_of(a, |capacity| capacity.size)
                .cmp(&size_of(b, |capacity| capacity.size))
                .then(by_place),
            ThisPcSort::FreeSpace => size_of(a, |capacity| capacity.free)
                .cmp(&size_of(b, |capacity| capacity.free))
                .then(by_place),
        };
        if self.descending {
            ordering.reverse()
        } else {
            ordering
        }
    }
}

/// A drive's size or room left, unmeasured ones first, as Windows lists
/// a drive it cannot measure.
fn size_of(facts: &DriveFacts, part: impl Fn(Capacity) -> u64) -> Option<u64> {
    facts.capacity.map(part)
}

/// What This PC sorts and groups a tile by.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct DriveFacts {
    /// The tile's place in This PC's own order.
    pub(super) order: usize,
    /// Its name, as the tile shows it.
    pub(super) name: String,
    /// Its type, as the Type column writes it.
    pub(super) kind: String,
    /// Whether it is a saved network location, not a drive.
    pub(super) network: bool,
    /// Its file system, such as NTFS or ext4; empty when unknown.
    pub(super) filesystem: String,
    /// A saved network location's address, as the address bar writes it;
    /// empty for a drive.
    pub(super) location: String,
    /// How full it is, once GIO has measured it.
    pub(super) capacity: Option<Capacity>,
}

impl DriveFacts {
    /// How full the drive is, in hundredths of a percent, once measured.
    fn percent_full(&self) -> Option<u64> {
        let capacity = self.capacity?;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a share from 0 to 1, times 10,000"
        )]
        let share = (capacity.used_share() * 10_000.0).round() as u64;
        Some(share)
    }

    /// The group the tile falls in when This PC is grouped by `group`:
    /// its title, a key that finds the group again after a redraw, and
    /// its place among the groups.
    pub(super) fn group(&self, group: ThisPcGroup) -> GroupKey {
        // Windows' group of a drive without the value, such as an empty
        // CD drive or a share that is not connected (VM, 2026-11-24).
        let unspecified = || GroupKey::new(ox_core::i18n::gettext("Unspecified"), i64::MAX);
        match group {
            ThisPcGroup::Type => {
                if self.network {
                    GroupKey::keyed(ox_core::i18n::gettext("Network locations"), "network", 1)
                } else {
                    GroupKey::keyed(ox_core::i18n::gettext("Devices and drives"), "drives", 0)
                }
            }
            ThisPcGroup::Name => {
                let name = ox_core::grouping::NameGroup::of(&self.name);
                GroupKey::new(ox_core::i18n::gettext(name.label()), name as i64)
            }
            ThisPcGroup::TotalSize => match self.capacity.filter(|capacity| capacity.size > 0) {
                Some(capacity) => {
                    let (title, rank) = capacity_band(capacity.size);
                    GroupKey::new(title, rank)
                }
                None => unspecified(),
            },
            ThisPcGroup::FileSystem if self.filesystem.is_empty() => unspecified(),
            ThisPcGroup::FileSystem => GroupKey::new(self.filesystem.clone(), 0),
            ThisPcGroup::FreeSpace | ThisPcGroup::PercentFull => {
                let Some(full) = self.percent_full() else {
                    return unspecified();
                };
                // Bands of a tenth: 0% - 9%, 10% - 19%, up to 90% - 100%.
                let percent = if group == ThisPcGroup::FreeSpace {
                    100 - full.div_ceil(100).min(100)
                } else {
                    full / 100
                };
                let tenth = (percent / 10).min(9);
                let low = tenth * 10;
                let high = if tenth == 9 { 100 } else { low + 9 };
                let pattern = if group == ThisPcGroup::FreeSpace {
                    ox_core::i18n::gettext("{low}% - {high}% free")
                } else {
                    ox_core::i18n::gettext("{low}% - {high}% full")
                };
                let title = ox_core::i18n::format_message(
                    &pattern,
                    &[("low", &low.to_string()), ("high", &high.to_string())],
                );
                GroupKey::new(title, i64::try_from(tenth).unwrap_or(0))
            }
        }
    }
}

/// The size bands of System.Capacity, Windows' Total Size of a drive:
/// each band's last size in bytes and its title. The limits are
/// Microsoft's (props-system-capacity, Windows 7 to 10); the titles take
/// the Windows 11 VM's wording, "16 GB - 80 GB" (2026-11-24); the others
/// are written alike and not seen in the VM.
const CAPACITY_BANDS: [(u64, &str); 6] = [
    (16 * GIB, message_id("0 GB - 16 GB")),
    (80 * GIB, message_id("16 GB - 80 GB")),
    (250 * GIB, message_id("80 GB - 250 GB")),
    (500 * GIB, message_id("250 GB - 500 GB")),
    (1024 * GIB, message_id("500 GB - 1 TB")),
    (u64::MAX, message_id(">1 TB")),
];

/// A gibibyte, the GB of Windows' bands.
const GIB: u64 = 1024 * 1024 * 1024;

/// The Total Size band of a drive of `size` bytes: its title and place.
fn capacity_band(size: u64) -> (String, i64) {
    let band = CAPACITY_BANDS
        .iter()
        .position(|(last, _)| size <= *last)
        .unwrap_or(CAPACITY_BANDS.len() - 1);
    (
        ox_core::i18n::gettext(CAPACITY_BANDS[band].1),
        i64::try_from(band).unwrap_or(0),
    )
}

/// What This PC's tiles are grouped by, as Windows 11's This PC offers
/// them where the app has the data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ThisPcGroup {
    /// "Devices and drives" and "Network locations".
    Type,
    /// The name's letter ranges, as a folder's.
    Name,
    /// Windows' bands of the drive's size (System.Capacity).
    TotalSize,
    /// Tenths of the room left, as Windows 11 groups Free Space.
    FreeSpace,
    /// The file system.
    FileSystem,
    /// Tenths of fullness, written as Free Space's are.
    PercentFull,
}

impl ThisPcGroup {
    /// Group by's choices in This PC, in Windows' order, (None) last.
    pub(super) const MENU: [GroupBy; 7] = [
        GroupBy::Name,
        GroupBy::Type,
        GroupBy::Size,
        GroupBy::FreeSpace,
        GroupBy::FileSystem,
        GroupBy::PercentFull,
        GroupBy::None,
    ];

    /// This PC's grouping for the window's `by`: Size is its Total Size; a
    /// choice This PC has no data for groups by type.
    pub(super) fn of(by: GroupBy) -> Option<Self> {
        match by {
            GroupBy::None => None,
            GroupBy::Name => Some(ThisPcGroup::Name),
            GroupBy::Size => Some(ThisPcGroup::TotalSize),
            GroupBy::FreeSpace => Some(ThisPcGroup::FreeSpace),
            GroupBy::FileSystem => Some(ThisPcGroup::FileSystem),
            GroupBy::PercentFull => Some(ThisPcGroup::PercentFull),
            GroupBy::Type | GroupBy::Modified | GroupBy::Created | GroupBy::SortKey => {
                Some(ThisPcGroup::Type)
            }
        }
    }

    /// The Group by menu's label for the window's `by` in This PC.
    pub(super) fn label(by: GroupBy) -> &'static str {
        match by {
            GroupBy::Size => ox_core::i18n::gettext_static("Total Size"),
            other => ox_core::i18n::gettext_static(other.label()),
        }
    }
}

/// One of This PC's groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GroupKey {
    /// "Devices and drives", "Huge (1 – 4 GB)", "NTFS"...
    pub(super) title: String,
    /// What the group's header and collapsed state are named by.
    pub(super) key: String,
    /// Its place among the groups; equal places go by title.
    pub(super) rank: i64,
}

impl GroupKey {
    /// A group named by its title.
    fn new(title: String, rank: i64) -> Self {
        let slug: String = title
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect();
        let key = format!("pc-group-{slug}");
        Self { title, key, rank }
    }

    /// The class the page carries while the group is collapsed, so a
    /// collapsed group stays collapsed when This PC is drawn again.
    pub(super) fn collapsed_class(&self) -> String {
        format!("{}-collapsed", self.key)
    }

    /// The class of the group's header, which finds it again after a
    /// redraw.
    pub(super) fn header_class(&self) -> String {
        format!("{}-header", self.key)
    }

    /// The groups of This PC that `tiles`, given with their facts, fall in
    /// when grouped by `group`, in Windows' order: by place, then title.
    pub(super) fn sorted<T>(tiles: Vec<(T, &DriveFacts)>, group: ThisPcGroup) -> Vec<(GroupKey, Vec<T>)> {
        let mut groups: Vec<(GroupKey, Vec<T>)> = Vec::new();
        for (tile, facts) in tiles {
            let key = facts.group(group);
            match groups.iter_mut().find(|(known, _)| known.key == key.key) {
                Some((_, members)) => members.push(tile),
                None => groups.push((key, vec![tile])),
            }
        }
        let text = |text: &str| crate::folder_view::sorting::SortKey::new(text);
        groups.sort_by(|(a, _), (b, _)| {
            a.rank
                .cmp(&b.rank)
                .then_with(|| text(&a.title).natural_cmp(&text(&b.title)))
        });
        groups
    }

    /// A group with its own `key`: Type's two, whose header and collapsed
    /// classes stay as before ("drives-header", "network-collapsed").
    fn keyed(title: String, key: &str, rank: i64) -> Self {
        Self {
            title,
            key: key.to_owned(),
            rank,
        }
    }
}

/// How a tile is laid out in a view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Shape {
    /// Tiles: a 48-pixel icon left of the name, the capacity bar and
    /// "N free of M", in a cell of this size.
    Tile(CellSize),
    /// Content: a row across the page, as tall as this, with a 32-pixel
    /// icon left of the name, the bar and "N free of M".
    Content(i32),
    /// The icon sizes: the icon, this many pixels, above the name and the
    /// bar, in a cell of this size.
    Icon(i32, CellSize),
    /// Small icons and List: a 16-pixel icon left of a one-line name, in
    /// a cell of this size.
    Line(CellSize),
    /// Details: a row as tall as this with a 16-pixel icon, the name, the
    /// type, the total size and the free space in their columns, at this
    /// text scale.
    Details(i32, f64),
}

impl Shape {
    /// The edge of the tile's icon.
    pub(super) fn icon_pixels(self) -> i32 {
        match self {
            Shape::Tile(_) => crate::folder_view::icon_size::TILES_ICON,
            Shape::Content(_) => 32,
            Shape::Icon(pixels, _) => pixels,
            Shape::Line(_) | Shape::Details(..) => 16,
        }
    }
}

/// Where a tile shows how full its drive is, once measured.
#[derive(Debug, Clone)]
pub(super) enum CapacitySlot {
    /// Under the name: the bar, and "N free of M" under it when the bool
    /// says so, else as the bar's tooltip.
    Lines(gtk::Box, bool),
    /// In the Total Size and Free Space columns.
    Columns(gtk::Label, gtk::Label),
    /// Nowhere: Small icons and List show the name alone.
    Hidden,
}

/// The parts of a tile a drive's tile fills in.
pub(super) struct TileParts {
    pub(super) child: gtk::FlowBoxChild,
    /// The lines under or beside the name, for a note such as "Click to
    /// connect"; in Details the Type column's cell.
    pub(super) lines: gtk::Box,
    pub(super) capacity: CapacitySlot,
}

/// One line of text with `css_class`, cut with "..." when too long.
pub(super) fn text_label(text: &str, css_class: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::End)
        .single_line_mode(true)
        .css_classes([css_class])
        .build()
}

/// A tile showing `icon` with `name` in `shape`, named `key`, the
/// location or volume it opens, which finds it again after a redraw; its
/// type is `kind`, for the Details' Type column.
pub(super) fn tile(icon: &gtk::Widget, name: &str, kind: &str, key: &str, shape: Shape) -> TileParts {
    let name_label = text_label(name, "tile-name");
    let lines = gtk::Box::new(gtk::Orientation::Vertical, 1);
    lines.add_css_class("tile-lines");
    let (content, capacity) = match shape {
        Shape::Tile(cell) => {
            let content = beside(icon, &name_label, &lines, super::landing::TILE_ICON_GAP);
            let (across, down) = super::landing::TILE_CHROME;
            content.set_size_request(cell.width - across, cell.height - down);
            lines.add_css_class("tile-bar-fixed");
            (content, CapacitySlot::Lines(lines.clone(), true))
        }
        Shape::Content(height) => {
            let content = beside(icon, &name_label, &lines, super::landing::TILE_ICON_GAP);
            content.set_size_request(-1, height - super::landing::TILE_CHROME.1);
            lines.set_hexpand(false);
            lines.set_size_request(CONTENT_LINES_WIDTH, -1);
            (content, CapacitySlot::Lines(lines.clone(), true))
        }
        Shape::Icon(pixels, cell) => {
            name_label.set_single_line_mode(false);
            name_label.set_wrap(true);
            name_label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
            name_label.set_lines(2);
            name_label.set_justify(gtk::Justification::Center);
            name_label.set_xalign(0.5);
            let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
            icon.set_halign(gtk::Align::Center);
            content.append(icon);
            lines.prepend(&name_label);
            // The bar as wide as the icon, at most the cell's width.
            lines.set_halign(gtk::Align::Center);
            lines.set_size_request(pixels.clamp(48, cell.width - ICON_CELL_CHROME), -1);
            content.append(&lines);
            content.set_size_request(cell.width - ICON_CELL_CHROME, -1);
            (content, CapacitySlot::Lines(lines.clone(), false))
        }
        Shape::Line(cell) => {
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            icon.set_valign(gtk::Align::Center);
            content.append(icon);
            name_label.set_hexpand(true);
            content.append(&name_label);
            lines.set_visible(false);
            content.append(&lines);
            content.set_size_request(cell.width - LINE_CELL_CHROME, cell.height - LINE_CELL_CHROME);
            (content, CapacitySlot::Hidden)
        }
        Shape::Details(height, scale) => {
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let name_cell = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            icon.set_valign(gtk::Align::Center);
            name_cell.append(icon);
            name_cell.append(&name_label);
            name_label.set_hexpand(true);
            name_cell.set_size_request(column_width(ThisPcSort::Name, scale) - DETAILS_NAME_INSET, -1);
            name_cell.add_css_class("pc-cell");
            content.append(&name_cell);
            let kind_label = text_label(kind, "pc-cell-text");
            lines.append(&kind_label);
            lines.set_valign(gtk::Align::Center);
            lines.set_size_request(column_width(ThisPcSort::Type, scale), -1);
            lines.add_css_class("pc-cell");
            content.append(&lines);
            let total = number_cell(column_width(ThisPcSort::TotalSize, scale));
            let free = number_cell(column_width(ThisPcSort::FreeSpace, scale));
            content.append(&total);
            content.append(&free);
            content.set_size_request(-1, height);
            content.set_halign(gtk::Align::Start);
            (content, CapacitySlot::Columns(total, free))
        }
    };
    let child = gtk::FlowBoxChild::builder().child(&content).name(key).build();
    child.update_property(&[gtk::accessible::Property::Label(name)]);
    TileParts {
        child,
        lines,
        capacity,
    }
}

/// The width of a Content row's name and capacity, as Windows 11 keeps
/// them at the left of the row.
const CONTENT_LINES_WIDTH: i32 = 220;

/// A tile's capacity bar across, its border included, as Windows 11's
/// This PC draws it at 100% (189 pixels on its capture).
const TILE_BAR_WIDTH: i32 = 189;

/// Pixels an icon tile's padding takes across (landing.css).
const ICON_CELL_CHROME: i32 = 12;

/// Pixels a Small icons or List tile's padding takes across and down.
const LINE_CELL_CHROME: i32 = 4;

/// How far Details' name cell starts in from its column, its row's
/// padding (landing.css).
const DETAILS_NAME_INSET: i32 = 12;

/// `column`'s width at the text `scale`.
fn column_width(column: ThisPcSort, scale: f64) -> i32 {
    text_size::ceil_pixels(f64::from(column.width()) * scale.max(1.0) - 0.5)
}

/// An empty, right-aligned number cell `width` pixels wide.
fn number_cell(width: i32) -> gtk::Label {
    let label = text_label("", "pc-cell-text");
    label.set_xalign(1.0);
    label.set_size_request(width, -1);
    label.add_css_class("pc-cell");
    label.add_css_class("pc-number");
    label
}

/// `icon` left of `name` over `lines`, `gap` pixels apart.
fn beside(icon: &gtk::Widget, name: &gtk::Label, lines: &gtk::Box, gap: i32) -> gtk::Box {
    lines.set_valign(gtk::Align::Center);
    lines.set_hexpand(true);
    lines.prepend(name);
    let content = gtk::Box::new(gtk::Orientation::Horizontal, gap);
    icon.set_valign(gtk::Align::Center);
    content.append(icon);
    content.append(lines);
    content
}

/// Shows `capacity` in `slot`: the bar and "N free of M", or the sizes in
/// their columns.
pub(super) fn show_capacity_in(slot: &CapacitySlot, capacity: Capacity) {
    match slot {
        CapacitySlot::Lines(lines, with_text) => {
            let bar = super::landing::capacity_bar(capacity);
            if lines.has_css_class("tile-bar-fixed") {
                // Windows 11's Tiles bar is as wide whatever the tile.
                bar.set_halign(gtk::Align::Start);
                bar.set_size_request(TILE_BAR_WIDTH, -1);
            }
            lines.append(&bar);
            if *with_text {
                lines.append(&text_label(&capacity.text(), "card-sub"));
            } else {
                bar.set_tooltip_text(Some(&capacity.text()));
            }
        }
        CapacitySlot::Columns(total, free) => {
            total.set_text(&ox_core::format::short_size(capacity.size));
            free.set_text(&ox_core::format::short_size(capacity.free));
        }
        CapacitySlot::Hidden => {}
    }
}

/// The titles of This PC's Details, Name, Type, Total Size and Free
/// Space, as Windows 11 heads them; a click sorts by the column, or turns
/// the order round when it is sorted by it already.
pub(super) fn details_titles(layout: ThisPcLayout, scale: f64) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row.add_css_class("pc-details-titles");
    row.set_halign(gtk::Align::Start);
    for column in ThisPcSort::ALL {
        let title = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let text = text_label(column.title(), "pc-title-text");
        title.append(&text);
        if column == layout.sort {
            let chevron = if layout.descending {
                Icon::ChevronDown16
            } else {
                Icon::ChevronUp16
            };
            title.append(&icons::image(chevron, 12));
        }
        if matches!(column, ThisPcSort::TotalSize | ThisPcSort::FreeSpace) {
            title.set_halign(gtk::Align::End);
        }
        let button = gtk::Button::builder()
            .child(&title)
            .css_classes(["pc-title"])
            .build();
        let width = column_width(column, scale);
        button.set_size_request(width, -1);
        let sorted = column == layout.sort;
        let descending = layout.descending;
        button.connect_clicked(move |button| {
            if sorted {
                let direction = if descending {
                    SortDirection::Ascending
                } else {
                    SortDirection::Descending
                };
                WindowAction::Direction.activate_from(button, Some(&direction.as_str().to_variant()));
            } else {
                WindowAction::Sort.activate_from(button, Some(&column.key().to_variant()));
            }
        });
        row.append(&button);
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(order: usize, kind: &str, size: u64, free: u64) -> DriveFacts {
        DriveFacts {
            order,
            name: format!("Drive {order}"),
            kind: kind.to_owned(),
            capacity: Some(Capacity { size, free }),
            ..DriveFacts::default()
        }
    }

    /// Name keeps This PC's own order, Windows' drive-letter order, and
    /// Descending turns it round; Type, Total Size, Free Space and Percent
    /// Full sort by their value, then by that order. This PC starts sorted
    /// by type, as Windows 11's.
    #[test]
    fn this_pc_sorts_as_windows_sorts_its_drives() {
        let drives = [
            facts(0, "Local Disk", 500, 100),
            facts(1, "CD Drive", 64, 0),
            facts(2, "Local Disk", 2000, 1800),
        ];
        let order = |sort: ThisPcSort, descending: bool| {
            let layout = ThisPcLayout {
                sort,
                descending,
                ..ThisPcLayout::default()
            };
            let mut sorted = drives.clone();
            sorted.sort_by(|a, b| layout.compare(a, b));
            sorted.iter().map(|drive| drive.order).collect::<Vec<_>>()
        };
        assert_eq!(ThisPcLayout::default().sort, ThisPcSort::Type);
        assert_eq!(order(ThisPcSort::Name, false), [0, 1, 2]);
        assert_eq!(order(ThisPcSort::Name, true), [2, 1, 0]);
        assert_eq!(
            order(ThisPcSort::Type, false),
            [1, 0, 2],
            "CD Drive first, as in Windows"
        );
        assert_eq!(order(ThisPcSort::TotalSize, false), [1, 0, 2]);
        assert_eq!(order(ThisPcSort::FreeSpace, true), [2, 0, 1]);
        assert_eq!(order(ThisPcSort::PercentFull, false), [2, 0, 1]);
    }

    /// The window's Sort menu keys reach This PC's columns: Size is its
    /// Total Size, and keys it has no column for sort by type.
    #[test]
    fn the_sort_menu_keys_reach_this_pcs_columns() {
        for column in ThisPcSort::MENU.into_iter().chain(ThisPcSort::MORE) {
            let by = SortBy::from_key(column.key()).expect("a sort key");
            assert_eq!(ThisPcSort::of(by), column);
        }
        assert_eq!(
            ThisPcSort::of(SortBy::Column(SortColumn::Modified)),
            ThisPcSort::Type
        );
    }

    /// Grouped, a drive falls in Windows' groups: by type, the drives and
    /// the network locations; by size, the files' size groups; by file
    /// system, its name; by fullness, tenths; an unmeasured drive under
    /// Unknown.
    #[test]
    fn drives_fall_in_windows_groups() {
        let disk = DriveFacts {
            filesystem: "ext4".to_owned(),
            ..facts(0, "Local Disk", 256 * 1024 * 1024 * 1024, 64 * 1024 * 1024 * 1024)
        };
        let share = DriveFacts {
            network: true,
            capacity: None,
            ..facts(1, "Network Drive", 0, 0)
        };
        let title = |facts: &DriveFacts, group: ThisPcGroup| facts.group(group).title;
        assert_eq!(title(&disk, ThisPcGroup::Type), "Devices and drives");
        assert_eq!(title(&share, ThisPcGroup::Type), "Network locations");
        assert_eq!(title(&disk, ThisPcGroup::TotalSize), "250 GB - 500 GB");
        assert_eq!(title(&disk, ThisPcGroup::FileSystem), "ext4");
        assert_eq!(title(&disk, ThisPcGroup::PercentFull), "70% - 79% full");
        assert_eq!(title(&share, ThisPcGroup::FreeSpace), "Unspecified");
        assert_eq!(title(&share, ThisPcGroup::TotalSize), "Unspecified");
        // The Windows 11 VM's Local Disk (C:): 62.8 GB, 41.6 GB free.
        let gib = |tenths: u64| tenths * 1024 * 1024 * 1024 / 10;
        let vm_disk = facts(2, "Local Disk", gib(628), gib(416));
        assert_eq!(title(&vm_disk, ThisPcGroup::TotalSize), "16 GB - 80 GB");
        assert_eq!(title(&vm_disk, ThisPcGroup::FreeSpace), "60% - 69% free");
        // The bands' edges, as System.Capacity sets them: 16 GB is the
        // first band's, a byte more the next one's.
        let at = |bytes: u64| title(&facts(3, "Local Disk", bytes, 0), ThisPcGroup::TotalSize);
        assert_eq!(at(16 * 1024 * 1024 * 1024), "0 GB - 16 GB");
        assert_eq!(at(16 * 1024 * 1024 * 1024 + 1), "16 GB - 80 GB");
        assert_eq!(at(2 * 1024 * 1024 * 1024 * 1024), ">1 TB");
        let full = facts(4, "Local Disk", 100, 0);
        assert_eq!(title(&full, ThisPcGroup::FreeSpace), "0% - 9% free");
        assert_eq!(title(&full, ThisPcGroup::PercentFull), "90% - 100% full");
        assert_eq!(title(&disk, ThisPcGroup::Name), "A – H");
        let groups = GroupKey::sorted(vec![("share", &share), ("disk", &disk)], ThisPcGroup::Type);
        let order: Vec<_> = groups
            .iter()
            .map(|(key, tiles)| (key.title.as_str(), tiles.clone()))
            .collect();
        assert_eq!(
            order,
            [
                ("Devices and drives", vec!["disk"]),
                ("Network locations", vec!["share"])
            ]
        );
        assert_eq!(ThisPcGroup::of(GroupBy::Size), Some(ThisPcGroup::TotalSize));
        assert_eq!(ThisPcGroup::of(GroupBy::None), None);
    }
}
