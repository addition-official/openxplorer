// SPDX-License-Identifier: AGPL-3.0-only
//! Tests of Windows Explorer's command pictures, on a synthetic folder
//! with the same layout built in the tests, of made-up pictures: never
//! Microsoft's.

use gtk::prelude::*;
use ox_core::settings::Preferences;

use super::*;
use crate::icons::{self, Icon};

/// Turns the pictures off again and draws the light look when a test
/// ends, also when it fails.
struct Reset;

impl Drop for Reset {
    fn drop(&mut self) {
        follow_preferences(&Preferences::default());
        set_look(false, false);
    }
}

/// A made-up picture: a square of `colour` filling the 16-unit square.
fn square(colour: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"16\" height=\"16\" fill=\"none\" \
         viewBox=\"0 0 16 16\">\n  <rect fill=\"{colour}\" width=\"16\" height=\"16\"/>\n</svg>\n"
    )
}

/// The colour of each look's made-up pictures.
const COLOURS: [(Look, &str, [u8; 3]); 4] = [
    (Look::Light, "#102030", [0x10, 0x20, 0x30]),
    (Look::Dark, "#E0D0C0", [0xE0, 0xD0, 0xC0]),
    (Look::HighContrastLight, "#00FF00", [0x00, 0xFF, 0x00]),
    (Look::HighContrastDark, "#FF00FF", [0xFF, 0x00, 0xFF]),
];

/// A folder laid out as Windows' `Assets\images`, with made-up pictures
/// of Cut, Copy (in upper case, as Windows' names may be), Details and the
/// address bar's Home in each look; Paste has none.
fn synthetic_folder() -> tempfile::TempDir {
    let folder = tempfile::tempdir().expect("a temporary folder");
    for (look, colour, _) in COLOURS {
        let dir = folder.path().join(look.folder());
        std::fs::create_dir_all(&dir).expect("a look folder");
        for name in [
            "windows.cut",
            "WINDOWS.COPY",
            "windows.iconsize.details",
            "addressbar.home",
        ] {
            std::fs::write(dir.join(format!("{name}.svg")), square(colour)).expect("written");
        }
        // Not a picture.
        std::fs::write(dir.join("notes.txt"), "not a picture").expect("written");
    }
    folder
}

/// Preferences with Windows icon files on and `folder` chosen.
fn with_folder(folder: &std::path::Path) -> Preferences {
    Preferences {
        use_icon_files: true,
        explorer_icons: Some(folder.display().to_string()),
        ..Preferences::default()
    }
}

/// The colour at the centre of `image`'s texture.
fn centre_colour(image: &gtk::Image) -> [u8; 3] {
    let texture = image
        .paintable()
        .and_downcast::<gtk::gdk::Texture>()
        .expect("a picture is shown");
    let mut downloader = gtk::gdk::TextureDownloader::new(&texture);
    downloader.set_format(gtk::gdk::MemoryFormat::R8g8b8a8);
    let (bytes, stride) = downloader.download_bytes();
    let (x, y) = (
        usize::try_from(texture.width() / 2).expect("a column"),
        usize::try_from(texture.height() / 2).expect("a row"),
    );
    let at = y * stride + x * 4;
    [bytes[at], bytes[at + 1], bytes[at + 2]]
}

/// The made-up picture is nested in one the size of the square, its own
/// size replaced by the 16-unit square it is drawn in; what is not one
/// `<svg>` element is refused.
#[test]
fn a_picture_is_nested_at_its_em_in_the_square() {
    let svg = "<?xml version=\"1.0\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" width='32' \
               height=\"32\" viewBox=\"0 0 16 16\"><rect width=\"1\" height=\"1\"/></svg>\n";
    let padded = padded_svg(svg, 20, 16, 40).expect("an svg");
    assert!(padded.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"40\" height=\"40\""));
    assert!(padded.contains("viewBox=\"-2 -2 20 20\""), "{padded}");
    assert!(
        padded.contains("<svg x=\"0\" y=\"0\" width=\"16\" height=\"16\" xmlns="),
        "{padded}"
    );
    assert!(!padded.contains("'32'") && !padded.contains("\"32\""), "{padded}");
    assert!(padded.ends_with("</svg></svg>"), "{padded}");
    assert_eq!(padded_svg("<html></html>", 16, 16, 16), None);
    assert_eq!(padded_svg("<svg viewBox=\"0 0 16 16\">", 16, 16, 16), None);
}

/// Every picture the app asks for is named once, the View menu's and the
/// menus' among them, and only command and address bar spots take them.
///
/// parity: LOOK-034
#[test]
fn the_commands_map_to_windows_explorers_file_names() {
    let names = mapped_files();
    for name in [
        "windows.iconsize.details",
        "windows.cut",
        "windows.ribbondelete",
        "windows.opennewtab",
        "windows.powershell.run",
        "addressbar.home",
        "SortBy",
    ] {
        assert!(names.contains(&name), "{name}");
    }
    assert_eq!(file_of(Icon::Cut, Spot::Plain), None);
    assert_eq!(file_of(Icon::Home, Spot::Command), None);
    assert_eq!(file_of(Icon::Home, Spot::AddressRoot), Some("addressbar.home"));
    assert_eq!(
        file_of(Icon::TextDescription, Spot::Command),
        Some("windows.iconsize.details")
    );
}

/// With the folder chosen, a command shows its picture from the folder in
/// the look drawn (dark, light, and either high contrast), in the
/// picture's own colours; a command without a picture, and an image that
/// is not a command, keep the app's icon; turning the folder off brings
/// the icon back. The folder and its pictures are made up in the test.
///
/// parity: LOOK-034
#[gtk::test]
fn commands_draw_windows_explorers_pictures_from_the_chosen_folder() {
    let _reset = Reset;
    let folder = synthetic_folder();
    let cut = icons::command_image(Icon::Cut, 16);
    let copy = icons::command_image(Icon::Copy, 18);
    let paste = icons::command_image(Icon::ClipboardPaste, 16);
    let plain = icons::image(Icon::Cut, 16);
    let address = gtk::Image::new();
    icons::set_icon_at(&address, Icon::FileFolder, 16, Spot::AddressRoot);
    follow_preferences(&with_folder(folder.path()));
    wait_until_read();
    assert!(
        matches!(status(), Status::Ready { found: 4, wanted } if wanted == mapped_files().len()),
        "{:?}",
        status()
    );
    for (look, _, colour) in COLOURS {
        let dark = matches!(look, Look::Dark | Look::HighContrastDark);
        let high = matches!(look, Look::HighContrastLight | Look::HighContrastDark);
        set_look(dark, high);
        assert_eq!(super::look(), look);
        for image in [&cut, &copy] {
            assert!(image.has_css_class(icons::EXPLORER_CLASS), "{look:?}");
            assert_eq!(image.icon_name(), None, "{look:?}");
            assert_eq!(centre_colour(image), colour, "{look:?}");
        }
    }
    assert_eq!(cut.pixel_size(), 16);
    assert_eq!(copy.pixel_size(), 18, "the square keeps its size");
    assert_eq!(paste.icon_name().as_deref(), Some(Icon::ClipboardPaste.name()));
    assert_eq!(
        plain.icon_name().as_deref(),
        Some(Icon::Cut.name()),
        "not a command"
    );
    // The address bar's root: Home has a picture, a folder has none.
    assert_eq!(address.icon_name().as_deref(), Some(Icon::FileFolder.name()));
    icons::set_icon(&address, Icon::Home, 16);
    assert!(address.has_css_class(icons::EXPLORER_CLASS), "the spot is kept");
    follow_preferences(&Preferences {
        use_icon_files: false,
        ..with_folder(folder.path())
    });
    assert_eq!(status(), Status::Off);
    assert_eq!(cut.icon_name().as_deref(), Some(Icon::Cut.name()));
    assert!(!cut.has_css_class(icons::EXPLORER_CLASS));
    assert_eq!(address.icon_name().as_deref(), Some(Icon::Home.name()));
}

/// A folder without Windows Explorer's look folders, or one missing,
/// draws nothing from it and says so; a damaged picture keeps the icon
/// and never fails.
///
/// parity: LOOK-034
#[gtk::test]
fn a_folder_without_the_pictures_keeps_the_apps_icons() {
    let _reset = Reset;
    let empty = tempfile::tempdir().expect("a temporary folder");
    follow_preferences(&with_folder(empty.path()));
    wait_until_read();
    assert_eq!(status(), Status::NotExplorerIcons);
    let cut = icons::command_image(Icon::Cut, 16);
    assert_eq!(cut.icon_name().as_deref(), Some(Icon::Cut.name()));

    let folder = synthetic_folder();
    std::fs::write(
        folder.path().join(Look::Light.folder()).join("windows.cut.svg"),
        "<svg damaged",
    )
    .expect("written");
    // The user chose the Assets folder above images.
    let assets = tempfile::tempdir().expect("a temporary folder");
    std::fs::rename(folder.path(), assets.path().join("images")).expect("moved");
    follow_preferences(&with_folder(assets.path()));
    wait_until_read();
    assert!(
        matches!(status(), Status::Ready { found: 4, .. }),
        "{:?}",
        status()
    );
    let cut = icons::command_image(Icon::Cut, 16);
    let copy = icons::command_image(Icon::Copy, 16);
    assert_eq!(cut.icon_name().as_deref(), Some(Icon::Cut.name()), "damaged");
    assert!(copy.has_css_class(icons::EXPLORER_CLASS));
}
