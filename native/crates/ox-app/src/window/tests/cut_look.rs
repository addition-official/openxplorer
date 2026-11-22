// SPDX-License-Identifier: AGPL-3.0-only
//! How an item a cut put on the clipboard looks (CLIP-002): as in Windows
//! 11 File Explorer, only its icon fades; its name and, in Details, its
//! Date modified, Type and Size stay at full strength. Checked on the
//! drawn pixels, against an item of the same type that is not cut.

use gtk::prelude::*;

use super::file_ops_support::select_names;
use super::icons::{pixel_rows, Pixel};
use crate::folder_view::cells::FileCell;
use crate::test_support::harness::{descendants, wait_for_frames, wait_until, Fixture, TestWindow};

/// A rectangle of the window, in whole pixels.
#[derive(Debug, Clone, Copy)]
struct Area {
    left: usize,
    top: usize,
    width: usize,
    height: usize,
}

impl Area {
    /// Where `widget` is drawn in `window`.
    fn of(widget: &impl IsA<gtk::Widget>, window: &gtk::Widget) -> Self {
        let bounds = widget.compute_bounds(window).expect("the widget is laid out");
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "window pixels"
        )]
        Self {
            left: bounds.x() as usize,
            top: bounds.y() as usize,
            width: bounds.width() as usize,
            height: bounds.height() as usize,
        }
    }
}

/// How light a pixel is, 0 to 255.
fn lightness([blue, green, red, _]: Pixel) -> u32 {
    (u32::from(red) * 299 + u32::from(green) * 587 + u32::from(blue) * 114) / 1000
}

/// The pixels of `area` in `rows`.
fn pixels_in(rows: &[Vec<Pixel>], area: Area) -> Vec<Pixel> {
    rows[area.top..area.top + area.height]
        .iter()
        .flat_map(|row| row[area.left..area.left + area.width].iter().copied())
        .collect()
}

/// The darkest pixel of `area`: how strong its text is drawn.
fn ink(rows: &[Vec<Pixel>], area: Area) -> u32 {
    pixels_in(rows, area)
        .into_iter()
        .map(lightness)
        .min()
        .unwrap_or(255)
}

/// The window as drawn now, as rows of BGRA pixels.
fn drawn(test: &TestWindow) -> Vec<Vec<Pixel>> {
    wait_for_frames(&test.window, 3);
    let window: gtk::Widget = test.window.clone().upcast();
    let paintable = gtk::WidgetPaintable::new(Some(&window));
    let snapshot = gtk::Snapshot::new();
    paintable.snapshot(&snapshot, f64::from(window.width()), f64::from(window.height()));
    let node = snapshot.to_node().expect("the window draws");
    let renderer = window
        .native()
        .and_then(|native| native.renderer())
        .expect("a drawn window");
    pixel_rows(&renderer.render_texture(&node, None::<&gtk::graphene::Rect>))
}

/// The name cell of the item shown as `name` (extensions are hidden, as
/// in Windows).
fn cell_named(test: &TestWindow, name: &str) -> FileCell {
    let view = test.window.folder_pane().view_widget();
    descendants::<FileCell>(&view)
        .into_iter()
        .find(|cell| cell.is_mapped() && cell.group_heading().is_none() && cell.name() == name)
        .unwrap_or_else(|| panic!("an item shows {name}"))
}

/// How the item called `name` is drawn: the ink of its icon, of its
/// name's ink and, with `columns`, the ink of the rest of its row.
fn item_look(test: &TestWindow, rows: &[Vec<Pixel>], name: &str, columns: bool) -> (u32, u32, Option<u32>) {
    let window: gtk::Widget = test.window.clone().upcast();
    let cell = cell_named(test, name);
    let (icon, label) = cell.icon_and_name();
    let cell_area = Area::of(&cell, &window);
    let rest = columns.then(|| {
        let left = cell_area.left + cell_area.width;
        let width = usize::try_from(window.width()).expect("a width") - left - 30;
        ink(
            rows,
            Area {
                left,
                width,
                ..cell_area
            },
        )
    });
    (
        ink(rows, Area::of(&icon, &window)),
        ink(rows, Area::of(&label, &window)),
        rest,
    )
}

/// Cuts `Notes 10.txt` of a standard fixture in `view` and leaves nothing
/// selected, so no selection colour is drawn behind either item.
fn cut_notes_in(view: &str) -> (Fixture, TestWindow) {
    let fixture = Fixture::standard();
    let test = TestWindow::open(&fixture.uri());
    test.activate("view", Some(view));
    test.wait_for_listing("the fixture folder");
    select_names(&test, &["Notes 10.txt"]);
    test.activate("cut", None);
    let model = test.window.folder_model();
    let position = (0..model.n_items())
        .find(|position| {
            model
                .item(*position)
                .and_downcast::<crate::folder_view::item::FileItem>()
                .is_some_and(|item| item.entry().name == "Notes 10.txt")
        })
        .expect("the cut item is listed");
    wait_until("the cut item to be marked", || {
        test.window.folder_pane().owners().is_shown_cut(position) == Some(true)
    });
    select_names(&test, &[]);
    (fixture, test)
}

/// Asserts that the cut item's text is drawn as strongly as the other's
/// and its icon visibly lighter.
fn assert_only_the_icon_fades(test: &TestWindow, columns: bool) {
    let rows = drawn(test);
    let (cut_icon, cut_name, cut_rest) = item_look(test, &rows, "Notes 10", columns);
    let (icon, name, rest) = item_look(test, &rows, "Notes 2", columns);
    assert!(
        cut_name.abs_diff(name) <= 8,
        "the cut item's name keeps its strength: {cut_name} against {name}"
    );
    if let (Some(cut_rest), Some(rest)) = (cut_rest, rest) {
        assert!(
            cut_rest.abs_diff(rest) <= 8,
            "the cut item's date, type and size keep their strength: {cut_rest} against {rest}"
        );
    }
    assert!(
        cut_icon >= icon + 40,
        "the cut item's icon fades: {cut_icon} against {icon}"
    );
}

/// parity: CLIP-002
#[gtk::test]
fn a_cut_item_fades_only_its_icon_in_details() {
    let (_fixture, test) = cut_notes_in("details");
    assert_only_the_icon_fades(&test, true);
}

/// parity: CLIP-002
#[gtk::test]
fn a_cut_item_fades_only_its_icon_in_large_icons() {
    let (_fixture, test) = cut_notes_in("large");
    assert_only_the_icon_fades(&test, false);
}
