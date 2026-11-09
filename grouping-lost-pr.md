# Title

fix(views): Keep Downloads grouped by date in the inactive pane of a split tab

# Body

Model: claude-opus-5-5

Downloads is grouped by Date modified until the user chooses otherwise, as in Windows Explorer. In a split tab, the pane beside the active one was shown in the style every other folder shares, which ignores that default, so Downloads there lost its groups. It happened when the split opened, when the other pane was clicked, and when the tab was shown again after another tab. With one style for all folders, that pane kept whatever groups it last showed.

The pane beside the active one is now styled the way the active pane is when it opens a folder: its folder's own style when each folder keeps one, and Downloads' groups in both settings modes.

Regression tests also cover the paths where users reported Downloads looking ungrouped. All of them kept the groups before this change and still do:

- Back then Forward, and Back then Downloads in the navigation pane, with each folder keeping its own style, with one style for all folders, and with a grouping the user chose
- another app saving into Downloads, either under a temporary name and renamed into place or written in several steps, keeping the groups and the selection on the same file
- another app's Save dialog opened at Downloads (at its last folder, a folder it passes or a file it passes), then a file selected, another file type, Back and Forward, and the navigation pane

## How tested

- New split-view test: it fails before the change, with the inactive Downloads pane ungrouped right after the split, and passes after it.
- Ran the group-by, split-view, view-style and file-dialog test suites, including all the new tests, on the branch merged with the latest main, on a private X display and D-Bus.
- Checked the real app under Xvfb with a fictional Downloads folder at 2x scale, in light and dark: Downloads beside the active pane loses its groups before the change and keeps them after.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
