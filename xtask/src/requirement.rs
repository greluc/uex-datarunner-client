// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Requirement identifiers such as `R-UI-10` or `R-CAP-1a`.

use std::fmt;

/// A requirement ID: an upper-case area, a number and an optional lower-case refinement letter.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequirementId {
    area: String,
    number: u32,
    refinement: Option<char>,
}

impl RequirementId {
    /// Builds an ID from its parts; the area is upper-cased.
    #[must_use]
    pub fn new(area: &str, number: u32, refinement: Option<char>) -> Self {
        Self {
            area: area.to_ascii_uppercase(),
            number,
            refinement,
        }
    }

    /// The IDs of the same area from this number up to and including `last`, without refinements.
    #[must_use]
    pub fn through(&self, last: u32) -> Vec<Self> {
        (self.number..=last)
            .map(|number| Self::new(&self.area, number, None))
            .collect()
    }

    /// Reads an ID such as `R-CAP-1a` from the start of `text` and returns it with the rest.
    ///
    /// Returns `None` when `text` does not start with an ID, or when the ID runs on into more
    /// letters or digits.
    #[must_use]
    pub fn parse_prefix(text: &str) -> Option<(Self, &str)> {
        let body = text.strip_prefix("R-")?;
        let (area, body) = body.split_once('-')?;
        let area_is_valid = area.starts_with(|c: char| c.is_ascii_uppercase())
            && area
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
        if !area_is_valid {
            return None;
        }
        let digits_end = body
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(body.len());
        let (digits, rest) = body.split_at(digits_end);
        let number = digits.parse().ok()?;
        let mut rest_chars = rest.chars();
        let refinement = rest_chars.next().filter(char::is_ascii_lowercase);
        let rest = if refinement.is_some() {
            rest_chars.as_str()
        } else {
            rest
        };
        if rest.starts_with(|c: char| c.is_ascii_alphanumeric()) {
            return None;
        }
        Some((Self::new(area, number, refinement), rest))
    }
}

impl fmt::Display for RequirementId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R-{}-{}", self.area, self.number)?;
        match self.refinement {
            Some(letter) => write!(f, "{letter}"),
            None => Ok(()),
        }
    }
}
