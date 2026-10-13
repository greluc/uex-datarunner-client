// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The steps of `cargo xtask check`.

/// One step of the check; a step fails the check when it reports any finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
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
mod tests {
    use super::{Step, plan};

    fn cargo_args(step: &Step) -> Vec<&str> {
        match step {
            Step::Cargo { args, .. } => args.iter().map(String::as_str).collect(),
            Step::RequirementTags { list } => list.iter().map(String::as_str).collect(),
        }
    }

    #[test]
    fn rustdoc_runs_with_warnings_denied_through_the_environment() {
        let plan = plan();
        assert_eq!(
            plan[2],
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
    fn the_plan_checks_format_clippy_rustdoc_tests_and_requirement_tags_in_that_order() {
        let plan = plan();
        let names: Vec<_> = plan.iter().map(Step::name).collect();
        assert_eq!(
            names,
            ["format", "clippy", "rustdoc", "tests", "requirement tags"]
        );
        assert_eq!(cargo_args(&plan[0]), ["fmt", "--all", "--check"]);
        assert_eq!(
            cargo_args(&plan[1]),
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
        assert_eq!(cargo_args(&plan[3]), ["test", "--workspace", "--locked"]);
        assert_eq!(
            cargo_args(&plan[4]),
            ["test", "--workspace", "--locked", "--", "--list"]
        );
    }
}
