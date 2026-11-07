# fix(transfers): Gray out More details during Replace or Skip Files, and return to it when the File Conflict window closes, as Windows 11 does

Model: claude-opus-5-5

## Summary

This PR makes two changes so Replace or Skip Files in the progress window behaves as on Windows 11.

**More details is grayed out while the question shows.** On Windows 11 the toggle is grayed out during Replace or Skip Files. The two files are compared only in the File Conflict window that **Compare info for both files** opens. OpenXplorer kept the toggle active and showed a comparison box under the question.

- While the window's one action asks Replace or Skip Files, the toggle is insensitive: grayed, out of Tab's reach and not clickable. It reads "More details" (Expanded false), and the action's details stay hidden whatever state was remembered.
- The comparison box under the question is removed (`compared_files`, `compare_in_details`, `forget_comparison`), together with its now unused strings.
- Once the question is answered and progress shows again, the toggle and the remembered More details / Fewer details state come back.
- The "There is not enough space" question is unchanged: no source shows Windows graying More details there. The failed-item questions, which use More details for source and destination, are also left alone.

**Closing the File Conflict window goes back to the question.** On Windows 11, **Cancel**, the close button (X) and **Esc** in the File Conflict window close only that window. The copy keeps waiting at the same Replace or Skip Files question, unanswered. OpenXplorer stopped the whole action instead.

- When the File Conflict window closes unanswered, the action now asks the same question again.
- Cancelling the action itself still stops it: the action's Cancel, or the progress window's close button.

## Tests

- `name_conflicts::closing_the_file_conflict_window_returns_to_the_question` (new) uses the real pointer and keyboard (XTEST) on the private Xvfb display. It opens Compare info, then closes the window with X, then Cancel, then Esc. Each time the same question still waits, unanswered, with nothing written, and at the end Skip still answers it.
  - Xvfb runs no window manager, so no title bar is drawn. X is therefore sent as the `WM_DELETE_WINDOW` close request a window manager sends for its X button (`Pointer::close_window_under_pointer`).
  - The test fails against the previous code.
- `progress_window::tests::replace_or_skip_greys_more_details_until_it_is_answered` (new)
- `progress_window::tests::a_full_drive_leaves_more_details_offered` (new)
- `restore_conflicts::the_question_greys_more_details_and_compares_the_files_only_in_the_file_conflict_window` replaces the test that expected the comparison box.
- The capture tests that closed the File Conflict window to end a copy now answer Skip after it.
- Results: 91 named progress window, transfer panel, conflict window, name-conflict, restore-conflict, capture and clipboard tests ran under Xvfb. 89 pass.
  - Two clipboard failed-item tests fail the same way on unmodified main in this environment. The tests ran as root, which bypasses the permission denial those tests set up.
- rustfmt and the parity, i18n and docs checks pass. With the local Clippy 1.97, Clippy warns only in files this change does not touch.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
