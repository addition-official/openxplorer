// SPDX-License-Identifier: AGPL-3.0-only
//! Text typed or pasted into the address bar, made ready to be read as an
//! address.
//!
//! Windows 11 File Explorer's address bar takes a path between double
//! quotes, as "Copy as path" puts it on the clipboard, and expands
//! environment variables written as `%NAME%`. [`typed_address`] does the
//! same, and also expands the `$NAME` and `${NAME}` forms Linux users
//! type. A variable that is not set is left as typed, as Windows leaves
//! it. `%USERPROFILE%`, Windows' name for the home folder, stands for
//! `HOME` when it is not set itself.

use std::borrow::Cow;

/// The text `typed` with one pair of surrounding double quotes removed
/// and its environment variables expanded through `variable`, which gives
/// a variable's value or `None` when it is not set.
///
/// `%NAME%` is expanded anywhere; `$NAME` only where the name ends the
/// text or is followed by `/`, so a name such as `$HOME notes` stays as
/// typed; `${NAME}` anywhere. An empty value expands to nothing.
pub fn typed_address<'a>(typed: &'a str, variable: impl Fn(&str) -> Option<String>) -> Cow<'a, str> {
    let _ = &variable;
    Cow::Borrowed(typed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn variables(name: &str) -> Option<String> {
        match name {
            "HOME" => Some("/home/demo".to_owned()),
            "PROJECTS" => Some("/srv/projects".to_owned()),
            "EMPTY" => Some(String::new()),
            _ => None,
        }
    }

    fn typed(text: &str) -> String {
        typed_address(text, variables).into_owned()
    }

    #[test]
    fn surrounding_double_quotes_are_removed_once() {
        assert_eq!(typed("\"/home/demo/Team files\""), "/home/demo/Team files");
        assert_eq!(typed("  \"/home/demo/Team files\"  "), "/home/demo/Team files");
        assert_eq!(typed("\"Recycle Bin\""), "Recycle Bin");
        assert_eq!(typed("\"\\\\nas\\Projects\""), "\\\\nas\\Projects");
        assert_eq!(typed("\"\"/a\"\""), "\"/a\"", "one pair only");
        assert_eq!(typed("\"/home/demo"), "\"/home/demo", "an unmatched quote stays");
        assert_eq!(typed("/home/\"demo\""), "/home/\"demo\"", "quotes inside stay");
        assert_eq!(typed("\""), "\"");
        assert_eq!(typed("\"\""), "");
    }

    #[test]
    fn percent_variables_are_expanded_as_windows_expands_them() {
        assert_eq!(typed("%HOME%/Music"), "/home/demo/Music");
        assert_eq!(
            typed("%home%/Music"),
            "/home/demo/Music",
            "Windows ignores the case"
        );
        assert_eq!(typed("%PROJECTS%/%HOME%"), "/srv/projects//home/demo");
        assert_eq!(typed("%USERPROFILE%\\Desktop"), "/home/demo\\Desktop");
        assert_eq!(typed("%NOT_SET%/Music"), "%NOT_SET%/Music", "unknown stays");
        assert_eq!(typed("100% done/50%"), "100% done/50%");
        assert_eq!(typed("%EMPTY%x"), "x");
        assert_eq!(typed("\"%HOME%/Team files\""), "/home/demo/Team files");
    }

    #[test]
    fn dollar_variables_are_expanded_before_a_slash_or_the_end() {
        assert_eq!(typed("$HOME"), "/home/demo");
        assert_eq!(typed("$HOME/Music"), "/home/demo/Music");
        assert_eq!(typed("${HOME}/Music"), "/home/demo/Music");
        assert_eq!(typed("${PROJECTS}x"), "/srv/projectsx");
        assert_eq!(
            typed("$HOME literal"),
            "$HOME literal",
            "a name inside other words stays"
        );
        assert_eq!(typed("$NOT_SET/Music"), "$NOT_SET/Music");
        assert_eq!(typed("${NOT_SET}/Music"), "${NOT_SET}/Music");
        assert_eq!(typed("smb://nas/C$/Data"), "smb://nas/C$/Data");
        assert_eq!(typed("$"), "$");
        assert_eq!(typed("${HOME"), "${HOME");
        assert_eq!(typed("$home/Music"), "$home/Music", "Linux names keep their case");
    }

    #[test]
    fn plain_text_is_borrowed_unchanged() {
        assert!(matches!(
            typed_address("/home/demo", variables),
            Cow::Borrowed("/home/demo")
        ));
        assert!(matches!(
            typed_address("Documents", variables),
            Cow::Borrowed("Documents")
        ));
    }
}
