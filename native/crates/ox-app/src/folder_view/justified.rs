// SPDX-License-Identifier: AGPL-3.0-only
//! Gallery's justified rows (HOME-026): Windows 11 File Explorer's Gallery
//! lays its photos out as a photo grid does, each thumbnail keeping its
//! photo's aspect ratio, in rows of one height that fill the width, with
//! the same gap between tiles and between rows.
//!
//! [`justify`] places photos of known aspect ratios in rows near a target
//! height: it adds photos to a row until it is as wide as the view, then
//! ends the row with or without the last photo, whichever leaves its
//! height nearer the target, and scales the row to fill the width
//! exactly. The last row keeps the target height and starts at the left.
//! Positions are whole pixels; a row's rounding goes into its tiles'
//! widths so its right edge meets the view's.

/// The narrowest and widest aspect ratios a tile takes: a photo narrower
/// or wider than these is cropped to them, so neither a sliver nor a strip
/// wider than the view shows.
pub(crate) const ASPECT_RANGE: (f64, f64) = (0.25, 5.0);

/// A tile's place, in whole pixels from the top left of the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Place {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Place {
    /// Whether (`x`, `y`) is on the tile.
    pub(crate) fn contains(&self, x: f64, y: f64) -> bool {
        x >= f64::from(self.x)
            && x < f64::from(self.x + self.width)
            && y >= f64::from(self.y)
            && y < f64::from(self.y + self.height)
    }

    /// The middle of the tile across.
    pub(crate) fn middle_x(&self) -> f64 {
        f64::from(self.x) + f64::from(self.width) / 2.0
    }
}

/// One row: the items it holds and where it is down the layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Row {
    /// The first item of the row.
    pub(crate) start: usize,
    /// One past the last item.
    pub(crate) end: usize,
    pub(crate) y: i32,
    pub(crate) height: i32,
}

/// Every tile's place, row by row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Layout {
    pub(crate) places: Vec<Place>,
    pub(crate) rows: Vec<Row>,
    /// The height of the whole layout.
    pub(crate) height: i32,
}

impl Layout {
    /// The row that holds `item`.
    pub(crate) fn row_of(&self, item: usize) -> Option<usize> {
        let index = self.rows.partition_point(|row| row.end <= item);
        (index < self.rows.len()).then_some(index)
    }

    /// The rows that show between `top` and `bottom`, as a range of rows.
    pub(crate) fn rows_between(&self, top: i32, bottom: i32) -> std::ops::Range<usize> {
        let first = self.rows.partition_point(|row| row.y + row.height <= top);
        let last = self.rows.partition_point(|row| row.y < bottom);
        first..last.max(first)
    }

    /// The item at (`x`, `y`), if a tile is there.
    pub(crate) fn item_at(&self, x: f64, y: f64) -> Option<usize> {
        #[expect(clippy::cast_possible_truncation, reason = "layout coordinates are small")]
        let rows = self.rows_between(y.floor() as i32, y.floor() as i32 + 1);
        rows.flat_map(|row| self.rows[row].start..self.rows[row].end)
            .find(|item| self.places[*item].contains(x, y))
    }

    /// The item in the row above (`up`) or below `item` nearest its middle.
    pub(crate) fn item_beside_vertically(&self, item: usize, up: bool) -> Option<usize> {
        let row = self.row_of(item)?;
        let other = if up { row.checked_sub(1)? } else { row + 1 };
        let other = self.rows.get(other)?;
        let middle = self.places[item].middle_x();
        (other.start..other.end).min_by(|a, b| {
            let distance = |item: &usize| (self.places[*item].middle_x() - middle).abs();
            distance(a).total_cmp(&distance(b))
        })
    }
}

/// Lays out tiles of `aspects` (width over height; `None` for unknown,
/// shown square) in rows `width` pixels wide near `target` pixels tall,
/// `gap` pixels apart.
pub(crate) fn justify(aspects: &[Option<f64>], width: i32, target: i32, gap: i32) -> Layout {
    justify_rows(aspects, width, target, gap, false)
}

/// Lays out as [`justify`] does, but fills the width with the last row
/// too, as the rows above others kept in place end where those begin: it
/// is stretched when that leaves it at most half as tall again as the
/// target, else it joins the row before it.
pub(crate) fn justify_filled(aspects: &[Option<f64>], width: i32, target: i32, gap: i32) -> Layout {
    justify_rows(aspects, width, target, gap, true)
}

/// What changed in a layout's items: `added` items took the place of
/// `removed` ones at `position`, numbered as before the change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Change {
    pub(crate) position: usize,
    pub(crate) removed: usize,
    pub(crate) added: usize,
}

/// Lays out `aspects`, the items after `change`, keeping `kept`, rows of
/// `old` on screen, exactly as they are (HOME-039): the items changed
/// only below them are laid out again below them, and those changed only
/// above them above them, every row there filling the width. Returns the
/// layout and how far down the kept rows moved, for the scroll position
/// to follow, or `None` when the change touches the kept rows.
pub(crate) fn steady(
    old: &Layout,
    kept: std::ops::Range<usize>,
    aspects: &[Option<f64>],
    change: Change,
    (width, target, gap): (i32, i32, i32),
) -> Option<(Layout, i32)> {
    let (first, last) = (old.rows.get(kept.start)?, old.rows.get(kept.end.checked_sub(1)?)?);
    let (start, end) = (first.start, last.end);
    let mut layout = Layout {
        places: vec![Place::default(); aspects.len()],
        rows: Vec::new(),
        height: 0,
    };
    if change.position >= end {
        // Below: the rows down to the last kept one stay.
        layout.rows.extend_from_slice(&old.rows[..kept.end]);
        layout.places[..end].copy_from_slice(&old.places[..end]);
        let below = justify(&aspects[end..], width, target, gap);
        let top = last.y + last.height + gap;
        layout.append(&below, end, top);
        layout.height = if below.rows.is_empty() {
            last.y + last.height
        } else {
            top + below.height
        };
        return Some((layout, 0));
    }
    if change.position + change.removed > start {
        return None;
    }
    // Above: the rows from the first kept one on stay, moved down.
    let new_start = start + change.added - change.removed;
    let above = justify_filled(&aspects[..new_start], width, target, gap);
    layout.append(&above, 0, 0);
    let top = if above.rows.is_empty() {
        0
    } else {
        above.height + gap
    };
    let shift = top - first.y;
    let moved = new_start.wrapping_sub(start);
    for row in &old.rows[kept.start..] {
        layout.rows.push(Row {
            start: row.start.wrapping_add(moved),
            end: row.end.wrapping_add(moved),
            y: row.y + shift,
            height: row.height,
        });
    }
    for (item, place) in old.places.iter().enumerate().skip(start) {
        layout.places[item.wrapping_add(moved)] = Place {
            y: place.y + shift,
            ..*place
        };
    }
    layout.height = old.height + shift;
    Some((layout, shift))
}

impl Layout {
    /// Adds `other`'s rows and places, its items from `first` on and its
    /// rows from `top` down.
    fn append(&mut self, other: &Layout, first: usize, top: i32) {
        for row in &other.rows {
            self.rows.push(Row {
                start: row.start + first,
                end: row.end + first,
                y: row.y + top,
                height: row.height,
            });
        }
        for (item, place) in other.places.iter().enumerate() {
            self.places[first + item] = Place {
                y: place.y + top,
                ..*place
            };
        }
    }
}

/// [`justify`], with the last row filling the width when `fill_last`.
fn justify_rows(aspects: &[Option<f64>], width: i32, target: i32, gap: i32, fill_last: bool) -> Layout {
    let width = width.max(1);
    let target = target.max(1);
    let aspects: Vec<f64> = aspects
        .iter()
        .map(|aspect| {
            aspect
                .filter(|aspect| aspect.is_finite() && *aspect > 0.0)
                .unwrap_or(1.0)
                .clamp(ASPECT_RANGE.0, ASPECT_RANGE.1)
        })
        .collect();
    let mut layout = Layout {
        places: vec![Place::default(); aspects.len()],
        rows: Vec::new(),
        height: 0,
    };
    let mut y = 0;
    let mut start = 0;
    while start < aspects.len() {
        let (end, height) = row_end(&aspects[start..], width, target, gap);
        let end = start + end;
        let mut height = height;
        if fill_last && height.is_none() {
            let filled = full_height(&aspects[start..end], width, gap);
            if filled <= f64::from(target) * 1.5 {
                height = Some(filled);
            } else if let Some(before) = layout.rows.pop() {
                // Too tall alone: one row with the row before it.
                y = before.y;
                start = before.start;
                height = Some(full_height(&aspects[start..end], width, gap));
            }
        }
        let full = height.is_some();
        let height = height.unwrap_or(f64::from(target));
        #[expect(clippy::cast_possible_truncation, reason = "row heights are small")]
        let row_height = (height.round() as i32).max(1);
        place_row(
            &mut layout.places[start..end],
            &aspects[start..end],
            y,
            row_height,
            width,
            gap,
            full,
        );
        layout.rows.push(Row {
            start,
            end,
            y,
            height: row_height,
        });
        y += row_height + gap;
        start = end;
    }
    layout.height = (y - gap).max(0);
    layout
}

/// The height at which `aspects` in one row fill `width` exactly.
fn full_height(aspects: &[f64], width: i32, gap: i32) -> f64 {
    let gaps = f64::from(u32::try_from(aspects.len().saturating_sub(1)).unwrap_or(u32::MAX));
    let sum: f64 = aspects.iter().sum();
    (f64::from(width) - f64::from(gap) * gaps) / sum.max(f64::MIN_POSITIVE)
}

/// How many of `aspects` the next row takes, and its height when it is
/// full; `None` when they all fit short of the width, as the last row.
fn row_end(aspects: &[f64], width: i32, target: i32, gap: i32) -> (usize, Option<f64>) {
    let mut sum = 0.0;
    let target = f64::from(target);
    for (index, aspect) in aspects.iter().enumerate() {
        let count = index + 1;
        sum += aspect;
        let gaps = f64::from(u32::try_from(index).unwrap_or(u32::MAX));
        let room = f64::from(width) - f64::from(gap) * gaps;
        if sum * target >= room {
            let with = room / sum;
            if index > 0 {
                let before = sum - aspect;
                let room_before = room + f64::from(gap);
                let without = room_before / before;
                // Nearer the target by ratio: a row twice as tall is as
                // far as one half as tall.
                if (without / target).ln().abs() < (with / target).ln().abs() {
                    return (index, Some(without));
                }
            }
            return (count, Some(with));
        }
    }
    (aspects.len(), None)
}

/// Places one row's tiles at `y`, `height` tall; a `full` row's widths
/// are stretched to end exactly at `width`.
fn place_row(places: &mut [Place], aspects: &[f64], y: i32, height: i32, width: i32, gap: i32, full: bool) {
    let room = f64::from(width - gap * (i32::try_from(places.len()).unwrap_or(i32::MAX) - 1));
    let sum: f64 = aspects.iter().sum();
    let mut x = 0;
    let mut covered = 0.0;
    for (index, (place, aspect)) in places.iter_mut().zip(aspects).enumerate() {
        let tile_width = if full {
            covered += aspect / sum * room;
            let gaps = gap * i32::try_from(index).unwrap_or(0);
            #[expect(clippy::cast_possible_truncation, reason = "row widths are small")]
            let end = covered.round() as i32 + gaps;
            end - x
        } else {
            #[expect(clippy::cast_possible_truncation, reason = "tile widths are small")]
            let natural = (aspect * f64::from(height)).round() as i32;
            natural.min(width)
        };
        *place = Place {
            x,
            y,
            width: tile_width.max(1),
            height,
        };
        x += tile_width + gap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full rows fill the width exactly with even gaps, keep one height
    /// near the target, and keep each tile's aspect ratio; the last row
    /// keeps the target height from the left.
    ///
    /// parity: HOME-026
    #[test]
    fn rows_fill_the_width_and_keep_aspect_ratios() {
        let aspects: Vec<Option<f64>> = [1.78, 1.0, 0.4, 3.0, 0.75, 1.5, 1.5, 0.42, 1.33, 1.33]
            .into_iter()
            .map(Some)
            .collect();
        let layout = justify(&aspects, 1000, 160, 4);
        assert!(layout.rows.len() >= 2);
        let last = layout.rows.len() - 1;
        for (index, row) in layout.rows.iter().enumerate() {
            let places = &layout.places[row.start..row.end];
            assert_eq!(places[0].x, 0);
            for pair in places.windows(2) {
                assert_eq!(pair[1].x - (pair[0].x + pair[0].width), 4);
            }
            let right = places.last().map(|place| place.x + place.width);
            if index < last || row.height != 160 {
                // A full row, the last one too when it reaches the width.
                assert_eq!(right, Some(1000));
                assert!((110..=230).contains(&row.height), "{row:?}");
            } else {
                assert!(right.is_some_and(|right| right < 1000));
            }
            for (place, aspect) in places.iter().zip(&aspects[row.start..row.end]) {
                let drawn = f64::from(place.width) / f64::from(place.height);
                let aspect = aspect.expect("known");
                assert!((drawn - aspect).abs() / aspect < 0.05, "{place:?} {aspect}");
            }
        }
        for pair in layout.rows.windows(2) {
            assert_eq!(pair[1].y, pair[0].y + pair[0].height + 4);
        }
        let end = layout.rows[last];
        assert_eq!(layout.height, end.y + end.height);
    }

    /// Unknown, extreme and empty inputs still lay out.
    ///
    /// parity: HOME-026
    #[test]
    fn odd_aspects_are_clamped_and_unknown_ones_are_square() {
        assert_eq!(justify(&[], 500, 160, 4), Layout::default());
        let layout = justify(&[None, Some(f64::NAN), Some(100.0), Some(0.01)], 2000, 100, 4);
        assert_eq!(layout.places[0].width, layout.places[0].height);
        assert_eq!(layout.places[1].width, layout.places[1].height);
        let wide = layout.places[2];
        assert!(wide.width <= wide.height * 5 + 1, "{wide:?}");
        // A panorama wider than the view gets a row to itself.
        let layout = justify(&[Some(5.0), Some(1.0)], 300, 160, 4);
        assert_eq!(layout.rows[0].end, 1);
        assert_eq!(layout.places[0].width, 300);
    }

    /// The rows on screen, hit testing and the tile above or below.
    ///
    /// parity: HOME-026
    #[test]
    fn the_layout_finds_rows_items_and_neighbours() {
        let layout = justify(&[Some(1.0); 12], 340, 100, 4);
        // Three tiles of 100 and their gaps fit 340 across as 108 or so.
        let first = layout.rows[0];
        assert_eq!(first.end - first.start, 3);
        assert_eq!(layout.rows_between(0, 1), 0..1);
        assert_eq!(layout.rows_between(0, layout.height), 0..layout.rows.len());
        let place = layout.places[4];
        assert_eq!(layout.item_at(place.middle_x(), f64::from(place.y + 2)), Some(4));
        assert_eq!(
            layout.item_at(f64::from(place.x - 2), f64::from(place.y + 2)),
            None
        );
        assert_eq!(layout.item_beside_vertically(4, true), Some(1));
        assert_eq!(layout.item_beside_vertically(4, false), Some(7));
        assert_eq!(layout.item_beside_vertically(1, true), None);
        assert_eq!(layout.row_of(11), Some(3));
        assert_eq!(layout.row_of(12), None);
    }

    /// Photos added below or above the rows on screen leave those rows
    /// exactly as they were, moved down by what was added above them;
    /// the rows above fill the width; a change among them is refused.
    ///
    /// parity: HOME-039
    #[test]
    fn rows_on_screen_stay_as_photos_come_above_and_below_them() {
        let aspects = |count: usize, from: usize| -> Vec<Option<f64>> {
            (from..from + count)
                .map(|index| Some([1.5, 0.75, 1.0, 1.33, 2.0][index % 5]))
                .collect()
        };
        let sizes = (1000, 160, 4);
        let old_aspects = aspects(60, 0);
        let old = justify(&old_aspects, 1000, 160, 4);
        let kept = 3..6;
        let (start, end) = (old.rows[3].start, old.rows[5].end);
        let kept_places = old.places[start..end].to_vec();
        let kept_rows = old.rows[kept.clone()].to_vec();

        // Below: 30 more after the kept rows.
        let mut more = old_aspects.clone();
        more.splice(end + 4..end + 4, aspects(30, 7));
        let change = Change {
            position: end + 4,
            removed: 0,
            added: 30,
        };
        let (below, shift) = steady(&old, kept.clone(), &more, change, sizes).expect("below");
        assert_eq!(shift, 0);
        assert_eq!(&below.places[start..end], &kept_places[..]);
        assert_eq!(&below.rows[kept.clone()], &kept_rows[..]);
        assert_eq!(below.places.len(), 90);
        let last = below.rows.last().expect("rows");
        assert_eq!(last.end, 90);
        assert_eq!(below.height, last.y + last.height);

        // Above: 23 more before them.
        let mut more = old_aspects.clone();
        more.splice(2..2, aspects(23, 3));
        let change = Change {
            position: 2,
            removed: 0,
            added: 23,
        };
        let (above, shift) = steady(&old, kept.clone(), &more, change, sizes).expect("above");
        assert!(shift > 0);
        for (index, place) in kept_places.iter().enumerate() {
            let moved = above.places[start + 23 + index];
            assert_eq!(
                (moved.x, moved.width, moved.height),
                (place.x, place.width, place.height)
            );
            assert_eq!(moved.y, place.y + shift);
        }
        let first_kept = above.row_of(start + 23).expect("a row");
        assert_eq!(above.rows[first_kept].y, kept_rows[0].y + shift);
        assert_eq!(above.height, old.height + shift);
        // Every row above them fills the width, one gap apart.
        for row in &above.rows[..first_kept] {
            let right = above.places[row.end - 1].x + above.places[row.end - 1].width;
            assert_eq!(right, 1000, "{row:?}");
        }
        for pair in above.rows.windows(2) {
            assert_eq!(pair[1].y, pair[0].y + pair[0].height + 4);
        }

        // Among them: refused.
        let mut more = old_aspects.clone();
        more.splice(start + 1..=start, aspects(2, 0));
        let change = Change {
            position: start + 1,
            removed: 0,
            added: 2,
        };
        assert_eq!(steady(&old, kept, &more, change, sizes), None);
    }

    /// A filled layout's last row reaches the width: stretched when that
    /// keeps it near the target, else joined to the row before it.
    ///
    /// parity: HOME-039
    #[test]
    fn a_filled_layout_ends_with_a_full_row() {
        for count in 1..40 {
            let aspects: Vec<Option<f64>> = (0..count)
                .map(|index| Some([1.5, 0.75, 1.0][index % 3]))
                .collect();
            let layout = justify_filled(&aspects, 1000, 160, 4);
            for row in &layout.rows {
                let right = layout.places[row.end - 1].x + layout.places[row.end - 1].width;
                if count > 3 {
                    assert_eq!(right, 1000, "{count} {row:?}");
                }
                assert!(row.height <= 240 || layout.rows.len() == 1, "{count} {row:?}");
            }
            assert_eq!(layout.rows.last().map(|row| row.end), Some(count));
        }
    }
}
