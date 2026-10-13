// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The requirement IDs and qualifiers that the planning documents define.

use std::collections::BTreeSet;

use crate::requirement::RequirementId;
use crate::tags::Tag;

/// Why a tag does not match the planning documents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// The requirement is not a row of the requirements table.
    UnknownRequirement(RequirementId),
    /// The roadmap does not split the requirement into a part with this qualifier.
    UnknownQualifier {
        /// The tagged requirement.
        id: RequirementId,
        /// The qualifier that the roadmap does not name for it.
        qualifier: String,
    },
}

/// The requirement IDs and qualifiers that the planning documents define.
#[derive(Clone, Debug)]
pub struct Registry {
    ids: BTreeSet<RequirementId>,
    qualified: BTreeSet<(RequirementId, String)>,
}

impl Registry {
    /// Reads the requirement IDs from the table rows of `requirements` (`| R-AREA-n | … |`) and
    /// the qualifiers from `roadmap`.
    ///
    /// A qualifier is read from ``R-ID:q``, from ``R-ID `:q` `` and from a group
    /// ``(`:q`): R-ID, R-ID…n`` that runs to the next semicolon or table cell end.
    #[must_use]
    pub fn from_documents(requirements: &str, roadmap: &str) -> Self {
        let ids = requirements
            .lines()
            .filter_map(|line| line.strip_prefix("| "))
            .filter_map(RequirementId::parse_prefix)
            .map(|(id, _)| id)
            .collect();
        Self {
            ids,
            qualified: qualified_ids(roadmap).collect(),
        }
    }

    /// Checks that the tagged requirement exists.
    ///
    /// # Errors
    ///
    /// Returns a [`RegistryError`] naming what the documents do not define.
    pub fn check(&self, tag: &Tag) -> Result<(), RegistryError> {
        if !self.ids.contains(&tag.id) {
            return Err(RegistryError::UnknownRequirement(tag.id.clone()));
        }
        match &tag.qualifier {
            Some(qualifier)
                if !self
                    .qualified
                    .contains(&(tag.id.clone(), qualifier.clone())) =>
            {
                Err(RegistryError::UnknownQualifier {
                    id: tag.id.clone(),
                    qualifier: qualifier.clone(),
                })
            }
            Some(_) | None => Ok(()),
        }
    }
}

fn qualified_ids(text: &str) -> impl Iterator<Item = (RequirementId, String)> + '_ {
    text.lines().flat_map(|line| {
        let inline = ids_in(line).into_iter().filter_map(|(id, rest)| {
            let qualifier = qualifier_after(rest)?;
            Some((id, qualifier))
        });
        inline.chain(grouped_ids(line)).collect::<Vec<_>>()
    })
}

fn ids_in(text: &str) -> Vec<(RequirementId, &str)> {
    text.match_indices("R-")
        .filter_map(|(start, _)| {
            let preceded_by_word = text
                .get(..start)
                .and_then(|before| before.chars().next_back())
                .is_some_and(char::is_alphanumeric);
            if preceded_by_word {
                return None;
            }
            RequirementId::parse_prefix(text.get(start..)?)
        })
        .collect()
}

fn qualifier_after(rest: &str) -> Option<String> {
    let rest = rest
        .strip_prefix(':')
        .or_else(|| rest.strip_prefix(" `:"))?;
    let end = rest
        .find(|c: char| !c.is_ascii_lowercase() && c != '-')
        .unwrap_or(rest.len());
    let qualifier = rest.get(..end).filter(|q| !q.is_empty())?;
    Some(qualifier.to_owned())
}

fn grouped_ids(line: &str) -> Vec<(RequirementId, String)> {
    const GROUP_START: &str = "(`:";
    let mut found = Vec::new();
    for (start, _) in line.match_indices(GROUP_START) {
        let Some(after) = line.get(start + GROUP_START.len()..) else {
            continue;
        };
        let Some((qualifier, after)) = after.split_once("`):") else {
            continue;
        };
        let list_end = after.find([';', '|']).unwrap_or(after.len());
        for (id, rest) in ids_in(after.get(..list_end).unwrap_or_default()) {
            for member in range_members(id, rest) {
                found.push((member, qualifier.to_owned()));
            }
        }
    }
    found
}

fn range_members(first: RequirementId, rest: &str) -> Vec<RequirementId> {
    let upper_bound = rest
        .strip_prefix('…')
        .or_else(|| rest.strip_prefix(".."))
        .map(|bound| bound.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|digits| digits?.parse().ok());
    match upper_bound {
        Some(last) => first.through(last),
        None => vec![first],
    }
}

#[cfg(test)]
mod tests {
    use super::{Registry, RegistryError};
    use crate::requirement::RequirementId;
    use crate::tags::parse_test_name;

    const REQUIREMENTS: &str = "\
Prose that mentions R-MAN-2 and R-NOPE-1 outside a table.

| ID | Requirement | Priority |
|---|---|---|
| R-UI-10 | Deviations are highlighted | M |
| R-CAP-1a | **Manual import (default):** a click | M |
| R-L10N-2 | Game-language fixtures | M |
| R-VAL-1 | Plausibility | M |
| R-UI-17 | Text size | M |
| R-UI-1 | Queue states | M |
| R-UI-11 | Staleness | M |
| R-UI-13 | Sessions | M |
| R-OCR-7 | Names | M |
| R-VAL-2 | Tolerances | M |
| R-CAP-3 | Environment | M |
| R-SUB-7 | Screenshot | M |
";

    const ROADMAP: &str = "\
its tests use the qualified tag (for example `R-VAL-1:ocr`); R-UI-17:manual needs no screen reader.
| M1 | R-UI-1 `:reports` (report and job states), R-UI-7 `:theme`; manual path (`:manual`): R-VAL-1, R-VAL-2, R-UI-10…12, R-SUB-7 (user-selected region + confirmed preview) |
| M3 | R-OCR-7 `:uex-names`, R-UI-1 `:captures` (capture states); OCR path (`:ocr`): R-VAL-1, R-VAL-2, R-UI-10…12, R-SUB-7; R-CAP-3 `:folder` (environment from the folder path) |
| M4 | R-UI-17 `:automated` (contrast), R-UI-17 `:manual` (keyboard walkthrough) |
";

    fn checked(test_name: &str) -> Result<(), RegistryError> {
        let registry = Registry::from_documents(REQUIREMENTS, ROADMAP);
        let tags = parse_test_name(test_name).unwrap();
        registry.check(&tags[0])
    }

    #[test]
    fn ids_in_requirement_table_rows_are_known() {
        for name in ["r_ui_10__d", "r_cap_1a__d", "r_l10n_2__d", "r_val_1__d"] {
            assert_eq!(checked(name), Ok(()), "{name}");
        }
    }

    #[test]
    fn qualifiers_are_those_the_roadmap_names_for_the_id() {
        for name in [
            "r_val_1_q_ocr__d",
            "r_val_1_q_manual__d",
            "r_ui_17_q_manual__d",
            "r_ui_17_q_automated__d",
            "r_ui_1_q_reports__d",
            "r_ui_1_q_captures__d",
            "r_ocr_7_q_uex_names__d",
            "r_cap_3_q_folder__d",
            "r_sub_7_q_manual__d",
            "r_ui_11_q_ocr__d",
            "r_ui_10_q_manual__d",
        ] {
            assert_eq!(checked(name), Ok(()), "{name}");
        }
    }

    #[test]
    fn a_qualifier_the_roadmap_does_not_name_for_the_id_is_unknown() {
        for (name, id, qualifier) in [
            (
                "r_val_1_q_other__d",
                RequirementId::new("VAL", 1, None),
                "other",
            ),
            (
                "r_ui_13_q_ocr__d",
                RequirementId::new("UI", 13, None),
                "ocr",
            ),
            (
                "r_val_1_q_folder__d",
                RequirementId::new("VAL", 1, None),
                "folder",
            ),
            (
                "r_ui_17_q_ocr__d",
                RequirementId::new("UI", 17, None),
                "ocr",
            ),
        ] {
            assert_eq!(
                checked(name),
                Err(RegistryError::UnknownQualifier {
                    id,
                    qualifier: qualifier.to_owned()
                }),
                "{name}"
            );
        }
    }

    #[test]
    fn ids_outside_requirement_table_rows_are_unknown() {
        assert_eq!(
            checked("r_man_2__d"),
            Err(RegistryError::UnknownRequirement(RequirementId::new(
                "MAN", 2, None
            )))
        );
        assert_eq!(
            checked("r_ui_99__d"),
            Err(RegistryError::UnknownRequirement(RequirementId::new(
                "UI", 99, None
            )))
        );
    }
}
