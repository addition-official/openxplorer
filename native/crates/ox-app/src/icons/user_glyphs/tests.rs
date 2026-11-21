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
        (Icon::ChevronRight16, 0xE970),
        (Icon::ChevronDown16, 0xE96E),
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
        assert_eq!(drawn_name(icon), icon.name(), "{icon:?}");
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
    let codepoints = [0xE710, 0xE72B, 0xE970, 0xE96E];
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
        Status::Ready {
            found: 4,
            wanted: glyph_count()
        }
    );
    assert_eq!(drawn_name(Icon::Add), "ox-win-add-20-symbolic");
    assert_eq!(drawn_name(Icon::Cut), Icon::Cut.name(), "the font lacks it");
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
    assert_eq!(back.icon_name().as_deref(), Some("ox-win-arrow-left-20-symbolic"));

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
    assert_eq!(drawn_name(Icon::Add), Icon::Add.name());
}

/// The chevrons the skin draws in CSS are drawn from the font through
/// rules that repeat only their icon declarations, the names replaced;
/// nothing when no name is.
///
/// parity: LOOK-031
#[gtk::test]
fn the_skins_chevrons_follow_through_an_override_stylesheet() {
    let rules = crate::theme::skin_rules();
    let drawn = |icon: Icon| -> &'static str {
        match icon {
            Icon::ChevronRight16 => "ox-win-chevron-right-16-symbolic",
            Icon::ChevronDown16 => "ox-win-chevron-down-16-symbolic",
            other => other.name(),
        }
    };
    let stylesheet = chevron_stylesheet(rules, drawn);
    assert!(stylesheet.contains(".address .crumb-divider { background-image: -gtk-icontheme(\"ox-win-chevron-right-16-symbolic\"); }"), "{stylesheet}");
    assert!(
        stylesheet.contains("-gtk-icon-source: -gtk-icontheme(\"ox-win-chevron-down-16-symbolic\")"),
        "{stylesheet}"
    );
    assert!(!stylesheet.contains("min-width"), "only the icon declarations");
    assert!(
        !stylesheet.contains("\"ox-chevron-right-16-symbolic\""),
        "every use replaced"
    );
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&stylesheet);
    assert!(chevron_stylesheet(rules, Icon::name).is_empty());
}
