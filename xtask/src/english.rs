// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The English check: German text in tracked files fails the build (ADR-0005 D12).

use crate::guard::SourceFile;

/// The path of the allowlist, relative to the repository root.
pub const ALLOWLIST_PATH: &str = "xtask/english-allowlist.toml";

const MIN_DISTINCT_WORDS: usize = 2;

const GERMAN_LETTERS: [char; 7] = ['ä', 'ö', 'ü', 'ß', 'Ä', 'Ö', 'Ü'];

const GERMAN_WORDS: &[&str] = &[
    "aber", "aus", "bei", "bitte", "damit", "dann", "das", "dass", "dem", "diese", "dieser",
    "dieses", "doch", "durch", "ein", "eine", "einem", "einen", "einer", "eines", "gegen", "haben",
    "hier", "ich", "immer", "ist", "jede", "jeder", "kann", "kein", "keine", "mehr", "nach",
    "nicht", "noch", "nur", "oder", "ohne", "schon", "sind", "sich", "sondern", "sowie", "und",
    "unter", "von", "weil", "wenn", "werden", "wir", "wird", "wurde", "wurden", "zum", "zur",
    "zwischen", "auf",
];

/// One entry of the allowlist: a path (or a directory ending in `/`) and why German text is
/// allowed there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception {
    /// A tracked path, or a directory prefix ending in `/`.
    pub path: String,
    /// Why the text is allowed; never empty.
    pub reason: String,
}

impl Exception {
    fn covers(&self, path: &str) -> bool {
        if self.path.ends_with('/') {
            path.starts_with(&self.path)
        } else {
            path == self.path
        }
    }
}

/// Reads the allowlist file.
///
/// The file holds `[[exception]]` tables, each with a `path` and a `reason` string and nothing
/// else.
///
/// # Errors
///
/// Returns a description when the text is not TOML, has a key other than `exception`, or an
/// entry has an unknown key, a missing or non-string field, an empty reason or a path that is
/// empty, absolute or contains `..`.
pub fn parse_allowlist(text: &str) -> Result<Vec<Exception>, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|error| format!("{ALLOWLIST_PATH} is not valid TOML: {error}"))?;
    if let Some(key) = table.keys().find(|key| key.as_str() != "exception") {
        return Err(format!("{ALLOWLIST_PATH} has the unknown key `{key}`"));
    }
    let Some(entries) = table.get("exception") else {
        return Ok(Vec::new());
    };
    let Some(entries) = entries.as_array() else {
        return Err(format!(
            "{ALLOWLIST_PATH}: `exception` must be an array of tables"
        ));
    };
    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| parse_entry(index + 1, entry))
        .collect()
}

fn parse_entry(number: usize, entry: &toml::Value) -> Result<Exception, String> {
    let fail = |problem: &str| format!("{ALLOWLIST_PATH}: exception {number}: {problem}");
    let Some(entry) = entry.as_table() else {
        return Err(fail("is not a table"));
    };
    if let Some(key) = entry
        .keys()
        .find(|key| !matches!(key.as_str(), "path" | "reason"))
    {
        return Err(fail(&format!("has the unknown key `{key}`")));
    }
    let text = |key: &str| -> Result<String, String> {
        entry
            .get(key)
            .and_then(toml::Value::as_str)
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| fail(&format!("needs a non-empty `{key}` string")))
    };
    let path = text("path")?;
    let reason = text("reason")?;
    if path.starts_with('/') || path.contains('\\') || path.split('/').any(|part| part == "..") {
        return Err(fail(
            "the path must be relative to the repository root with `/` separators",
        ));
    }
    Ok(Exception { path, reason })
}

/// Names the German text in `files` that no entry of `allowlist` excuses.
///
/// A line is German when it holds a German letter (`ä`, `ö`, `ü`, `ß`) or at least two distinct
/// words of the built-in list of German function words. An entry that excuses nothing is reported
/// as well, so the allowlist cannot outlive the text it was written for.
#[must_use]
pub fn violations(files: &[SourceFile], allowlist: &[Exception]) -> Vec<String> {
    let mut found = Vec::new();
    let mut used = vec![false; allowlist.len()];
    for file in files {
        let excusing: Vec<usize> = allowlist
            .iter()
            .enumerate()
            .filter(|(_, exception)| exception.covers(&file.path))
            .map(|(index, _)| index)
            .collect();
        for (index, line) in file.text.lines().enumerate() {
            let Some(evidence) = german_evidence(line) else {
                continue;
            };
            if excusing.is_empty() {
                found.push(format!(
                    "{}:{}: German text ({evidence})",
                    file.path,
                    index + 1
                ));
            } else {
                for &position in &excusing {
                    if let Some(flag) = used.get_mut(position) {
                        *flag = true;
                    }
                }
            }
        }
    }
    for (exception, was_used) in allowlist.iter().zip(&used) {
        if !was_used {
            found.push(format!(
                "{ALLOWLIST_PATH}: the exception for `{}` excuses no German text; remove it",
                exception.path
            ));
        }
    }
    found
}

fn german_evidence(line: &str) -> Option<String> {
    if let Some(letter) = line.chars().find(|c| GERMAN_LETTERS.contains(c)) {
        return Some(format!("the letter `{letter}`"));
    }
    let mut words: Vec<String> = line
        .split(|c: char| !c.is_alphabetic())
        .map(str::to_lowercase)
        .filter(|word| GERMAN_WORDS.contains(&word.as_str()))
        .collect();
    words.sort();
    words.dedup();
    (words.len() >= MIN_DISTINCT_WORDS).then(|| format!("the words {}", words.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::{Exception, parse_allowlist, violations};
    use crate::guard::SourceFile;

    fn file(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }

    fn exception(path: &str) -> Exception {
        Exception {
            path: path.to_owned(),
            reason: "a verbatim quote".to_owned(),
        }
    }

    #[test]
    fn english_text_passes() {
        let text = "The terminal is open and the price is not shown.\nIf this fails, retry.\n";
        assert_eq!(violations(&[file("a.md", text)], &[]), Vec::<String>::new());
    }

    #[test]
    fn a_german_sentence_fails_with_path_line_and_words() {
        let text = "Fine line.\nDas ist nicht erlaubt.\n";
        let found = violations(&[file("docs/a.md", text)], &[]);
        assert_eq!(
            found,
            ["docs/a.md:2: German text (the words das, ist, nicht)"]
        );
    }

    #[test]
    fn a_german_letter_alone_fails() {
        for letter in ["ä", "ö", "ü", "ß", "Ä", "Ö", "Ü"] {
            let found = violations(&[file("a.md", &format!("word {letter}x\n"))], &[]);
            assert_eq!(found.len(), 1, "{letter}");
        }
    }

    #[test]
    fn one_german_function_word_alone_passes() {
        for text in [
            "The MIT licence",
            "a von Neumann machine",
            "the word der in a table",
        ] {
            assert_eq!(
                violations(&[file("a.md", text)], &[]),
                Vec::<String>::new(),
                "{text}"
            );
        }
    }

    #[test]
    fn english_technical_terms_that_look_like_german_words_pass() {
        for line in [
            "MIT, DER-encoded keys",
            "DES and DER in the table",
            "DER MIT DES",
        ] {
            assert_eq!(
                violations(&[file("a.md", line)], &[]),
                Vec::<String>::new(),
                "{line}"
            );
        }
    }

    #[test]
    fn the_same_word_twice_counts_once() {
        assert_eq!(
            violations(&[file("a.md", "und und und")], &[]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn words_are_found_in_identifiers_and_ignoring_case() {
        let found = violations(&[file("a.rs", "let Und_Nicht = 1;")], &[]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn a_word_inside_a_longer_word_does_not_count() {
        let text = "understand the hundred and the reader isolate";
        assert_eq!(violations(&[file("a.md", text)], &[]), Vec::<String>::new());
    }

    #[test]
    fn an_allowlisted_file_passes_and_the_entry_counts_as_used() {
        let files = [file("docs/quote.md", "Das ist nicht erlaubt.\n")];
        assert_eq!(
            violations(&files, &[exception("docs/quote.md")]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_directory_entry_covers_the_files_below_it_but_not_its_siblings() {
        let allow = [exception("corpus/")];
        let covered = [file("corpus/a/expected.json", "Das ist nicht erlaubt.\n")];
        assert_eq!(violations(&covered, &allow), Vec::<String>::new());
        let sibling = [
            file("corpus/a/expected.json", "Das ist nicht erlaubt.\n"),
            file("corpus-private/x.md", "Das ist nicht erlaubt.\n"),
        ];
        let found = violations(&sibling, &allow);
        assert_eq!(found.len(), 1);
        assert!(found[0].starts_with("corpus-private/x.md:1:"), "{found:?}");
    }

    #[test]
    fn a_file_entry_does_not_cover_a_longer_path_with_the_same_prefix() {
        let files = [file("docs/quote.md.bak", "Das ist nicht erlaubt.\n")];
        assert_eq!(violations(&files, &[exception("docs/quote.md")]).len(), 2);
    }

    #[test]
    fn an_entry_whose_file_holds_no_german_text_is_reported() {
        let files = [file("docs/quote.md", "All English now.\n")];
        let found = violations(&files, &[exception("docs/quote.md")]);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("excuses no German text"), "{found:?}");
        assert!(found[0].contains("docs/quote.md"), "{found:?}");
    }

    #[test]
    fn an_entry_for_a_path_that_is_not_tracked_is_reported() {
        let found = violations(&[file("a.md", "English.\n")], &[exception("gone.md")]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn the_allowlist_is_read_with_path_and_reason() {
        let text = "[[exception]]\npath = \"docs/a.md\"\nreason = \"verbatim quote\"\n\n[[exception]]\npath = \"corpus/\"\nreason = \"in-game text\"\n";
        let allow = parse_allowlist(text).unwrap();
        assert_eq!(allow.len(), 2);
        assert_eq!(
            allow[0],
            Exception {
                path: "docs/a.md".into(),
                reason: "verbatim quote".into()
            }
        );
        assert_eq!(allow[1].path, "corpus/");
    }

    #[test]
    fn an_empty_allowlist_is_valid() {
        assert_eq!(parse_allowlist("").unwrap(), Vec::new());
    }

    #[test]
    fn an_entry_without_a_reason_is_rejected() {
        for text in [
            "[[exception]]\npath = \"a.md\"\n",
            "[[exception]]\npath = \"a.md\"\nreason = \"\"\n",
            "[[exception]]\npath = \"a.md\"\nreason = \"   \"\n",
            "[[exception]]\npath = \"a.md\"\nreason = 3\n",
        ] {
            let error = parse_allowlist(text).unwrap_err();
            assert!(
                error.contains("exception 1") && error.contains("reason"),
                "{error}"
            );
        }
    }

    #[test]
    fn an_entry_without_a_usable_path_is_rejected() {
        for path in ["", "/etc/x", "a/../b", "a\\\\b"] {
            let text = format!("[[exception]]\npath = \"{path}\"\nreason = \"r\"\n");
            assert!(parse_allowlist(&text).is_err(), "{path}");
        }
    }

    #[test]
    fn unknown_keys_and_invalid_toml_are_rejected() {
        assert!(
            parse_allowlist("[[exception]]\npath = \"a\"\nreason = \"r\"\nglob = \"*\"\n").is_err()
        );
        assert!(parse_allowlist("[other]\nx = 1\n").is_err());
        assert!(parse_allowlist("exception = 3\n").is_err());
        assert!(parse_allowlist("[[exception]\n").is_err());
    }
}
