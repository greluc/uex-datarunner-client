// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The requirement tags of the tests that `cargo test -- --list` prints.

use crate::registry::{Registry, RegistryError};
use crate::tags::{TagError, parse_test_name};

/// What is wrong with the tags of one test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// The name does not follow the tag grammar.
    Syntax(TagError),
    /// A tag names a requirement or qualifier that the documents do not define.
    Registry(RegistryError),
}

/// A test whose tags are invalid.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    /// The full test path as `cargo test -- --list` prints it.
    pub test: String,
    /// What is wrong with it.
    pub problem: Problem,
}

/// Checks the tags of every test in the output of `cargo test -- --list`.
///
/// Only lines that end in `: test` are tests; the first problem of each test is reported.
#[must_use]
pub fn check_listing(listing: &str, registry: &Registry) -> Vec<Violation> {
    listing
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .filter_map(|test| {
            first_problem(test, registry).map(|problem| Violation {
                test: test.to_owned(),
                problem,
            })
        })
        .collect()
}

fn first_problem(test: &str, registry: &Registry) -> Option<Problem> {
    match parse_test_name(test) {
        Err(error) => Some(Problem::Syntax(error)),
        Ok(tags) => tags
            .iter()
            .find_map(|tag| registry.check(tag).err())
            .map(Problem::Registry),
    }
}

#[cfg(test)]
mod tests {
    use super::{Problem, Violation, check_listing};
    use crate::registry::{Registry, RegistryError};
    use crate::requirement::RequirementId;
    use crate::tags::TagError;

    const LISTING: &str = "\
r_ui_10__deviation_blocks_release: test
plain_test: test
tags::tests::r_val_1_q_ocr_r_ui_99__two_tags: test
r_val_2: test
speed::r_val_1__bench: benchmark
src/lib.rs - documented (line 3): test

4 tests, 1 benchmarks
";

    fn registry() -> Registry {
        Registry::from_documents(
            "| R-UI-10 | x | M |\n| R-VAL-1 | y | M |\n| R-VAL-2 | z | M |\n",
            "R-VAL-1:ocr",
        )
    }

    #[test]
    fn every_tagged_test_with_a_problem_is_reported_with_its_path() {
        assert_eq!(
            check_listing(LISTING, &registry()),
            [
                Violation {
                    test: "tags::tests::r_val_1_q_ocr_r_ui_99__two_tags".to_owned(),
                    problem: Problem::Registry(RegistryError::UnknownRequirement(
                        RequirementId::new("UI", 99, None)
                    )),
                },
                Violation {
                    test: "r_val_2".to_owned(),
                    problem: Problem::Syntax(TagError::MissingDescription),
                },
            ]
        );
    }
}
