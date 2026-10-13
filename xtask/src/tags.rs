// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Requirement tags carried in test function names.

use std::fmt;
use std::iter::Peekable;

use crate::requirement::RequirementId;

/// One requirement a test is tagged with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tag {
    /// The tagged requirement.
    pub id: RequirementId,
    /// The part of a split requirement the test covers, such as `ocr`.
    pub qualifier: Option<String>,
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.id.fmt(f)?;
        match &self.qualifier {
            Some(qualifier) => write!(f, ":{qualifier}"),
            None => Ok(()),
        }
    }
}

/// Why a test name does not follow the tag grammar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagError {
    /// A tag group contains a token that is not part of the grammar.
    MalformedTag(String),
    /// The tag group is not closed by a double underscore followed by a description.
    MissingDescription,
    /// A `q` marker is not followed by a qualifier.
    MissingQualifier,
}

/// Reads the requirement tags from a test path such as `module::r_ui_10__description`.
///
/// A test name is tagged when it starts with `r_<area>_<digit>`. The tag group ends at the first
/// double underscore; a tag is `r_<area>_<number><letter?>`.
///
/// # Errors
///
/// Returns a [`TagError`] when a tagged name does not follow the grammar.
pub fn parse_test_name(test_path: &str) -> Result<Vec<Tag>, TagError> {
    let name = test_path.rsplit("::").next().unwrap_or(test_path);
    if !is_tagged(name) {
        return Ok(Vec::new());
    }
    let (group, description) = name.split_once("__").ok_or(TagError::MissingDescription)?;
    if description.is_empty() {
        return Err(TagError::MissingDescription);
    }
    let mut tokens = group.split('_').peekable();
    let mut tags = Vec::new();
    while tokens.peek().is_some() {
        tags.push(parse_tag(&mut tokens)?);
    }
    Ok(tags)
}

fn parse_tag<'a>(tokens: &mut Peekable<impl Iterator<Item = &'a str>>) -> Result<Tag, TagError> {
    match tokens.next() {
        Some("r") => {}
        other => return Err(malformed(other)),
    }
    let area = tokens
        .next()
        .filter(|t| is_area(t))
        .ok_or_else(|| malformed(None))?;
    let number = tokens.next();
    let (digits, refinement) = number
        .and_then(split_number)
        .ok_or_else(|| malformed(number))?;
    let qualifier = if tokens.next_if_eq(&"q").is_some() {
        Some(parse_qualifier(tokens)?)
    } else {
        None
    };
    Ok(Tag {
        id: RequirementId::new(area, digits, refinement),
        qualifier,
    })
}

fn parse_qualifier<'a>(
    tokens: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Result<String, TagError> {
    let mut words = Vec::new();
    while let Some(word) = tokens.next_if(|&token| token != "r") {
        if word.is_empty() || !word.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(malformed(Some(word)));
        }
        words.push(word);
    }
    if words.is_empty() {
        return Err(TagError::MissingQualifier);
    }
    Ok(words.join("-"))
}

fn malformed(token: Option<&str>) -> TagError {
    TagError::MalformedTag(token.unwrap_or_default().to_owned())
}

fn is_area(token: &str) -> bool {
    token.starts_with(|c: char| c.is_ascii_lowercase())
        && token
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn split_number(token: &str) -> Option<(u32, Option<char>)> {
    let digits_end = token
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(token.len());
    let (digits, rest) = token.split_at(digits_end);
    let mut letters = rest.chars();
    let refinement = match (letters.next(), letters.next()) {
        (None, _) => None,
        (Some(letter), None) if letter.is_ascii_lowercase() => Some(letter),
        (Some(_), _) => return None,
    };
    Some((digits.parse().ok()?, refinement))
}

fn is_tagged(name: &str) -> bool {
    let mut tokens = name.split('_');
    let starts_with_letter = |token: Option<&str>| {
        token.is_some_and(|t| t.starts_with(|c: char| c.is_ascii_lowercase()))
    };
    let starts_with_digit =
        |token: Option<&str>| token.is_some_and(|t| t.starts_with(|c: char| c.is_ascii_digit()));
    tokens.next() == Some("r")
        && starts_with_letter(tokens.next())
        && starts_with_digit(tokens.next())
}

#[cfg(test)]
mod tests {
    use super::{TagError, parse_test_name};

    fn rendered(test_path: &str) -> Vec<String> {
        parse_test_name(test_path)
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn names_that_do_not_start_with_an_id_are_untagged() {
        for path in [
            "plain_test",
            "module::r_squared_is_positive",
            "r_2_is_not_an_id",
            "tags::tests::real_thing",
        ] {
            assert_eq!(rendered(path), Vec::<String>::new(), "{path}");
        }
    }

    #[test]
    fn a_tag_group_needs_a_description_after_the_double_underscore() {
        for path in ["r_val_2", "mod::r_val_2__", "r_val_2_r_ui_1"] {
            assert_eq!(
                parse_test_name(path),
                Err(TagError::MissingDescription),
                "{path}"
            );
        }
    }

    #[test]
    fn a_qualifier_marker_needs_its_qualifier() {
        assert_eq!(
            parse_test_name("r_val_1_q__description"),
            Err(TagError::MissingQualifier)
        );
    }

    #[test]
    fn tokens_outside_the_grammar_are_malformed() {
        for (path, token) in [
            ("r_val_1_foo__description", "foo"),
            ("r_val_12ab__description", "12ab"),
            ("r_val_1b2__description", "1b2"),
            ("r_val_1_r_ui__description", ""),
        ] {
            assert_eq!(
                parse_test_name(path),
                Err(TagError::MalformedTag(token.to_owned())),
                "{path}"
            );
        }
    }

    #[test]
    fn a_qualifier_of_several_words_joins_them_with_hyphens() {
        assert_eq!(
            rendered("r_ocr_7_q_uex_names_r_val_1__description"),
            ["R-OCR-7:uex-names", "R-VAL-1"]
        );
    }

    #[test]
    fn qualifier_follows_its_id_after_q() {
        assert_eq!(
            rendered("r_val_1_q_ocr_r_val_2__parses_cap_height"),
            ["R-VAL-1:ocr", "R-VAL-2"]
        );
    }

    #[test]
    fn refinement_letter_and_digits_in_the_area_are_part_of_the_id() {
        assert_eq!(
            rendered("r_cap_1a_r_l10n_2__imports_once"),
            ["R-CAP-1a", "R-L10N-2"]
        );
    }

    #[test]
    fn several_ids_before_the_double_underscore_are_all_tags() {
        assert_eq!(
            rendered("pricing::r_ui_10_r_val_2__major_deviation_blocks_release"),
            ["R-UI-10", "R-VAL-2"]
        );
    }
}
