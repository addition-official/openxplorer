// SPDX-License-Identifier: AGPL-3.0-only
//! Tests of reading icons from Windows icon files, on synthetic files
//! built in the tests ([`super::synthetic`]): no Windows file is used.

use std::path::{Path, PathBuf};

use super::synthetic::{palette_bitmap, rgba_bitmap, test_pattern, SyntheticIconFile, SyntheticImage};
use super::*;
use crate::test_support::temporary_folder;

/// The roles `library` draws.
fn roles_of(library: IconLibrary) -> Vec<IconRole> {
    IconRole::ALL
        .into_iter()
        .filter(|role| role.source().0 == library)
        .collect()
}

/// The group IDs of `library`'s roles, each once: several roles may share
/// a group.
fn groups_of(library: IconLibrary) -> Vec<u16> {
    let mut groups: Vec<u16> = roles_of(library)
        .into_iter()
        .map(|role| role.source().1)
        .collect();
    groups.sort_unstable();
    groups.dedup();
    groups
}

/// Writes a synthetic file called `name` with every group of `library`
/// into `folder`.
fn write_library(folder: &Path, name: &str, library: IconLibrary) -> PathBuf {
    let path = folder.join(name);
    let bytes = SyntheticIconFile::with_groups(&groups_of(library)).build();
    std::fs::write(&path, bytes).expect("the temporary folder takes the file");
    path
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

/// Reads every group of `bytes` and decodes every image, as loading does;
/// what matters is that it returns.
fn read_everything(bytes: Vec<u8>) {
    let Ok(file) = IconFile::from_bytes(bytes) else {
        return;
    };
    let ids: Vec<u16> = file.group_ids().collect();
    for id in ids {
        for image in file.group(id).unwrap_or_default() {
            if let Ok(decoded) = decode(&image.data) {
                let _ = decoded.to_png();
            }
        }
    }
}

/// parity: LOOK-030
#[test]
fn a_synthetic_file_gives_its_groups_and_images() {
    for wide in [false, true] {
        let synthetic = SyntheticIconFile {
            wide,
            ..SyntheticIconFile::with_groups(&[3, 55])
        };
        let file = IconFile::from_bytes(synthetic.build()).expect("a valid file");
        assert_eq!(file.group_ids().collect::<Vec<_>>(), [3, 55], "PE32+ {wide}");
        let images = file.group(3).expect("a valid group");
        let edges: Vec<u32> = images.iter().map(|image| image.width).collect();
        assert_eq!(edges, [16, 32, 256]);
        assert!(file.group(4).expect("a missing group is no error").is_empty());
    }
}

/// parity: LOOK-030
#[test]
fn bitmaps_and_pngs_decode_to_the_pattern_drawn() {
    let pattern = test_pattern(16, [200, 40, 90]);
    assert_eq!(
        decode(&rgba_bitmap(&pattern)),
        Ok(Decoded::Bitmap(pattern.clone()))
    );
    assert_eq!(
        decode(&palette_bitmap(&pattern)),
        Ok(Decoded::Bitmap(pattern.clone()))
    );
    let png = encode_png(&pattern);
    let decoded = decode(&png).expect("a valid PNG");
    assert_eq!(decoded.size(), (16, 16));
    assert_eq!(decoded.to_png(), png, "a PNG is kept as it is");
}

/// A 32-bit bitmap without any alpha takes its transparency from the
/// mask, as older icons do.
///
/// parity: LOOK-030
#[test]
fn a_bitmap_without_alpha_uses_its_mask() {
    let pattern = test_pattern(32, [10, 120, 220]);
    let mut data = rgba_bitmap(&pattern);
    // Clear every alpha byte of the color rows, which follow the header.
    for pixel in data[40..40 + 32 * 32 * 4].chunks_exact_mut(4) {
        pixel[3] = 0;
    }
    assert_eq!(decode(&data), Ok(Decoded::Bitmap(pattern)));
}

/// parity: LOOK-030
#[test]
fn damaged_or_unsupported_images_are_refused() {
    let pattern = test_pattern(16, [1, 2, 3]);
    let bitmap = rgba_bitmap(&pattern);
    let with = |at: usize, bytes: &[u8]| {
        let mut changed = bitmap.clone();
        changed[at..at + bytes.len()].copy_from_slice(bytes);
        changed
    };
    let cases = [
        ("cut short", bitmap[..bitmap.len() / 2].to_vec()),
        ("too wide", with(4, &512u32.to_le_bytes())),
        ("no width", with(4, &0u32.to_le_bytes())),
        ("top down", with(8, &(-32i32).to_le_bytes())),
        ("odd height", with(8, &31u32.to_le_bytes())),
        ("16-bit", with(14, &16u16.to_le_bytes())),
        ("compressed", with(16, &1u32.to_le_bytes())),
        ("odd header", with(0, &12u32.to_le_bytes())),
        ("empty", Vec::new()),
    ];
    for (case, data) in cases {
        assert!(decode(&data).is_err(), "{case}");
    }
    let mut palette = palette_bitmap(&pattern);
    palette[32..36].copy_from_slice(&300u32.to_le_bytes());
    assert!(decode(&palette).is_err(), "more colors than 8 bits hold");

    let png = encode_png(&pattern);
    let mut bad_checksum = png.clone();
    bad_checksum[29] ^= 0xff;
    let mut too_large = png.clone();
    too_large[16..20].copy_from_slice(&1024u32.to_be_bytes());
    for (case, data) in [
        ("checksum", bad_checksum),
        ("1024 wide", too_large),
        ("cut", png[..20].to_vec()),
    ] {
        assert!(decode(&data).is_err(), "PNG {case}");
    }
    assert_eq!(decode(&vec![0; 2 << 20]), Err(IconFileError::OverLimit));
}

/// Files that are not PE files, or whose headers point outside the file,
/// give an error and no icons.
///
/// parity: LOOK-030
#[test]
fn files_that_are_not_icon_files_are_refused() {
    let good = SyntheticIconFile::with_groups(&[3]).build();
    let with = |at: usize, bytes: &[u8]| {
        let mut changed = good.clone();
        changed[at..at + bytes.len()].copy_from_slice(bytes);
        changed
    };
    let cases = [
        ("empty", Vec::new(), IconFileError::NotPe),
        ("text", b"Fictional notes".to_vec(), IconFileError::NotPe),
        ("no MZ", with(0, b"ZM"), IconFileError::NotPe),
        (
            "PE header past the end",
            with(0x3c, &u32::MAX.to_le_bytes()),
            IconFileError::NotPe,
        ),
        ("no PE signature", with(64, b"NE\0\0"), IconFileError::NotPe),
        (
            "unknown optional header",
            with(88, &0x107u16.to_le_bytes()),
            IconFileError::NotPe,
        ),
        (
            "too many sections",
            with(70, &1000u16.to_le_bytes()),
            IconFileError::OverLimit,
        ),
        (
            "resources outside every section",
            with(88 + 96 + 16, &0x9000u32.to_le_bytes()),
            IconFileError::Damaged,
        ),
        (
            "no resource directory",
            with(88 + 96 + 16, &0u32.to_le_bytes()),
            IconFileError::NoIcons,
        ),
        (
            "section past the end",
            with(312 + 20, &0x7fff_0000u32.to_le_bytes()),
            IconFileError::Damaged,
        ),
    ];
    for (case, bytes, expected) in cases {
        let error = IconFile::from_bytes(bytes).err();
        assert_eq!(error, Some(expected), "{case}");
    }
}

/// A file cut at any length, or with any byte changed, never panics or
/// reads out of bounds; at worst it gives no icons.
///
/// parity: LOOK-030
#[test]
fn truncated_and_mutated_files_never_panic() {
    let good = SyntheticIconFile::with_groups(&[3, 2, 102]).build();
    for length in (0..good.len()).step_by(7) {
        read_everything(good[..length].to_vec());
    }
    let mut shuffle = Shuffle(0x9e37_79b9_7f4a_7c15);
    for _ in 0..3000 {
        let mut bytes = good.clone();
        for _ in 0..=shuffle.next() % 4 {
            let at = usize::try_from(shuffle.next() % bytes.len() as u64).expect("within the file");
            bytes[at] = shuffle.next().to_le_bytes()[0];
        }
        read_everything(bytes);
    }
    // Every byte of the headers and directories set to its extremes.
    for at in 0..0x400.min(good.len()) {
        for value in [0x00, 0x7f, 0x80, 0xff] {
            let mut bytes = good.clone();
            bytes[at] = value;
            read_everything(bytes);
        }
    }
}

/// A group that lists more images than the cap, or whose images point
/// past the section, gives what it can and nothing else.
///
/// parity: LOOK-030
#[test]
fn groups_are_capped_and_missing_images_left_out() {
    let image = |id: u16| SyntheticImage {
        id,
        edge: 16,
        bit_count: 32,
        data: rgba_bitmap(&test_pattern(16, [9, 9, 9])),
    };
    let crowded = SyntheticIconFile {
        groups: vec![(3, (1..=65).map(image).collect())],
        ..SyntheticIconFile::default()
    };
    let file = IconFile::from_bytes(crowded.build()).expect("a valid file");
    assert_eq!(file.group(3).err(), Some(IconFileError::OverLimit));

    let stub = SyntheticIconFile {
        without_images: true,
        ..SyntheticIconFile::with_groups(&[3])
    };
    let file = IconFile::from_bytes(stub.build()).expect("a valid file");
    assert!(
        file.group(3).expect("a valid group").is_empty(),
        "a group without its images"
    );
}

/// parity: LOOK-030
#[test]
fn files_are_recognized_by_name_with_or_without_mun() {
    let cases = [
        (
            "C:/Windows/SystemResources/imageres.dll.mun",
            Some(IconLibrary::Imageres),
        ),
        (
            "/media/windows/Windows/SystemResources/SHELL32.dll.MUN",
            Some(IconLibrary::Shell32),
        ),
        ("/srv/icons/imageres.dll", Some(IconLibrary::Imageres)),
        ("/srv/icons/shell32.dll", Some(IconLibrary::Shell32)),
        ("/srv/icons/explorer.exe", None),
        ("/srv/icons/imageres.dll.bak", None),
        ("/srv/icons/my-imageres.dll", None),
    ];
    for (path, expected) in cases {
        assert_eq!(IconLibrary::of_file(Path::new(path)), expected, "{path}");
    }
}

/// parity: LOOK-030
#[test]
fn every_role_has_its_windows_group() {
    use IconLibrary::{Imageres, Shell32};
    use Provenance::{BestMatch, Verified};
    // The special folders are the flat tiles of Windows 11 build 26200's
    // known-folder DefaultIcon values, not the yellow 1001 to 1005 ones;
    // every fixed drive, Local Disk included, is the plain drive (32).
    let expected = [
        (IconRole::Folder, Imageres, 3, Verified),
        (IconRole::Desktop, Imageres, 183, Verified),
        (IconRole::Downloads, Imageres, 184, Verified),
        (IconRole::Documents, Imageres, 112, Verified),
        (IconRole::Pictures, Imageres, 113, Verified),
        (IconRole::Music, Imageres, 108, Verified),
        (IconRole::Videos, Imageres, 189, Verified),
        (IconRole::Home, Shell32, 51380, Verified),
        (IconRole::Gallery, Shell32, 51586, Verified),
        (IconRole::ThisPc, Imageres, 109, Verified),
        (IconRole::Network, Imageres, 25, Verified),
        (IconRole::NetworkComputer, Imageres, 104, BestMatch),
        (IconRole::NetworkShare, Imageres, 28, BestMatch),
        (IconRole::NetworkDrive, Imageres, 33, BestMatch),
        (IconRole::NetworkDriveDisconnected, Imageres, 31, BestMatch),
        (IconRole::Drive, Imageres, 32, Verified),
        (IconRole::RemovableDrive, Imageres, 30, BestMatch),
        (IconRole::OpticalDrive, Imageres, 56, BestMatch),
        (IconRole::RecycleBinEmpty, Imageres, 55, Verified),
        (IconRole::RecycleBinFull, Imageres, 54, Verified),
        (IconRole::ZipFolder, Imageres, 175, Verified),
        (IconRole::TextFile, Imageres, 102, Verified),
        (IconRole::ImageFile, Imageres, 132, BestMatch),
        (IconRole::AudioFile, Imageres, 131, BestMatch),
        (IconRole::VideoFile, Imageres, 133, BestMatch),
        (IconRole::Executable, Imageres, 15, BestMatch),
        (IconRole::Script, Imageres, 68, BestMatch),
        (IconRole::Shortcut, Imageres, 15, BestMatch),
        (IconRole::GenericFile, Imageres, 2, Verified),
        (IconRole::Settings, Shell32, 16826, BestMatch),
    ];
    assert_eq!(expected.len(), IconRole::ALL.len());
    for (role, library, group, provenance) in expected {
        assert_eq!(role.source(), (library, group, provenance), "{role:?}");
        assert_eq!(IconRole::from_key(role.key()), Some(role));
    }
    for old in [1002, 1003, 1004, 1005, 36] {
        assert!(
            IconRole::ALL.iter().all(|role| role.source().1 != old),
            "group {old} is no longer used"
        );
    }
}

/// Both libraries together draw every role; the cache holds a PNG per
/// role and edge, and the best image for a size is the smallest at least
/// that large.
///
/// parity: LOOK-030
#[test]
fn chosen_files_draw_every_role_from_the_cache() {
    let folder = temporary_folder();
    let cache = folder.path().join("cache");
    let files = vec![
        write_library(folder.path(), "imageres.dll.mun", IconLibrary::Imageres),
        write_library(folder.path(), "shell32.dll", IconLibrary::Shell32),
    ];
    let loaded = load_icons(&files, &cache);
    assert_eq!(loaded.found(), IconRole::ALL.len());
    for role in IconRole::ALL {
        let edges: Vec<u32> = loaded.images(role).iter().map(|image| image.edge).collect();
        assert_eq!(edges, [16, 32, 256], "{role:?}");
        for image in loaded.images(role) {
            assert!(image.path.starts_with(&cache), "only the cache holds images");
            let png = std::fs::read(&image.path).expect("the cached image exists");
            assert_eq!(
                decode(&png).map(|decoded| decoded.size()),
                Ok((image.edge, image.edge))
            );
        }
    }
    let best = |pixels| loaded.best(IconRole::Folder, pixels).map(|image| image.edge);
    assert_eq!(best(16), Some(16));
    assert_eq!(best(20), Some(32));
    assert_eq!(best(48), Some(256));
    assert_eq!(best(512), Some(256), "the largest when none is large enough");

    // Found again in the cache, at the same paths.
    assert_eq!(load_icons(&files, &cache), loaded);
}

/// Missing, unreadable, unknown and damaged files give no icons and leave
/// the roles to the app's own; a stub without images gives way to the
/// next file of the same name.
///
/// parity: LOOK-030
#[test]
fn files_that_give_nothing_leave_the_built_in_icons() {
    let folder = temporary_folder();
    let cache = folder.path().join("cache");
    let stub = folder.path().join("stub").join("imageres.dll");
    std::fs::create_dir(stub.parent().expect("a parent")).expect("a folder");
    let synthetic = SyntheticIconFile {
        without_images: true,
        ..SyntheticIconFile::with_groups(&groups_of(IconLibrary::Imageres))
    };
    std::fs::write(&stub, synthetic.build()).expect("written");
    let damaged = folder.path().join("damaged").join("shell32.dll.mun");
    std::fs::create_dir(damaged.parent().expect("a parent")).expect("a folder");
    std::fs::write(&damaged, b"MZ but nothing else").expect("written");
    let unknown = write_library(folder.path(), "explorer.exe", IconLibrary::Imageres);
    let missing = folder.path().join("missing").join("imageres.dll.mun");
    let a_folder = folder.path().join("imageres.dll");
    std::fs::create_dir(&a_folder).expect("a folder named like the file");

    let nothing = load_icons(
        &[stub.clone(), damaged.clone(), unknown, missing, a_folder],
        &cache,
    );
    assert_eq!(nothing.found(), 0);
    assert!(nothing.best(IconRole::Folder, 16).is_none());

    let real = write_library(folder.path(), "imageres.dll.mun", IconLibrary::Imageres);
    let after_stub = load_icons(&[stub, damaged, real], &cache);
    assert_eq!(after_stub.found(), roles_of(IconLibrary::Imageres).len());
    assert!(
        after_stub.images(IconRole::Home).is_empty(),
        "shell32 was damaged"
    );
}

/// A changed file is read afresh, a broken cache folder is made again,
/// and folders of files no longer chosen are removed.
///
/// parity: LOOK-030
#[test]
fn the_cache_follows_the_chosen_files() {
    let folder = temporary_folder();
    let cache = folder.path().join("cache");
    let imageres = write_library(folder.path(), "imageres.dll.mun", IconLibrary::Imageres);
    let folders = || -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = std::fs::read_dir(&cache)
            .expect("the cache exists")
            .map(|entry| entry.expect("readable").path())
            .collect();
        found.sort();
        found
    };
    let imageres_roles = roles_of(IconLibrary::Imageres).len();
    assert_eq!(
        load_icons(std::slice::from_ref(&imageres), &cache).found(),
        imageres_roles
    );
    let first = folders();
    assert_eq!(first.len(), 1);

    // Only a folder group now.
    std::fs::write(&imageres, SyntheticIconFile::with_groups(&[3]).build()).expect("rewritten");
    assert_eq!(load_icons(std::slice::from_ref(&imageres), &cache).found(), 1);
    let second = folders();
    assert_eq!(second.len(), 1, "the old folder is removed");
    assert_ne!(first, second);

    std::fs::remove_file(second[0].join("index")).expect("the index exists");
    assert_eq!(
        load_icons(std::slice::from_ref(&imageres), &cache).found(),
        1,
        "made again"
    );

    let unrelated = cache.join("notes");
    std::fs::create_dir(&unrelated).expect("a folder");
    assert_eq!(load_icons(&[], &cache).found(), 0);
    assert_eq!(
        folders(),
        [unrelated],
        "only the reader's own folders are removed"
    );
}
