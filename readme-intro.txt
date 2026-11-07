STATUS: PR-READY
HEAD: readme-intro 2545cbf27c23083bd6d79e057d3f5d3e746237fb
QUESTION: none
NOTE: README.md only, on newest upstream main (e4520a3), pushed without force-push. The supported-systems note moves word for word to the start of Install (lead-in "A Windows File Explorer-inspired file manager, developed" becomes "OpenXplorer is developed"); the feature paragraph is merged into the opening. No docs or website copy reads README.md: tools/sync-docs.py --check passes and tools/audit-public-data.py reports no issues. Per the manager's request no cargo build, clippy or tests were run for this docs-only change (an earlier native/tools/check.py run with the container's Rust 1.97 stopped on six existing clippy Duration lints in ox-core, unrelated to this branch). PR title and body in readme-intro-pr.md.

BEFORE (first paragraph, upstream main):
A Windows File Explorer-inspired file manager, **developed for Zorin OS first and foremost**. Zorin is the primary target for its desktop experience and integration; Ubuntu and Debian are secondary compatibility targets and require compatible system packages.

AFTER (README.md, first 20 lines):
# OpenXplorer

A Windows File Explorer-inspired file manager for Linux, written in Rust with GTK 4. Browse local folders, SMB shares and SFTP, FTP, WebDAV and NFS locations with tabs, split panes, a folder tree, Details, Compact, List and icon views with thumbnails, clickable paths, pinned folders, search, undoable file operations and light/dark themes.

**[Releases](https://github.com/AKolenda/openxplorer/releases)** · **[Website](https://openxplorer.app)** · **[Installation](docs/installation.md)** · **[Documentation](docs/introduction.md)**

![OpenXplorer browsing fictional sample files](apps/web/public/assets/screenshots/explorer-light.png)

![Two panes side by side showing the same fictional folder](apps/web/public/assets/screenshots/split-view.png)

*The native GTK 4 app, captured in an isolated session with fictional files: the Details view, and split panes. Network examples in these pictures do not establish live SMB compatibility.*

## Install

OpenXplorer is **developed for Zorin OS first and foremost**. Zorin is the primary target for its desktop experience and integration; Ubuntu and Debian are secondary compatibility targets and require compatible system packages.

Version **2.0.14** is a native GTK 4 application written in Rust (`native/`). It keeps the same look, settings, pins and saved passwords as 1.x; see the [changelog](CHANGELOG.md) for what 2.0.14 adds. Get it from [GitHub Releases](https://github.com/AKolenda/openxplorer/releases):

- **Zorin OS 18, Ubuntu 24.04 and newer, Debian 13:** `openxplorer_2.0.14_all.deb` (x86-64). OpenXplorer 1.1.x offers it in **Check for updates**.
- **Fedora, openSUSE Tumbleweed, Arch Linux:** the `.rpm` or `.pkg.tar.zst` of the release, when the release lists one.
