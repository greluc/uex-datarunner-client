// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `cargo xtask` entry point: parses the subcommand and runs the steps of `xtask::check`.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use xtask::check::{Step, plan};
use xtask::exit::exit_code;
use xtask::listing::check_listing;
use xtask::registry::Registry;

const REQUIREMENTS: &str = "docs/plan/01-requirements.md";
const ROADMAP: &str = "docs/plan/04-roadmap.md";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["check"] => run_check(&workspace_root()),
        _ => usage(),
    };
    exit_code(outcome, &mut io::stderr())
}

fn usage() -> io::Result<ExitCode> {
    writeln!(io::stderr(), "usage: cargo xtask check")?;
    Ok(ExitCode::FAILURE)
}

fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(manifest_dir).to_path_buf()
}

fn cargo_program() -> OsString {
    std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn run_check(root: &Path) -> io::Result<ExitCode> {
    let mut failed = Vec::new();
    for step in plan() {
        writeln!(io::stderr(), "==> {}", step.name())?;
        if !run_step(&step, root)? {
            failed.push(step.name());
        }
    }
    if failed.is_empty() {
        writeln!(io::stderr(), "check passed")?;
        Ok(ExitCode::SUCCESS)
    } else {
        writeln!(io::stderr(), "check failed: {}", failed.join(", "))?;
        Ok(ExitCode::FAILURE)
    }
}

fn run_step(step: &Step, root: &Path) -> io::Result<bool> {
    match step {
        Step::Cargo { args, env, .. } => Ok(Command::new(cargo_program())
            .args(args)
            .envs(env.iter().copied())
            .current_dir(root)
            .status()?
            .success()),
        Step::RequirementTags { list } => check_requirement_tags(list, root),
    }
}

fn check_requirement_tags(list: &[String], root: &Path) -> io::Result<bool> {
    let listing = Command::new(cargo_program())
        .args(list)
        .current_dir(root)
        .output()?;
    if !listing.status.success() {
        io::stderr().write_all(&listing.stderr)?;
        return Ok(false);
    }
    let registry = Registry::from_documents(
        &fs::read_to_string(root.join(REQUIREMENTS))?,
        &fs::read_to_string(root.join(ROADMAP))?,
    );
    let violations = check_listing(&String::from_utf8_lossy(&listing.stdout), &registry);
    for violation in &violations {
        writeln!(io::stderr(), "{}: {:?}", violation.test, violation.problem)?;
    }
    Ok(violations.is_empty())
}
