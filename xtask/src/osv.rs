// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The vulnerability gate of 10 S-19 over the JSON that `osv-scanner` prints.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// The CVSS score from which a vulnerability in a shipped package blocks the merge.
pub const BLOCKING_SCORE: f64 = 7.0;

const CRATES_IO_ECOSYSTEM: &str = "crates.io";
const TOOLING_PACKAGE: &str = "xtask";
const TOOLING_DIRECTORY: &str = "tools";

/// A package in the resolved dependency graph.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageId {
    /// The package name.
    pub name: String,
    /// The resolved version.
    pub version: String,
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

/// The severity OSV-Scanner reports for a group of vulnerability records.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Severity {
    /// No usable score; blocks until the advisory is triaged.
    Unscored,
    /// The highest CVSS base score of the group.
    Scored(f64),
}

impl Severity {
    /// Whether a vulnerability of this severity in a shipped package blocks the merge.
    #[must_use]
    pub fn blocks(self) -> bool {
        match self {
            Self::Unscored => true,
            Self::Scored(score) => score >= BLOCKING_SCORE,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unscored => write!(f, "no score"),
            Self::Scored(score) => write!(f, "CVSS {score:.1}"),
        }
    }
}

/// One group of vulnerability records for one package.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// The affected package.
    pub package: PackageId,
    /// The advisory identifiers of the group.
    pub ids: Vec<String>,
    /// The severity of the group.
    pub severity: Severity,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.package,
            self.ids.join(", "),
            self.severity
        )
    }
}

/// The outcome of the gate.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Verdict {
    /// Findings in shipped packages that block the merge.
    pub blocking: Vec<Finding>,
    /// Findings that are shown but do not block: tooling packages and shipped findings below the limit.
    pub reported: Vec<Finding>,
}

/// Why the gate cannot judge an OSV-Scanner report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateError {
    /// The text is not valid JSON.
    Syntax(String),
    /// The JSON lacks a part of the report format the gate relies on.
    Shape(String),
}

impl fmt::Display for GateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(message) => {
                write!(f, "the OSV-Scanner report is not valid JSON: {message}")
            }
            Self::Shape(message) => write!(f, "the OSV-Scanner report is unexpected: {message}"),
        }
    }
}

impl std::error::Error for GateError {}

/// Judges the JSON `report` of `osv-scanner scan source --format json` against the `shipped` packages.
///
/// A group blocks when its package is shipped and its severity is at least [`BLOCKING_SCORE`] or
/// missing. A vulnerability record that no group names is treated as an unscored group, so a
/// report cannot hide a record by omitting its group.
///
/// # Errors
///
/// Returns [`GateError`] when `report` is not JSON, has no `results` array, or names a package of
/// another ecosystem than crates.io; the gate fails closed on a report it cannot read.
pub fn judge(report: &str, shipped: &BTreeSet<PackageId>) -> Result<Verdict, GateError> {
    let document: Value =
        serde_json::from_str(report).map_err(|error| GateError::Syntax(error.to_string()))?;
    let results = document
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| GateError::Shape("no `results` array".to_owned()))?;
    let mut verdict = Verdict::default();
    for result in results {
        let packages = result
            .get("packages")
            .and_then(Value::as_array)
            .ok_or_else(|| GateError::Shape("a result has no `packages` array".to_owned()))?;
        for entry in packages {
            let package = package_id(entry)?;
            let blocks_here = shipped.contains(&package);
            for finding in findings(entry, &package) {
                if blocks_here && finding.severity.blocks() {
                    verdict.blocking.push(finding);
                } else {
                    verdict.reported.push(finding);
                }
            }
        }
    }
    Ok(verdict)
}

fn package_id(entry: &Value) -> Result<PackageId, GateError> {
    let package = entry
        .get("package")
        .ok_or_else(|| GateError::Shape("a package entry has no `package`".to_owned()))?;
    let text = |field: &str| {
        package
            .get(field)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| GateError::Shape(format!("a package has no `{field}`")))
    };
    let ecosystem = text("ecosystem")?;
    if ecosystem != CRATES_IO_ECOSYSTEM {
        return Err(GateError::Shape(format!(
            "a package of the ecosystem `{ecosystem}`, but only `{CRATES_IO_ECOSYSTEM}` is scanned"
        )));
    }
    Ok(PackageId {
        name: text("name")?,
        version: text("version")?,
    })
}

fn findings(entry: &Value, package: &PackageId) -> Vec<Finding> {
    let groups = array(entry, "groups");
    let mut named: BTreeSet<String> = BTreeSet::new();
    let mut found: Vec<Finding> = groups
        .iter()
        .map(|group| {
            let ids = strings(group, "ids");
            named.extend(ids.iter().cloned());
            Finding {
                package: package.clone(),
                ids,
                severity: severity(group),
            }
        })
        .collect();
    let unnamed: Vec<String> = array(entry, "vulnerabilities")
        .iter()
        .filter_map(|record| record.get("id").and_then(Value::as_str))
        .filter(|id| !named.contains(*id))
        .map(str::to_owned)
        .collect();
    if !unnamed.is_empty() {
        found.push(Finding {
            package: package.clone(),
            ids: unnamed,
            severity: Severity::Unscored,
        });
    }
    found
}

fn array<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value
        .get(field)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn strings(value: &Value, field: &str) -> Vec<String> {
    array(value, field)
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

fn severity(group: &Value) -> Severity {
    let score = match group.get("max_severity") {
        Some(Value::String(text)) => text.trim().parse::<f64>().ok(),
        Some(Value::Number(number)) => number.as_f64(),
        _ => None,
    };
    match score {
        Some(score) if score.is_finite() => Severity::Scored(score),
        _ => Severity::Unscored,
    }
}

/// The packages named in the output of `cargo tree --prefix none --format "{p}"`.
///
/// Every line of the form `name vVERSION …` contributes one package; a line of another form,
/// such as a section header, is skipped.
#[must_use]
pub fn shipped_packages(tree_output: &str) -> BTreeSet<PackageId> {
    tree_output
        .lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            let name = words.next()?;
            let version = words.next()?.strip_prefix('v')?;
            Some(PackageId {
                name: name.to_owned(),
                version: version.to_owned(),
            })
        })
        .collect()
}

/// The names of the workspace members that ship, from the JSON of `cargo metadata --no-deps`.
///
/// Every member is shipped except `xtask` and the crates under the `tools/` directory of the
/// workspace, which are build and test tooling (10 S-19).
///
/// # Errors
///
/// Returns [`GateError`] when `metadata` is not JSON or lacks the `workspace_root` and `packages`
/// fields.
pub fn shipped_roots(metadata: &str) -> Result<Vec<String>, GateError> {
    let document: Value =
        serde_json::from_str(metadata).map_err(|error| GateError::Syntax(error.to_string()))?;
    let root = document
        .get("workspace_root")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .ok_or_else(|| GateError::Shape("cargo metadata has no `workspace_root`".to_owned()))?;
    let tools = root.join(TOOLING_DIRECTORY);
    let packages = document
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| GateError::Shape("cargo metadata has no `packages`".to_owned()))?;
    let mut names = BTreeSet::new();
    for package in packages {
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| GateError::Shape("a package has no `name`".to_owned()))?;
        let manifest = package
            .get("manifest_path")
            .and_then(Value::as_str)
            .map(Path::new)
            .ok_or_else(|| GateError::Shape("a package has no `manifest_path`".to_owned()))?;
        if name != TOOLING_PACKAGE && !manifest.starts_with(&tools) {
            names.insert(name.to_owned());
        }
    }
    Ok(names.into_iter().collect())
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use std::collections::BTreeSet;

    use super::{GateError, PackageId, Severity, judge, shipped_packages, shipped_roots};

    const VULNERABLE: &str = include_str!("../fixtures/osv/vulnerable.json");
    const CLEAN: &str = include_str!("../fixtures/osv/clean.json");

    fn package(name: &str, version: &str) -> PackageId {
        PackageId {
            name: name.to_owned(),
            version: version.to_owned(),
        }
    }

    fn shipped(packages: &[(&str, &str)]) -> BTreeSet<PackageId> {
        packages
            .iter()
            .map(|(name, version)| package(name, version))
            .collect()
    }

    fn report(package: &str, groups: &str, vulnerabilities: &str) -> String {
        format!(
            r#"{{"results":[{{"source":{{"path":"Cargo.lock","type":"lockfile"}},"packages":[
            {{"package":{{"name":"{package}","version":"1.0.0","ecosystem":"crates.io"}},
              "vulnerabilities":[{vulnerabilities}],"groups":[{groups}]}}]}}]}}"#
        )
    }

    fn group(ids: &str, max_severity: &str) -> String {
        format!(r#"{{"ids":[{ids}],"aliases":[],"max_severity":{max_severity}}}"#)
    }

    #[test]
    fn r_sec_5__a_report_without_results_is_clean() {
        let verdict = judge(CLEAN, &shipped(&[("smallvec", "0.6.9")])).unwrap();
        assert!(verdict.blocking.is_empty());
        assert!(verdict.reported.is_empty());
    }

    #[test]
    fn r_sec_5__a_high_severity_vulnerability_in_a_shipped_package_blocks() {
        let verdict = judge(VULNERABLE, &shipped(&[("smallvec", "0.6.9")])).unwrap();
        let blocked: Vec<_> = verdict
            .blocking
            .iter()
            .map(|finding| finding.ids[0].as_str())
            .collect();
        assert_eq!(
            blocked,
            [
                "RUSTSEC-2018-0018",
                "RUSTSEC-2019-0009",
                "RUSTSEC-2019-0012",
                "RUSTSEC-2021-0003"
            ]
        );
        assert_eq!(verdict.blocking[0].severity, Severity::Scored(7.5));
        assert_eq!(verdict.blocking[1].package, package("smallvec", "0.6.9"));
    }

    #[test]
    fn r_sec_5__a_vulnerability_below_the_limit_in_a_shipped_package_is_only_reported() {
        let verdict = judge(VULNERABLE, &shipped(&[("time", "0.1.43")])).unwrap();
        assert!(verdict.blocking.is_empty(), "{verdict:?}");
        let reported: Vec<_> = verdict
            .reported
            .iter()
            .map(|finding| finding.package.name.as_str())
            .collect();
        assert_eq!(
            reported,
            ["smallvec", "smallvec", "smallvec", "smallvec", "time"]
        );
    }

    #[test]
    fn r_sec_5__the_limit_itself_blocks() {
        let json = report("pkg", &group(r#""RUSTSEC-2099-0001""#, r#""7.0""#), "");
        let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
        assert_eq!(verdict.blocking.len(), 1);
        let json = report("pkg", &group(r#""RUSTSEC-2099-0001""#, r#""6.9""#), "");
        let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
        assert!(verdict.blocking.is_empty());
    }

    #[test]
    fn r_sec_5__a_vulnerability_in_a_tooling_package_is_only_reported() {
        let verdict = judge(VULNERABLE, &BTreeSet::new()).unwrap();
        assert!(verdict.blocking.is_empty());
        assert_eq!(verdict.reported.len(), 5);
    }

    #[test]
    fn r_sec_5__another_version_of_a_shipped_package_does_not_block() {
        let verdict = judge(VULNERABLE, &shipped(&[("smallvec", "1.13.2")])).unwrap();
        assert!(verdict.blocking.is_empty());
    }

    #[test]
    fn r_sec_5__a_group_without_a_score_blocks_until_it_is_triaged() {
        for max_severity in ["\"\"", "null", "\"n/a\"", "\"NaN\"", "[]"] {
            let json = report("pkg", &group(r#""RUSTSEC-2099-0001""#, max_severity), "");
            let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
            assert_eq!(verdict.blocking.len(), 1, "{max_severity}");
            assert_eq!(verdict.blocking[0].severity, Severity::Unscored);
        }
    }

    #[test]
    fn r_sec_5__a_group_without_a_max_severity_field_blocks() {
        let json = report("pkg", r#"{"ids":["RUSTSEC-2099-0001"],"aliases":[]}"#, "");
        let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
        assert_eq!(verdict.blocking.len(), 1);
    }

    #[test]
    fn r_sec_5__a_numeric_max_severity_is_read() {
        let json = report("pkg", &group(r#""RUSTSEC-2099-0001""#, "9.1"), "");
        let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
        assert_eq!(verdict.blocking[0].severity, Severity::Scored(9.1));
    }

    #[test]
    fn r_sec_5__a_vulnerability_that_no_group_names_blocks_as_unscored() {
        let json = report(
            "pkg",
            &group(r#""RUSTSEC-2099-0001""#, r#""2.0""#),
            r#"{"id":"RUSTSEC-2099-0001"},{"id":"RUSTSEC-2099-0002"}"#,
        );
        let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
        assert_eq!(verdict.blocking.len(), 1);
        assert_eq!(verdict.blocking[0].ids, ["RUSTSEC-2099-0002"]);
        assert_eq!(verdict.blocking[0].severity, Severity::Unscored);
        assert_eq!(verdict.reported.len(), 1);
    }

    #[test]
    fn r_sec_5__a_package_with_neither_groups_nor_vulnerabilities_has_no_finding() {
        let json = report("pkg", "", "");
        let verdict = judge(&json, &shipped(&[("pkg", "1.0.0")])).unwrap();
        assert_eq!(verdict, super::Verdict::default());
    }

    #[test]
    fn r_sec_5__text_that_is_not_json_fails_closed() {
        let error = judge("Scanning dir .", &BTreeSet::new()).unwrap_err();
        assert!(matches!(error, GateError::Syntax(_)), "{error:?}");
        assert!(error.to_string().contains("not valid JSON"), "{error}");
    }

    #[test]
    fn r_sec_5__a_report_without_a_results_array_fails_closed() {
        for json in ["{}", r#"{"results":null}"#, r#"{"results":{}}"#, "[]"] {
            let error = judge(json, &BTreeSet::new()).unwrap_err();
            assert!(matches!(error, GateError::Shape(_)), "{json}: {error:?}");
        }
    }

    #[test]
    fn r_sec_5__a_result_without_packages_fails_closed() {
        let error = judge(r#"{"results":[{"source":{}}]}"#, &BTreeSet::new()).unwrap_err();
        assert!(matches!(error, GateError::Shape(_)), "{error:?}");
    }

    #[test]
    fn r_sec_5__a_package_of_another_ecosystem_fails_closed() {
        let json = report("pkg", "", "").replace("crates.io", "npm");
        let error = judge(&json, &BTreeSet::new()).unwrap_err();
        assert!(error.to_string().contains("npm"), "{error}");
    }

    #[test]
    fn r_sec_5__a_package_without_a_name_or_version_fails_closed() {
        for field in ["name", "version"] {
            let json = report("pkg", "", "").replace(&format!("\"{field}\""), "\"other\"");
            let error = judge(&json, &BTreeSet::new()).unwrap_err();
            assert!(error.to_string().contains(field), "{field}: {error}");
        }
    }

    #[test]
    fn r_sec_5__a_finding_reads_as_package_ids_and_severity() {
        let verdict = judge(VULNERABLE, &shipped(&[("time", "0.1.43")])).unwrap();
        let finding = verdict.reported.last().unwrap();
        assert_eq!(
            finding.to_string(),
            "time 0.1.43: RUSTSEC-2020-0071, GHSA-wcg3-cvx6-7396 (CVSS 6.2)"
        );
        assert_eq!(Severity::Unscored.to_string(), "no score");
    }

    const TREE: &str = "xtask v0.0.0 (D:\\work\\xtask)\n\
        serde_json v1.0.151\n\
        itoa v1.0.18\n\
        memchr v2.8.3\n\
        serde_json v1.0.151 (*)\n\
        proc-macro-thing v0.2.1 (proc-macro)\n\
        \n\
        [build-dependencies]\n\
        cc v1.2.3\n";

    #[test]
    fn r_sec_5__the_shipped_set_is_read_from_cargo_tree_output() {
        let packages = shipped_packages(TREE);
        assert_eq!(
            packages,
            shipped(&[
                ("xtask", "0.0.0"),
                ("serde_json", "1.0.151"),
                ("itoa", "1.0.18"),
                ("memchr", "2.8.3"),
                ("proc-macro-thing", "0.2.1"),
                ("cc", "1.2.3"),
            ])
        );
    }

    #[test]
    fn r_sec_5__lines_that_are_not_packages_are_skipped() {
        assert!(shipped_packages("").is_empty());
        assert!(shipped_packages("[dev-dependencies]\nerror: no such package\n").is_empty());
    }

    fn metadata(root: &str, members: &[(&str, &str)]) -> String {
        let packages: Vec<String> = members
            .iter()
            .map(|(name, manifest)| format!(r#"{{"name":"{name}","manifest_path":"{manifest}"}}"#))
            .collect();
        format!(
            r#"{{"workspace_root":"{root}","packages":[{}]}}"#,
            packages.join(",")
        )
    }

    #[test]
    fn r_sec_5__xtask_and_the_tools_directory_are_tooling_and_every_other_member_ships() {
        let json = metadata(
            "/work",
            &[
                ("xtask", "/work/xtask/Cargo.toml"),
                ("ocr-eval", "/work/tools/ocr-eval/Cargo.toml"),
                ("app", "/work/app/Cargo.toml"),
                ("shared-kernel", "/work/shared-kernel/Cargo.toml"),
                ("tools-lookalike", "/work/tools-lookalike/Cargo.toml"),
            ],
        );
        assert_eq!(
            shipped_roots(&json).unwrap(),
            ["app", "shared-kernel", "tools-lookalike"]
        );
    }

    #[test]
    fn r_sec_5__a_workspace_that_holds_only_tooling_ships_nothing() {
        let json = metadata("/work", &[("xtask", "/work/xtask/Cargo.toml")]);
        assert!(shipped_roots(&json).unwrap().is_empty());
    }

    #[test]
    fn r_sec_5__metadata_that_cannot_be_read_fails_closed() {
        for json in ["", "{}", r#"{"workspace_root":"/w"}"#, r#"{"packages":[]}"#] {
            assert!(shipped_roots(json).is_err(), "{json}");
        }
        assert!(shipped_roots(r#"{"workspace_root":"/w","packages":[{"name":"a"}]}"#).is_err());
        assert!(
            shipped_roots(r#"{"workspace_root":"/w","packages":[{"manifest_path":"/w/a"}]}"#)
                .is_err()
        );
    }
}
