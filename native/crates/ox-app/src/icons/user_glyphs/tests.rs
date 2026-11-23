// SPDX-License-Identifier: AGPL-3.0-only
//! Tests of glyphs from a Windows symbol font, on a synthetic font built
//! in the tests ([`ox_core::windows_glyphs::synthetic`]): never a
//! Microsoft font.

use gtk::prelude::*;
use ox_core::settings::Preferences;
use ox_core::windows_glyphs::synthetic::SyntheticFont;

use super::*;

/// Turns the glyphs off again when a test ends, also when it fails.
struct Reset;

impl Drop for Reset {
    fn drop(&mut self) {
        follow_preferences(&Preferences::default());
    }
}

/// Preferences with Windows icon files on and `font` chosen.
fn with_font(font: &std::path::Path) -> Preferences {
    Preferences {
        use_icon_files: true,
        symbol_font: Some(font.display().to_string()),
        ..Preferences::default()
    }
}

/// The codepoint `icon` is drawn with.
fn codepoint(icon: Icon) -> Option<u32> {
    glyph_of(icon).map(|glyph| glyph.codepoint)
}

/// Every bundled icon has its Windows counterpart recorded: a glyph of
/// the symbol font, by Microsoft's published codepoints in the Private
/// Use Area; a colour picture; or the reason there is none. The places
/// the owner listed are all glyphs.
///
/// parity: LOOK-031
#[test]
fn every_icon_has_its_windows_counterpart_recorded() {
    for icon in ALL_ICONS {
        let symbolic = icon.name().ends_with("-symbolic");
        match windows_source(icon) {
            WindowsSource::Glyph(glyph) => {
                assert!(symbolic, "{icon:?} is a glyph, so symbolic");
                assert!((0xE000..=0xF8FF).contains(&glyph.codepoint), "{icon:?}");
                assert!(!glyph.name.is_empty());
            }
            WindowsSource::Picture => assert!(!symbolic, "{icon:?} is a colour picture"),
            WindowsSource::None(reason) => {
                assert!(symbolic, "{icon:?}");
                assert!(!reason.is_empty());
            }
        }
    }
    // The command bar, the navigation buttons, the address bar, search,
    // the panes, the tabs, the menus, the drop-down carets, Settings and
    // the status bar's view buttons.
    let expected = [
        (Icon::Add, 0xE710),
        (Icon::Cut, 0xE8C6),
        (Icon::Copy, 0xE8C8),
        (Icon::ClipboardPaste, 0xE77F),
        (Icon::Rename, 0xE8AC),
        (Icon::Share, 0xE72D),
        (Icon::Delete, 0xE74D),
        (Icon::ArrowSort, 0xE8CB),
        (Icon::Grid, 0xE8A9),
        (Icon::MoreHorizontal, 0xE712),
        (Icon::Settings, 0xE713),
        (Icon::PanelRight, 0xE90D),
        (Icon::PanelRightGallery, 0xE8A1),
        (Icon::ArrowLeft, 0xE72B),
        (Icon::ArrowRight, 0xE72A),
        (Icon::ArrowUp, 0xE74A),
        (Icon::ArrowClockwise, 0xE72C),
        (Icon::ChevronRight16, 0xE76C),
        (Icon::ChevronDown16, 0xE70D),
        (Icon::Dismiss, 0xE711),
        (Icon::Search, 0xE721),
        (Icon::Dismiss16, 0xE711),
        (Icon::Checkmark, 0xE73E),
        (Icon::CircleSmallFilled, 0xE915),
        (Icon::ChevronDown, 0xE70D),
        (Icon::TextBulletList, 0xE8FD),
    ];
    for (icon, expected) in expected {
        assert_eq!(codepoint(icon), Some(expected), "{icon:?}");
    }
    assert!(glyph_count() > 70, "most glyphs have one");
}

/// Off by default: every icon is drawn by its bundled name.
///
/// parity: LOOK-031
#[gtk::test]
fn glyphs_are_off_by_default() {
    let _reset = Reset;
    follow_preferences(&Preferences::default());
    assert_eq!(status(), Status::Off);
    for icon in ALL_ICONS {
        assert_eq!(drawn_name(icon, 16), icon.name(), "{icon:?}");
    }
}

/// With the font chosen and the switch on, an icon whose glyph the font
/// has is drawn by the glyph's symbolic icon, in the image's colour, and
/// one the font lacks keeps its own; images already in a window follow,
/// and turning it off brings every bundled icon back.
///
/// parity: LOOK-031
#[gtk::test]
fn the_fonts_glyphs_replace_the_bundled_ones_and_go_away_again() {
    let _reset = Reset;
    let folder = tempfile::tempdir().expect("a temporary folder");
    let font = folder.path().join("SegoeIcons.ttf");
    let codepoints = [0xE710, 0xE72B, 0xE76C, 0xE70D];
    std::fs::write(&font, SyntheticFont::with_codepoints(&codepoints).build()).expect("written");
    let window = gtk::Window::new();
    let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let add = crate::icons::image(Icon::Add, 16);
    let cut = crate::icons::image(Icon::Cut, 16);
    column.append(&add);
    column.append(&cut);
    window.set_child(Some(&column));

    follow_preferences(&with_font(&font));
    wait_until_read();
    assert_eq!(
        status(),
        // E70D draws both chevrons down.
        Status::Ready {
            found: 5,
            wanted: glyph_count()
        }
    );
    assert_eq!(drawn_name(Icon::Add, 16), "ox-win-add-20-symbolic");
    assert_eq!(drawn_name(Icon::Cut, 16), Icon::Cut.name(), "the font lacks it");
    assert_eq!(
        add.icon_name().as_deref(),
        Some("ox-win-add-20-symbolic"),
        "a shown image follows"
    );
    assert_eq!(add.pixel_size(), 16);
    assert_eq!(cut.icon_name().as_deref(), Some(Icon::Cut.name()));
    let theme = gtk::IconTheme::for_display(&add.display());
    let looked_up = theme.lookup_icon(
        "ox-win-add-20-symbolic",
        &[],
        16,
        1,
        gtk::TextDirection::Ltr,
        gtk::IconLookupFlags::empty(),
    );
    // Whether an icon is symbolic is known once it is loaded.
    looked_up.snapshot(&gtk::Snapshot::new(), 16.0, 16.0);
    assert!(looked_up.is_symbolic(), "drawn in the image's colour");
    let file = looked_up
        .file()
        .and_then(|file| file.path())
        .expect("from the cache");
    assert!(file.starts_with(glib::user_cache_dir()), "{file:?}");
    let back = crate::icons::image(Icon::ArrowLeft, 20);
    assert_eq!(
        back.icon_name().as_deref(),
        Some("ox-win-arrow-left-20-box20-symbolic"),
        "a 20-pixel square draws the 16-pixel em"
    );

    follow_preferences(&Preferences::default());
    assert_eq!(
        add.icon_name().as_deref(),
        Some(Icon::Add.name()),
        "back to its own"
    );
    assert!(!theme
        .search_path()
        .iter()
        .any(|path| path.starts_with(ox_core::windows_glyphs::glyph_cache_folder())));
    window.destroy();
}

/// A damaged font gives no glyph, and every icon keeps its own.
///
/// parity: LOOK-031
#[gtk::test]
fn a_damaged_font_keeps_the_bundled_glyphs() {
    let _reset = Reset;
    let folder = tempfile::tempdir().expect("a temporary folder");
    let font = folder.path().join("segmdl2.ttf");
    std::fs::write(&font, b"\0\x01\0\0 and nothing a font needs").expect("written");
    follow_preferences(&with_font(&font));
    wait_until_read();
    assert_eq!(
        status(),
        Status::Ready {
            found: 0,
            wanted: glyph_count()
        }
    );
    assert_eq!(drawn_name(Icon::Add, 16), Icon::Add.name());
}

/// The chevrons the skin draws in CSS are drawn from the font through
/// rules that repeat only their icon declarations, the names replaced;
/// nothing when no name is.
///
/// parity: LOOK-031
#[gtk::test]
fn the_skins_chevrons_follow_through_an_override_stylesheet() {
    let rules = crate::theme::skin_rules();
    // The names the squares give, so each rule shows which square it took.
    let drawn = |icon: Icon, square: i32| -> &'static str {
        match (icon, square) {
            (Icon::ChevronRight16, 12) => "ox-win-chevron-right-16-box12-symbolic",
            (Icon::ChevronLeft16, 12) => "ox-win-chevron-left-16-box12-symbolic",
            (Icon::ChevronDown16, 16) => "ox-win-chevron-down-16-symbolic",
            (Icon::ChevronDown16, 10) => "ox-win-chevron-down-16-box10-symbolic",
            (other, _) => other.name(),
        }
    };
    let stylesheet = chevron_stylesheet(rules, drawn);
    let has = |text: &str| stylesheet.contains(text);
    assert!(
        has(".address .crumb-divider { background-image: -gtk-icontheme(\"ox-win-chevron-right-16-box12-symbolic\"); }"),
        "the divider's 12-pixel square: {stylesheet}"
    );
    assert!(
        has(".address .crumb-divider:dir(rtl) { background-image: -gtk-icontheme(\"ox-win-chevron-left-16-box12-symbolic\"); }"),
        "the right-to-left divider takes the divider's square: {stylesheet}"
    );
    assert!(
        has("-gtk-icon-source: -gtk-icontheme(\"ox-win-chevron-down-16-symbolic\")"),
        "the expander's 16-pixel square: {stylesheet}"
    );
    assert!(
        has("-gtk-icontheme(\"ox-win-chevron-down-16-box10-symbolic\")"),
        "the sort arrow's 10-pixel square: {stylesheet}"
    );
    assert!(!stylesheet.contains("min-width"), "only the icon declarations");
    assert!(
        !stylesheet.contains("\"ox-chevron-right-16-symbolic\""),
        "every use replaced"
    );
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&stylesheet);
    assert!(chevron_stylesheet(rules, |icon, _| icon.name()).is_empty());
}

/// The rows and columns of `texture` any ink covers: its inked bounding
/// box, `None` when nothing is drawn.
fn inked_box(texture: &gtk::gdk::Texture) -> Option<(i32, i32, i32, i32)> {
    let downloader = gtk::gdk::TextureDownloader::new(texture);
    let (bytes, stride) = downloader.download_bytes();
    let (width, height) = (texture.width(), texture.height());
    let mut found: Option<(i32, i32, i32, i32)> = None;
    for y in 0..height {
        for x in 0..width {
            let alpha = bytes
                [usize::try_from(y).expect("a row") * stride + usize::try_from(x).expect("a column") * 4 + 3];
            if alpha > 127 {
                found = Some(match found {
                    None => (x, y, x, y),
                    Some((left, top, right, bottom)) => {
                        (left.min(x), top.min(y), right.max(x), bottom.max(y))
                    }
                });
            }
        }
    }
    found
}

/// The inked box of the icon `name` as the icon theme of `image`'s
/// display paints it `size` pixels square.
fn painted_box(image: &gtk::Image, name: &str, size: i32) -> Option<(i32, i32, i32, i32)> {
    let theme = gtk::IconTheme::for_display(&image.display());
    let icon = theme.lookup_icon(
        name,
        &[],
        size,
        1,
        gtk::TextDirection::Ltr,
        gtk::IconLookupFlags::empty(),
    );
    let renderer = gtk::gsk::CairoRenderer::new();
    renderer
        .realize_for_display(&image.display())
        .expect("the Cairo renderer needs no GPU");
    let snapshot = gtk::Snapshot::new();
    let edge = f64::from(size);
    icon.snapshot(&snapshot, edge, edge);
    let node = snapshot.to_node()?;
    #[expect(clippy::cast_precision_loss, reason = "a small icon")]
    let bounds = gtk::graphene::Rect::new(0.0, 0.0, size as f32, size as f32);
    let texture = renderer.render_texture(node, Some(&bounds));
    renderer.unrealize();
    inked_box(&texture)
}

/// Drawn through GTK at a 16-pixel em, a glyph keeps the font's own
/// side bearings and baseline: its ink is the outline's box in font units
/// scaled by 16 / units per em, smaller than the em, not stretched to it.
///
/// parity: LOOK-031
#[gtk::test]
fn a_glyph_is_drawn_at_its_em_with_the_fonts_own_padding() {
    use ox_core::windows_glyphs::synthetic::{SyntheticGlyph, UNITS_PER_EM};

    let _reset = Reset;
    let folder = tempfile::tempdir().expect("a temporary folder");
    let font_path = folder.path().join("SegoeIcons.ttf");
    // A square from 256 to 768 units of a 1024-unit em: 4 to 12 pixels at
    // a 16-pixel em, its baseline at the em's bottom.
    let font = SyntheticFont {
        glyphs: vec![SyntheticGlyph::Simple(vec![vec![
            (256, 256, true),
            (768, 256, true),
            (768, 768, true),
            (256, 768, true),
        ]])],
        mapping: vec![(0xE8C6, 1)],
        format_12: false,
    };
    assert_eq!(UNITS_PER_EM, 1024);
    std::fs::write(&font_path, font.build()).expect("written");
    follow_preferences(&with_font(&font_path));
    wait_until_read();
    let image = crate::icons::image(Icon::Cut, 16);
    let name = image.icon_name().expect("a name").to_string();
    assert!(name.starts_with("ox-win-"), "{name}");
    let ink = painted_box(&image, &name, 16).expect("ink");
    assert_eq!(ink, (4, 4, 11, 11), "the outline at 16 / 1024 of its units");
}

/// Windows' em in each square: the square itself up to 16 pixels, 16
/// pixels in the app's 17 to 24 pixel squares, and the square again for
/// icons the user enlarged; a padded square has an icon name of its own,
/// which maps back to the bundled icon.
///
/// parity: LOOK-031
#[test]
fn each_square_draws_the_glyph_at_windows_em() {
    for (size, em) in [
        (12, 12),
        (16, 16),
        (17, 16),
        (20, 16),
        (24, 16),
        (32, 32),
        (48, 48),
    ] {
        assert_eq!(windows_em(size), em, "{size}");
    }
    assert_eq!(glyph_icon_name(Icon::Add, 16), "ox-win-add-20-symbolic");
    assert_eq!(glyph_icon_name(Icon::Add, 17), "ox-win-add-20-box17-symbolic");
}

/// A glyph in the command bar's 17-pixel square is drawn at Windows'
/// 16-pixel em, centred, not grown to fill the square.
///
/// parity: LOOK-031
#[gtk::test]
fn a_larger_square_draws_the_glyph_at_a_16_pixel_em() {
    use ox_core::windows_glyphs::synthetic::SyntheticGlyph;

    let _reset = Reset;
    let folder = tempfile::tempdir().expect("a temporary folder");
    let font_path = folder.path().join("SegoeIcons.ttf");
    // The whole em, so its ink shows the em's size.
    let font = SyntheticFont {
        glyphs: vec![SyntheticGlyph::Simple(vec![vec![
            (0, 0, true),
            (1024, 0, true),
            (1024, 1024, true),
            (0, 1024, true),
        ]])],
        mapping: vec![(0xE8C6, 1)],
        format_12: false,
    };
    std::fs::write(&font_path, font.build()).expect("written");
    follow_preferences(&with_font(&font_path));
    wait_until_read();
    let image = crate::icons::image(Icon::Cut, 20);
    let name = image.icon_name().expect("a name").to_string();
    assert_eq!(name, "ox-win-cut-20-box20-symbolic");
    assert_eq!(image.pixel_size(), 20, "the square keeps its size");
    let ink = painted_box(&image, &name, 20).expect("ink");
    assert_eq!(ink, (2, 2, 17, 17), "16 pixels of em in the middle of 20");
}

/// Chevrons are drawn at the ems measured on Windows 11 at 1x: 12 in a
/// 16-pixel square (a submenu's arrow), 10 in a 12-pixel one (a
/// breadcrumb separator) and 8 elsewhere; a drop-down button's caret in
/// the command bar's 17-pixel square inks 8 whole pixels, not the 16 of
/// the glyphs beside it, and not 9 blurred across a half pixel.
///
/// parity: LOOK-031
#[gtk::test]
fn chevrons_are_drawn_at_windows_ems() {
    use ox_core::windows_glyphs::synthetic::SyntheticGlyph;

    for (size, em) in [(12, 10), (16, 12), (17, 8), (20, 8), (10, 8), (8, 8)] {
        assert_eq!(em_of(Icon::ChevronRight16, size), em, "{size}");
    }
    for size in [12, 17] {
        assert_eq!(em_of(Icon::ChevronDown, size), 8, "the caret in {size}");
    }
    assert_eq!(em_of(Icon::Cut, 17), 16);
    let _reset = Reset;
    let folder = tempfile::tempdir().expect("a temporary folder");
    let font_path = folder.path().join("SegoeIcons.ttf");
    let font = SyntheticFont {
        glyphs: vec![SyntheticGlyph::Simple(vec![vec![
            (0, 0, true),
            (1024, 0, true),
            (1024, 1024, true),
            (0, 1024, true),
        ]])],
        mapping: vec![(0xE70D, 1)],
        format_12: false,
    };
    std::fs::write(&font_path, font.build()).expect("written");
    follow_preferences(&with_font(&font_path));
    wait_until_read();
    let image = crate::icons::image(Icon::ChevronDown, 17);
    let name = image.icon_name().expect("a name").to_string();
    assert_eq!(name, "ox-win-chevron-down-20-box17-symbolic");
    assert_eq!(
        painted_box(&image, &name, 17),
        Some((4, 4, 11, 11)),
        "an 8-pixel em on whole pixels in 17"
    );
}
