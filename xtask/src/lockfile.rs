// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The packages recorded in a `Cargo.lock`.

use std::fmt;

/// The `source` Cargo records for a package from crates.io.
pub const CRATES_IO: &str = "registry+https://github.com/rust-lang/crates.io-index";

/// One `[[package]]` entry of a lockfile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockedPackage {
    /// The package name.
    pub name: String,
    /// The resolved version, including any build metadata.
    pub version: String,
    /// Where the package comes from; absent for the workspace's own packages.
    pub source: Option<String>,
    /// The SHA-256 of the package archive as lowercase hexadecimal; absent for packages without a registry source.
    pub checksum: Option<String>,
}

impl LockedPackage {
    /// The name of the directory `cargo vendor --versioned-dirs` writes this package to.
    #[must_use]
    pub fn directory_name(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }
}

/// Why a lockfile cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LockfileError {
    /// The text is not valid TOML.
    Syntax(String),
    /// A `[[package]]` entry lacks a string field the lockfile format guarantees.
    MissingField {
        /// The zero-based position of the entry.
        index: usize,
        /// The missing field.
        field: &'static str,
    },
}

impl fmt::Display for LockfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax(message) => write!(f, "Cargo.lock is not valid TOML: {message}"),
            Self::MissingField { index, field } => {
                write!(f, "Cargo.lock package #{index} has no `{field}`")
            }
        }
    }
}

/// Reads every `[[package]]` entry of `text`.
///
/// # Errors
///
/// Returns a [`LockfileError`] when the text is not TOML or an entry has no name or version.
pub fn parse(text: &str) -> Result<Vec<LockedPackage>, LockfileError> {
    let table: toml::Table = text
        .parse()
        .map_err(|error: toml::de::Error| LockfileError::Syntax(error.message().to_owned()))?;
    let Some(packages) = table.get("package").and_then(toml::Value::as_array) else {
        return Ok(Vec::new());
    };
    packages
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let field = |field: &'static str| entry.get(field).and_then(toml::Value::as_str);
            let required = |name: &'static str| {
                field(name)
                    .map(str::to_owned)
                    .ok_or(LockfileError::MissingField { index, field: name })
            };
            Ok(LockedPackage {
                name: required("name")?,
                version: required("version")?,
                source: field("source").map(str::to_owned),
                checksum: field("checksum").map(str::to_owned),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{CRATES_IO, LockedPackage, LockfileError, parse};

    const LOCK: &str = r#"
version = 4

[[package]]
name = "serde_core"
version = "1.0.229"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "abc123"

[[package]]
name = "xtask"
version = "0.0.0"
dependencies = ["serde_core"]
"#;

    #[test]
    fn a_registry_package_carries_its_source_and_checksum() {
        let packages = parse(LOCK).unwrap();
        assert_eq!(
            packages[0],
            LockedPackage {
                name: "serde_core".to_owned(),
                version: "1.0.229".to_owned(),
                source: Some(CRATES_IO.to_owned()),
                checksum: Some("abc123".to_owned()),
            }
        );
    }

    #[test]
    fn a_workspace_package_has_neither_source_nor_checksum() {
        let packages = parse(LOCK).unwrap();
        assert_eq!(packages[1].source, None);
        assert_eq!(packages[1].checksum, None);
    }

    #[test]
    fn a_lockfile_without_packages_is_empty() {
        assert_eq!(parse("version = 4\n").unwrap(), Vec::new());
    }

    #[test]
    fn text_that_is_not_toml_is_a_syntax_error() {
        assert!(matches!(parse("[[package"), Err(LockfileError::Syntax(_))));
    }

    #[test]
    fn an_entry_without_a_version_is_reported_with_its_position() {
        let text = "[[package]]\nname = \"a\"\nversion = \"1.0.0\"\n[[package]]\nname = \"b\"\n";
        assert_eq!(
            parse(text),
            Err(LockfileError::MissingField {
                index: 1,
                field: "version"
            })
        );
    }

    #[test]
    fn the_vendor_directory_is_the_name_and_version_joined_by_a_hyphen() {
        let package = LockedPackage {
            name: "toml".to_owned(),
            version: "1.1.6+spec-1.1.0".to_owned(),
            source: None,
            checksum: None,
        };
        assert_eq!(package.directory_name(), "toml-1.1.6+spec-1.1.0");
    }
}
