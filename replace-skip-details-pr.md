# fix(transfers): Gray out More details while Replace or Skip Files is asked, as Windows 11 does

Model: claude-opus-5-5

## Summary

Windows 11 shows the progress window's **More details** toggle grayed out while it asks **Replace or Skip Files**. The two files are compared only in the File Conflict window that **Compare info for both files** opens. OpenXplorer kept the toggle active and showed a comparison box (name, "From ..." / "Already in ...", dates and sizes) under the question.

- While the window's one action asks Replace or Skip Files, the toggle is insensitive: grayed, out of Tab's reach and not clickable. It reads "More details" (Expanded false), and the action's details stay hidden whatever state was remembered.
- The comparison box under the question is removed (`compared_files`, `compare_in_details`, `forget_comparison`), together with its now unused strings.
- Once the question is answered and progress shows again, the toggle and the remembered More details / Fewer details state come back.
- **Compare info for both files** still opens the File Conflict window.
- The "There is not enough space" question is unchanged: no source shows Windows graying More details there. The failed-item questions on the access-denied-continue branch, which use More details for source and destination, are also left alone.
- The File Conflict window's close button (X) still works like its Cancel and stops the action. Microsoft's dialog guidelines say the title-bar Close "should have the same effect as the Cancel or Close button", and the Win32 dialog manager turns Close into a click on IDCANCEL. No public source documents the File Conflict window on its own, so this was not checked on Windows itself.

## Tests

- `progress_window::tests::replace_or_skip_greys_more_details_until_it_is_answered` (new)
- `progress_window::tests::a_full_drive_leaves_more_details_offered` (new)
- `restore_conflicts::the_question_greys_more_details_and_compares_the_files_only_in_the_file_conflict_window` (replaces the test that expected the comparison box). It also captures `replace-skip-details-*.png`.
- All 52 progress window, transfer panel, conflict window, name-conflict and restore-conflict tests pass under Xvfb. rustfmt and the parity, i18n and docs checks pass. With the local Clippy 1.97, Clippy warns only in files this change does not touch.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
