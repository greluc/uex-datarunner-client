// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! A throwaway directory for tests that need real files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::tracked::git_command;

const MAX_ATTEMPTS: u32 = 10_000;

/// A fresh directory below the system temporary directory, removed when dropped.
pub(crate) struct Scratch {
    root: PathBuf,
}

impl Scratch {
    /// Creates an empty directory.
    pub(crate) fn new() -> Self {
        let base = std::env::temp_dir();
        let mut attempt = 0_u32;
        loop {
            let root = base.join(format!("xtask-test-{}-{attempt}", std::process::id()));
            match fs::create_dir(&root) {
                Ok(()) => return Self { root },
                Err(error)
                    if attempt < MAX_ATTEMPTS
                        && matches!(
                            error.kind(),
                            std::io::ErrorKind::AlreadyExists
                                | std::io::ErrorKind::PermissionDenied
                        ) =>
                {
                    attempt += 1;
                }
                Err(error) => panic!("cannot create {}: {error}", root.display()),
            }
        }
    }

    /// The directory.
    pub(crate) fn path(&self) -> &Path {
        &self.root
    }

    /// Runs `git` with `args` in the directory as Ada Lovelace, ignoring the `GIT_*` variables
    /// that a Git hook exports, and returns the exit success and standard output.
    pub(crate) fn git(&self, args: &[&str]) -> (bool, String) {
        let output = git_command(&self.root)
            .args([
                "-c",
                "user.name=Ada Lovelace",
                "-c",
                "user.email=ada@example.org",
            ])
            .args(["-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .unwrap();
        (
            output.status.success(),
            String::from_utf8(output.stdout).unwrap(),
        )
    }

    /// Writes `contents` to `relative` (with `/` separators), creating the directories above it.
    pub(crate) fn write(&self, relative: &str, contents: &str) {
        let target = self.root.join(relative);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(target, contents).unwrap();
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).ok();
    }
}
