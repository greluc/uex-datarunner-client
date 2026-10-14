// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The `cargo xtask` entry point: parses the subcommand and runs the steps of `xtask::check`.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use xtask::check::{Step, plan};
use xtask::deny::unjustified_suppressions;
use xtask::dependabot;
use xtask::exit::exit_code;
use xtask::guard::scan;
use xtask::listing::check_listing;
use xtask::osv::{judge, shipped_packages, shipped_roots};
use xtask::registry::Registry;
use xtask::sources::collect;
use xtask::vendor::{actual_list, regenerate, verify};
use xtask::vet::violations;

const REQUIREMENTS: &str = "docs/plan/01-requirements.md";
const ROADMAP: &str = "docs/plan/04-roadmap.md";
const ENGINEERING_PRINCIPLES: &str = "docs/plan/09-engineering-principles.md";
const DENY_CONFIG: &str = "deny.toml";
const VET_CONFIG: &str = "supply-chain/config.toml";
const VET_BASELINE: &str = "supply-chain/bootstrap-exemptions.txt";
const LOCKFILE: &str = "Cargo.lock";
const HASH_LIST: &str = "supply-chain/vendor-hashes.txt";
const VENDOR_WORK_DIR: &str = "target/xtask-vendor";
const SHIPPED_TARGETS: [&str; 2] = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"];

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let root = workspace_root();
    let outcome = match arguments
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["check"] => run_check(&root),
        ["guard"] => run_guard(&root),
        ["osv-gate", report] => run_osv_gate(&root, Path::new(report)),
        ["vendor-hashes", "write"] => write_hash_list(&root),
        ["vendor-hashes", "verify"] => verify_hash_list(&root),
        _ => usage(),
    };
    exit_code(outcome, &mut io::stderr())
}

fn usage() -> io::Result<ExitCode> {
    writeln!(
        io::stderr(),
        "usage: cargo xtask check | guard | osv-gate <report.json> | vendor-hashes write | vendor-hashes verify"
    )?;
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
        Step::SupplyChainGuard => guard_passes(root),
        Step::DenySuppressions => deny_suppressions_pass(root),
        Step::VetExemptions => vet_exemptions_pass(root),
        Step::Dependabot => dependabot_passes(root),
        Step::Cargo { args, env, .. } => Ok(Command::new(cargo_program())
            .args(args)
            .envs(env.iter().copied())
            .current_dir(root)
            .status()?
            .success()),
        Step::RequirementTags { list } => check_requirement_tags(list, root),
    }
}

fn run_guard(root: &Path) -> io::Result<ExitCode> {
    Ok(exit_status(guard_passes(root)?))
}

fn cargo_stdout(root: &Path, args: &[&str]) -> io::Result<String> {
    let output = Command::new(cargo_program())
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        io::stderr().write_all(&output.stderr)?;
        return Err(io::Error::other(format!("cargo {} failed", args.join(" "))));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn shipped_graph(root: &Path) -> io::Result<(Vec<String>, BTreeSet<xtask::osv::PackageId>)> {
    let metadata = cargo_stdout(
        root,
        &["metadata", "--locked", "--no-deps", "--format-version", "1"],
    )?;
    let roots = shipped_roots(&metadata).map_err(io::Error::other)?;
    let mut shipped = BTreeSet::new();
    for name in &roots {
        for target in SHIPPED_TARGETS {
            let tree = cargo_stdout(
                root,
                &[
                    "tree",
                    "--locked",
                    "--package",
                    name,
                    "--target",
                    target,
                    "--edges",
                    "normal",
                    "--all-features",
                    "--prefix",
                    "none",
                    "--format",
                    "{p}",
                ],
            )?;
            shipped.extend(shipped_packages(&tree));
        }
    }
    Ok((roots, shipped))
}

fn run_osv_gate(root: &Path, report: &Path) -> io::Result<ExitCode> {
    let (roots, shipped) = shipped_graph(root)?;
    let verdict = judge(&fs::read_to_string(report)?, &shipped).map_err(io::Error::other)?;
    writeln!(
        io::stderr(),
        "shipped packages: {} in {} workspace members ({})",
        shipped.len(),
        roots.len(),
        if roots.is_empty() {
            "none yet".to_owned()
        } else {
            roots.join(", ")
        }
    )?;
    for finding in &verdict.reported {
        writeln!(io::stderr(), "reported: {finding}")?;
    }
    for finding in &verdict.blocking {
        writeln!(io::stderr(), "BLOCKING: {finding}")?;
    }
    Ok(exit_status(verdict.blocking.is_empty()))
}

fn exit_status(passed: bool) -> ExitCode {
    if passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn guard_passes(root: &Path) -> io::Result<bool> {
    let findings = scan(&collect(root)?);
    for finding in &findings {
        writeln!(io::stderr(), "{}: {}", finding.file, finding.problem)?;
    }
    Ok(findings.is_empty())
}

fn deny_suppressions_pass(root: &Path) -> io::Result<bool> {
    let findings = unjustified_suppressions(
        &fs::read_to_string(root.join(DENY_CONFIG))?,
        &fs::read_to_string(root.join(ENGINEERING_PRINCIPLES))?,
    );
    for finding in &findings {
        writeln!(io::stderr(), "{finding}")?;
    }
    Ok(findings.is_empty())
}

fn dependabot_passes(root: &Path) -> io::Result<bool> {
    let findings = dependabot::violations(&collect(root)?);
    for finding in &findings {
        writeln!(io::stderr(), "{finding}")?;
    }
    Ok(findings.is_empty())
}

fn vet_exemptions_pass(root: &Path) -> io::Result<bool> {
    let findings = violations(
        &fs::read_to_string(root.join(VET_CONFIG))?,
        &fs::read_to_string(root.join(VET_BASELINE))?,
        &fs::read_to_string(root.join(ENGINEERING_PRINCIPLES))?,
    );
    for finding in &findings {
        writeln!(io::stderr(), "{finding}")?;
    }
    Ok(findings.is_empty())
}

fn with_vendor_tree<T>(
    root: &Path,
    action: impl FnOnce(&str, &Path) -> io::Result<T>,
) -> io::Result<T> {
    let lock_text = fs::read_to_string(root.join(LOCKFILE))?;
    let work_dir = root.join(VENDOR_WORK_DIR);
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir)?;
    }
    fs::create_dir_all(&work_dir)?;
    let outcome = regenerate(root, &cargo_program(), &work_dir)
        .and_then(|vendor_dir| action(&lock_text, &vendor_dir));
    fs::remove_dir_all(&work_dir).ok();
    outcome
}

fn write_hash_list(root: &Path) -> io::Result<ExitCode> {
    let list = with_vendor_tree(root, |lock_text, vendor_dir| {
        actual_list(lock_text, vendor_dir).map_err(io::Error::other)
    })?;
    let target = root.join(HASH_LIST);
    if let Some(directory) = target.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(&target, list.render())?;
    writeln!(
        io::stderr(),
        "wrote {HASH_LIST}: {} packages",
        list.entries().len()
    )?;
    Ok(ExitCode::SUCCESS)
}

fn verify_hash_list(root: &Path) -> io::Result<ExitCode> {
    let list_text = fs::read_to_string(root.join(HASH_LIST))?;
    let differences = with_vendor_tree(root, |lock_text, vendor_dir| {
        verify(lock_text, &list_text, vendor_dir).map_err(io::Error::other)
    })?;
    for difference in &differences {
        writeln!(io::stderr(), "{HASH_LIST}: {difference}")?;
    }
    if differences.is_empty() {
        writeln!(io::stderr(), "{HASH_LIST} matches the vendored tree")?;
    }
    Ok(exit_status(differences.is_empty()))
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
