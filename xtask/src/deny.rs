// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The justification rule for the suppressions in `deny.toml` (10 S-6).

use crate::register::{self, Row};

/// Names the suppression entries of `deny_toml` that lack a `reason`, or a row with a revisit date
/// in the suppression register `register` (09 §3a).
///
/// The entries read are `bans.skip`, `bans.skip-tree`, `advisories.ignore`, `licenses.exceptions`
/// and `licenses.clarify`. A `reason` is demanded where the key accepts one; every entry needs a
/// register row that names `deny.toml` and the entry in backticks and ends with a date.
#[must_use]
pub fn unjustified_suppressions(deny_toml: &str, register: &str) -> Vec<String> {
    let table = match deny_toml.parse::<toml::Table>() {
        Ok(table) => table,
        Err(error) => return vec![format!("deny.toml is not valid TOML: {error}")],
    };
    let rows = register::rows(register);
    entries(&table)
        .into_iter()
        .flat_map(|entry| problems(&entry, &rows))
        .collect()
}

const REASONED_KEYS: [(&str, &str); 3] = [
    ("bans", "skip"),
    ("bans", "skip-tree"),
    ("advisories", "ignore"),
];
const REGISTERED_KEYS: [(&str, &str); 2] = [("licenses", "exceptions"), ("licenses", "clarify")];

struct Entry {
    key: String,
    name: String,
    reason: Option<String>,
    reason_required: bool,
}

fn entries(table: &toml::Table) -> Vec<Entry> {
    let reasoned = REASONED_KEYS.map(|key| (key, true));
    let registered = REGISTERED_KEYS.map(|key| (key, false));
    reasoned
        .into_iter()
        .chain(registered)
        .flat_map(|((section, key), reason_required)| {
            array(table, section, key)
                .iter()
                .map(move |value| entry(format!("{section}.{key}"), value, reason_required))
        })
        .collect()
}

fn array<'a>(table: &'a toml::Table, section: &str, key: &str) -> &'a [toml::Value] {
    table
        .get(section)
        .and_then(toml::Value::as_table)
        .and_then(|section| section.get(key))
        .and_then(toml::Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn entry(key: String, value: &toml::Value, reason_required: bool) -> Entry {
    let text_of = |field: &str| {
        value
            .get(field)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
    };
    let name = value
        .as_str()
        .map(str::to_owned)
        .or_else(|| text_of("crate"))
        .or_else(|| text_of("id"))
        .unwrap_or_else(|| "(unnamed)".to_owned());
    Entry {
        key,
        name,
        reason: text_of("reason"),
        reason_required,
    }
}

fn problems(entry: &Entry, rows: &[Row<'_>]) -> Vec<String> {
    let subject = format!("deny.toml {} entry {}", entry.key, entry.name);
    let mut problems = Vec::new();
    let has_reason = entry
        .reason
        .as_deref()
        .is_some_and(|reason| !reason.trim().is_empty());
    if entry.reason_required && !has_reason {
        problems.push(format!("{subject}: no reason"));
    }
    let marker = format!("`{}`", entry.name);
    let named: Vec<_> = rows
        .iter()
        .filter(|row| row.text.contains("`deny.toml`") && row.text.contains(&marker))
        .collect();
    if named.is_empty() {
        problems.push(format!(
            "{subject}: no row in the suppression register (09 §3a) naming `deny.toml` and {marker}"
        ));
    } else if !named.iter().any(|row| row.dated()) {
        problems.push(format!(
            "{subject}: its suppression register row has no revisit date"
        ));
    }
    problems
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::unjustified_suppressions;

    const REGISTER: &str = "| Location | Rule | Reason | Date | Revisit when |\n\
        |---|---|---|---|---|\n\
        | `deny.toml` `bans.skip` entry `syn@1` | duplicate | pulled in by a macro crate | 2026-10-14 | 2027-04-14 |\n";

    fn findings(deny_toml: &str, register: &str) -> Vec<String> {
        unjustified_suppressions(deny_toml, register)
    }

    #[test]
    fn r_sec_10__a_config_without_suppressions_has_no_findings() {
        let deny = "[bans]\nskip = []\nskip-tree = []\n[licenses]\nexceptions = []\n";
        assert_eq!(findings(deny, ""), Vec::<String>::new());
        assert_eq!(findings("", ""), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_register_without_section_headings_is_read_whole() {
        let deny =
            "[bans]\nskip = [{ crate = \"syn@1\", reason = \"pulled in by a macro crate\" }]\n";
        assert_eq!(findings(deny, REGISTER), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_skip_entry_with_a_reason_and_a_register_row_is_justified() {
        let deny =
            "[bans]\nskip = [{ crate = \"syn@1\", reason = \"pulled in by a macro crate\" }]\n";
        assert_eq!(findings(deny, REGISTER), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_bare_string_skip_entry_has_no_reason() {
        let findings = findings("[bans]\nskip = [\"syn@1\"]\n", REGISTER);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("bans.skip"), "{findings:?}");
        assert!(findings[0].contains("syn@1"), "{findings:?}");
        assert!(findings[0].contains("reason"), "{findings:?}");
    }

    #[test]
    fn r_sec_10__an_empty_or_blank_reason_is_no_reason() {
        for reason in ["", "   "] {
            let deny = format!("[bans]\nskip = [{{ crate = \"syn@1\", reason = \"{reason}\" }}]\n");
            let findings = findings(&deny, REGISTER);
            assert_eq!(findings.len(), 1, "{reason:?}: {findings:?}");
            assert!(findings[0].contains("reason"), "{findings:?}");
        }
    }

    #[test]
    fn r_sec_10__a_skip_entry_without_a_register_row_is_unjustified() {
        let deny =
            "[bans]\nskip = [{ crate = \"syn@1\", reason = \"pulled in by a macro crate\" }]\n";
        let findings = findings(deny, "");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("register"), "{findings:?}");
        assert!(findings[0].contains("syn@1"), "{findings:?}");
    }

    #[test]
    fn r_sec_10__a_register_row_without_a_revisit_date_does_not_count() {
        let register = "| `deny.toml` `bans.skip` entry `syn@1` | duplicate | reason | 2026-10-14 | when syn 2 lands |\n";
        let deny =
            "[bans]\nskip = [{ crate = \"syn@1\", reason = \"pulled in by a macro crate\" }]\n";
        let findings = findings(deny, register);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("revisit date"), "{findings:?}");
    }

    #[test]
    fn r_sec_10__a_register_row_that_does_not_name_deny_toml_does_not_count() {
        let register = "| a lint in `syn@1` | duplicate | reason | 2026-10-14 | 2027-04-14 |\n";
        let deny =
            "[bans]\nskip = [{ crate = \"syn@1\", reason = \"pulled in by a macro crate\" }]\n";
        assert_eq!(findings(deny, register).len(), 1);
    }

    #[test]
    fn r_sec_10__a_row_for_another_entry_does_not_justify_this_one() {
        let deny =
            "[bans]\nskip = [{ crate = \"syn@2\", reason = \"pulled in by a macro crate\" }]\n";
        assert_eq!(findings(deny, REGISTER).len(), 1);
    }

    #[test]
    fn r_sec_10__the_prefix_of_another_entry_is_not_a_match() {
        let register = "| `deny.toml` `bans.skip` entry `syn@10` | duplicate | reason | 2026-10-14 | 2027-04-14 |\n";
        let deny =
            "[bans]\nskip = [{ crate = \"syn@1\", reason = \"pulled in by a macro crate\" }]\n";
        assert_eq!(findings(deny, register).len(), 1);
    }

    #[test]
    fn r_sec_10__skip_tree_and_advisory_ignores_are_checked_like_skips() {
        let deny = "[bans]\nskip-tree = [{ crate = \"old@0.1\", depth = 2 }]\n\
            [advisories]\nignore = [\"RUSTSEC-2099-0001\", { id = \"RUSTSEC-2099-0002\" }]\n";
        let findings = findings(deny, "");
        let joined = findings.join("\n");
        for entry in ["old@0.1", "RUSTSEC-2099-0001", "RUSTSEC-2099-0002"] {
            assert!(joined.contains(entry), "{entry}: {findings:?}");
        }
        assert!(joined.contains("bans.skip-tree"), "{findings:?}");
        assert!(joined.contains("advisories.ignore"), "{findings:?}");
    }

    #[test]
    fn r_sec_10__a_yanked_crate_ignore_is_named_by_its_crate_spec() {
        let deny = "[advisories]\nignore = [{ crate = \"gone@0.1.1\", reason = \"yanked\" }]\n";
        let findings = findings(deny, "");
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("gone@0.1.1"), "{findings:?}");
    }

    #[test]
    fn r_sec_10__licence_exceptions_and_clarifications_need_a_register_row() {
        let deny = "[licenses]\nexceptions = [{ allow = [\"OpenSSL\"], crate = \"ring\" }]\n\
            [[licenses.clarify]]\ncrate = \"webpki\"\nexpression = \"ISC\"\nlicense-files = []\n";
        let findings = findings(deny, "");
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert!(findings[0].contains("licenses.exceptions"), "{findings:?}");
        assert!(findings[0].contains("ring"), "{findings:?}");
        assert!(findings[1].contains("licenses.clarify"), "{findings:?}");
        assert!(findings[1].contains("webpki"), "{findings:?}");
    }

    #[test]
    fn r_sec_10__a_licence_exception_with_a_register_row_is_justified() {
        let register = "| `deny.toml` `licenses.exceptions` entry `ring` | licence | reason | 2026-10-14 | 2027-04-14 |\n";
        let deny = "[licenses]\nexceptions = [{ allow = [\"OpenSSL\"], crate = \"ring\" }]\n";
        assert_eq!(findings(deny, register), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_row_outside_the_suppression_register_section_does_not_count() {
        let register = "## 2. Rules\n\n\
            | `deny.toml` `bans.skip` entry `syn@1` | tool | x | 2026-10-14 | 2027-04-14 |\n\n\
            ## 3a. Suppression register\n\n| Location | Rule | Reason | Date | Revisit when |\n|---|---|---|---|---|\n\n\
            ## 3b. Other\n\n\
            | `deny.toml` `bans.skip` entry `syn@2` | tool | x | 2026-10-14 | 2027-04-14 |\n";
        let skip = |name: &str| {
            format!(
                "[bans]\nskip = [{{ crate = \"{name}\", reason = \"pulled in by a macro crate\" }}]\n"
            )
        };
        assert_eq!(findings(&skip("syn@1"), register).len(), 1);
        assert_eq!(findings(&skip("syn@2"), register).len(), 1);
    }

    #[test]
    fn r_sec_10__the_repository_config_has_no_unjustified_suppression() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let deny = std::fs::read_to_string(root.join("deny.toml")).unwrap();
        let register =
            std::fs::read_to_string(root.join("docs/plan/09-engineering-principles.md")).unwrap();
        assert_eq!(findings(&deny, &register), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_config_that_is_not_toml_is_a_finding() {
        let findings = findings("[bans\nskip = ", REGISTER);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("not valid TOML"), "{findings:?}");
    }
}
