// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The steps of `cargo xtask check`.

/// One step of the check; a step fails the check when it reports any finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// The supply-chain guard of 10 S-4 over the repository files; it runs before any Cargo call.
    SupplyChainGuard,
    /// The justification rule of 10 S-6 over the suppressions in `deny.toml`.
    DenySuppressions,
    /// The bootstrap rule of 10 S-6 over the `cargo-vet` exemptions in `supply-chain/`.
    VetExemptions,
    /// The Dependabot rules of 10 S-7 and S-8 over `.github/`.
    Dependabot,
    /// A Cargo invocation that must exit successfully.
    Cargo {
        /// The name shown in the report.
        name: &'static str,
        /// The arguments after `cargo`.
        args: Vec<String>,
        /// The environment variables set for the invocation.
        env: Vec<(&'static str, &'static str)>,
    },
    /// The requirement-tag check over the test listing printed by the `list` invocation.
    RequirementTags {
        /// The arguments after `cargo` that print the test listing.
        list: Vec<String>,
    },
}

impl Step {
    /// The name shown in the report.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::SupplyChainGuard => "supply-chain guard",
            Self::DenySuppressions => "cargo-deny suppressions",
            Self::VetExemptions => "cargo-vet exemptions",
            Self::Dependabot => "dependabot",
            Self::Cargo { name, .. } => name,
            Self::RequirementTags { .. } => "requirement tags",
        }
    }
}

fn cargo(name: &'static str, args: &[&str]) -> Step {
    Step::Cargo {
        name,
        args: args.iter().map(|&arg| arg.to_owned()).collect(),
        env: Vec::new(),
    }
}

/// The steps of `cargo xtask check` in the order they run.
#[must_use]
pub fn plan() -> Vec<Step> {
    vec![
        Step::SupplyChainGuard,
        cargo(
            "cargo-deny",
            &["deny", "--locked", "check", "licenses", "sources", "bans"],
        ),
        Step::DenySuppressions,
        Step::VetExemptions,
        Step::Dependabot,
        cargo("format", &["fmt", "--all", "--check"]),
        cargo(
            "clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        Step::Cargo {
            name: "rustdoc",
            args: ["doc", "--workspace", "--no-deps", "--locked"]
                .map(str::to_owned)
                .to_vec(),
            env: vec![("RUSTDOCFLAGS", "-D warnings")],
        },
        cargo("tests", &["test", "--workspace", "--locked"]),
        Step::RequirementTags {
            list: ["test", "--workspace", "--locked", "--", "--list"]
                .map(str::to_owned)
                .to_vec(),
        },
    ]
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use std::collections::BTreeMap;

    use super::{Step, plan};
    use crate::guard::invocation_is_locked;

    fn cargo_args(step: &Step) -> Vec<&str> {
        match step {
            Step::Cargo { args, .. } => args.iter().map(String::as_str).collect(),
            Step::RequirementTags { list } => list.iter().map(String::as_str).collect(),
            Step::SupplyChainGuard
            | Step::DenySuppressions
            | Step::VetExemptions
            | Step::Dependabot => Vec::new(),
        }
    }

    #[test]
    fn rustdoc_runs_with_warnings_denied_through_the_environment() {
        let plan = plan();
        assert_eq!(
            plan[7],
            Step::Cargo {
                name: "rustdoc",
                args: ["doc", "--workspace", "--no-deps", "--locked"]
                    .map(str::to_owned)
                    .to_vec(),
                env: vec![("RUSTDOCFLAGS", "-D warnings")],
            }
        );
    }

    #[test]
    fn the_plan_runs_the_supply_chain_steps_first_then_format_clippy_rustdoc_tests_and_tags() {
        let plan = plan();
        let names: Vec<_> = plan.iter().map(Step::name).collect();
        assert_eq!(
            names,
            [
                "supply-chain guard",
                "cargo-deny",
                "cargo-deny suppressions",
                "cargo-vet exemptions",
                "dependabot",
                "format",
                "clippy",
                "rustdoc",
                "tests",
                "requirement tags"
            ]
        );
        assert_eq!(
            cargo_args(&plan[1]),
            ["deny", "--locked", "check", "licenses", "sources", "bans"]
        );
        assert_eq!(cargo_args(&plan[5]), ["fmt", "--all", "--check"]);
        assert_eq!(
            cargo_args(&plan[6]),
            [
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings"
            ]
        );
        assert_eq!(cargo_args(&plan[8]), ["test", "--workspace", "--locked"]);
        assert_eq!(
            cargo_args(&plan[9]),
            ["test", "--workspace", "--locked", "--", "--list"]
        );
    }

    #[test]
    fn r_sec_1__every_cargo_command_of_the_plan_that_resolves_dependencies_passes_locked() {
        for step in plan() {
            let args = cargo_args(&step);
            assert!(
                invocation_is_locked(&args, &BTreeMap::new()),
                "{}: cargo {}",
                step.name(),
                args.join(" ")
            );
        }
    }
}
