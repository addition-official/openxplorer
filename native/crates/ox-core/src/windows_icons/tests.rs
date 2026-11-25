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
        .filter(|role| role.sources().iter().any(|(source, _, _)| *source == library))
        .collect()
}

/// The group IDs of `library`'s roles, each once: several roles may share
/// a group.
fn groups_of(library: IconLibrary) -> Vec<u16> {
    let mut groups: Vec<u16> = IconRole::ALL
        .into_iter()
        .flat_map(IconRole::sources)
        .filter(|(source, _, _)| *source == library)
        .map(|(_, group, _)| *group)
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

/// Every Windows file Explorer draws icons from is recognized by its name,
/// with or without `.mun`, in any letter case; another file is not.
///
/// parity: LOOK-030
#[test]
fn every_explorer_icon_file_is_recognized() {
    for (name, library) in [
        ("imageres.dll.mun", IconLibrary::Imageres),
        ("SHELL32.DLL", IconLibrary::Shell32),
        ("DDORes.dll.mun", IconLibrary::Ddores),
        ("wpdshext.dll.mun", IconLibrary::Wpdshext),
        ("networkexplorer.dll.mun", IconLibrary::NetworkExplorer),
        ("netshell.dll.mun", IconLibrary::Netshell),
        ("ntshrui.dll.mun", IconLibrary::Ntshrui),
        ("zipfldr.dll.mun", IconLibrary::Zipfldr),
        ("mydocs.dll.mun", IconLibrary::Mydocs),
        ("mycomput.dll.mun", IconLibrary::Mycomput),
        ("ExplorerFrame.dll.mun", IconLibrary::ExplorerFrame),
        ("dmdskres.dll", IconLibrary::Dmdskres),
    ] {
        let path = Path::new("/mnt/windows/SystemResources").join(name);
        assert_eq!(IconLibrary::of_file(&path), Some(library), "{name}");
    }
    assert_eq!(IconLibrary::of_file(Path::new("/srv/notepad.exe")), None);
    for role in IconRole::ALL {
        assert!(!role.sources().is_empty(), "{role:?} has a source");
        assert_eq!(role.source(), role.sources()[0], "{role:?}");
    }
}

/// A role takes its images from the file of its first source the user
/// chose, whatever the order of the chosen files: imageres' ZIP folder
/// before zipfldr's, and zipfldr's when imageres is not chosen; a phone
/// from wpdshext.dll, else DDORes.dll. Each file's group is looked up in
/// that file only.
///
/// parity: LOOK-030
#[test]
fn each_role_takes_its_first_chosen_source() {
    let folder = temporary_folder();
    let cache = folder.path().join("cache");
    let source_group = |role: IconRole, library: IconLibrary| -> u16 {
        role.sources()
            .iter()
            .find(|(source, _, _)| *source == library)
            .map(|(_, group, _)| *group)
            .expect("a source in that file")
    };
    let write = |name: &str, groups: &[u16]| {
        let path = folder.path().join(name);
        std::fs::write(&path, SyntheticIconFile::with_groups(groups).build()).expect("written");
        path
    };
    // The cache folder a role's images were read into tells the file.
    let drawn_from = |files: &[PathBuf], role: IconRole| -> Option<PathBuf> {
        let loaded = load_icons(files, &cache);
        let image = loaded.images(role).first()?;
        image.path.parent().map(Path::to_path_buf)
    };
    let zip_in_imageres = source_group(IconRole::ZipFolder, IconLibrary::Imageres);
    let zip_in_zipfldr = source_group(IconRole::ZipFolder, IconLibrary::Zipfldr);
    // zipfldr.dll also has imageres' number, which is not its role's.
    let zipfldr = write("zipfldr.dll.mun", &[zip_in_zipfldr, zip_in_imageres]);
    let imageres = write("imageres.dll.mun", &[zip_in_imageres]);
    let from_imageres = drawn_from(std::slice::from_ref(&imageres), IconRole::ZipFolder);
    let from_zipfldr = drawn_from(std::slice::from_ref(&zipfldr), IconRole::ZipFolder);
    assert!(from_imageres.is_some() && from_zipfldr.is_some());
    assert_ne!(from_imageres, from_zipfldr);
    assert_eq!(
        drawn_from(&[zipfldr.clone(), imageres], IconRole::ZipFolder),
        from_imageres,
        "imageres first, whatever the order chosen"
    );
    let only_imageres_number = write("zipfldr.dll", &[zip_in_imageres]);
    assert_eq!(
        drawn_from(&[only_imageres_number], IconRole::ZipFolder),
        None,
        "zipfldr's role is looked up by zipfldr's number"
    );

    let phone_in_wpdshext = source_group(IconRole::PortableDevice, IconLibrary::Wpdshext);
    let phone_in_ddores = source_group(IconRole::PortableDevice, IconLibrary::Ddores);
    let ddores = write("DDORes.dll.mun", &[phone_in_ddores]);
    let wpdshext = write("wpdshext.dll.mun", &[phone_in_wpdshext]);
    let from_wpdshext = drawn_from(std::slice::from_ref(&wpdshext), IconRole::PortableDevice);
    let from_ddores = drawn_from(std::slice::from_ref(&ddores), IconRole::PortableDevice);
    assert!(from_wpdshext.is_some() && from_ddores.is_some());
    assert_eq!(
        drawn_from(&[ddores, wpdshext], IconRole::PortableDevice),
        from_wpdshext,
        "wpdshext first"
    );
    // A file no role draws from yet is read and gives nothing.
    let frame = write("ExplorerFrame.dll.mun", &[1, 2]);
    assert_eq!(load_icons(&[frame], &cache).found(), 0);
}
