// SPDX-License-Identifier: AGPL-3.0-only
//! One entry per visible application for Open with, and the code-editor
//! shortcuts. Ports `CatalogTests` of `v2.0.0:desktop/tests/test_v06.py`.

use ox_core::integration::{default_text_editor, editor_shortcuts, unique_applications, ApplicationInfo};

use super::{app, TestApplication, OWN_ID};

/// Visual Studio Code under `id`, the Python helper's default name.
fn code(id: &str) -> TestApplication {
    app(id, "Visual Studio Code")
}

fn ids(applications: &[TestApplication]) -> Vec<&str> {
    applications
        .iter()
        .map(|application| application.id.as_deref().unwrap_or_default())
        .collect()
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_duplicate_desktop_id`
/// parity: OPEN-011
#[test]
fn a_launcher_listed_twice_is_offered_once() {
    let editor = code("code.desktop");

    assert_eq!(unique_applications([editor.clone(), editor], None).len(), 1);
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_duplicate_visible_name_prefers_primary`
/// parity: OPEN-011, OPEN-015
#[test]
fn of_two_launchers_with_one_name_the_primary_one_is_offered() {
    let primary = code("code.desktop");
    let flatpak = code("com.visualstudio.code.desktop");

    assert_eq!(unique_applications([flatpak, primary.clone()], None), [primary]);
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_preferred_default_wins`
/// parity: OPEN-011
#[test]
fn the_current_default_represents_its_name() {
    let primary = code("code.desktop");
    let flatpak = code("com.visualstudio.code.desktop");

    let offered = unique_applications([primary, flatpak.clone()], Some("com.visualstudio.code.desktop"));

    assert_eq!(offered, [flatpak]);
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_url_helper_omitted`
/// parity: OPEN-011, OPEN-015
#[test]
fn url_handler_helpers_are_not_offered() {
    assert!(unique_applications([code("code-url-handler.desktop")], None).is_empty());
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_hidden_omitted`
/// parity: OPEN-011
#[test]
fn hidden_launchers_are_not_offered() {
    let hidden = TestApplication {
        is_shown: false,
        ..code("code.desktop")
    };

    assert!(unique_applications([hidden], None).is_empty());
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_missing_identifier_omitted`
/// parity: OPEN-011
#[test]
fn launchers_without_an_id_are_not_offered() {
    let without_id = TestApplication {
        id: None,
        ..code("code.desktop")
    };

    assert!(unique_applications([without_id], None).is_empty());
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_self_excluded`
/// parity: OPEN-011
#[test]
fn openxplorer_is_not_offered_to_open_items() {
    assert!(unique_applications([code(OWN_ID)], None).is_empty());
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_no_argument_support_omitted`
/// parity: OPEN-011
#[test]
fn launchers_that_take_neither_files_nor_uris_are_not_offered() {
    let no_arguments = TestApplication {
        accepts_files: false,
        accepts_uris: false,
        ..code("x.desktop")
    };

    assert!(unique_applications([no_arguments], None).is_empty());
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_distinct_editors_preserved`
/// parity: OPEN-015
#[test]
fn distinct_editors_each_get_a_shortcut() {
    let editors = [
        code("code.desktop"),
        app("code-insiders.desktop", "Code Insiders"),
        app("codium.desktop", "VSCodium"),
    ];

    assert_eq!(editor_shortcuts(editors).len(), 3);
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_other_app_not_editor_shortcut`
/// parity: OPEN-015
#[test]
fn other_applications_get_no_editor_shortcut() {
    assert!(editor_shortcuts([app("evince.desktop", "Document Viewer")]).is_empty());
}

/// Ported from `v2.0.0:desktop/tests/test_v06.py::CatalogTests::test_name_case_and_space_dedup`
/// parity: OPEN-011
#[test]
fn names_differing_in_case_and_spacing_are_one_application() {
    let offered = unique_applications([app("a.desktop", " Code "), app("b.desktop", "code")], None);

    assert_eq!(ids(&offered), ["a.desktop"]);
}

/// parity: OPEN-011
#[test]
fn applications_are_listed_by_name() {
    let offered = unique_applications(
        [
            app("z.desktop", "Text Editor"),
            app("a.desktop", "Videos"),
            app("m.desktop", "Archive Manager"),
        ],
        None,
    );

    assert_eq!(ids(&offered), ["m.desktop", "z.desktop", "a.desktop"]);
}

/// An application with the desktop file's `Categories`.
struct Categorized(TestApplication, &'static str);

impl ApplicationInfo for Categorized {
    fn id(&self) -> Option<String> {
        self.0.id()
    }

    fn display_name(&self) -> String {
        self.0.display_name()
    }

    fn should_show(&self) -> bool {
        self.0.should_show()
    }

    fn supports_files(&self) -> bool {
        self.0.supports_files()
    }

    fn supports_uris(&self) -> bool {
        self.0.supports_uris()
    }

    fn categories(&self) -> Option<String> {
        Some(self.1.to_owned())
    }
}

/// "Edit in <editor>" names the default text editor, as Windows names
/// Notepad for a .txt whatever opens it: the default application for
/// plain text when it is a text editor, else the first text editor among
/// the others. A word processor that opens .txt files is no text editor,
/// and the app itself is never offered.
///
/// parity: OPEN-015
#[test]
fn the_default_text_editor_is_a_text_editor_even_when_a_word_processor_opens_text() {
    let writer = || {
        Categorized(
            app("fictional-writer.desktop", "Fictional Writer"),
            "Office;WordProcessor;",
        )
    };
    let editor = || {
        Categorized(
            app("fictional-editor.desktop", "Fictional Editor"),
            "Utility;TextEditor;",
        )
    };
    let other = || {
        Categorized(
            app("other-editor.desktop", "Other Editor"),
            "Development;TextEditor;",
        )
    };
    let own = || Categorized(app(OWN_ID, "OpenXplorer"), "Utility;TextEditor;");

    let chosen = default_text_editor(Some(writer()), [own(), writer(), editor(), other()]);
    let chosen = chosen.expect("a text editor");
    assert_eq!(chosen.id, "fictional-editor.desktop");
    assert_eq!(chosen.name, "Fictional Editor");

    let chosen = default_text_editor(Some(other()), [editor(), other()]);
    assert_eq!(
        chosen.map(|editor| editor.id).as_deref(),
        Some("other-editor.desktop")
    );

    assert!(default_text_editor(Some(writer()), [writer()]).is_none());
    assert!(default_text_editor(None::<Categorized>, []).is_none());
}
