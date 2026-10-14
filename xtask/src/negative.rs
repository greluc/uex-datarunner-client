// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The M0 negative tests of R-SEC-1 that need a real Cargo: a build with `--locked` against a
//! stale `Cargo.lock` fails.

use std::ffi::OsString;
use std::process::{Command, Output};

use crate::scratch::Scratch;

fn project(lock: Option<&str>) -> Scratch {
    let project = Scratch::new();
    project.write(
        "Cargo.toml",
        "[package]\nname = \"stale\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    project.write("src/lib.rs", "");
    if let Some(lock) = lock {
        project.write("Cargo.lock", lock);
    }
    project
}

fn lock_for(version: &str) -> String {
    format!("version = 4\n\n[[package]]\nname = \"stale\"\nversion = \"{version}\"\n")
}

fn build(project: &Scratch, flag: &str) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    Command::new(cargo)
        .args(["build", "--offline", flag])
        .current_dir(project.path())
        .env("CARGO_TARGET_DIR", project.path().join("target"))
        .output()
        .unwrap()
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::{build, lock_for, project};

    #[test]
    fn r_sec_1__a_locked_build_against_an_up_to_date_lockfile_succeeds() {
        let project = project(Some(&lock_for("0.1.0")));
        let output = build(&project, "--locked");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn r_sec_1__a_locked_build_against_a_stale_lockfile_fails() {
        let project = project(Some(&lock_for("0.0.9")));
        let output = build(&project, "--locked");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{stderr}");
        assert!(stderr.contains("--locked"), "{stderr}");
    }

    #[test]
    fn r_sec_1__a_locked_build_without_a_lockfile_fails() {
        let project = project(None);
        let output = build(&project, "--locked");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{stderr}");
        assert!(stderr.contains("--locked"), "{stderr}");
    }

    #[test]
    fn r_sec_1__the_same_stale_lockfile_is_silently_rewritten_without_the_flag() {
        let project = project(Some(&lock_for("0.0.9")));
        let output = build(&project, "--quiet");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
