// SPDX-License-Identifier: AGPL-3.0-only
//! The archive readers as the archive browser, Properties and Extract All
//! use them, on an archive held in memory.
//!
//! The first byte of the input picks the name the archive is opened under
//! (the readers look at the extension) and which options are used; the
//! rest is the archive.

use std::fs;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, OnceLock};

use gio::prelude::FileExt;

use crate::archive::{
    remove_preview_copy, ArchiveBrowser, ArchiveEntryKind, ArchiveError, ArchiveOpener, ArchivePassword,
    ArchiveStream, ExtractionLimits, ExtractionRequest, GioExtractionOutput, ZipExtractor,
};
use crate::gio_node::GioNode;
use crate::transfer::{Cancellation, Node, NodeFactory};

/// The names an archive is opened under, picked by the low three bits of
/// the first byte.
const NAMES: [&str; 8] = [
    "Fuzz.zip",
    "Fuzz.7z",
    "Fuzz.tar",
    "Fuzz.tar.gz",
    "Fuzz.tar.bz2",
    "Fuzz.tar.xz",
    "Fuzz.tar.zst",
    "Fuzz.bin",
];

/// The folders listed below the top, and the members previewed, at most.
const MAX_VISITS: usize = 6;

/// The options an input asks for.
struct Input<'a> {
    uri: String,
    archive: &'a [u8],
    /// Decrypt with a password.
    password: bool,
    /// Extract only the members the browser lists.
    selection: bool,
    /// Extract as the private step of an extraction into an existing folder.
    into_existing: bool,
}

fn input(data: &[u8]) -> Option<Input<'_>> {
    let (&options, archive) = data.split_first()?;
    Some(Input {
        uri: format!("file:///home/alex/Downloads/{}", NAMES[usize::from(options & 7)]),
        archive,
        password: options & 0x08 != 0,
        selection: options & 0x10 != 0,
        into_existing: options & 0x20 != 0,
    })
}

/// Opens every URI as `archive`.
fn memory_opener(archive: &[u8]) -> Arc<dyn ArchiveOpener> {
    let archive = archive.to_vec();
    let opener = move |_uri: &str, cancel: &Cancellation| -> Result<Box<dyn ArchiveStream>, ArchiveError> {
        cancel.check()?;
        Ok(Box::new(Cursor::new(archive.clone())))
    };
    Arc::new(opener)
}

fn gio_factory() -> NodeFactory {
    Arc::new(|uri: &str| Ok(Box::new(GioNode::new(uri)) as Box<dyn Node>))
}

/// A private folder for this process's copies and extractions, removed
/// with the process's temporary folder.
fn scratch() -> &'static Path {
    static SCRATCH: OnceLock<tempfile::TempDir> = OnceLock::new();
    SCRATCH
        .get_or_init(|| {
            tempfile::Builder::new()
                .prefix("ox-fuzz-")
                .tempdir()
                .expect("a temporary folder")
        })
        .path()
}

/// Lower limits than the app's, so a fuzzed archive that is within every
/// limit still extracts in well under a second.
fn fuzz_limits() -> ExtractionLimits {
    ExtractionLimits {
        max_entries: 2000,
        max_paths: 4000,
        max_depth: 64,
        max_total_bytes: 16 * 1024 * 1024,
        max_member_bytes: 8 * 1024 * 1024,
        max_ratio: 1000,
    }
}

fn browser(input: &Input<'_>) -> ArchiveBrowser {
    let browser = ArchiveBrowser::new(memory_opener(input.archive), scratch().join("previews"));
    if input.password {
        browser.with_password(ArchivePassword::from("lighthouse"))
    } else {
        browser
    }
}

/// Lists the top of the archive and a few of its folders, previews a few
/// members, and checks the whole archive as Extract All does first.
pub fn list_inspect_preview(data: &[u8]) {
    let Some(input) = input(data) else {
        return;
    };
    let cancel = Cancellation::new();
    let browser = browser(&input);
    let mut folders = vec![String::new()];
    let mut visited = 0;
    let mut previewed = 0;
    while let Some(prefix) = folders.pop() {
        visited += 1;
        let Ok(listing) = browser.list(&input.uri, &prefix, &cancel) else {
            continue;
        };
        for entry in &listing.entries {
            match entry.kind {
                ArchiveEntryKind::Folder if visited + folders.len() < MAX_VISITS => {
                    folders.push(entry.member.clone());
                }
                ArchiveEntryKind::File { .. } if entry.can_open && previewed < MAX_VISITS => {
                    previewed += 1;
                    if let Ok(copy) = browser.preview_member(&input.uri, &entry.member, &cancel) {
                        remove_preview_copy(&copy.path);
                    }
                }
                _ => {}
            }
        }
    }
    let extractor = ZipExtractor::new(
        memory_opener(input.archive),
        gio_factory(),
        Arc::new(GioExtractionOutput),
    )
    .with_limits(fuzz_limits());
    let _ = extractor.inspect(&input.uri, &cancel);
}

/// Extracts the archive into a new folder in an empty folder, as Extract
/// All does, then checks that nothing was written outside the new folder
/// and that every link it holds stays inside it.
///
/// # Panics
///
/// When the extraction wrote outside its new folder or extracted a link
/// leading out of it.
pub fn extract(data: &[u8]) {
    let Some(input) = input(data) else {
        return;
    };
    let cancel = Cancellation::new();
    let root = tempfile::Builder::new()
        .prefix("extract-")
        .tempdir_in(scratch())
        .expect("a temporary folder");
    let destination = root.path().join("Destination");
    fs::create_dir(&destination).expect("the destination folder");
    let mut extractor = ZipExtractor::new(
        memory_opener(input.archive),
        gio_factory(),
        Arc::new(GioExtractionOutput),
    )
    .with_limits(fuzz_limits());
    if input.password {
        extractor = extractor.with_password(ArchivePassword::from("lighthouse"));
    }
    if input.selection {
        let listed = browser(&input)
            .list(&input.uri, "", &cancel)
            .map(|listing| listing.entries.into_iter().map(|entry| entry.member).collect::<Vec<_>>())
            .unwrap_or_default();
        extractor = extractor.with_selection(&listed);
    }
    if input.into_existing {
        extractor = extractor.into_existing_folder();
    }
    let request = ExtractionRequest {
        archive_uri: input.uri.clone(),
        destination_uri: gio::File::for_path(&destination).uri().to_string(),
        folder_name: "Extracted".to_owned(),
    };
    let result = extractor.extract(&request, &cancel);
    let new_folder = destination.join("Extracted");
    check_only_child(root.path(), "Destination");
    match result {
        Ok(_) => {
            check_only_child(&destination, "Extracted");
            check_contained(&new_folder, &new_folder);
        }
        Err(ArchiveError::StagingLeftBehind { .. }) => {}
        Err(_) => {
            let left = names(&destination);
            assert!(left.is_empty(), "a failed extraction left {left:?} behind");
        }
    }
}

/// The second byte of a fix-up input: which compression wraps the TAR.
const COMPRESSIONS: [&str; 5] = ["Fuzz.tar", "Fuzz.tar.gz", "Fuzz.tar.bz2", "Fuzz.tar.xz", "Fuzz.tar.zst"];

/// [`list_inspect_preview`] and [`extract`] on an archive whose checksums
/// are made right first, so mutations reach the parsers behind them: the
/// first byte is the options byte, the second picks a compression for a
/// TAR, and the rest is a TAR (each header's checksum is set, walking the
/// headers as a reader does) or a 7z (its start and next header CRCs are
/// set), compressed here as the second byte says.
pub fn fixed_up(data: &[u8]) {
    let [options, compression, archive @ ..] = data else {
        return;
    };
    let mut archive = archive.to_vec();
    let (name_index, packed) = if archive.starts_with(b"7z\xBC\xAF\x27\x1C") {
        fix_seven_zip(&mut archive);
        (1, archive)
    } else {
        fix_tar(&mut archive);
        let kind = usize::from(*compression) % COMPRESSIONS.len();
        (2 + kind, compress(kind, &archive))
    };
    let mut input = vec![(options & !7) | u8::try_from(name_index).unwrap_or(7)];
    input.extend_from_slice(&packed);
    list_inspect_preview(&input);
    extract(&input);
}

fn compress(kind: usize, tar: &[u8]) -> Vec<u8> {
    use std::io::Write;
    match kind {
        1 => {
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
            let _ = encoder.write_all(tar);
            encoder.finish().unwrap_or_default()
        }
        2 => {
            let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::fast());
            let _ = encoder.write_all(tar);
            encoder.finish().unwrap_or_default()
        }
        3 => {
            let Ok(mut encoder) = lzma_rust2::XzWriter::new(Vec::new(), lzma_rust2::XzOptions::with_preset(0)) else {
                return Vec::new();
            };
            let _ = encoder.write_all(tar);
            encoder.finish().unwrap_or_default()
        }
        4 => ruzstd::encoding::compress_to_vec(tar, ruzstd::encoding::CompressionLevel::Fastest),
        _ => tar.to_vec(),
    }
}

/// Sets the checksum of each TAR header, from the first, skipping each
/// member's data by the size its header gives.
fn fix_tar(tar: &mut [u8]) {
    let mut at = 0_usize;
    while let Some(block) = tar.get_mut(at..at + 512) {
        if block.iter().all(|&byte| byte == 0) {
            at += 512;
            continue;
        }
        block[148..156].fill(b' ');
        let sum: u64 = block.iter().map(|&byte| u64::from(byte)).sum();
        let digits = format!("{sum:06o}\0 ");
        block[148..156].copy_from_slice(&digits.as_bytes()[digits.len() - 8..]);
        let size = tar_size(&block[124..136]).unwrap_or(0);
        let data_blocks = size.div_ceil(512).min(u64::try_from(tar.len()).unwrap_or(u64::MAX));
        at = at
            .saturating_add(512)
            .saturating_add(usize::try_from(data_blocks * 512).unwrap_or(usize::MAX));
    }
}

/// A TAR size field: octal, or base-256 when its first bit is set.
fn tar_size(field: &[u8]) -> Option<u64> {
    if field.first().is_some_and(|byte| byte & 0x80 != 0) {
        return field[1..]
            .iter()
            .try_fold(0_u64, |value, &byte| value.checked_mul(256)?.checked_add(u64::from(byte)));
    }
    let digits: String = field
        .iter()
        .map(|&byte| char::from(byte))
        .filter(char::is_ascii_digit)
        .collect();
    u64::from_str_radix(&digits, 8).ok()
}

/// Sets a 7z's next header CRC, when the next header is inside it, and
/// its start header CRC.
fn fix_seven_zip(archive: &mut [u8]) {
    if archive.len() < 32 {
        return;
    }
    let word = |bytes: &[u8], at: usize| u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap_or_default());
    let offset = word(archive, 12);
    let size = word(archive, 20);
    let next = usize::try_from(offset)
        .ok()
        .and_then(|offset| offset.checked_add(32))
        .zip(usize::try_from(size).ok())
        .and_then(|(start, size)| Some(start..start.checked_add(size)?));
    if let Some(next) = next.filter(|next| next.end <= archive.len()) {
        let crc = crc32fast::hash(&archive[next]);
        archive[28..32].copy_from_slice(&crc.to_le_bytes());
    }
    let crc = crc32fast::hash(&archive[12..32]);
    archive[8..12].copy_from_slice(&crc.to_le_bytes());
}

fn names(folder: &Path) -> Vec<String> {
    fs::read_dir(folder)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn check_only_child(folder: &Path, child: &str) {
    let found = names(folder);
    assert!(
        found.iter().all(|name| name == child),
        "the extraction wrote {found:?} into {}",
        folder.display()
    );
}

/// Checks that every item below `folder` is inside `top`, and that every
/// link resolves, by its text alone, to a place inside `top`.
fn check_contained(folder: &Path, top: &Path) {
    let Ok(entries) = fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(&path).expect("a link's target");
            let resolved = lexical(&folder.join(&target));
            assert!(
                resolved.starts_with(top),
                "the link {} leads to {} outside the extracted folder",
                path.display(),
                target.display()
            );
            // Through other links, a link can lead further than its text.
            if let (Ok(real), Ok(real_top)) = (fs::canonicalize(&path), fs::canonicalize(top)) {
                assert!(
                    real.starts_with(&real_top),
                    "the link {} leads to {} outside the extracted folder",
                    path.display(),
                    real.display()
                );
            }
        } else if metadata.is_dir() {
            check_contained(&path, top);
        }
    }
}

/// `path` with `.` and `..` resolved by their text.
fn lexical(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            other => resolved.push(other),
        }
    }
    resolved
}
