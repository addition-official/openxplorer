// SPDX-License-Identifier: AGPL-3.0-only
//! A trace of what the previews cost, for measuring them: with
//! `OX_THUMBNAIL_TRACE` naming a file, every preview a cell asks for,
//! shows or drops, every read of the thumbnail cache, every picture
//! decoded and every thumbnailer started is written there, one line each,
//! as `<microseconds of CLOCK_MONOTONIC> <event> <URI or detail>`.
//! `native/tools/thumbnail-bench.py` reads it. Without the variable
//! nothing is written, and each call costs one look at a flag.

use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock, PoisonError};

/// The trace file, when the variable names one that could be created.
static TRACE: OnceLock<Option<Mutex<File>>> = OnceLock::new();

fn trace() -> Option<&'static Mutex<File>> {
    TRACE
        .get_or_init(|| {
            let path = std::env::var_os("OX_THUMBNAIL_TRACE")?;
            File::options()
                .create(true)
                .append(true)
                .open(path)
                .ok()
                .map(Mutex::new)
        })
        .as_ref()
}

/// Whether the trace is written.
pub(crate) fn enabled() -> bool {
    trace().is_some()
}

/// Writes `event` about `detail`, when the trace is written.
pub(crate) fn event(event: &str, detail: &str) {
    let Some(file) = trace() else {
        return;
    };
    let now = gtk::glib::monotonic_time();
    // One write per line, so lines of several threads never interleave.
    let line = format!("{now} {event} {detail}\n");
    let mut file = file.lock().unwrap_or_else(PoisonError::into_inner);
    let _ = file.write_all(line.as_bytes());
}
