# Title

fix(native): Keep the window running and responsive whatever a file holds

# Body

Model: claude-opus-5-5
CLA: I agree

## Summary

Windows Explorer stays open whatever file it shows. OpenXplorer could close or freeze:

- Every picture shown in the Preview and Details panes, every thumbnail and every custom icon was decoded by gdk-pixbuf's C loaders inside the app. One damaged file could abort, crash or hang the window.
- Opening a damaged `.7z` as a folder closed the app.
- A share that stopped answering froze the window.

This PR fixes all three:

- **Pictures decode in a pool of separate processes.** `picture_guard` keeps up to four decoders: the app started again with `--ox-decode-picture`, each serving many pictures in turn.
  - The app streams a picture's bytes to a decoder and reads the pixels back. The decoder runs the same decoding, with the same size and time limits, as the app did before.
  - It limits its own processor time, memory, file size, core dumps and open files. Where bubblewrap works, it runs as the desktop thumbnailers do: without network access, the session bus or the user's files.
  - The app stops a decoder when the selection moves on, or after the picture's time limit plus 3 seconds. A decoder is also replaced after 500 pictures, or as soon as it crashes, hangs or sends an answer that doesn't add up.
  - In each of those cases the item shows its type icon, or the Preview pane says "No preview available". The app checks every size in an answer before using it.
- **SVG pictures.** Some SVG files take far more time and memory to draw than their size suggests; a single line of very long text is enough. gdk-pixbuf draws an SVG only once its loader is closed, so the picture's time limit couldn't stop it.
  - A picture whose first bytes look like markup or name an SVG is now drawn by a decoder under tighter limits: 10 seconds of processor time and 1 GiB of memory. That decoder ends once it has answered, so the next picture gets a fresh one.
  - The picture's time limit plus the pool's 3 seconds still applies, and SVG files over 32 MiB are not drawn.
  - Other formats are decoded as before.
- **Every caller goes through the pool:** the Preview pane, the Details pane picture, thumbnails, cached thumbnails, thumbnailer and PDF page output, custom folder icons, and the gallery's picture sizes.
  - Properties' **Change Icon** pickers used to decode on the main thread, and PNG rotation used to decode in the app. Both now use the pool.
  - The gallery reads picture sizes from the file header in Rust where the format allows.
- **7z:** a damaged files record could index past its bits and panic. The archive worker re-raised that panic on the main thread, which closed the app. The read is fixed, and the worker now reports any panic as a damaged archive.
- **No freeze on a share that stops answering.** A path can belong to a kernel, sshfs or FUSE mount of a share, where every `stat` and `read` waits for as long as the server is gone.
  - These used to read files on the main thread: Properties (to decide its Shortcut tab and whether a folder is a link), opening a `.desktop` link, the startup folder check, adding a gallery folder, and a failed copy's drive name.
  - They now ask a worker thread and wait at most 400 ms. A local file answers at once.
  - If a file doesn't answer, the window carries on without that fact: Properties opens without its Shortcut, Sharing and Customize tabs, and a link opens as a plain file.
- **Launchers:** a pipe named `something.desktop` no longer hangs the window, in Properties or on open. Launchers over 1 MiB are not read.
- **Saving a shortcut:** saving already ran on a worker thread. It now gives up after 10 seconds on a share that stopped answering, and says so.
- **Panics in pane updates:** a panic while the details pane, the Preview pane, a file's media details or an archive's listing is being shown is caught, logged to `crash.log` and skipped. Before, it closed the app through GTK's C callbacks.
- **After a crash:**
  - Panics are written with a backtrace to `~/.cache/winspace/crash.log`.
  - The app notices at the next start when the last run ended abnormally.
  - **Reopen my tabs** still brings the tabs back after a crash and shows nothing about it. The tabs are saved every 3 seconds.
  - If the last two runs both ended within their first minute, the restore is skipped once, quietly, so a tab that closes the app can't close it at every start.

Media playback in the Preview pane is not changed here; #368 moves it out of process.

## Cost

Measured on 2,000 photos in Large icons, paging through the whole folder, two runs per build (debug builds):

| | Thumbnails made by the end of a fast scroll | CPU, first and second scroll | CPU after a restart, read from the thumbnail cache | Frame gaps over 50 ms (first scroll) | Gaps over 100 ms |
|---|---|---|---|---|---|
| main | 2,000 | 49 s | 15.5 s | 7–8 | 0 |
| one process per picture (an earlier version, not kept) | about 1,100 | 172 s | 52 s | 22–23 | 0 |
| decoder pool (this PR) | 2,000 | 52 s | 16 s | 5–11 | 0 |

## Testing

- Tests written first. The 7z test panics on the old code and passes with the fix.
- The decoders, tested against:
  - a decoder that aborts or hangs, and one cancelled mid-picture;
  - endless input;
  - 2,000 mangled answers;
  - a corpus of broken pictures generated in the test: cut-short files, huge and negative sizes, SVGs whose entities expand without end, deep nesting, self-references and seeded mutations.
- SVG tests:
  - SVGs are drawn at the size shown, and broken ones are refused.
  - A long-text SVG too large to draw whole is drawn in part or refused.
  - An SVG slow to draw is refused at the time limit.
  - A decoder ends after a vector picture, and the next picture is decoded.
- Seeded mutation fuzzing:
  - 40,000 rounds each of ZIP, 7z and four tar flavors;
  - 600,000 per sample for the RTF reader, the text decoder and the picture header readers.
- Decoder pool tests: one process serves many pictures; a decoder that crashes mid-stream is replaced; a large picture follows small ones in the same decoder. That last test caught a bug where a decoder kept the previous picture's size limit; it is fixed.
- Stalled-mount tests: Properties of a launcher or folder on a stalled mount, and of a pipe named `.desktop`, opens in under 3 seconds. A test can't mount a real share, so a stalled mount is simulated by delaying every look under a folder.
- A run under a private Xvfb: 300 generated files, a third of them broken, arrowed through with the Preview pane and then the Details pane. The app stayed open, with nothing on stderr matching "Bail out", CRITICAL, "panicked" or a signal.
- `native/parity/check.py`, rustfmt and Clippy with Rust 1.92.0, and every test executable in `check.py`'s isolation. The test container only has root, and the app refuses to run as root, so `check.py` itself stops at the first test that starts the app. The run outside it: 4,885 tests passed. The 54 failures are the same tests that fail on unchanged main here (permission tests that root bypasses, Python interop fixtures, the bubblewrap thumbnailer tests), plus one layout test that passed when run again alone.
