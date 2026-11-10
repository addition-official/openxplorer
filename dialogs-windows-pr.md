# Title

feat(dialogs): Ask about extensions and replacing files in Windows 11's compact message boxes, and hide known extensions in the Save dialog's File name

# Body

Model: claude-opus-5-5

Three dialogs now match Windows 11 File Explorer.

- **Changing an extension while renaming** asks in a small message box titled "Rename", with the warning icon, "If you change a file name extension, the file might become unusable." and on the next line "Are you sure you want to change it?". Yes is focused, so Enter renames. No, Escape and the close button keep the old name. Alt+Y and Alt+N press the buttons. This covers F2 in every view, the Rename dialog, renaming several items, and the name in Properties.
- **Save dialog File name**: when "hide extensions for known file types" is on (the default), File name shows the name without its extension ("extra_2" for extra_2.txt). This applies to the caller's suggested name, a file selected in the list, and a file double-clicked. Save still writes the extension, and choosing another type swaps it. With extensions shown, the whole name appears as before.
- **Saving over an existing file** asks in a compact "Confirm Save As" box: the warning icon, "extra_2.txt already exists." and on the next line "Do you want to replace it?", with Yes and No. No is focused, so Enter and Escape replace nothing and return to the dialog with the name kept. The box is sized to its words, and a long name wraps instead of widening it.

Both message boxes share one new message-box style in the dialog frame: a caption with the title and a close button, the icon beside the words, Alt with each button's first letter, and a size that fits the content. The warning icon is `warning_32_color` from the same pinned Fluent UI System Icons package as the app's other icons, listed in `SOURCES.md` with its checksum.

| Dialog | Before | After | Windows 11 |
|---|---|---|---|
| Extension change on rename | "Change the extension?" heading, one-line question, Cancel / Change extension, 510 px wide | "Rename" message box, warning icon, two lines, Yes (focused) / No, sized to its words | "Rename" message box, warning icon, two lines, Yes (default) / No |
| Save dialog File name, extensions hidden | Whole name ("extra_2.txt") | Name without the extension ("extra_2"); Save adds it | "extra_2" |
| Save dialog File name, extensions shown | Whole name | Whole name | "extra_2.txt" |
| Replacing a file on Save | "Confirm Save As" with a quoted one-line question, Replace (focused) / Cancel, 510 px wide with empty space | Compact "Confirm Save As" box, warning icon, two lines, Yes / No (focused) | "Confirm Save As", warning icon, two lines, Yes / No (No preselected) |

## Changed owner decisions

- OPS-049: the extension question was "Change the extension?" with Cancel and "Change extension". It now uses Windows 11's "Rename" message box with Yes and No. The parity entry and OPS-050's note say "No" where they said "Cancel".
- INT-032: the replace question had Replace focused and Cancel. Saving over one file now uses Yes and No with No focused, as in Windows. Several files keep "Confirm Save" with Replace and Cancel, in the same compact box.
- INT-032: File name used to show the whole name. With extensions hidden it now leaves out a known type's extension, and the whole name is selected rather than the name without its extension.

## How it was tested

- Tests were written first and failed on the unchanged code: the Rename message box (title, two lines, Yes/No, focus, icon, Alt keys, size), Confirm Save As (text, Yes/No, No focused, Escape, No keeping the name, Yes replacing, compact size with a short and a long name), and File name with extensions hidden and shown, including type changes and the saved name.
- rustfmt, Clippy with warnings as errors, the parity inventory check and the translation template check pass.
- The app was driven with real keyboard and mouse input on a virtual display at 1x and 2x, in the light and dark themes, using fictional files. Covered: F2 in Details, List, Tiles, Large icons and Compact view, Properties, and renaming several items; Yes, No, Enter, Escape, Alt+Y, Alt+N and clicking; no question when the extension is kept, for dotted folder names, or with extensions hidden; Save with a new, typed or selected name, with extensions hidden and shown, and after a type change; Confirm Save As answered with Enter, Escape, a click on No, Alt+Y and a click on Yes, with a long name.
- Pictures of the three dialogs at 1x and 2x, in light and dark, were taken through the test suite's capture hook.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
