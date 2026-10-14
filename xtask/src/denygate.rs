// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The M0 negative tests of R-SEC-10 and R-SEC-1 that need a real `cargo-deny`: a dependency under
//! a disallowed licence, from a git source or in two versions fails the gate that `cargo xtask
//! check` runs with the repository's own `deny.toml`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::scratch::Scratch;

const ROOT_LICENSE: &str = "GPL-3.0-or-later";

struct Member {
    alias: String,
    package: String,
    version: String,
    license: Option<String>,
}

impl Member {
    fn licensed(alias: &str, license: &str) -> Self {
        Self {
            alias: alias.to_owned(),
            package: alias.to_owned(),
            version: "1.0.0".to_owned(),
            license: Some(license.to_owned()),
        }
    }

    fn unlicensed(alias: &str) -> Self {
        Self {
            license: None,
            ..Self::licensed(alias, "")
        }
    }

    fn version_of(alias: &str, package: &str, version: &str) -> Self {
        Self {
            package: package.to_owned(),
            version: version.to_owned(),
            ..Self::licensed(alias, "MIT")
        }
    }

    fn manifest(&self) -> String {
        let license = self
            .license
            .as_ref()
            .map_or_else(String::new, |license| format!("license = \"{license}\"\n"));
        format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2024\"\n{license}",
            self.package, self.version
        )
    }

    fn dependency(&self) -> String {
        format!(
            "{} = {{ path = \"{}\", package = \"{}\" }}\n",
            self.alias, self.alias, self.package
        )
    }
}

fn repository_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap_or(manifest_dir).to_path_buf()
}

fn root_manifest(dependencies: &str) -> String {
    format!(
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\
         license = \"{ROOT_LICENSE}\"\npublish = false\n\n[dependencies]\n{dependencies}"
    )
}

fn project(members: &[Member]) -> Scratch {
    let project = Scratch::new();
    let dependencies: String = members.iter().map(Member::dependency).collect();
    project.write("Cargo.toml", &root_manifest(&dependencies));
    project.write("src/lib.rs", "");
    for member in members {
        project.write(&format!("{}/Cargo.toml", member.alias), &member.manifest());
        project.write(&format!("{}/src/lib.rs", member.alias), "");
    }
    let config = std::fs::read_to_string(repository_root().join("deny.toml")).unwrap();
    project.write("deny.toml", &config);
    project
}

fn run(project: &Scratch, program: &str, arguments: &[&str]) -> Output {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| program.into());
    let program = if program == "cargo" {
        cargo
    } else {
        program.into()
    };
    Command::new(program)
        .args(arguments)
        .current_dir(project.path())
        .env("CARGO_HOME", project.path().join("cargo-home"))
        .env("CARGO_TARGET_DIR", project.path().join("target"))
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn gate(project: &Scratch) -> Output {
    let lock = run(project, "cargo", &["generate-lockfile"]);
    assert!(lock.status.success(), "{}", stderr(&lock));
    run(
        project,
        "cargo",
        &["deny", "--frozen", "check", "licenses", "sources", "bans"],
    )
}

fn git_dependency(project: &Scratch) -> String {
    project.write(
        "gitdep/Cargo.toml",
        &Member::licensed("gitdep", "MIT").manifest(),
    );
    project.write("gitdep/src/lib.rs", "");
    let identity = [
        "-c",
        "user.name=fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "-c",
        "commit.gpgsign=false",
    ];
    for arguments in [
        &["init", "--quiet"][..],
        &["add", "."][..],
        &["commit", "--quiet", "-m", "fixture"][..],
    ] {
        let mut command = vec!["-C", "gitdep"];
        command.extend(identity);
        command.extend(arguments);
        let output = run(project, "git", &command);
        assert!(output.status.success(), "{}", stderr(&output));
    }
    let directory = project.path().join("gitdep");
    let url = format!(
        "file:///{}",
        directory.display().to_string().replace('\\', "/")
    );
    format!("gitdep = {{ git = \"{url}\" }}\n")
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::{Member, gate, git_dependency, project, repository_root, root_manifest, stderr};

    const ALLOWED: [&str; 16] = [
        "Apache-2.0",
        "Apache-2.0 WITH LLVM-exception",
        "MIT",
        "ISC",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "Unicode-3.0",
        "Zlib",
        "BSL-1.0",
        "CC0-1.0",
        "LGPL-2.1-only",
        "LGPL-2.1-or-later",
        "LGPL-3.0-only",
        "LGPL-3.0-or-later",
        "MPL-2.0",
        "GPL-3.0-or-later",
    ];

    const DISALLOWED: [&str; 7] = [
        "AGPL-3.0-only",
        "GPL-3.0-only",
        "GPL-2.0-only",
        "BSD-4-Clause",
        "EPL-2.0",
        "CDDL-1.0",
        "SSPL-1.0",
    ];

    fn config() -> toml::Table {
        std::fs::read_to_string(repository_root().join("deny.toml"))
            .unwrap()
            .parse()
            .unwrap()
    }

    fn string_list(table: &toml::Table, section: &str, key: &str) -> Vec<String> {
        table[section][key]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn r_sec_10__the_licence_allow_list_is_the_policy_of_the_stack_rules() {
        assert_eq!(string_list(&config(), "licenses", "allow"), ALLOWED);
    }

    #[test]
    fn r_sec_1__only_crates_io_is_an_allowed_source() {
        let config = config();
        assert_eq!(config["sources"]["unknown-registry"].as_str(), Some("deny"));
        assert_eq!(config["sources"]["unknown-git"].as_str(), Some("deny"));
        assert_eq!(
            string_list(&config, "sources", "allow-registry"),
            ["https://github.com/rust-lang/crates.io-index"]
        );
        assert!(string_list(&config, "sources", "allow-git").is_empty());
        for host in ["github", "gitlab", "bitbucket"] {
            assert!(
                config["sources"]["allow-org"][host]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                "{host}"
            );
        }
    }

    #[test]
    fn r_sec_1__wildcard_requirements_and_duplicate_versions_are_denied() {
        let config = config();
        assert_eq!(config["bans"]["wildcards"].as_str(), Some("deny"));
        assert_eq!(config["bans"]["multiple-versions"].as_str(), Some("deny"));
    }

    #[test]
    fn r_sec_10__the_gate_covers_the_windows_and_linux_targets_without_dev_dependencies() {
        let config = config();
        assert_eq!(
            string_list(&config, "graph", "targets"),
            ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]
        );
        assert_eq!(config["graph"]["exclude-dev"].as_bool(), Some(true));
        assert!(config["graph"].get("exclude").is_none());
    }

    #[test]
    fn r_sec_10__a_dependency_under_every_allowed_licence_passes_the_gate() {
        let members: Vec<_> = ALLOWED
            .iter()
            .enumerate()
            .map(|(index, license)| Member::licensed(&format!("allowed{index}"), license))
            .collect();
        let output = gate(&project(&members));
        assert!(output.status.success(), "{}", stderr(&output));
    }

    #[test]
    fn r_sec_10__a_dependency_under_a_disallowed_licence_fails_the_gate() {
        for license in DISALLOWED {
            let output = gate(&project(&[Member::licensed("culprit", license)]));
            let report = stderr(&output);
            assert!(!output.status.success(), "{license}: {report}");
            assert!(report.contains("error[rejected]"), "{license}: {report}");
            assert!(report.contains("culprit"), "{license}: {report}");
            assert!(report.contains(license), "{license}: {report}");
        }
    }

    #[test]
    fn r_sec_10__a_dependency_without_licence_information_fails_the_gate() {
        let output = gate(&project(&[Member::unlicensed("culprit")]));
        let report = stderr(&output);
        assert!(!output.status.success(), "{report}");
        assert!(report.contains("error[unlicensed]"), "{report}");
        assert!(report.contains("culprit"), "{report}");
    }

    #[test]
    fn r_sec_10__a_dual_licensed_dependency_passes_under_its_allowed_option() {
        let output = gate(&project(&[Member::licensed("dual", "MIT OR GPL-3.0-only")]));
        assert!(output.status.success(), "{}", stderr(&output));
    }

    #[test]
    fn r_sec_10__a_dependency_that_needs_a_disallowed_licence_in_addition_fails_the_gate() {
        let output = gate(&project(&[Member::licensed(
            "both",
            "MIT AND GPL-3.0-only",
        )]));
        let report = stderr(&output);
        assert!(!output.status.success(), "{report}");
        assert!(report.contains("error[rejected]"), "{report}");
        assert!(report.contains("both"), "{report}");
    }

    #[test]
    fn r_sec_1__a_git_dependency_fails_the_gate() {
        let fixture = project(&[]);
        let dependency = git_dependency(&fixture);
        fixture.write("Cargo.toml", &root_manifest(&dependency));
        let output = gate(&fixture);
        let report = stderr(&output);
        assert!(!output.status.success(), "{report}");
        assert!(report.contains("error[source-not-allowed]"), "{report}");
        assert!(report.contains("gitdep"), "{report}");
    }

    #[test]
    fn r_sec_1__a_path_dependency_inside_the_workspace_without_a_version_passes_the_gate() {
        let output = gate(&project(&[Member::licensed("sibling", "MIT")]));
        assert!(output.status.success(), "{}", stderr(&output));
    }

    #[test]
    fn r_sec_1__two_versions_of_one_crate_fail_the_gate() {
        let output = gate(&project(&[
            Member::version_of("first", "twice", "1.0.0"),
            Member::version_of("second", "twice", "2.0.0"),
        ]));
        let report = stderr(&output);
        assert!(!output.status.success(), "{report}");
        assert!(report.contains("error[duplicate]"), "{report}");
        assert!(report.contains("twice"), "{report}");
    }
}
