// SPDX-License-Identifier: AGPL-3.0-only
//! Entry points for the fuzz targets in `native/fuzz`, built only with the
//! `fuzzing` feature, so the app and its packages never contain them.
//!
//! Each function takes arbitrary bytes and runs the app's own readers of
//! untrusted data on them, the way the app does; it panics only when a
//! reader breaks a promise its callers rely on.

use std::time::Duration;

use crate::picture_guard::Limits;

/// The first bytes of a picture input: the places the picture is cut into
/// the pieces it arrives in, each a little-endian `u16`, then the picture.
const CUT_COUNT: usize = 4;

/// Splits off the cut points the input starts with.
fn cuts_and_rest(data: &[u8]) -> (Vec<usize>, &[u8]) {
    let header = CUT_COUNT * 2;
    if data.len() < header {
        return (Vec::new(), data);
    }
    let mut cuts: Vec<usize> = data[..header]
        .chunks_exact(2)
        .map(|pair| usize::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect();
    cuts.sort_unstable();
    (cuts, &data[header..])
}

/// The app's own picture header readers and stream cutters, the answers
/// the picture decoder and the vector drawer write, and the app's
/// requests as the decoder reads them.
pub fn picture_headers(data: &[u8]) {
    let (cuts, picture) = cuts_and_rest(data);
    crate::picture_guard::fuzz_headers(picture, &cuts);
    crate::picture_guard::fuzz_answer(data);
    crate::picture_guard::fuzz_vector_answer(data);
}

/// A picture decoded as the decoder process does, in this process, with
/// the display limits and a time limit under the fuzzing job's.
pub fn picture_decode(data: &[u8]) {
    let (cuts, picture) = cuts_and_rest(data);
    let limits = Limits {
        time: Duration::from_millis(1500),
        ..Limits::DISPLAY
    };
    crate::picture_guard::fuzz_serve(picture, &cuts, (256, 256), limits);
}

/// What Properties and the Preview pane read from a picture's start and
/// from the media reader's output, the gallery's EXIF dates, Rotate left
/// and right, and a desktop thumbnailer's entry and picture.
pub fn media_details(data: &[u8]) {
    crate::media_details::fuzz_details(data);
    let _ = crate::folder_view::gallery::exif_facts_in(data);
    crate::photo_rotate::fuzz_turn(data);
    crate::thumbnails::fuzz_thumbnailer(data);
}

/// The Preview pane's text and rich text readers, and desktop entries the
/// app may follow as links.
pub fn text_preview(data: &[u8]) {
    crate::window::fuzz_text(data);
    crate::window::fuzz_rtf(data);
    crate::window::fuzz_link(&String::from_utf8_lossy(data));
}
