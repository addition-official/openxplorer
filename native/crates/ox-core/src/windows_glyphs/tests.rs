// SPDX-License-Identifier: AGPL-3.0-only
//! Tests of reading glyphs from a symbol font, on synthetic fonts built in
//! the tests ([`super::synthetic`]): no Microsoft font is used.

use super::synthetic::{SyntheticFont, SyntheticGlyph};
use super::*;
use crate::test_support::temporary_folder;

/// The commands of the path in `svg`, each with its coordinates: "M0 0",
/// "L1 1", "Q1 0 2 2", "Z".
fn commands(svg: &str) -> Vec<String> {
    let start = svg.find(" d=\"").expect("a path") + 4;
    let data = &svg[start..svg[start..].find('"').expect("a closed path") + start];
    let mut commands: Vec<String> = Vec::new();
    for character in data.chars() {
        if character.is_ascii_alphabetic() {
            commands.push(character.to_string());
        } else if let Some(last) = commands.last_mut() {
            if last.len() == 1 {
                last.push(' ');
            }
            last.push(character);
        }
    }
    for command in &mut commands {
        // "M 0 1024" back to "M0 1024".
        *command = command.replacen(' ', "", 1);
    }
    commands
}

/// A tiny pseudo-random sequence, the same in every run.
struct Shuffle(u64);

impl Shuffle {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Reads every glyph of `bytes` at `codepoints`; what matters is that it
/// returns.
fn read_everything(bytes: Vec<u8>, codepoints: &[u32]) {
    if let Ok(font) = Font::parse(bytes) {
        for &codepoint in codepoints {
            let _ = font.glyph_svg(codepoint, 16);
        }
    }
}

/// parity: LOOK-031
#[test]
fn glyphs_are_found_by_codepoint_in_format_4_and_12() {
    for format_12 in [false, true] {
        let font = SyntheticFont {
            format_12,
            ..SyntheticFont::with_codepoints(&[0xE710, 0xE72B, 0xE8C6])
        };
        let font = Font::parse(font.build()).expect("a valid font");
        assert_eq!(font.glyph_index(0xE710), Some(1), "format 12: {format_12}");
        assert_eq!(font.glyph_index(0xE72B), Some(2));
        assert_eq!(font.glyph_index(0xE8C6), Some(3));
        assert_eq!(font.glyph_index(0xE711), None, "a codepoint the font lacks");
        assert_eq!(font.glyph_index(0x41), None);
    }
}

/// A glyph's outline becomes an SVG path of lines and quadratic curves in
/// a square, its em box flipped to SVG's downward y axis.
///
/// parity: LOOK-031
#[test]
fn a_glyph_becomes_a_square_svg_of_its_outline() {
    let font = SyntheticFont {
        glyphs: vec![SyntheticGlyph::Simple(vec![vec![
            (0, 0, true),
            (1024, 0, true),
            (1024, 1024, false),
            (0, 1024, true),
        ]])],
        mapping: vec![(0xE710, 1)],
        format_12: false,
    };
    let font = Font::parse(font.build()).expect("a valid font");
    let svg = font
        .glyph_svg(0xE710, 16)
        .expect("a valid glyph")
        .expect("an outline");
    assert!(
        svg.contains("width=\"16\" height=\"16\" viewBox=\"0 0 1024 1024\""),
        "{svg}"
    );
    assert_eq!(
        commands(&svg),
        ["M0 1024", "L1024 1024", "Q1024 0 0 0", "L0 1024", "Z"]
    );
    assert_eq!(font.glyph_svg(0xE711, 16), Ok(None));
}

/// Off-curve points in a row imply an on-curve point between them, and a
/// contour of only off-curve points starts between its last and first.
///
/// parity: LOOK-031
#[test]
fn implied_points_close_quadratic_contours() {
    let font = SyntheticFont {
        glyphs: vec![SyntheticGlyph::Simple(vec![vec![
            (0, 512, false),
            (512, 1024, false),
            (1024, 512, false),
            (512, 0, false),
        ]])],
        mapping: vec![(0xE700, 1)],
        format_12: true,
    };
    let font = Font::parse(font.build()).expect("a valid font");
    let svg = font.glyph_svg(0xE700, 16).expect("valid").expect("an outline");
    assert_eq!(
        commands(&svg),
        [
            "M256 768",
            "Q0 512 256 256",
            "Q512 0 768 256",
            "Q1024 512 768 768",
            "Q512 1024 256 768",
            "Z"
        ]
    );
}

/// A composite glyph places another glyph's outline at its offset.
///
/// parity: LOOK-031
#[test]
fn composite_glyphs_place_their_components() {
    let font = SyntheticFont {
        glyphs: vec![
            SyntheticGlyph::Simple(vec![vec![(0, 0, true), (100, 0, true), (100, 100, true)]]),
            SyntheticGlyph::Composite {
                glyph: 1,
                offset: (200, 300),
            },
        ],
        mapping: vec![(0xE710, 2)],
        format_12: false,
    };
    let font = Font::parse(font.build()).expect("a valid font");
    let svg = font.glyph_svg(0xE710, 16).expect("valid").expect("an outline");
    assert_eq!(
        commands(&svg),
        ["M200 724", "L300 724", "L300 624", "L200 724", "Z"]
    );
}

/// A composite glyph that places itself is refused for nesting too
/// deeply, not followed for ever.
///
/// parity: LOOK-031
#[test]
fn a_composite_that_places_itself_is_refused() {
    let font = SyntheticFont {
        glyphs: vec![SyntheticGlyph::Composite {
            glyph: 1,
            offset: (1, 1),
        }],
        mapping: vec![(0xE710, 1)],
        format_12: false,
    };
    let font = Font::parse(font.build()).expect("a valid font");
    assert_eq!(font.glyph_svg(0xE710, 16), Err(FontError::OverLimit));
}

/// Files that are not TrueType fonts, or whose tables point outside the
/// file, are refused.
///
/// parity: LOOK-031
#[test]
fn files_that_are_not_truetype_fonts_are_refused() {
    let good = SyntheticFont::with_codepoints(&[0xE710]).build();
    let with = |at: usize, bytes: &[u8]| {
        let mut changed = good.clone();
        changed[at..at + bytes.len()].copy_from_slice(bytes);
        changed
    };
    let cases = [
        ("empty", Vec::new(), FontError::NotTrueType),
        ("PostScript outlines", with(0, b"OTTO"), FontError::NotTrueType),
        ("text", b"Fictional notes".to_vec(), FontError::NotTrueType),
        (
            "too many tables",
            with(4, &500u16.to_be_bytes()),
            FontError::OverLimit,
        ),
        (
            "a table past the end",
            with(12 + 8, &u32::MAX.to_be_bytes()),
            FontError::Damaged,
        ),
        ("no glyf", with(12 + 16, b"xxxx"), FontError::Damaged),
    ];
    for (case, bytes, expected) in cases {
        assert_eq!(Font::parse(bytes).err(), Some(expected), "{case}");
    }
    assert_eq!(Font::parse(vec![0; 33 << 20]).err(), Some(FontError::TooLarge));
}

/// A font cut at any length, or with any byte changed, never panics; at
/// worst it gives no glyph.
///
/// parity: LOOK-031
#[test]
fn truncated_and_mutated_fonts_never_panic() {
    let codepoints = [0xE710, 0xE72B, 0xE8C6];
    let mut font = SyntheticFont::with_codepoints(&codepoints);
    font.glyphs.push(SyntheticGlyph::Composite {
        glyph: 1,
        offset: (5, 5),
    });
    font.mapping.push((0xE74D, 4));
    let good = font.build();
    let all = [0xE710, 0xE72B, 0xE8C6, 0xE74D, 0x41];
    for length in 0..good.len() {
        read_everything(good[..length].to_vec(), &all);
    }
    let mut shuffle = Shuffle(0x2545_f491_4f6c_dd1d);
    for _ in 0..4000 {
        let mut bytes = good.clone();
        for _ in 0..=shuffle.next() % 4 {
            let at = usize::try_from(shuffle.next() % bytes.len() as u64).expect("within the file");
            bytes[at] = shuffle.next().to_le_bytes()[0];
        }
        read_everything(bytes, &all);
    }
    for at in 0..good.len() {
        for value in [0x00, 0x7f, 0x80, 0xff] {
            let mut bytes = good.clone();
            bytes[at] = value;
            read_everything(bytes, &all);
        }
    }
}

/// The glyphs are written once as symbolic icons into the cache, found
/// there again, and a font that gives none leaves no folder in use.
///
/// parity: LOOK-031
#[test]
fn glyphs_are_written_into_the_cache_as_symbolic_icons() {
    let folder = temporary_folder();
    let cache = folder.path().join("cache");
    let font = folder.path().join("SegoeIcons.ttf");
    std::fs::write(&font, SyntheticFont::with_codepoints(&[0xE710, 0xE8C6]).build()).expect("written");

    let wanted = [
        ("ox-win-add-20-symbolic".to_owned(), 0xE710),
        ("ox-win-cut-20-symbolic".to_owned(), 0xE8C6),
        ("ox-win-dismiss-16-symbolic".to_owned(), 0xE711),
        ("../escape-symbolic".to_owned(), 0xE710),
        ("ox-win-add-20".to_owned(), 0xE710),
    ];
    let loaded = load_glyphs(&font, &wanted, &cache);
    assert_eq!(loaded.found(), 2);
    assert!(loaded.contains("ox-win-add-20-symbolic"));
    assert!(
        !loaded.contains("ox-win-dismiss-16-symbolic"),
        "a glyph the font lacks"
    );
    let icon_folder = loaded.folder().expect("a folder of icons");
    assert!(icon_folder.starts_with(&cache), "only the cache holds them");
    let icons = icon_folder.join("hicolor/scalable/actions");
    let svg = std::fs::read_to_string(icons.join("ox-win-cut-20-symbolic.svg")).expect("written");
    assert!(svg.starts_with("<svg"));
    let files = std::fs::read_dir(&icons).expect("readable").count();
    assert_eq!(files, 2, "two icons, no name outside the pattern");
    assert!(
        icon_folder.join("hicolor/index.theme").is_file(),
        "a themed directory"
    );
    assert_eq!(load_glyphs(&font, &wanted, &cache), loaded, "found again");
    let beside: Vec<_> = std::fs::read_dir(folder.path()).expect("readable").collect();
    assert_eq!(beside.len(), 2, "nothing written beside the font");

    let damaged = folder.path().join("segmdl2.ttf");
    std::fs::write(&damaged, b"\0\x01\0\0 and nothing a font needs").expect("written");
    let one = [("ox-win-add-20-symbolic".to_owned(), 0xE710)];
    let nothing = load_glyphs(&damaged, &one, &cache);
    assert_eq!(nothing.found(), 0);
    assert!(nothing.folder().is_none());
    assert_eq!(
        load_glyphs(&folder.path().join("missing.ttf"), &one, &cache).found(),
        0
    );
}
