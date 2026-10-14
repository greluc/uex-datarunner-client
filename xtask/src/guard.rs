// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The supply-chain guard of 10 S-4: the rules that no configuration may relax.

use std::collections::BTreeMap;
use std::fmt;

use crate::lockfile::{self, CRATES_IO, LockedPackage};

/// A file the guard reads, with its path relative to the repository root and `/` separators.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    /// The path relative to the repository root.
    pub path: String,
    /// The file text.
    pub text: String,
}

/// What a rule found wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Problem {
    /// A TOML file cannot be parsed, so none of its rules can be checked.
    Unparseable(String),
    /// A `[[package]]` has a source but no checksum.
    PackageWithoutChecksum(String),
    /// A `[[package]]` comes from a source other than the crates.io registry.
    PackageFromOtherSource {
        /// The package as `name version`.
        package: String,
        /// The source the lockfile records.
        source: String,
    },
    /// A top-level section that redirects or overrides dependency resolution.
    ForbiddenSection(String),
    /// A configuration key that changes how Cargo runs the compiler or fetches sources.
    ForbiddenConfigKey(String),
    /// A dependency that names a git repository or an alternative registry.
    ForbiddenDependencyKey {
        /// The dependency name.
        dependency: String,
        /// The offending key.
        key: String,
    },
    /// A path dependency that leaves the workspace.
    PathOutsideWorkspace {
        /// The dependency name.
        dependency: String,
        /// The path as written.
        path: String,
    },
    /// An environment variable that redirects sources or wraps the compiler.
    ForbiddenEnvironmentOverride(String),
    /// A resolving Cargo command without `--locked` (or `--frozen`) before any `--`.
    CargoWithoutLocked {
        /// The one-based line of the command.
        line: usize,
        /// The command as written, starting at `cargo`.
        command: String,
    },
    /// An alias for a resolving Cargo command without `--locked` (or `--frozen`).
    AliasWithoutLocked(String),
    /// The toolchain channel is missing or is not an exact `MAJOR.MINOR.PATCH` version.
    ToolchainNotExact(Option<String>),
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unparseable(message) => write!(f, "not valid TOML: {message}"),
            Self::PackageWithoutChecksum(package) => {
                write!(f, "{package} has a source but no checksum")
            }
            Self::PackageFromOtherSource { package, source } => {
                write!(f, "{package} comes from {source}, not from crates.io")
            }
            Self::ForbiddenSection(section) => write!(f, "`[{section}]` is not allowed"),
            Self::ForbiddenConfigKey(key) => write!(f, "`{key}` is not allowed"),
            Self::ForbiddenDependencyKey { dependency, key } => {
                write!(f, "dependency `{dependency}` uses `{key}`")
            }
            Self::PathOutsideWorkspace { dependency, path } => {
                write!(
                    f,
                    "dependency `{dependency}` has the path `{path}` outside the workspace"
                )
            }
            Self::ForbiddenEnvironmentOverride(name) => write!(f, "`{name}` is not allowed"),
            Self::CargoWithoutLocked { line, command } => {
                write!(f, "line {line}: `{command}` has no --locked")
            }
            Self::AliasWithoutLocked(alias) => write!(f, "alias `{alias}` has no --locked"),
            Self::ToolchainNotExact(Some(channel)) => {
                write!(f, "toolchain channel `{channel}` is not an exact version")
            }
            Self::ToolchainNotExact(None) => write!(f, "toolchain names no channel"),
        }
    }
}

/// A problem in one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// The path relative to the repository root.
    pub file: String,
    /// What is wrong.
    pub problem: Problem,
}

/// Checks the packages of `Cargo.lock`: every one with a source needs a checksum and the
/// crates.io registry as its source.
#[must_use]
pub fn check_lockfile(packages: &[LockedPackage]) -> Vec<Problem> {
    let mut problems = Vec::new();
    for package in packages {
        let Some(source) = &package.source else {
            continue;
        };
        let label = format!("{} {}", package.name, package.version);
        if package.checksum.is_none() {
            problems.push(Problem::PackageWithoutChecksum(label.clone()));
        }
        if source != CRATES_IO {
            problems.push(Problem::PackageFromOtherSource {
                package: label,
                source: source.clone(),
            });
        }
    }
    problems
}

const FORBIDDEN_SECTIONS: [&str; 4] = ["patch", "replace", "source", "registries"];
const FORBIDDEN_CONFIG_KEYS: [(&str, &str); 4] = [
    ("build", "rustc"),
    ("build", "rustc-wrapper"),
    ("build", "rustc-workspace-wrapper"),
    ("net", "git-fetch-with-cli"),
];
const DEPENDENCY_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];
const FORBIDDEN_DEPENDENCY_KEYS: [&str; 3] = ["git", "registry", "registry-index"];

fn parse_table(text: &str) -> Result<toml::Table, Problem> {
    text.parse()
        .map_err(|error: toml::de::Error| Problem::Unparseable(error.message().to_owned()))
}

fn forbidden_sections(table: &toml::Table) -> Vec<Problem> {
    FORBIDDEN_SECTIONS
        .iter()
        .filter(|section| table.contains_key(**section))
        .map(|section| Problem::ForbiddenSection((*section).to_owned()))
        .collect()
}

/// Checks a `Cargo.toml` at `path` (relative to the repository root).
///
/// It may not carry `[patch]`, `[replace]`, `[source]` or `[registries]`, and no dependency may
/// name a git repository, an alternative registry or a path outside the workspace.
#[must_use]
pub fn check_manifest(path: &str, text: &str) -> Vec<Problem> {
    let table = match parse_table(text) {
        Ok(table) => table,
        Err(problem) => return vec![problem],
    };
    let mut problems = forbidden_sections(&table);
    let manifest_dir = path.rsplit_once('/').map_or("", |(directory, _)| directory);
    for dependencies in dependency_tables(&table) {
        for (name, spec) in dependencies {
            let Some(spec) = spec.as_table() else {
                continue;
            };
            for key in FORBIDDEN_DEPENDENCY_KEYS {
                if spec.contains_key(key) {
                    problems.push(Problem::ForbiddenDependencyKey {
                        dependency: name.clone(),
                        key: key.to_owned(),
                    });
                }
            }
            if let Some(dependency_path) = spec.get("path").and_then(toml::Value::as_str)
                && leaves_workspace(manifest_dir, dependency_path)
            {
                problems.push(Problem::PathOutsideWorkspace {
                    dependency: name.clone(),
                    path: dependency_path.to_owned(),
                });
            }
        }
    }
    problems
}

fn direct_tables(table: &toml::Table) -> Vec<&toml::Table> {
    DEPENDENCY_TABLES
        .iter()
        .filter_map(|name| table.get(*name).and_then(toml::Value::as_table))
        .collect()
}

fn dependency_tables(manifest: &toml::Table) -> Vec<&toml::Table> {
    let mut tables = direct_tables(manifest);
    if let Some(workspace) = manifest.get("workspace").and_then(toml::Value::as_table) {
        tables.extend(
            workspace
                .get("dependencies")
                .and_then(toml::Value::as_table),
        );
    }
    if let Some(targets) = manifest.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            tables.extend(direct_tables(target));
        }
    }
    tables
}

fn leaves_workspace(manifest_dir: &str, path: &str) -> bool {
    let absolute = path.starts_with(['/', '\\']) || path.as_bytes().get(1) == Some(&b':');
    if absolute {
        return true;
    }
    let mut depth = manifest_dir
        .split('/')
        .filter(|part| !part.is_empty())
        .count();
    for part in path.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => match depth.checked_sub(1) {
                Some(shallower) => depth = shallower,
                None => return true,
            },
            _ => depth += 1,
        }
    }
    false
}

/// Checks a `.cargo/config.toml`.
///
/// It may not carry `[patch]`, `[replace]`, `[source]` or `[registries]`, a compiler wrapper or
/// `net.git-fetch-with-cli`, and every alias for a resolving command needs `--locked`.
#[must_use]
pub fn check_config(text: &str) -> Vec<Problem> {
    let table = match parse_table(text) {
        Ok(table) => table,
        Err(problem) => return vec![problem],
    };
    let mut problems = forbidden_sections(&table);
    if table.contains_key("paths") {
        problems.push(Problem::ForbiddenSection("paths".to_owned()));
    }
    for (section, key) in FORBIDDEN_CONFIG_KEYS {
        let present = table
            .get(section)
            .and_then(toml::Value::as_table)
            .is_some_and(|section| section.contains_key(key));
        if present {
            problems.push(Problem::ForbiddenConfigKey(format!("{section}.{key}")));
        }
    }
    let known = aliases(text);
    for (name, expansion) in &known {
        let args: Vec<&str> = expansion.iter().map(String::as_str).collect();
        if !is_locked(&args, &known, 0) {
            problems.push(Problem::AliasWithoutLocked(name.clone()));
        }
    }
    problems
}

/// Checks a `rust-toolchain.toml`: the channel must be an exact version.
#[must_use]
pub fn check_toolchain(text: &str) -> Vec<Problem> {
    let table = match parse_table(text) {
        Ok(table) => table,
        Err(problem) => return vec![problem],
    };
    let channel = table
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|toolchain| toolchain.get("channel"))
        .and_then(toml::Value::as_str);
    match channel {
        Some(channel) if is_exact_version(channel) => Vec::new(),
        other => vec![Problem::ToolchainNotExact(other.map(str::to_owned))],
    }
}

fn is_exact_version(channel: &str) -> bool {
    let parts: Vec<&str> = channel.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

fn is_forbidden_variable(name: &str) -> bool {
    name.contains("RUSTC_WRAPPER")
        || name.contains("RUSTC_WORKSPACE_WRAPPER")
        || name.contains("CARGO_NET_GIT_FETCH_WITH_CLI")
        || name.starts_with("CARGO_REGISTRIES_")
        || name.starts_with("CARGO_SOURCE_")
}

/// Finds the forbidden environment variable names in `text`: a compiler wrapper,
/// `CARGO_REGISTRIES_*`, `CARGO_SOURCE_*` and `CARGO_NET_GIT_FETCH_WITH_CLI`.
#[must_use]
pub fn check_environment_overrides(text: &str) -> Vec<Problem> {
    let mut found: Vec<&str> = Vec::new();
    for name in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        if is_forbidden_variable(name) && !found.contains(&name) {
            found.push(name);
        }
    }
    found
        .into_iter()
        .map(|name| Problem::ForbiddenEnvironmentOverride(name.to_owned()))
        .collect()
}

/// Finds the headers `[patch]`, `[replace]`, `[source]` and `[registries]` (also as the start
/// of a longer header such as `[patch.crates-io]`) anywhere in `text`.
#[must_use]
pub fn check_section_headers(text: &str) -> Vec<Problem> {
    FORBIDDEN_SECTIONS
        .iter()
        .filter(|section| {
            text.contains(&format!("[{section}]")) || text.contains(&format!("[{section}."))
        })
        .map(|section| Problem::ForbiddenSection((*section).to_owned()))
        .collect()
}

const RESOLVING_COMMANDS: [&str; 26] = [
    "b", "bench", "build", "c", "check", "clippy", "d", "deny", "doc", "fetch", "fix", "install",
    "llvm-cov", "metadata", "mutants", "nextest", "package", "publish", "r", "run", "rustc",
    "rustdoc", "t", "test", "tree", "vendor",
];
const ALIAS_DEPTH: usize = 8;

/// Whether the arguments after `cargo` pass `--locked`, or run an alias that does.
///
/// A command that does not resolve dependencies needs no flag.
#[must_use]
pub fn invocation_is_locked(args: &[&str], aliases: &BTreeMap<String, Vec<String>>) -> bool {
    is_locked(args, aliases, 0)
}

fn is_locked(args: &[&str], aliases: &BTreeMap<String, Vec<String>>, depth: usize) -> bool {
    let mut rest = args.iter().copied();
    let mut global_locked = false;
    let subcommand = loop {
        match rest.next() {
            None => return true,
            Some(option) if option.starts_with('+') => {}
            Some("--locked" | "--frozen") => global_locked = true,
            Some("-Z" | "-C" | "--config" | "--color") => {
                rest.next();
            }
            Some(option) if option.starts_with('-') => {}
            Some(subcommand) => break subcommand,
        }
    };
    if RESOLVING_COMMANDS.contains(&subcommand) {
        return global_locked
            || rest
                .take_while(|arg| *arg != "--")
                .any(|arg| arg == "--locked" || arg == "--frozen");
    }
    match aliases.get(subcommand) {
        Some(expansion) if depth < ALIAS_DEPTH => {
            let expanded: Vec<&str> = expansion.iter().map(String::as_str).chain(rest).collect();
            is_locked(&expanded, aliases, depth + 1)
        }
        Some(_) | None => true,
    }
}

fn logical_lines(text: &str) -> Vec<(usize, String)> {
    let mut lines: Vec<(usize, String)> = Vec::new();
    let mut continued = false;
    for (index, physical) in text.lines().enumerate() {
        let trimmed = physical.trim_end();
        let (content, continues) = match trimmed.strip_suffix(['\\', '`']) {
            Some(content) => (content, true),
            None => (trimmed, false),
        };
        match lines.last_mut() {
            Some((_, line)) if continued => {
                line.push(' ');
                line.push_str(content);
            }
            _ => lines.push((index + 1, content.to_owned())),
        }
        continued = continues;
    }
    lines
}

fn is_cargo_program(token: &str) -> bool {
    token == "cargo"
        || token == "cargo.exe"
        || token.ends_with("/cargo")
        || token.ends_with("\\cargo")
}

/// Finds the Cargo commands in `text` that resolve dependencies without `--locked`.
///
/// A command ends at `&&`, `||`, `;`, `|` or the end of its logical line; a line that ends in a
/// backslash or backtick continues on the next.
#[must_use]
pub fn check_cargo_invocations(
    text: &str,
    aliases: &BTreeMap<String, Vec<String>>,
) -> Vec<Problem> {
    let mut problems = Vec::new();
    for (line, content) in logical_lines(text) {
        let separated = content
            .replace("&&", " ; ")
            .replace("||", " ; ")
            .replace(['|', ';', '(', ')'], " ; ");
        let tokens: Vec<&str> = separated
            .split_whitespace()
            .map(|token| token.trim_matches(['"', '\'']))
            .collect();
        for command in tokens.split(|token| *token == ";") {
            let Some(start) = command.iter().position(|token| is_cargo_program(token)) else {
                continue;
            };
            let Some(invocation) = command.get(start..) else {
                continue;
            };
            let args = invocation.get(1..).unwrap_or_default();
            if !invocation_is_locked(args, aliases) {
                problems.push(Problem::CargoWithoutLocked {
                    line,
                    command: format!("cargo {}", args.join(" ")),
                });
            }
        }
    }
    problems
}

/// Reads the `[alias]` table of a `.cargo/config.toml`; a string alias is split on whitespace.
///
/// A file that is not TOML has no aliases; [`check_config`] reports it.
#[must_use]
pub fn aliases(config: &str) -> BTreeMap<String, Vec<String>> {
    let Ok(table) = config.parse::<toml::Table>() else {
        return BTreeMap::new();
    };
    let Some(aliases) = table.get("alias").and_then(toml::Value::as_table) else {
        return BTreeMap::new();
    };
    aliases
        .iter()
        .filter_map(|(name, value)| {
            let expansion = match value {
                toml::Value::String(text) => text.split_whitespace().map(str::to_owned).collect(),
                toml::Value::Array(items) => items
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_owned)
                    .collect(),
                toml::Value::Integer(_)
                | toml::Value::Float(_)
                | toml::Value::Boolean(_)
                | toml::Value::Datetime(_)
                | toml::Value::Table(_) => return None,
            };
            Some((name.clone(), expansion))
        })
        .collect()
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn is_cargo_config(path: &str) -> bool {
    path == ".cargo/config.toml"
        || path == ".cargo/config"
        || path.ends_with("/.cargo/config.toml")
        || path.ends_with("/.cargo/config")
}

/// Applies every rule to the files and returns the findings in the order of the files.
///
/// `Cargo.lock` gets the lockfile rules, a `Cargo.toml` the manifest rules, a Cargo configuration
/// the configuration rules, `rust-toolchain.toml` the toolchain rule, and every file under
/// `.github/` the section-header and `--locked` rules. All of them also get the
/// environment-variable rule.
#[must_use]
pub fn scan(files: &[SourceFile]) -> Vec<Finding> {
    let known_aliases: BTreeMap<String, Vec<String>> = files
        .iter()
        .filter(|file| is_cargo_config(&file.path))
        .flat_map(|file| aliases(&file.text))
        .collect();
    let mut findings = Vec::new();
    for file in files {
        let mut problems = Vec::new();
        if file.path == "Cargo.lock" {
            match lockfile::parse(&file.text) {
                Ok(packages) => problems.extend(check_lockfile(&packages)),
                Err(error) => problems.push(Problem::Unparseable(error.to_string())),
            }
        } else if file_name(&file.path) == "Cargo.toml" {
            problems.extend(check_manifest(&file.path, &file.text));
            problems.extend(check_environment_overrides(&file.text));
        } else if is_cargo_config(&file.path) {
            problems.extend(check_config(&file.text));
            problems.extend(check_environment_overrides(&file.text));
        } else if file_name(&file.path) == "rust-toolchain.toml" {
            problems.extend(check_toolchain(&file.text));
            problems.extend(check_environment_overrides(&file.text));
        } else if file.path.starts_with(".github/") && !file.path.ends_with(".md") {
            problems.extend(check_environment_overrides(&file.text));
            problems.extend(check_section_headers(&file.text));
            problems.extend(check_cargo_invocations(&file.text, &known_aliases));
        }
        findings.extend(problems.into_iter().map(|problem| Finding {
            file: file.path.clone(),
            problem,
        }));
    }
    findings
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        Finding, Problem, SourceFile, aliases, check_cargo_invocations, check_config,
        check_environment_overrides, check_lockfile, check_manifest, check_section_headers,
        check_toolchain, invocation_is_locked, scan,
    };
    use crate::lockfile::{CRATES_IO, LockedPackage};

    fn package(source: Option<&str>, checksum: Option<&str>) -> LockedPackage {
        LockedPackage {
            name: "evil".to_owned(),
            version: "1.0.0".to_owned(),
            source: source.map(str::to_owned),
            checksum: checksum.map(str::to_owned),
        }
    }

    fn locked_aliases() -> BTreeMap<String, Vec<String>> {
        aliases("[alias]\nxtask = \"run --package xtask --locked --quiet --\"\n")
    }

    fn file(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }

    #[test]
    fn r_sec_1__a_registry_package_with_a_checksum_passes() {
        assert_eq!(
            check_lockfile(&[package(Some(CRATES_IO), Some("abc"))]),
            Vec::new()
        );
    }

    #[test]
    fn r_sec_1__a_workspace_package_without_source_passes() {
        assert_eq!(check_lockfile(&[package(None, None)]), Vec::new());
    }

    #[test]
    fn r_sec_1__a_package_with_a_source_but_no_checksum_is_refused() {
        assert_eq!(
            check_lockfile(&[package(Some(CRATES_IO), None)]),
            [Problem::PackageWithoutChecksum("evil 1.0.0".to_owned())]
        );
    }

    #[test]
    fn r_sec_1__a_git_package_is_refused() {
        let source = "git+https://github.com/evil/evil#0123456789abcdef";
        assert_eq!(
            check_lockfile(&[package(Some(source), None)]),
            [
                Problem::PackageWithoutChecksum("evil 1.0.0".to_owned()),
                Problem::PackageFromOtherSource {
                    package: "evil 1.0.0".to_owned(),
                    source: source.to_owned()
                }
            ]
        );
    }

    #[test]
    fn r_sec_1__a_package_from_another_registry_is_refused_even_with_a_checksum() {
        let source = "registry+https://example.com/index";
        assert_eq!(
            check_lockfile(&[package(Some(source), Some("abc"))]),
            [Problem::PackageFromOtherSource {
                package: "evil 1.0.0".to_owned(),
                source: source.to_owned()
            }]
        );
    }

    #[test]
    fn r_sec_1__a_sparse_registry_source_other_than_crates_io_is_refused() {
        let source = "sparse+https://index.crates.io/";
        assert_eq!(
            check_lockfile(&[package(Some(source), Some("abc"))]).len(),
            1
        );
    }

    #[test]
    fn r_sec_1__patch_replace_source_and_registries_sections_are_refused_in_a_manifest() {
        for section in ["patch", "replace", "source", "registries"] {
            let text = format!("[package]\nname = \"a\"\n\n[{section}.crates-io]\nx = 1\n");
            assert_eq!(
                check_manifest("Cargo.toml", &text),
                [Problem::ForbiddenSection(section.to_owned())],
                "{section}"
            );
        }
    }

    #[test]
    fn r_sec_1__a_dotted_patch_key_is_refused_as_well() {
        let text = "patch.crates-io.foo = { path = \"foo\" }\n";
        assert_eq!(
            check_manifest("Cargo.toml", text),
            [Problem::ForbiddenSection("patch".to_owned())]
        );
    }

    #[test]
    fn r_sec_1__a_git_dependency_is_refused() {
        let text = "[dependencies]\nfoo = { git = \"https://github.com/x/foo\" }\n";
        assert_eq!(
            check_manifest("Cargo.toml", text),
            [Problem::ForbiddenDependencyKey {
                dependency: "foo".to_owned(),
                key: "git".to_owned()
            }]
        );
    }

    #[test]
    fn r_sec_1__a_registry_dependency_is_refused_in_every_dependency_table() {
        let text = "[workspace.dependencies]\na = { version = \"1\", registry = \"r\" }\n\
                    [dev-dependencies]\nb = { version = \"1\", registry-index = \"https://r\" }\n\
                    [build-dependencies]\nc = { version = \"1\", registry = \"r\" }\n\
                    [target.'cfg(unix)'.dependencies]\nd = { version = \"1\", registry = \"r\" }\n";
        let problems = check_manifest("Cargo.toml", text);
        let mut names: Vec<_> = problems
            .iter()
            .map(|problem| {
                let Problem::ForbiddenDependencyKey { dependency, .. } = problem else {
                    panic!("unexpected {problem:?}");
                };
                dependency.as_str()
            })
            .collect();
        names.sort_unstable();
        assert_eq!(names, ["a", "b", "c", "d"]);
    }

    #[test]
    fn r_sec_1__a_path_dependency_inside_the_workspace_passes() {
        let text = "[dependencies]\ngame = { path = \"../game\" }\n";
        assert_eq!(check_manifest("xtask/Cargo.toml", text), Vec::new());
    }

    #[test]
    fn r_sec_1__a_path_dependency_that_leaves_the_workspace_is_refused() {
        let text = "[dependencies]\nout = { path = \"../../out\" }\n";
        assert_eq!(
            check_manifest("xtask/Cargo.toml", text),
            [Problem::PathOutsideWorkspace {
                dependency: "out".to_owned(),
                path: "../../out".to_owned()
            }]
        );
    }

    #[test]
    fn r_sec_1__an_absolute_path_dependency_is_refused() {
        let text = "[dependencies]\nout = { path = \"/opt/out\" }\n";
        assert_eq!(check_manifest("Cargo.toml", text).len(), 1);
    }

    #[test]
    fn r_sec_1__a_workspace_inherited_dependency_passes() {
        let text = "[dependencies]\nsha2.workspace = true\ntoml = { workspace = true }\n";
        assert_eq!(check_manifest("xtask/Cargo.toml", text), Vec::new());
    }

    #[test]
    fn r_sec_1__a_manifest_that_is_not_toml_is_reported() {
        assert!(matches!(
            check_manifest("Cargo.toml", "[dependencies").as_slice(),
            [Problem::Unparseable(_)]
        ));
    }

    #[test]
    fn r_sec_1__a_source_replacement_is_refused_in_a_cargo_config() {
        let text = "[source.crates-io]\nreplace-with = \"mirror\"\n";
        assert_eq!(
            check_config(text),
            [Problem::ForbiddenSection("source".to_owned())]
        );
    }

    #[test]
    fn r_sec_1__registries_and_patch_are_refused_in_a_cargo_config() {
        let text =
            "[registries.mine]\nindex = \"https://r\"\n[patch.crates-io]\nx = { path = \"x\" }\n";
        assert_eq!(
            check_config(text),
            [
                Problem::ForbiddenSection("patch".to_owned()),
                Problem::ForbiddenSection("registries".to_owned())
            ]
        );
    }

    #[test]
    fn r_sec_1__a_compiler_wrapper_and_git_cli_fetching_are_refused_in_a_cargo_config() {
        let text = "[build]\nrustc-wrapper = \"w\"\nrustc-workspace-wrapper = \"w\"\n\
                    [net]\ngit-fetch-with-cli = true\n";
        assert_eq!(
            check_config(text),
            [
                Problem::ForbiddenConfigKey("build.rustc-wrapper".to_owned()),
                Problem::ForbiddenConfigKey("build.rustc-workspace-wrapper".to_owned()),
                Problem::ForbiddenConfigKey("net.git-fetch-with-cli".to_owned())
            ]
        );
    }

    #[test]
    fn r_sec_1__an_alias_only_cargo_config_passes() {
        assert_eq!(
            check_config("[alias]\nxtask = \"run --package xtask --locked --quiet --\"\n"),
            Vec::new()
        );
    }

    #[test]
    fn r_sec_1__an_alias_for_a_resolving_command_needs_locked() {
        let text = "[alias]\nxtask = \"run --package xtask --quiet --\"\n";
        assert_eq!(
            check_config(text),
            [Problem::AliasWithoutLocked("xtask".to_owned())]
        );
    }

    #[test]
    fn r_sec_3__a_toolchain_with_an_exact_version_passes() {
        assert_eq!(
            check_toolchain("[toolchain]\nchannel = \"1.99.0\"\n"),
            Vec::new()
        );
    }

    #[test]
    fn r_sec_3__a_floating_or_partial_channel_is_refused() {
        for channel in [
            "stable",
            "beta",
            "nightly",
            "1.99",
            "1",
            "1.99.x",
            "nightly-2026-10-01",
            "1.99.0-beta.1",
        ] {
            let text = format!("[toolchain]\nchannel = \"{channel}\"\n");
            assert_eq!(
                check_toolchain(&text),
                [Problem::ToolchainNotExact(Some(channel.to_owned()))],
                "{channel}"
            );
        }
    }

    #[test]
    fn r_sec_3__a_toolchain_without_a_channel_is_refused() {
        assert_eq!(
            check_toolchain("[toolchain]\nprofile = \"minimal\"\n"),
            [Problem::ToolchainNotExact(None)]
        );
    }

    #[test]
    fn r_sec_1__every_listed_environment_override_is_refused() {
        let text = "env:\n  RUSTC_WRAPPER: sccache\n  CARGO_REGISTRIES_MINE_INDEX: https://r\n  \
                    CARGO_SOURCE_CRATES_IO_REPLACE_WITH: mirror\n  CARGO_NET_GIT_FETCH_WITH_CLI: true\n";
        assert_eq!(
            check_environment_overrides(text),
            [
                Problem::ForbiddenEnvironmentOverride("RUSTC_WRAPPER".to_owned()),
                Problem::ForbiddenEnvironmentOverride("CARGO_REGISTRIES_MINE_INDEX".to_owned()),
                Problem::ForbiddenEnvironmentOverride(
                    "CARGO_SOURCE_CRATES_IO_REPLACE_WITH".to_owned()
                ),
                Problem::ForbiddenEnvironmentOverride("CARGO_NET_GIT_FETCH_WITH_CLI".to_owned())
            ]
        );
    }

    #[test]
    fn r_sec_1__the_workspace_wrapper_and_the_build_wrapper_variable_are_refused() {
        let text = "RUSTC_WORKSPACE_WRAPPER=x CARGO_BUILD_RUSTC_WRAPPER=y";
        assert_eq!(check_environment_overrides(text).len(), 2);
    }

    #[test]
    fn r_sec_1__text_without_an_override_passes() {
        assert_eq!(
            check_environment_overrides("env:\n  CARGO_TERM_COLOR: always\n"),
            Vec::new()
        );
    }

    #[test]
    fn r_sec_1__a_toml_header_for_a_forbidden_section_is_refused_in_a_workflow_file() {
        let text = "run: |\n  printf '[patch.crates-io]' > Cargo.toml\n[source.crates-io]\n  [registries.x]\n";
        assert_eq!(
            check_section_headers(text),
            [
                Problem::ForbiddenSection("patch".to_owned()),
                Problem::ForbiddenSection("source".to_owned()),
                Problem::ForbiddenSection("registries".to_owned())
            ]
        );
    }

    #[test]
    fn r_sec_1__a_yaml_list_that_mentions_source_is_not_a_header() {
        assert_eq!(
            check_section_headers("branches: [source, main]\n- [patchwork]\n"),
            Vec::new()
        );
    }

    #[test]
    fn r_sec_1__aliases_are_read_from_strings_and_arrays() {
        let found = aliases("[alias]\na = \"run --locked\"\nb = [\"test\", \"--locked\"]\n");
        assert_eq!(found["a"], ["run", "--locked"]);
        assert_eq!(found["b"], ["test", "--locked"]);
    }

    #[test]
    fn r_sec_1__a_resolving_command_needs_locked_or_frozen() {
        let none = BTreeMap::new();
        assert!(invocation_is_locked(&["build", "--locked"], &none));
        assert!(invocation_is_locked(&["test", "--frozen"], &none));
        assert!(!invocation_is_locked(&["build", "--release"], &none));
        assert!(!invocation_is_locked(&["fetch"], &none));
    }

    #[test]
    fn r_sec_1__locked_after_a_double_dash_does_not_count() {
        assert!(!invocation_is_locked(
            &["test", "--workspace", "--", "--locked"],
            &BTreeMap::new()
        ));
    }

    #[test]
    fn r_sec_1__commands_that_do_not_resolve_need_no_flag() {
        let none = BTreeMap::new();
        assert!(invocation_is_locked(&["fmt", "--all", "--check"], &none));
        assert!(invocation_is_locked(&["--version"], &none));
    }

    #[test]
    fn r_sec_1__a_toolchain_override_and_global_options_do_not_hide_the_subcommand() {
        let none = BTreeMap::new();
        assert!(!invocation_is_locked(&["+nightly", "build"], &none));
        assert!(!invocation_is_locked(
            &["-Z", "unstable-options", "test"],
            &none
        ));
        assert!(invocation_is_locked(
            &["+nightly", "build", "--locked"],
            &none
        ));
    }

    #[test]
    fn r_sec_1__an_alias_is_locked_when_its_expansion_is() {
        let mut found = locked_aliases();
        assert!(invocation_is_locked(&["xtask", "check"], &found));
        found.insert("loose".to_owned(), vec!["run".to_owned()]);
        assert!(!invocation_is_locked(&["loose"], &found));
    }

    #[test]
    fn r_sec_1__a_dropped_locked_flag_in_a_workflow_is_found_with_its_line() {
        let text = "steps:\n  - run: cargo build --locked\n  - run: cargo test --workspace\n";
        assert_eq!(
            check_cargo_invocations(text, &BTreeMap::new()),
            [Problem::CargoWithoutLocked {
                line: 3,
                command: "cargo test --workspace".to_owned()
            }]
        );
    }

    #[test]
    fn r_sec_1__every_command_of_a_chain_is_checked() {
        let text = "run: cargo fetch --locked && cargo build; cargo test --locked | tee log\n";
        assert_eq!(
            check_cargo_invocations(text, &BTreeMap::new()),
            [Problem::CargoWithoutLocked {
                line: 1,
                command: "cargo build".to_owned()
            }]
        );
    }

    #[test]
    fn r_sec_1__a_continued_command_is_read_as_one() {
        let text = "run: |\n  cargo build \\\n    --release \\\n    --locked\n";
        assert_eq!(check_cargo_invocations(text, &BTreeMap::new()), Vec::new());
    }

    #[test]
    fn r_sec_1__an_alias_call_and_cargo_fmt_pass_in_a_workflow() {
        let text = "run: cargo xtask check\nrun: cargo fmt --all --check\n";
        assert_eq!(check_cargo_invocations(text, &locked_aliases()), Vec::new());
    }

    #[test]
    fn r_sec_1__scan_applies_the_rule_set_of_each_file_kind() {
        let files = [
            file(
                "Cargo.lock",
                "[[package]]\nname = \"evil\"\nversion = \"1.0.0\"\nsource = \"git+https://x/y#1\"\n",
            ),
            file("Cargo.toml", "[patch.crates-io]\nx = { path = \"x\" }\n"),
            file("rust-toolchain.toml", "[toolchain]\nchannel = \"stable\"\n"),
            file(
                ".cargo/config.toml",
                "[source.crates-io]\nreplace-with = \"m\"\n",
            ),
            file(
                ".github/workflows/ci.yml",
                "env:\n  RUSTC_WRAPPER: w\nrun: cargo test\n",
            ),
        ];
        let findings = scan(&files);
        let by_file = |name: &str| -> usize {
            findings
                .iter()
                .filter(|finding| finding.file == name)
                .count()
        };
        assert_eq!(by_file("Cargo.lock"), 2);
        assert_eq!(by_file("Cargo.toml"), 1);
        assert_eq!(by_file("rust-toolchain.toml"), 1);
        assert_eq!(by_file(".cargo/config.toml"), 1);
        assert_eq!(by_file(".github/workflows/ci.yml"), 2);
    }

    #[test]
    fn r_sec_1__scan_reads_the_aliases_of_the_config_for_the_workflows() {
        let files = [
            file(
                ".cargo/config.toml",
                "[alias]\nxtask = \"run --package xtask --locked --quiet --\"\n",
            ),
            file(".github/workflows/ci.yml", "run: cargo xtask check\n"),
        ];
        assert_eq!(scan(&files), Vec::<Finding>::new());
    }

    #[test]
    fn r_sec_1__scan_finds_manifests_in_member_directories() {
        let files = [file(
            "tools/ocr-eval/Cargo.toml",
            "[dependencies]\nfoo = { git = \"https://x/foo\" }\n",
        )];
        let findings = scan(&files);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, "tools/ocr-eval/Cargo.toml");
    }

    #[test]
    fn r_sec_1__a_clean_set_of_files_has_no_findings() {
        let files = [
            file(
                "Cargo.lock",
                "[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n\
                 source = \"registry+https://github.com/rust-lang/crates.io-index\"\n\
                 checksum = \"abc\"\n",
            ),
            file("Cargo.toml", "[workspace]\nmembers = [\"xtask\"]\n"),
            file("rust-toolchain.toml", "[toolchain]\nchannel = \"1.99.0\"\n"),
        ];
        assert_eq!(scan(&files), Vec::<Finding>::new());
    }

    #[test]
    fn r_sec_1__a_global_locked_flag_before_the_subcommand_counts() {
        let none = BTreeMap::new();
        assert!(invocation_is_locked(&["--locked", "build"], &none));
        assert!(invocation_is_locked(&["--frozen", "test"], &none));
    }

    #[test]
    fn r_sec_1__the_value_of_a_global_option_is_not_taken_for_the_subcommand() {
        let none = BTreeMap::new();
        assert!(!invocation_is_locked(
            &["--color", "always", "build"],
            &none
        ));
        assert!(invocation_is_locked(
            &["--color", "always", "build", "--locked"],
            &none
        ));
    }

    #[test]
    fn r_sec_1__the_planned_third_party_subcommands_need_locked_too() {
        let none = BTreeMap::new();
        for subcommand in ["deny", "llvm-cov", "mutants", "nextest"] {
            assert!(!invocation_is_locked(&[subcommand], &none), "{subcommand}");
            assert!(
                invocation_is_locked(&[subcommand, "--locked"], &none),
                "{subcommand}"
            );
        }
    }

    #[test]
    fn r_sec_1__path_overrides_and_a_compiler_path_are_refused_in_a_cargo_config() {
        let text = "paths = [\"../evil\"]\n[build]\nrustc = \"/tmp/rustc\"\n";
        assert_eq!(
            check_config(text),
            [
                Problem::ForbiddenSection("paths".to_owned()),
                Problem::ForbiddenConfigKey("build.rustc".to_owned())
            ]
        );
    }

    #[test]
    fn r_sec_1__markdown_under_github_is_prose_and_not_scanned() {
        let files = [file(
            ".github/PULL_REQUEST_TEMPLATE.md",
            "- [ ] [source] added\nRUSTC_WRAPPER and cargo build in prose\n",
        )];
        assert_eq!(scan(&files), Vec::<Finding>::new());
    }
}
