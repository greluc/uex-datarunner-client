// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rule for the `cargo-vet` exemptions: the shrinking bootstrap set and justified exceptions (10 S-6).

use std::collections::BTreeSet;

use crate::register;

/// The `notes` text every bootstrap exemption carries.
pub const BOOTSTRAP_NOTES: &str = "initial adoption, not yet audited";

/// The revisit date of the shared suppression-register row for the bootstrap exemptions.
pub const BOOTSTRAP_REVISIT_DATE: &str = "2027-04-13";

/// The first line of the baseline file.
pub const BASELINE_HEADER: &str = "bootstrap-exemptions 1";

const CONFIG_MARKER: &str = "`supply-chain/config.toml`";
const BASELINE_MARKER: &str = "`supply-chain/bootstrap-exemptions.txt`";

/// Names the violations of the exemption rule in the `cargo-vet` `config` (10 S-6).
///
/// An exemption whose `notes` are exactly [`BOOTSTRAP_NOTES`] is a bootstrap exemption. The
/// crates they exempt are the crates of the `baseline` (the header line [`BASELINE_HEADER`], then
/// one crate name per line), no more and no fewer, so a new exemption and a removal both show in
/// the baseline's diff and the set only shrinks by review. While bootstrap exemptions exist, the
/// suppression `register` (09 §3a) has one row that names `supply-chain/config.toml` and the
/// baseline file and ends with [`BOOTSTRAP_REVISIT_DATE`]; when none is left, the row is gone.
///
/// Any other exemption is a justified exception for a crate that cannot be audited: it carries
/// non-empty `notes` and has its own register row that names `supply-chain/config.toml` and the
/// crate in backticks and ends with a date.
#[must_use]
pub fn violations(config: &str, baseline: &str, register: &str) -> Vec<String> {
    let table = match config.parse::<toml::Table>() {
        Ok(table) => table,
        Err(error) => {
            return vec![format!(
                "supply-chain/config.toml is not valid TOML: {error}"
            )];
        }
    };
    let (bootstrap, justified): (Vec<_>, Vec<_>) = exemptions(&table)
        .into_iter()
        .partition(|exemption| exemption.notes.as_deref() == Some(BOOTSTRAP_NOTES));
    let rows = register::rows(register);
    let mut found = Vec::new();
    for exemption in &justified {
        found.extend(justified_violations(exemption, &rows));
    }
    found.extend(baseline_violations(
        &bootstrap
            .iter()
            .map(|exemption| exemption.name.clone())
            .collect(),
        baseline,
    ));
    found.extend(shared_row_violations(!bootstrap.is_empty(), &rows));
    found
}

struct Exemption {
    name: String,
    version: String,
    notes: Option<String>,
}

fn exemptions(table: &toml::Table) -> Vec<Exemption> {
    let Some(by_crate) = table.get("exemptions").and_then(toml::Value::as_table) else {
        return Vec::new();
    };
    by_crate
        .iter()
        .flat_map(|(name, entries)| {
            entries
                .as_array()
                .map_or(&[][..], Vec::as_slice)
                .iter()
                .map(move |entry| Exemption {
                    name: name.clone(),
                    version: text(entry, "version").unwrap_or_default(),
                    notes: text(entry, "notes"),
                })
        })
        .collect()
}

fn text(value: &toml::Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}

fn baseline_violations(exempted: &BTreeSet<String>, baseline: &str) -> Vec<String> {
    let mut lines = baseline.lines();
    if lines.next() != Some(BASELINE_HEADER) {
        return vec![format!(
            "supply-chain/bootstrap-exemptions.txt does not start with \"{BASELINE_HEADER}\""
        )];
    }
    let listed: BTreeSet<String> = lines
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect();
    let added = exempted.difference(&listed).map(|name| {
        format!(
            "cargo-vet exemption {name} is not in supply-chain/bootstrap-exemptions.txt: a new dependency is audited or imported, never exempted (10 S-6)"
        )
    });
    let stale = listed.difference(exempted).map(|name| {
        format!(
            "supply-chain/bootstrap-exemptions.txt lists {name}, which has no exemption any more: remove the line"
        )
    });
    added.chain(stale).collect()
}

fn justified_violations(exemption: &Exemption, rows: &[register::Row<'_>]) -> Vec<String> {
    let subject = format!(
        "cargo-vet exemption {} {}",
        exemption.name, exemption.version
    );
    let mut found = Vec::new();
    if exemption
        .notes
        .as_deref()
        .is_none_or(|notes| notes.trim().is_empty())
    {
        found.push(format!(
            "{subject}: no `notes`; the shared bootstrap text is reserved for the crates of supply-chain/bootstrap-exemptions.txt, and a new dependency is audited or imported (10 S-6)"
        ));
    }
    let marker = format!("`{}`", exemption.name);
    let named: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.text.contains(CONFIG_MARKER)
                && row.text.contains(&marker)
                && !row.text.contains(BASELINE_MARKER)
        })
        .collect();
    if named.is_empty() {
        found.push(format!(
            "{subject}: no row in the suppression register (09 §3a) naming {CONFIG_MARKER} and {marker}"
        ));
    } else if !named.iter().any(|row| row.dated()) {
        found.push(format!(
            "{subject}: its suppression register row has no revisit date"
        ));
    }
    found
}

fn shared_row_violations(bootstrap_exists: bool, rows: &[register::Row<'_>]) -> Vec<String> {
    let shared: Vec<_> = rows
        .iter()
        .filter(|row| row.text.contains(BASELINE_MARKER))
        .collect();
    match (bootstrap_exists, shared.as_slice()) {
        (true, []) => vec![format!(
            "the suppression register (09 §3a) has no row for the cargo-vet bootstrap exemptions: it names {CONFIG_MARKER} and {BASELINE_MARKER} and ends with {BOOTSTRAP_REVISIT_DATE}"
        )],
        (true, [row]) if !row.revisit.contains(BOOTSTRAP_REVISIT_DATE) => vec![format!(
            "the suppression register row for the cargo-vet bootstrap exemptions does not end with the revisit date {BOOTSTRAP_REVISIT_DATE}"
        )],
        (true, [_]) | (false, []) => Vec::new(),
        (true, several) => vec![format!(
            "the suppression register (09 §3a) has {} rows that name {BASELINE_MARKER}; the bootstrap exemptions share one",
            several.len()
        )],
        (false, _) => vec![
            "the suppression register (09 §3a) still has a row for the cargo-vet bootstrap exemptions, but none is left: remove the row"
                .to_owned(),
        ],
    }
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::{BOOTSTRAP_NOTES, violations};

    const REGISTER: &str = "| `supply-chain/config.toml` `[[exemptions]]`, the crates listed in `supply-chain/bootstrap-exemptions.txt` | cargo-vet unaudited crate | initial adoption, not yet audited | 2026-10-14 | 2027-04-13 |\n";
    const BASELINE: &str = "bootstrap-exemptions 1\nsha2\ntoml\n";
    const EMPTY_BASELINE: &str = "bootstrap-exemptions 1\n";
    const NO_EXEMPTIONS: &str = "[cargo-vet]\nversion = \"0.10\"\n";
    const JUSTIFICATION: &str = "vendor ships a binary that cannot be read";

    fn config(exemptions: &[(&str, &str, &str)]) -> String {
        let mut text = String::from("[cargo-vet]\nversion = \"0.10\"\n\n");
        for (name, version, notes) in exemptions {
            text.push_str(&format!(
                "[[exemptions.{name}]]\nversion = \"{version}\"\ncriteria = \"safe-to-deploy\"\n"
            ));
            if !notes.is_empty() {
                text.push_str(&format!("notes = \"{notes}\"\n"));
            }
            text.push('\n');
        }
        text
    }

    fn bootstrap() -> String {
        config(&[
            ("sha2", "0.11.0", BOOTSTRAP_NOTES),
            ("toml", "1.1.6", BOOTSTRAP_NOTES),
        ])
    }

    fn justified_row(name: &str, revisit: &str) -> String {
        format!(
            "| `supply-chain/config.toml` `[[exemptions.{name}]]` entry `{name}` | not auditable | binary blob | 2026-10-14 | {revisit} |\n"
        )
    }

    #[test]
    fn r_sec_2__the_bootstrap_set_with_its_baseline_and_row_has_no_violation() {
        assert_eq!(
            violations(&bootstrap(), BASELINE, REGISTER),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_2__a_store_without_exemptions_needs_neither_baseline_entries_nor_a_row() {
        assert_eq!(
            violations(NO_EXEMPTIONS, EMPTY_BASELINE, ""),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_2__a_new_bootstrap_exemption_outside_the_baseline_is_a_violation() {
        let config = config(&[
            ("sha2", "0.11.0", BOOTSTRAP_NOTES),
            ("toml", "1.1.6", BOOTSTRAP_NOTES),
            ("serde_json", "1.0.151", BOOTSTRAP_NOTES),
        ]);
        let found = violations(&config, BASELINE, REGISTER);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("serde_json"), "{found:?}");
        assert!(found[0].contains("never exempted"), "{found:?}");
    }

    #[test]
    fn r_sec_2__a_baseline_entry_without_an_exemption_is_a_violation() {
        let config = config(&[("sha2", "0.11.0", BOOTSTRAP_NOTES)]);
        let found = violations(&config, BASELINE, REGISTER);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("toml"), "{found:?}");
        assert!(found[0].contains("remove the line"), "{found:?}");
    }

    #[test]
    fn r_sec_2__two_versions_of_one_exempted_crate_are_one_baseline_entry() {
        let config = config(&[
            ("sha2", "0.10.0", BOOTSTRAP_NOTES),
            ("sha2", "0.11.0", BOOTSTRAP_NOTES),
            ("toml", "1.1.6", BOOTSTRAP_NOTES),
        ]);
        assert_eq!(
            violations(&config, BASELINE, REGISTER),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_2__a_baseline_without_its_header_is_a_violation() {
        let found = violations(&bootstrap(), "sha2\ntoml\n", REGISTER);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("bootstrap-exemptions 1"), "{found:?}");
    }

    #[test]
    fn r_sec_2__bootstrap_exemptions_without_a_register_row_are_a_violation() {
        let found = violations(&bootstrap(), BASELINE, "");
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("no row"), "{found:?}");
    }

    #[test]
    fn r_sec_2__a_register_row_with_another_revisit_date_is_a_violation() {
        let register = REGISTER.replace("2027-04-13", "2027-10-13");
        let found = violations(&bootstrap(), BASELINE, &register);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("2027-04-13"), "{found:?}");
    }

    #[test]
    fn r_sec_2__the_revisit_date_must_be_in_the_last_cell() {
        let register = REGISTER.replace(
            "| 2026-10-14 | 2027-04-13 |",
            "| 2027-04-13 | 2026-10-14 | when audited |",
        );
        assert_eq!(violations(&bootstrap(), BASELINE, &register).len(), 1);
    }

    #[test]
    fn r_sec_2__several_shared_rows_are_a_violation() {
        let register = format!("{REGISTER}{REGISTER}");
        let found = violations(&bootstrap(), BASELINE, &register);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("share one"), "{found:?}");
    }

    #[test]
    fn r_sec_2__a_shared_row_without_bootstrap_exemptions_is_stale() {
        let found = violations(NO_EXEMPTIONS, EMPTY_BASELINE, REGISTER);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("remove the row"), "{found:?}");
    }

    #[test]
    fn r_sec_2__an_exemption_without_notes_is_a_violation() {
        let config = config(&[("blob", "1.0.0", "")]);
        let register = justified_row("blob", "2027-01-01");
        let found = violations(&config, EMPTY_BASELINE, &register);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("blob 1.0.0"), "{found:?}");
        assert!(found[0].contains("no `notes`"), "{found:?}");
    }

    #[test]
    fn r_sec_2__a_blank_notes_text_is_no_notes() {
        let config = config(&[("blob", "1.0.0", " ")]);
        let register = justified_row("blob", "2027-01-01");
        let found = violations(&config, EMPTY_BASELINE, &register);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("no `notes`"), "{found:?}");
    }

    #[test]
    fn r_sec_2__the_bootstrap_text_with_a_trailing_space_is_a_justification_of_its_own() {
        let config = config(&[("blob", "1.0.0", "initial adoption, not yet audited ")]);
        let register = justified_row("blob", "2027-01-01");
        assert_eq!(
            violations(&config, EMPTY_BASELINE, &register),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_2__a_justified_exemption_with_notes_and_its_own_row_is_allowed() {
        let config = config(&[
            ("sha2", "0.11.0", BOOTSTRAP_NOTES),
            ("toml", "1.1.6", BOOTSTRAP_NOTES),
            ("blob", "1.0.0", JUSTIFICATION),
        ]);
        let register = format!("{REGISTER}{}", justified_row("blob", "2027-01-01"));
        assert_eq!(
            violations(&config, BASELINE, &register),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_2__a_justified_exemption_without_its_own_row_is_a_violation() {
        let config = config(&[("blob", "1.0.0", JUSTIFICATION)]);
        let found = violations(&config, EMPTY_BASELINE, "");
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("blob"), "{found:?}");
        assert!(found[0].contains("no row"), "{found:?}");
    }

    #[test]
    fn r_sec_2__the_shared_row_does_not_justify_another_exemption() {
        let config = config(&[
            ("sha2", "0.11.0", BOOTSTRAP_NOTES),
            ("toml", "1.1.6", BOOTSTRAP_NOTES),
            ("blob", "1.0.0", JUSTIFICATION),
        ]);
        let found = violations(&config, BASELINE, REGISTER);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("blob"), "{found:?}");
    }

    #[test]
    fn r_sec_2__a_justified_row_without_a_revisit_date_does_not_count() {
        let config = config(&[("blob", "1.0.0", JUSTIFICATION)]);
        let register = justified_row("blob", "when the vendor publishes source");
        let found = violations(&config, EMPTY_BASELINE, &register);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("revisit date"), "{found:?}");
    }

    #[test]
    fn r_sec_2__a_config_that_is_not_toml_is_a_violation() {
        let found = violations("[cargo-vet\n", BASELINE, REGISTER);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("not valid TOML"), "{found:?}");
    }

    #[test]
    fn r_sec_2__the_repository_store_follows_the_exemption_rule() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let read = |path: &str| std::fs::read_to_string(root.join(path)).unwrap();
        let found = violations(
            &read("supply-chain/config.toml"),
            &read("supply-chain/bootstrap-exemptions.txt"),
            &read("docs/plan/09-engineering-principles.md"),
        );
        assert_eq!(found, Vec::<String>::new());
    }
}
