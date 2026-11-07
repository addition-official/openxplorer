Model: claude-opus-5-5

# fix(thumbnails): Cap picture decoding and limit desktop thumbnailers' resources

## Summary

This continues the paused thumbnail-safety work on top of #238 (desktop thumbnailers) and #251 (thumbnailers outside /usr and on NixOS), merged with the current main.

- **Picture decoding limits.** The app decodes pictures in several places: its own thumbnails, thumbnails read back from the shared cache, the details pane's preview and custom icons. All of these now go through a new `picture_guard` module:
  - A picture whose header asks for more than 64 megapixels is refused before gdk-pixbuf allocates it. A JPEG may have up to 256 megapixels, because gdk-pixbuf decodes it scaled down.
  - A damaged or truncated picture is refused. Bytes after a PNG's IEND chunk are ignored.
  - An animated GIF is decoded to its first frame only.
  - Decoding stops after about 10 seconds; the limit is checked between reads.
  - A cache file larger than 1024 x 1024 is ignored. A folder's own picture keeps the larger limits of a shown picture.

  The item keeps its type icon, and the details pane still lists the picture's dimensions.
- **No fallback for refused pictures.** A picture refused this way does not start a desktop thumbnailer, which would read the same file. A picture that timed out is tried again later.
- **Thumbnailer resource limits.**
  - Desktop thumbnailers run in a systemd user scope when one is available, whether or not `prlimit` and `nice` are installed: 2 GiB of memory, 64 tasks and one processor's worth of CPU for all their processes together.
  - They also start through `prlimit` and `nice`: 2 GiB of data memory, 64 MiB per file written, 60 s of CPU time per process, at most 64 processes beyond those the user already runs, no core dumps, and a niceness 10 above the app's.
  - Each step falls back when it is not available, and either `prlimit` or `nice` is used alone when it is the only one installed. A warning is logged whenever thumbnailers run without any limits, or when bubblewrap cannot make its sandbox.
- **Leftover temporary folders.** If the app closes while a thumbnailer is still running, its temporary folder stays behind. The next run removes such folders once they are older than any live one can be. Only folders named exactly as the app names them (its prefix and eight hexadecimal digits) are removed.
- **Network files listed elsewhere.** With "Also in network folders" off, these files keep their icon wherever they are listed, including Recent, search results and links:
  - items whose URI or target is on a server or device (smb, sftp, mtp, ...)
  - files on a network file system: NFS, CIFS/SMB, AFS, Ceph, or a FUSE network client such as sshfs, rclone or the GVfs mounts

  A Recently used files row is judged by the file it stands for. The file system is checked off the main thread.
- Docs (`docs.json` and the generated Markdown) and the parity notes for VIEW-057, VIEW-058 and PROP-011 are updated.

#238 and #251 already cover the sandbox, the timeout and cancellation, cleanup of the thumbnailer's temporary folder during a run, the checks on its output PNG and the cleared environment, so none of that is changed here.

## Behavior compared with Windows

- Folder Options "Always show icons, never thumbnails" behaves as before: every item shows its icon.
- Removable drives show thumbnails, as in Windows.
- Network folders keep the existing default: thumbnails are off unless Settings turns them on. By default, Windows shows thumbnails there; a Group Policy can turn them off.
- A PNG with extra data after its end and the first frame of an animated GIF are shown, as Windows shows them.

## Known limitation

On Ubuntu 24.04 and later, AppArmor's default user-namespace restriction keeps bubblewrap from making its sandbox for an app without its own AppArmor profile. Desktop thumbnailers therefore do not start there, and PDFs and videos keep their icons. This is the same as on main since #238. The app's own picture thumbnails are not affected, and a warning is now logged.

## Testing

- `cargo fmt --check`. Clippy reports nothing in the changed files.
- `python3 native/parity/check.py`.
- Tests run by name in an isolated session:
  - `thumbnails` (including `network_fs`)
  - `picture_guard`
  - `a_picture_too_large_to_preview_keeps_its_icon_and_lists_its_dimensions`
  - `the_details_pane_previews_an_image_and_names_its_dimensions`
  - `always_show_icons_turns_thumbnails_off_everywhere`
  - `a_custom_icon_shows_in_the_view_until_the_default_is_restored`
  - `a_folders_new_picture_shows_in_the_pane_at_once`
- Two tests fail when TMPDIR is under /tmp, because the sandbox's /tmp hides the test's install folder: `a_sandboxed_thumbnailer_sees_only_its_item` (existing; it fails the same way on unchanged main) and the new `a_sandboxed_thumbnailer_runs_within_its_limits`. Both pass with TMPDIR elsewhere.
- Manual runs under Xvfb with fictional files, as an unprivileged user:
  - Large icons at 1x and 2x, including a 96-megapixel JPEG and a PNG with appended bytes.
  - A thumbnailer that hangs: it is stopped after 30 s and the window stays responsive.
  - A 102 MB file and a 30000 x 30000 PNG.
  - An SMB share, with network previews off and on.
  - A link to a file on the share's FUSE path.
  - A folder under /media.
  - "Always show icons".
  - No temporary files are left afterwards.
- The systemd scope could not be exercised in the test environment, which runs no user service manager; the `prlimit` fallback was exercised.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
