// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Dependabot rules of 10 S-7 and S-8: a cooldown on every ecosystem and no auto-merge.

use crate::guard::SourceFile;

/// The path of the Dependabot configuration.
pub const CONFIG_PATH: &str = ".github/dependabot.yml";

/// The shortest cooldown, in days, that any ecosystem of the configuration may declare.
pub const MIN_COOLDOWN_DAYS: u32 = 7;

const REQUIRED_ECOSYSTEMS: [&str; 2] = ["cargo", "github-actions"];
const ECOSYSTEM_KEY: &str = "package-ecosystem:";
const COOLDOWN_KEYS: [&str; 4] = [
    "default-days:",
    "semver-major-days:",
    "semver-minor-days:",
    "semver-patch-days:",
];
const AUTO_MERGE_MARKERS: [&str; 4] = ["gh pr merge", "automerge", "auto-merge", "fetch-metadata"];

/// Names the violations of the Dependabot rules in the repository `files`.
///
/// `.github/dependabot.yml` must exist and declare the ecosystems `cargo` and `github-actions`.
/// Every ecosystem needs a `cooldown` with a `default-days` of at least [`MIN_COOLDOWN_DAYS`], no
/// shorter `semver-*-days` and no `exclude` list. No workflow or other non-Markdown file under
/// `.github/` may merge a pull request or enable auto-merge, so Dependabot never merges itself.
#[must_use]
pub fn violations(files: &[SourceFile]) -> Vec<String> {
    let mut found = Vec::new();
    match files.iter().find(|file| file.path == CONFIG_PATH) {
        Some(config) => found.extend(config_violations(&config.text)),
        None => found.push(format!("{CONFIG_PATH} does not exist")),
    }
    found.extend(auto_merge_violations(files));
    found
}

fn config_violations(text: &str) -> Vec<String> {
    let entries = entries(text);
    let mut found: Vec<String> = REQUIRED_ECOSYSTEMS
        .iter()
        .filter(|required| !entries.iter().any(|entry| entry.ecosystem == **required))
        .map(|required| format!("{CONFIG_PATH} has no update entry for `{required}`"))
        .collect();
    for entry in &entries {
        found.extend(entry_violations(entry));
    }
    found
}

struct Entry<'a> {
    ecosystem: String,
    lines: Vec<&'a str>,
}

fn entries(text: &str) -> Vec<Entry<'_>> {
    let mut entries: Vec<Entry<'_>> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start().trim_start_matches("- ").trim_start();
        if let Some(value) = trimmed.strip_prefix(ECOSYSTEM_KEY) {
            entries.push(Entry {
                ecosystem: value.trim().trim_matches(['"', '\'']).to_owned(),
                lines: Vec::new(),
            });
        } else if let Some(entry) = entries.last_mut() {
            entry.lines.push(line);
        }
    }
    entries
}

fn entry_violations(entry: &Entry<'_>) -> Vec<String> {
    let subject = format!("{CONFIG_PATH} entry `{}`", entry.ecosystem);
    let mut lines = entry
        .lines
        .iter()
        .copied()
        .skip_while(|line| !line.trim_start().starts_with("cooldown:"));
    let Some(header) = lines.next() else {
        return vec![format!(
            "{subject}: no `cooldown`; a version update must wait at least {MIN_COOLDOWN_DAYS} days (10 S-7)"
        )];
    };
    let cooldown: Vec<&str> = lines
        .take_while(|line| is_deeper_than(line, header))
        .collect();
    let mut found = Vec::new();
    let mut default_days = None;
    for line in &cooldown {
        let trimmed = line.trim();
        for key in COOLDOWN_KEYS {
            let Some(value) = trimmed.strip_prefix(key) else {
                continue;
            };
            if !value
                .trim()
                .parse::<u32>()
                .is_ok_and(|days| days >= MIN_COOLDOWN_DAYS)
            {
                found.push(format!(
                    "{subject}: `{key}` {} is not a number of at least {MIN_COOLDOWN_DAYS} days",
                    value.trim()
                ));
            }
            if key == "default-days:" {
                default_days = Some(value.trim().to_owned());
            }
        }
        if trimmed == "exclude:" {
            found.push(format!(
                "{subject}: the cooldown has an `exclude` list, which takes dependencies out of it"
            ));
        }
    }
    if default_days.is_none() {
        found.push(format!("{subject}: the cooldown has no `default-days`"));
    }
    found
}

fn is_deeper_than(line: &str, parent: &str) -> bool {
    line.trim().is_empty() || indentation(line) > indentation(parent)
}

fn indentation(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn auto_merge_violations(files: &[SourceFile]) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.path.starts_with(".github/") && !file.path.ends_with(".md"))
        .flat_map(|file| {
            let lowered = file.text.to_lowercase();
            AUTO_MERGE_MARKERS
                .iter()
                .filter(move |marker| lowered.contains(**marker))
                .map(|marker| {
                    format!(
                        "{}: mentions `{marker}`; pull requests, Dependabot's included, are never merged by automation (10 S-8)",
                        file.path
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::{CONFIG_PATH, violations};
    use crate::guard::SourceFile;

    const GOOD: &str = "version: 2\nupdates:\n  - package-ecosystem: cargo\n    directory: /\n    schedule:\n      interval: weekly\n    cooldown:\n      default-days: 7\n  - package-ecosystem: github-actions\n    directory: /\n    schedule:\n      interval: weekly\n    cooldown:\n      default-days: 7\n";

    fn file(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }

    fn check(config: &str) -> Vec<String> {
        violations(&[file(CONFIG_PATH, config)])
    }

    #[test]
    fn r_sec_5__a_cooldown_of_seven_days_on_both_ecosystems_has_no_violation() {
        assert_eq!(check(GOOD), Vec::<String>::new());
    }

    #[test]
    fn r_sec_5__a_longer_cooldown_is_allowed() {
        assert_eq!(
            check(&GOOD.replace("default-days: 7", "default-days: 14")),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_5__a_missing_configuration_is_a_violation() {
        let found = violations(&[]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("does not exist"), "{found:?}");
    }

    #[test]
    fn r_sec_5__an_ecosystem_without_a_cooldown_is_a_violation() {
        let config = GOOD.replacen("    cooldown:\n      default-days: 7\n", "", 1);
        let found = check(&config);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`cargo`"), "{found:?}");
        assert!(found[0].contains("no `cooldown`"), "{found:?}");
    }

    #[test]
    fn r_sec_5__a_cooldown_shorter_than_seven_days_is_a_violation() {
        for days in ["0", "3", "6"] {
            let config = GOOD.replacen("default-days: 7", &format!("default-days: {days}"), 1);
            let found = check(&config);
            assert_eq!(found.len(), 1, "{days}: {found:?}");
            assert!(found[0].contains("at least 7 days"), "{found:?}");
        }
    }

    #[test]
    fn r_sec_5__a_cooldown_that_is_not_a_number_is_a_violation() {
        let found = check(&GOOD.replacen("default-days: 7", "default-days: week", 1));
        assert_eq!(found.len(), 1, "{found:?}");
    }

    #[test]
    fn r_sec_5__a_short_semver_cooldown_undoes_the_default_for_that_kind_of_update() {
        let config = GOOD.replacen(
            "default-days: 7\n",
            "default-days: 7\n      semver-patch-days: 1\n",
            1,
        );
        let found = check(&config);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("semver-patch-days"), "{found:?}");
    }

    #[test]
    fn r_sec_5__a_cooldown_without_default_days_is_a_violation() {
        let config = GOOD.replacen("default-days: 7", "semver-major-days: 30", 1);
        let found = check(&config);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("no `default-days`"), "{found:?}");
    }

    #[test]
    fn r_sec_5__an_exclude_list_in_the_cooldown_is_a_violation() {
        let config = GOOD.replacen(
            "default-days: 7\n",
            "default-days: 7\n      exclude:\n        - serde\n",
            1,
        );
        let found = check(&config);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("exclude"), "{found:?}");
    }

    #[test]
    fn r_sec_5__a_cooldown_belongs_to_its_own_entry() {
        let config = "updates:\n  - package-ecosystem: cargo\n    directory: /\n  - package-ecosystem: github-actions\n    cooldown:\n      default-days: 7\n";
        let found = check(config);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`cargo`"), "{found:?}");
    }

    #[test]
    fn r_sec_5__both_required_ecosystems_must_be_present() {
        let config =
            "updates:\n  - package-ecosystem: cargo\n    cooldown:\n      default-days: 7\n";
        let found = check(config);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`github-actions`"), "{found:?}");
        let found = check("version: 2\n");
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn r_sec_5__quoted_ecosystem_names_are_read() {
        let config = GOOD.replace("package-ecosystem: cargo", "package-ecosystem: \"cargo\"");
        assert_eq!(check(&config), Vec::<String>::new());
    }

    #[test]
    fn r_sec_5__a_workflow_that_merges_or_enables_auto_merge_is_a_violation() {
        for text in [
            "run: gh pr merge --auto --merge \"$PR\"",
            "uses: reitermarkus/automerge@v2",
            "uses: dependabot/fetch-metadata@v2",
            "run: echo AUTO-MERGE",
            "uses: peter-evans/enable-pull-request-automerge@v3",
        ] {
            let found = violations(&[
                file(CONFIG_PATH, GOOD),
                file(".github/workflows/merge.yml", text),
            ]);
            assert_eq!(found.len(), 1, "{text}: {found:?}");
            assert!(
                found[0].contains(".github/workflows/merge.yml"),
                "{found:?}"
            );
        }
    }

    #[test]
    fn r_sec_5__prose_under_github_may_mention_auto_merge() {
        let found = violations(&[
            file(CONFIG_PATH, GOOD),
            file(".github/SECURITY.md", "We never auto-merge pull requests."),
        ]);
        assert_eq!(found, Vec::<String>::new());
    }

    #[test]
    fn r_sec_5__the_repository_files_follow_the_dependabot_rules() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap();
        let files = crate::sources::collect(root).unwrap();
        assert_eq!(violations(&files), Vec::<String>::new());
    }
}
