// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The committed list of per-package hashes of the vendored dependency tree.

use std::collections::BTreeMap;
use std::fmt;

use sha2::{Digest, Sha256};

use crate::lockfile::LockedPackage;

const HEADER: &str = "vendor-hashes 1";

/// The hashes recorded for one vendored package.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    /// The package name.
    pub name: String,
    /// The package version, including any build metadata.
    pub version: String,
    /// The SHA-256 of the package archive, as recorded in `Cargo.lock`.
    pub checksum: String,
    /// The hash of the vendored files, see [`tree_hash`].
    pub tree: String,
}

impl Entry {
    fn package(&self) -> String {
        format!("{} {}", self.name, self.version)
    }
}

/// The entries of the hash list, ordered by name and version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HashList {
    entries: Vec<Entry>,
}

/// Why a hash list cannot be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ListError {
    /// The first line is not the expected format header.
    Header,
    /// A line does not have the four fields `name version checksum tree`, or a hash is not
    /// 64 lowercase hexadecimal digits.
    Line(usize),
    /// Two lines name the same package version.
    Duplicate(String),
}

impl fmt::Display for ListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Header => write!(f, "the hash list does not start with `{HEADER}`"),
            Self::Line(number) => write!(f, "hash list line {number} is malformed"),
            Self::Duplicate(package) => write!(f, "hash list names {package} twice"),
        }
    }
}

impl HashList {
    /// Builds a list from `entries`, ordering them by name and version.
    #[must_use]
    pub fn new(mut entries: Vec<Entry>) -> Self {
        entries.sort();
        Self { entries }
    }

    /// The entries in list order.
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The file text: a header line, then one `name version checksum tree` line per entry.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text = format!("{HEADER}\n");
        for entry in &self.entries {
            text.push_str(&format!(
                "{} {} {} {}\n",
                entry.name, entry.version, entry.checksum, entry.tree
            ));
        }
        text
    }

    /// Reads the file text written by [`HashList::render`].
    ///
    /// # Errors
    ///
    /// Returns a [`ListError`] for a missing header, a malformed line or a repeated package.
    pub fn parse(text: &str) -> Result<Self, ListError> {
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return Err(ListError::Header);
        }
        let mut entries: Vec<Entry> = Vec::new();
        for (index, line) in lines.enumerate() {
            let number = index + 2;
            let fields: Vec<&str> = line.split(' ').collect();
            let [name, version, checksum, tree] = fields[..] else {
                return Err(ListError::Line(number));
            };
            if name.is_empty() || version.is_empty() || !is_digest(checksum) || !is_digest(tree) {
                return Err(ListError::Line(number));
            }
            let entry = Entry {
                name: name.to_owned(),
                version: version.to_owned(),
                checksum: checksum.to_owned(),
                tree: tree.to_owned(),
            };
            if entries
                .iter()
                .any(|known| known.package() == entry.package())
            {
                return Err(ListError::Duplicate(entry.package()));
            }
            entries.push(entry);
        }
        Ok(Self::new(entries))
    }
}

fn is_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// A way the vendored tree departs from the committed list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Difference {
    /// The list names a package that the tree does not contain.
    Missing {
        /// The package as `name version`.
        package: String,
    },
    /// The tree contains a package that the list does not name.
    Unexpected {
        /// The package as `name version`.
        package: String,
    },
    /// The archive checksum differs.
    Checksum {
        /// The package as `name version`.
        package: String,
    },
    /// The vendored files differ.
    Tree {
        /// The package as `name version`.
        package: String,
    },
}

impl fmt::Display for Difference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { package } => write!(f, "{package}: in the hash list, not vendored"),
            Self::Unexpected { package } => write!(f, "{package}: vendored, not in the hash list"),
            Self::Checksum { package } => write!(f, "{package}: archive checksum differs"),
            Self::Tree { package } => write!(f, "{package}: vendored files differ"),
        }
    }
}

/// Lists every difference between the `committed` list and the `actual` one.
#[must_use]
pub fn compare(committed: &HashList, actual: &HashList) -> Vec<Difference> {
    let index = |list: &HashList| -> BTreeMap<String, Entry> {
        list.entries
            .iter()
            .map(|entry| (entry.package(), entry.clone()))
            .collect()
    };
    let committed = index(committed);
    let actual = index(actual);
    let mut differences = Vec::new();
    for (package, expected) in &committed {
        match actual.get(package) {
            None => differences.push(Difference::Missing {
                package: package.clone(),
            }),
            Some(found) => {
                if found.checksum != expected.checksum {
                    differences.push(Difference::Checksum {
                        package: package.clone(),
                    });
                }
                if found.tree != expected.tree {
                    differences.push(Difference::Tree {
                        package: package.clone(),
                    });
                }
            }
        }
    }
    for package in actual
        .keys()
        .filter(|package| !committed.contains_key(*package))
    {
        differences.push(Difference::Unexpected {
            package: package.clone(),
        });
    }
    differences
}

/// Why a hash list cannot be built from a lockfile and a vendored tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// A package with a source has no checksum in the lockfile.
    NoChecksum(String),
    /// The vendored tree has no directory for a package with a source.
    NotVendored(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoChecksum(package) => write!(f, "{package} has a source but no checksum"),
            Self::NotVendored(package) => write!(f, "{package} was not vendored"),
        }
    }
}

/// Builds the list for the packages of a lockfile from the hashes of the vendored directories.
///
/// `trees` maps a directory name (see [`LockedPackage::directory_name`]) to its [`tree_hash`].
/// Packages without a source belong to the workspace and are not vendored.
///
/// # Errors
///
/// Returns a [`BuildError`] for a package without checksum or without a vendored directory.
pub fn build(
    packages: &[LockedPackage],
    trees: &BTreeMap<String, String>,
) -> Result<HashList, BuildError> {
    let mut entries = Vec::new();
    for package in packages.iter().filter(|package| package.source.is_some()) {
        let label = format!("{} {}", package.name, package.version);
        let checksum = package
            .checksum
            .clone()
            .ok_or_else(|| BuildError::NoChecksum(label.clone()))?;
        let tree = trees
            .get(&package.directory_name())
            .cloned()
            .ok_or(BuildError::NotVendored(label))?;
        entries.push(Entry {
            name: package.name.clone(),
            version: package.version.clone(),
            checksum,
            tree,
        });
    }
    Ok(HashList::new(entries))
}

/// The lowercase hexadecimal SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The hash of a vendored package directory.
///
/// `files` pairs the `/`-separated path of every file below the package directory with the
/// [`sha256_hex`] of its contents. The result does not depend on their order.
#[must_use]
pub fn tree_hash(files: &[(String, String)]) -> String {
    let mut lines: Vec<String> = files
        .iter()
        .map(|(path, digest)| format!("{digest}  {path}\n"))
        .collect();
    lines.sort();
    sha256_hex(lines.concat().as_bytes())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        BuildError, Difference, Entry, HashList, ListError, build, compare, sha256_hex, tree_hash,
    };
    use crate::lockfile::{CRATES_IO, LockedPackage};

    fn digest(seed: char) -> String {
        seed.to_string().repeat(64)
    }

    fn entry(name: &str, version: &str, seed: char) -> Entry {
        Entry {
            name: name.to_owned(),
            version: version.to_owned(),
            checksum: digest(seed),
            tree: digest('0'),
        }
    }

    fn registry(name: &str, version: &str, checksum: Option<&str>) -> LockedPackage {
        LockedPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            source: Some(CRATES_IO.to_owned()),
            checksum: checksum.map(str::to_owned),
        }
    }

    #[test]
    fn sha256_of_the_empty_input_is_the_known_digest() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn the_tree_hash_does_not_depend_on_file_order() {
        let a = ("src/lib.rs".to_owned(), sha256_hex(b"lib"));
        let b = ("Cargo.toml".to_owned(), sha256_hex(b"manifest"));
        assert_eq!(tree_hash(&[a.clone(), b.clone()]), tree_hash(&[b, a]));
    }

    #[test]
    fn the_tree_hash_changes_with_a_file_content() {
        let before = [("src/lib.rs".to_owned(), sha256_hex(b"lib"))];
        let after = [("src/lib.rs".to_owned(), sha256_hex(b"lib!"))];
        assert_ne!(tree_hash(&before), tree_hash(&after));
    }

    #[test]
    fn the_tree_hash_changes_with_a_file_path() {
        let before = [("src/lib.rs".to_owned(), sha256_hex(b"lib"))];
        let after = [("src/main.rs".to_owned(), sha256_hex(b"lib"))];
        assert_ne!(tree_hash(&before), tree_hash(&after));
    }

    #[test]
    fn the_tree_hash_is_a_sha256_digest() {
        let hash = tree_hash(&[("a".to_owned(), sha256_hex(b"a"))]);
        assert_eq!(hash.len(), 64);
        assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn a_rendered_list_is_read_back_unchanged() {
        let list = HashList::new(vec![entry("b", "2.0.0", 'b'), entry("a", "1.0.0", 'a')]);
        assert_eq!(HashList::parse(&list.render()), Ok(list));
    }

    #[test]
    fn entries_are_ordered_by_name_then_version() {
        let list = HashList::new(vec![
            entry("b", "1.0.0", 'b'),
            entry("a", "2.0.0", 'a'),
            entry("a", "1.0.0", 'c'),
        ]);
        let keys: Vec<_> = list
            .entries()
            .iter()
            .map(|e| format!("{} {}", e.name, e.version))
            .collect();
        assert_eq!(keys, ["a 1.0.0", "a 2.0.0", "b 1.0.0"]);
    }

    #[test]
    fn a_list_without_the_header_is_refused() {
        assert_eq!(HashList::parse("a 1.0.0 x y\n"), Err(ListError::Header));
    }

    #[test]
    fn a_line_with_a_short_digest_is_refused_with_its_line_number() {
        let text = format!("vendor-hashes 1\na 1.0.0 {} abc\n", digest('a'));
        assert_eq!(HashList::parse(&text), Err(ListError::Line(2)));
    }

    #[test]
    fn a_line_with_an_uppercase_digest_is_refused() {
        let text = format!("vendor-hashes 1\na 1.0.0 {} {}\n", digest('a'), digest('A'));
        assert_eq!(HashList::parse(&text), Err(ListError::Line(2)));
    }

    #[test]
    fn a_package_listed_twice_is_refused() {
        let line = format!("a 1.0.0 {} {}\n", digest('a'), digest('b'));
        let text = format!("vendor-hashes 1\n{line}{line}");
        assert_eq!(
            HashList::parse(&text),
            Err(ListError::Duplicate("a 1.0.0".to_owned()))
        );
    }

    #[test]
    fn identical_lists_have_no_difference() {
        let list = HashList::new(vec![entry("a", "1.0.0", 'a')]);
        assert_eq!(compare(&list, &list.clone()), Vec::new());
    }

    #[test]
    fn a_changed_tree_hash_is_reported_as_a_tree_difference() {
        let committed = HashList::new(vec![entry("a", "1.0.0", 'a')]);
        let mut changed = entry("a", "1.0.0", 'a');
        changed.tree = digest('f');
        assert_eq!(
            compare(&committed, &HashList::new(vec![changed])),
            [Difference::Tree {
                package: "a 1.0.0".to_owned()
            }]
        );
    }

    #[test]
    fn a_changed_archive_checksum_is_reported_as_a_checksum_difference() {
        let committed = HashList::new(vec![entry("a", "1.0.0", 'a')]);
        let mut changed = entry("a", "1.0.0", 'a');
        changed.checksum = digest('f');
        assert_eq!(
            compare(&committed, &HashList::new(vec![changed])),
            [Difference::Checksum {
                package: "a 1.0.0".to_owned()
            }]
        );
    }

    #[test]
    fn a_package_only_in_one_list_is_missing_or_unexpected() {
        let committed = HashList::new(vec![entry("a", "1.0.0", 'a')]);
        let actual = HashList::new(vec![entry("b", "1.0.0", 'b')]);
        assert_eq!(
            compare(&committed, &actual),
            [
                Difference::Missing {
                    package: "a 1.0.0".to_owned()
                },
                Difference::Unexpected {
                    package: "b 1.0.0".to_owned()
                }
            ]
        );
    }

    #[test]
    fn a_new_version_of_a_listed_package_is_a_missing_and_an_unexpected_package() {
        let committed = HashList::new(vec![entry("a", "1.0.0", 'a')]);
        let actual = HashList::new(vec![entry("a", "1.0.1", 'a')]);
        assert_eq!(compare(&committed, &actual).len(), 2);
    }

    #[test]
    fn the_list_is_built_from_registry_packages_and_their_vendored_trees() {
        let packages = [
            registry("a", "1.0.0", Some(&digest('a'))),
            LockedPackage {
                name: "xtask".to_owned(),
                version: "0.0.0".to_owned(),
                source: None,
                checksum: None,
            },
        ];
        let trees = BTreeMap::from([("a-1.0.0".to_owned(), digest('b'))]);
        let list = build(&packages, &trees).unwrap();
        assert_eq!(
            list.entries(),
            [Entry {
                name: "a".to_owned(),
                version: "1.0.0".to_owned(),
                checksum: digest('a'),
                tree: digest('b'),
            }]
        );
    }

    #[test]
    fn a_registry_package_without_a_checksum_cannot_be_listed() {
        let packages = [registry("a", "1.0.0", None)];
        let trees = BTreeMap::from([("a-1.0.0".to_owned(), digest('b'))]);
        assert_eq!(
            build(&packages, &trees),
            Err(BuildError::NoChecksum("a 1.0.0".to_owned()))
        );
    }

    #[test]
    fn a_registry_package_without_a_vendored_directory_cannot_be_listed() {
        let packages = [registry("a", "1.0.0", Some(&digest('a')))];
        assert_eq!(
            build(&packages, &BTreeMap::new()),
            Err(BuildError::NotVendored("a 1.0.0".to_owned()))
        );
    }
}
