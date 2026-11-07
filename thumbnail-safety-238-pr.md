Model: claude-opus-5-5

# fix(thumbnails): Cap picture decoding and limit desktop thumbnailers' resources

## Summary

This continues the paused thumbnail-safety work on top of #238 (desktop thumbnailers) and #251 (thumbnailers outside /usr and on NixOS).

- **Picture decoding limits.** The app decodes pictures in several places: its own thumbnails, thumbnails read back from the shared cache, the details pane's preview and custom icons. All of these now go through a new `picture_guard` module:
  - A raster picture whose header asks for more than 64 megapixels is refused before gdk-pixbuf allocates it. A smaller one is decoded to the size shown.
  - A damaged or truncated picture is refused.
  - An animated GIF is decoded to its first frame only.
  - Decoding stops after 10 seconds.
  - A cache file larger than 1024 x 1024 is ignored.

  The item keeps its type icon, and the details pane still lists the picture's dimensions.
- **No fallback for refused pictures.** A picture refused this way does not start a desktop thumbnailer, which would read the same file.
- **Thumbnailer resource limits.** Desktop thumbnailers start through `prlimit` and `nice`: 2 GiB of data memory, 64 MiB per file written, no core dumps, and a niceness 10 above the app's. If either tool is missing, they run as before, within the existing 30-second timeout.
- **Remote items in local folders.** With "Also in network folders" off, an item whose URI or target is on a server or device (smb, sftp, mtp, ...) keeps its icon in any folder, including Recent and search results.
- Docs (`docs.json` and the generated Markdown) and the parity notes for VIEW-057, VIEW-058 and PROP-011 are updated.

#238 and #251 already cover the sandbox, the timeout and cancellation, cleanup of the thumbnailer's temporary folder, the checks on its output PNG and the cleared environment, so none of that is changed here.

## Behavior compared with Windows

- Folder Options "Always show icons, never thumbnails" behaves as before: every item shows its icon.
- Removable drives show thumbnails, as in Windows.
- Network folders keep the existing default: thumbnails are off unless Settings turns them on. By default, Windows shows thumbnails there; a Group Policy can turn them off.

## Testing

- `cargo fmt --check`. Clippy reports nothing in the changed files.
- `python3 native/parity/check.py`.
- Tests run by name in an isolated session:
  - `picture_guard`
  - `thumbnails`
  - `a_picture_too_large_to_preview_keeps_its_icon_and_lists_its_dimensions`
  - `the_details_pane_previews_an_image_and_names_its_dimensions`
  - `always_show_icons_turns_thumbnails_off_everywhere`
  - `a_custom_icon_shows_in_the_view_until_the_default_is_restored`
- Two tests fail when TMPDIR is under /tmp, because the sandbox's /tmp hides the test's install folder: `a_sandboxed_thumbnailer_sees_only_its_item` (existing; it fails the same way on unchanged main) and the new `a_sandboxed_thumbnailer_runs_within_its_limits`. Both pass with TMPDIR elsewhere.
- Manual run under Xvfb with fictional files, as an unprivileged user:
  - Large icons at 1x and 2x.
  - A thumbnailer that hangs: it is stopped after 30 s and the window stays responsive.
  - A 102 MB file and a 30000 x 30000 PNG.
  - An SMB share, with network previews off and on.
  - A folder under /media.
  - "Always show icons".
  - No temporary files are left afterwards.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
