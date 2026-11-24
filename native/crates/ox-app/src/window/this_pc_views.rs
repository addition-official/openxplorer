// SPDX-License-Identifier: AGPL-3.0-only
//! This PC in every view Windows 11 offers there (VIEW-071, DEV-001):
//! Extra large, Large, Medium and Small icons, List, Details, Tiles and
//! Content, sorted by Name, Type, Total Size or Free Space, and grouped by
//! type ("Devices and drives", "Network locations") or not at all.
//!
//! This PC keeps a style of its own, as Network does
//! (`ox_core::settings::own_style_place`): it starts in Tiles grouped by
//! type, and a view, sort or grouping chosen there is saved for it alone.
//! The window keeps the chosen style in the folder pane's own state, as
//! for any folder, and [`super::landing`] draws the tiles from it
//! ([`ThisPcLayout`]).
//!
//! Each view only changes how a tile is laid out ([`Shape`]); the tiles
//! keep their keys, menus, selection and drags, whatever the view.
//! Windows' Name sort orders the drives by their letter, not by their
//! label, so This PC's own order (Local Disk, then the drives as they are
//! listed) is its Name order; Descending turns it round.

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
}

impl ThisPcSort {
    /// Every column of This PC's Details, in Windows' order.
    pub(super) const ALL: [ThisPcSort; 4] = [
        ThisPcSort::Name,
        ThisPcSort::Type,
        ThisPcSort::TotalSize,
        ThisPcSort::FreeSpace,
    ];

    /// This PC's sort for the window's sort key `by`: Size is its Total
    /// Size; a key This PC has no column for sorts by name.
    pub(super) fn of(by: SortBy) -> Self {
        match by {
            SortBy::Column(SortColumn::Type) => ThisPcSort::Type,
            SortBy::Column(SortColumn::Size) => ThisPcSort::TotalSize,
            SortBy::Role(SortRole::FreeSpace) => ThisPcSort::FreeSpace,
            _ => ThisPcSort::Name,
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
        }
    }

    /// The column's title, as Windows 11's This PC heads it.
    pub(super) fn title(self) -> &'static str {
        match self {
            ThisPcSort::Name => ox_core::i18n::gettext_static("Name"),
            ThisPcSort::Type => ox_core::i18n::gettext_static("Type"),
            ThisPcSort::TotalSize => ox_core::i18n::gettext_static("Total Size"),
            ThisPcSort::FreeSpace => ox_core::i18n::gettext_static("Free Space"),
        }
    }

    /// The column's width at 100% text, Name's included.
    fn width(self) -> i32 {
        match self {
            ThisPcSort::Name => 256,
            ThisPcSort::Type => 144,
            ThisPcSort::TotalSize | ThisPcSort::FreeSpace => 112,
        }
    }
}

/// How This PC is laid out: the view, sort and grouping it keeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ThisPcLayout {
    pub(super) view: FolderView,
    pub(super) sort: ThisPcSort,
    pub(super) descending: bool,
    /// Under "Devices and drives" and "Network locations", or in one run.
    pub(super) grouped: bool,
    /// Windows' Compact view, whose Details rows stand closer.
    pub(super) compact: bool,
}

impl Default for ThisPcLayout {
    /// Windows 11's This PC: Tiles grouped by type, in drive order.
    fn default() -> Self {
        Self {
            view: FolderView::Tiles,
            sort: ThisPcSort::Name,
            descending: false,
            grouped: true,
            compact: false,
        }
    }
}

impl ThisPcLayout {
    /// The layout for the window's `view`, sort key `by` in `direction`,
    /// grouped or not.
    pub(super) fn new(
        view: FolderView,
        by: SortBy,
        direction: SortDirection,
        grouped: bool,
        compact: bool,
    ) -> Self {
        Self {
            view,
            sort: ThisPcSort::of(by),
            descending: direction == SortDirection::Descending,
            grouped,
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
        let ordering = match self.sort {
            ThisPcSort::Name => by_place,
            ThisPcSort::Type => a.kind.cmp(&b.kind).then(by_place),
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

/// What This PC sorts a tile by.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct DriveFacts {
    /// The tile's place in This PC's own order.
    pub(super) order: usize,
    /// Its type, as the Type column writes it.
    pub(super) kind: String,
    /// How full it is, once GIO has measured it.
    pub(super) capacity: Option<Capacity>,
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
            kind: kind.to_owned(),
            capacity: Some(Capacity { size, free }),
        }
    }

    /// Name keeps This PC's own order, Windows' drive-letter order, and
    /// Descending turns it round; Type, Total Size and Free Space sort by
    /// their value, then by that order.
    #[test]
    fn this_pc_sorts_as_windows_sorts_its_drives() {
        let drives = [
            facts(0, "Local Disk", 500, 100),
            facts(1, "Removable Disk", 64, 60),
            facts(2, "Local Disk", 2000, 10),
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
        assert_eq!(order(ThisPcSort::Name, false), [0, 1, 2]);
        assert_eq!(order(ThisPcSort::Name, true), [2, 1, 0]);
        assert_eq!(order(ThisPcSort::Type, false), [0, 2, 1]);
        assert_eq!(order(ThisPcSort::TotalSize, false), [1, 0, 2]);
        assert_eq!(order(ThisPcSort::FreeSpace, true), [0, 1, 2]);
    }

    /// The window's Sort menu keys reach This PC's columns: Size is its
    /// Total Size, and keys it has no column for sort by name.
    #[test]
    fn the_sort_menu_keys_reach_this_pcs_columns() {
        for column in ThisPcSort::ALL {
            let by = SortBy::from_key(column.key()).expect("a sort key");
            assert_eq!(ThisPcSort::of(by), column);
        }
        assert_eq!(
            ThisPcSort::of(SortBy::Column(SortColumn::Modified)),
            ThisPcSort::Name
        );
    }
}
