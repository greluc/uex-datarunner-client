// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding the files the supply-chain guard reads.

use std::fs;
use std::io;
use std::path::Path;

use crate::guard::SourceFile;

const SKIPPED_DIRECTORIES: [&str; 4] = ["target", ".git", ".idea", "corpus-private"];

/// Reads `Cargo.lock`, every `Cargo.toml`, every `.cargo/config.toml` (or `.cargo/config`),
/// every `rust-toolchain.toml` and every file below `.github/` under `root`.
///
/// Directories named `target`, `.git`, `.idea` and `corpus-private` are not entered. The result
/// is ordered by path.
///
/// # Errors
///
/// Returns an error when a directory or a selected file cannot be read, is not valid UTF-8 or is a
/// symbolic link, because the guard cannot vouch for what such a file contains.
pub fn collect(root: &Path) -> io::Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    walk(root, "", &mut files)?;
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn walk(dir: &Path, prefix: &str, files: &mut Vec<SourceFile>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let relative = format!("{prefix}{name}");
        let kind = entry.file_type()?;
        if kind.is_dir() {
            if !SKIPPED_DIRECTORIES.contains(&name.as_str()) {
                walk(&entry.path(), &format!("{relative}/"), files)?;
            }
        } else if is_read_by_the_guard(&relative) {
            if !kind.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{relative} is not a regular file"),
                ));
            }
            let text = fs::read_to_string(entry.path())
                .map_err(|error| io::Error::new(error.kind(), format!("{relative}: {error}")))?;
            files.push(SourceFile {
                path: relative,
                text,
            });
        }
    }
    Ok(())
}

fn is_read_by_the_guard(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    path == "Cargo.lock"
        || name == "Cargo.toml"
        || name == "rust-toolchain.toml"
        || path.starts_with(".github/")
        || path.ends_with(".cargo/config.toml")
        || path.ends_with(".cargo/config")
}

#[cfg(test)]
mod tests {
    use super::collect;
    use crate::scratch::Scratch;

    fn paths(scratch: &Scratch) -> Vec<String> {
        collect(scratch.path())
            .unwrap()
            .into_iter()
            .map(|file| file.path)
            .collect()
    }

    #[test]
    fn the_files_the_guard_reads_are_found_at_any_depth_with_slash_paths() {
        let scratch = Scratch::new();
        scratch.write("Cargo.lock", "version = 4\n");
        scratch.write("Cargo.toml", "[workspace]\n");
        scratch.write("rust-toolchain.toml", "[toolchain]\n");
        scratch.write(".cargo/config.toml", "[alias]\n");
        scratch.write("tools/ocr-eval/Cargo.toml", "[package]\n");
        scratch.write("tools/ocr-eval/.cargo/config.toml", "[alias]\n");
        scratch.write(".github/workflows/ci.yml", "name: CI\n");
        scratch.write(".github/requirements/zizmor.txt", "zizmor==1\n");
        assert_eq!(
            paths(&scratch),
            [
                ".cargo/config.toml",
                ".github/requirements/zizmor.txt",
                ".github/workflows/ci.yml",
                "Cargo.lock",
                "Cargo.toml",
                "rust-toolchain.toml",
                "tools/ocr-eval/.cargo/config.toml",
                "tools/ocr-eval/Cargo.toml",
            ]
        );
    }

    #[test]
    fn other_files_are_not_read() {
        let scratch = Scratch::new();
        scratch.write("README.md", "# readme\n");
        scratch.write("xtask/src/main.rs", "fn main() {}\n");
        scratch.write("docs/config.toml", "x = 1\n");
        assert_eq!(paths(&scratch), Vec::<String>::new());
    }

    #[test]
    fn build_output_and_version_control_are_not_entered() {
        let scratch = Scratch::new();
        scratch.write("target/debug/build/x/Cargo.toml", "[package]\n");
        scratch.write(".git/hooks/Cargo.toml", "[package]\n");
        scratch.write("xtask/Cargo.toml", "[package]\n");
        assert_eq!(paths(&scratch), ["xtask/Cargo.toml"]);
    }

    #[test]
    fn the_text_of_a_file_is_read_whole() {
        let scratch = Scratch::new();
        scratch.write("Cargo.toml", "[workspace]\nmembers = []\n");
        let files = collect(scratch.path()).unwrap();
        assert_eq!(files[0].text, "[workspace]\nmembers = []\n");
    }

    #[test]
    fn a_missing_root_is_an_error() {
        let scratch = Scratch::new();
        assert!(collect(&scratch.path().join("absent")).is_err());
    }

    #[test]
    fn a_selected_file_that_is_not_utf8_is_an_error_and_not_skipped() {
        let scratch = Scratch::new();
        std::fs::write(scratch.path().join("Cargo.toml"), [0xff_u8, 0xfe, 0x00]).unwrap();
        let error = collect(scratch.path()).unwrap_err();
        assert!(error.to_string().contains("Cargo.toml"), "{error}");
    }
}
