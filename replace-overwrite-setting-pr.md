# feat(transfers): Let Replace overwrite the old file as Windows does (Recycle Bin stays the default)

Model: claude-opus-5-5

Nothing changes by default. When "Replace the file in the destination" is chosen, OpenXplorer still moves the replaced file to the Recycle Bin first, so Undo and a cancelled copy can bring it back. Windows 11 overwrites the old file instead, and it is gone. This adds that behavior as a setting.

## What changes

- Settings > Files & folders > Copying and moving has a new switch, "When replacing a file, move the old one to the Recycle Bin". It is on by default and is stored as `recycleReplacedFiles` only when turned off.
- With the switch off, Replace overwrites the old file in every Replace of a file: paste, drag, copy, move (and their Redo), and Recycle Bin restore. Nothing goes to the Recycle Bin.
- Cancel stays safe with the switch off. A copy is written into private staging and swapped in with one rename at the end, a move renames over the old file, and a restore takes the item out of the Recycle Bin under a private name beside the old file and renames it over it. A Replace cancelled before the swap keeps the old file.
- A folder in the way of a restore still goes to the Recycle Bin. The switch is about files, and a folder tree is never deleted as a side effect of Replace.
- Undo cannot bring an overwritten file back, so it leaves the replacement in place, as it already did on shares without a Recycle Bin.

## Windows behavior

No Microsoft documentation describes what Explorer's Replace does with the old file. Microsoft Q&A answers and data-recovery guides say it is overwritten and does not go to the Recycle Bin. This was not tested on a Windows machine.

## Implementation

- `ReplacedFiles` on `OperationContext` (`ox-core/src/ops/context.rs`), set by the window from the preference when an operation or transfer starts.
- `resolved_transfer.rs`: with `Overwrite`, Replace skips the Recycle Bin step and runs the engine's Replace.
- `restore_conflicts.rs`: `restore_over` restores a file over a file through a private staging name and one rename.
- Docs (`apps/web/lib/docs.json` and the generated Markdown), the parity notes for OPS-028 and OPS-041, and the message template are updated.

## Tests

- ox-core: `replace_recycles_the_old_file_by_default`, `replace_overwrites_without_the_recycle_bin_when_chosen`, `a_moves_replace_overwrites_without_the_recycle_bin_when_chosen`, `cancelling_an_overwriting_copy_midway_keeps_the_old_file` (ops_name_conflicts), `replace_overwrites_the_newer_file_when_chosen`, `a_cancelled_overwriting_restore_keeps_the_newer_file` (ops_restore_conflicts).
- ox-app: `a_pasted_replace_overwrites_the_old_file_when_the_switch_is_off`, `restore_replace_overwrites_the_newer_file_when_the_switch_is_off`, `restore_replace_recycles_the_newer_file_by_default`, `the_replaced_files_switch_is_on_by_default_and_saves_only_its_turning_off`.
- Run: rustfmt and clippy (Rust 1.92.0) clean; i18n, parity, docs sync and public-data audit pass; the ox-core suites ops_name_conflicts, ops_restore_conflicts, ops_recycle_bin, transfer, ops_transfer, ops_undo, ops_tab_transfer and the ox-app name_conflicts, restore_conflicts, recycle_bin and settings_page tests pass under Xvfb. The full `tools/check.py` suite was not run.
- Real input under Xvfb, as an unprivileged user with fictional files: copy-Replace, restore-Replace and Cancel in the middle of a 2 GB Replace, each with the switch on and off, with file contents and the Recycle Bin checked each time. All six pass.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
