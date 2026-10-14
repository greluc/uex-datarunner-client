// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The DCO 1.1 sign-off check over the commits of a pull request (ADR-0003).

/// The `--format` argument of `git log` that [`parse_log`] reads.
pub const LOG_FORMAT: &str = "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%B%x1e";

const FIELD_SEPARATOR: char = '\u{1f}';
const RECORD_SEPARATOR: char = '\u{1e}';
const SIGN_OFF_KEY: &str = "signed-off-by:";
const NOREPLY_DOMAIN: &str = "users.noreply.github.com";

/// One commit as `git log` printed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    /// The full object name.
    pub id: String,
    /// How many parents the commit has; a merge commit has more than one.
    pub parents: usize,
    /// The author name as recorded in the commit.
    pub author_name: String,
    /// The author e-mail address as recorded in the commit.
    pub author_email: String,
    /// The whole commit message.
    pub message: String,
}

/// Reads the output of `git log` run with [`LOG_FORMAT`].
///
/// # Errors
///
/// Returns a description of the first record that does not have the five fields of the format.
pub fn parse_log(output: &str) -> Result<Vec<Commit>, String> {
    let mut commits = Vec::new();
    for record in output.split(RECORD_SEPARATOR) {
        let record = record.trim_start_matches(['\n', '\r']);
        if record.is_empty() {
            continue;
        }
        let fields: Vec<&str> = record.splitn(5, FIELD_SEPARATOR).collect();
        let [id, parents, author_name, author_email, message] = fields.as_slice() else {
            return Err(format!(
                "the git log record {:?} does not have five fields",
                record.chars().take(48).collect::<String>()
            ));
        };
        commits.push(Commit {
            id: (*id).to_owned(),
            parents: parents.split_whitespace().count(),
            author_name: (*author_name).to_owned(),
            author_email: (*author_email).to_owned(),
            message: (*message).to_owned(),
        });
    }
    Ok(commits)
}

/// Names the commits of `commits` that carry no sign-off matching their author.
///
/// A commit passes when the last paragraph of its message, the trailer block, has a
/// `Signed-off-by: Name <address>` line starting in the first column whose name equals
/// the author name and whose address equals the author address, ignoring case; a GitHub noreply
/// address with and without its numeric prefix counts as the same address. Merge commits are not
/// checked.
#[must_use]
pub fn violations(commits: &[Commit]) -> Vec<String> {
    commits
        .iter()
        .filter(|commit| commit.parents < 2)
        .filter_map(|commit| {
            let sign_offs = sign_offs(&commit.message);
            if sign_offs.is_empty() {
                return Some(format!(
                    "{}: no `Signed-off-by` line; sign off with `git commit -s`",
                    describe(commit)
                ));
            }
            let matches_author = sign_offs.iter().any(|(name, email)| {
                *name == commit.author_name && same_address(email, &commit.author_email)
            });
            (!matches_author).then(|| {
                let found: Vec<String> = sign_offs
                    .iter()
                    .map(|(name, email)| format!("{name} <{email}>"))
                    .collect();
                format!(
                    "{}: the sign-off ({}) does not match the author {} <{}>",
                    describe(commit),
                    found.join(", "),
                    commit.author_name,
                    commit.author_email
                )
            })
        })
        .collect()
}

fn describe(commit: &Commit) -> String {
    let short: String = commit.id.chars().take(10).collect();
    let subject = commit.message.lines().next().unwrap_or_default().trim();
    format!("{short} ({subject})")
}

fn last_paragraph(message: &str) -> Vec<&str> {
    let lines: Vec<&str> = message.trim_end().lines().collect();
    let start = lines
        .iter()
        .rposition(|line| line.trim().is_empty())
        .map_or(0, |blank| blank + 1);
    lines.get(start..).unwrap_or_default().to_vec()
}

fn sign_offs(message: &str) -> Vec<(&str, &str)> {
    last_paragraph(message)
        .into_iter()
        .filter_map(|line| {
            let line = line.trim_end();
            let key_length = SIGN_OFF_KEY.len();
            let key = line.get(..key_length)?;
            if !key.eq_ignore_ascii_case(SIGN_OFF_KEY) {
                return None;
            }
            let value = line.get(key_length..)?.trim();
            let (name, rest) = value.rsplit_once('<')?;
            let email = rest.strip_suffix('>')?;
            let name = name.trim();
            (!name.is_empty() && !email.is_empty()).then_some((name, email))
        })
        .collect()
}

fn same_address(left: &str, right: &str) -> bool {
    canonical_address(left) == canonical_address(right)
}

fn canonical_address(address: &str) -> String {
    let address = address.trim().to_ascii_lowercase();
    match address.split_once('@') {
        Some((local, domain)) if domain == NOREPLY_DOMAIN => {
            let handle = match local.split_once('+') {
                Some((digits, handle))
                    if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    handle
                }
                Some(_) | None => local,
            };
            format!("{handle}@{domain}")
        }
        Some(_) | None => address,
    }
}

#[cfg(test)]
mod tests {
    use super::{Commit, LOG_FORMAT, parse_log, violations};
    use crate::scratch::Scratch;

    fn commit(author_name: &str, author_email: &str, message: &str) -> Commit {
        Commit {
            id: "0123456789abcdef".to_owned(),
            parents: 1,
            author_name: author_name.to_owned(),
            author_email: author_email.to_owned(),
            message: message.to_owned(),
        }
    }

    fn ada(message: &str) -> Commit {
        commit("Ada Lovelace", "ada@example.org", message)
    }

    #[test]
    fn a_matching_sign_off_passes() {
        let message = "feat(x): add y\n\nBody.\n\nSigned-off-by: Ada Lovelace <ada@example.org>\n";
        assert_eq!(violations(&[ada(message)]), Vec::<String>::new());
    }

    #[test]
    fn a_commit_without_a_sign_off_fails_and_names_commit_and_subject() {
        let found = violations(&[ada("feat(x): add y\n\nBody.\n")]);
        assert_eq!(found.len(), 1);
        assert!(
            found[0].starts_with("0123456789 (feat(x): add y): no `Signed-off-by`"),
            "{found:?}"
        );
    }

    #[test]
    fn a_sign_off_quoted_in_the_body_or_indented_does_not_count() {
        for message in [
            "revert: x\n\nSigned-off-by: Ada Lovelace <ada@example.org>\n\nThis reverts the commit.\n",
            "m\n\n    Signed-off-by: Ada Lovelace <ada@example.org>\n",
            "m\n\n> Signed-off-by: Ada Lovelace <ada@example.org>\n",
        ] {
            assert_eq!(violations(&[ada(message)]).len(), 1, "{message:?}");
        }
    }

    #[test]
    fn a_sign_off_in_the_trailer_block_among_other_trailers_passes() {
        let message = "m\n\nBody.\n\nSigned-off-by: Ada Lovelace <ada@example.org>\nCo-Authored-By: Grace <g@example.org>\n";
        assert_eq!(violations(&[ada(message)]), Vec::<String>::new());
    }

    #[test]
    fn a_sign_off_without_a_blank_line_before_it_passes_in_a_single_paragraph() {
        let message = "m\nSigned-off-by: Ada Lovelace <ada@example.org>";
        assert_eq!(violations(&[ada(message)]), Vec::<String>::new());
    }

    #[test]
    fn a_sign_off_with_another_address_fails() {
        let found = violations(&[ada(
            "m\n\nSigned-off-by: Ada Lovelace <other@example.org>\n",
        )]);
        assert!(
            found[0].contains("does not match the author Ada Lovelace <ada@example.org>"),
            "{found:?}"
        );
    }

    #[test]
    fn a_sign_off_with_another_name_fails() {
        let found = violations(&[ada("m\n\nSigned-off-by: Grace Hopper <ada@example.org>\n")]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn a_misspelt_address_domain_fails() {
        let author = commit(
            "Ada Lovelace",
            "ada@gmail.com",
            "m\n\nSigned-off-by: Ada Lovelace <ada@gmial.com>\n",
        );
        assert_eq!(violations(&[author]).len(), 1);
    }

    #[test]
    fn the_address_is_compared_ignoring_case() {
        let found = violations(&[ada("m\n\nSigned-off-by: Ada Lovelace <ADA@Example.ORG>\n")]);
        assert_eq!(found, Vec::<String>::new());
    }

    #[test]
    fn the_trailer_key_is_compared_ignoring_case() {
        let found = violations(&[ada("m\n\nsigned-off-by: Ada Lovelace <ada@example.org>\n")]);
        assert_eq!(found, Vec::<String>::new());
    }

    #[test]
    fn one_matching_sign_off_among_several_passes() {
        let message = "m\n\nSigned-off-by: Grace Hopper <grace@example.org>\nSigned-off-by: Ada Lovelace <ada@example.org>\n";
        assert_eq!(violations(&[ada(message)]), Vec::<String>::new());
    }

    #[test]
    fn a_github_noreply_address_matches_with_or_without_its_numeric_prefix() {
        let author = commit(
            "Ada Lovelace",
            "12345+ada@users.noreply.github.com",
            "m\n\nSigned-off-by: Ada Lovelace <ada@users.noreply.github.com>\n",
        );
        assert_eq!(violations(&[author]), Vec::<String>::new());
        let author = commit(
            "Ada Lovelace",
            "ada@users.noreply.github.com",
            "m\n\nSigned-off-by: Ada Lovelace <12345+ada@users.noreply.github.com>\n",
        );
        assert_eq!(violations(&[author]), Vec::<String>::new());
    }

    #[test]
    fn a_noreply_address_of_another_handle_fails() {
        let author = commit(
            "Ada Lovelace",
            "12345+ada@users.noreply.github.com",
            "m\n\nSigned-off-by: Ada Lovelace <67890+grace@users.noreply.github.com>\n",
        );
        assert_eq!(violations(&[author]).len(), 1);
    }

    #[test]
    fn a_plus_tag_on_an_ordinary_address_is_not_stripped() {
        let author = commit(
            "Ada Lovelace",
            "ada+work@example.org",
            "m\n\nSigned-off-by: Ada Lovelace <ada@example.org>\n",
        );
        assert_eq!(violations(&[author]).len(), 1);
    }

    #[test]
    fn a_sign_off_without_an_address_or_name_does_not_count() {
        for line in [
            "Signed-off-by: Ada Lovelace",
            "Signed-off-by: Ada Lovelace <>",
            "Signed-off-by: <ada@example.org>",
            "Signed-off-by:",
        ] {
            assert_eq!(
                violations(&[ada(&format!("m\n\n{line}\n"))]).len(),
                1,
                "{line}"
            );
        }
    }

    #[test]
    fn a_merge_commit_is_not_checked() {
        let mut merge = ada("Merge branch 'main' into feature\n");
        merge.parents = 2;
        assert_eq!(violations(&[merge]), Vec::<String>::new());
    }

    #[test]
    fn every_failing_commit_is_reported() {
        let found = violations(&[
            ada("one\n"),
            ada("two\n\nSigned-off-by: Ada Lovelace <ada@example.org>\n"),
            ada("three\n"),
        ]);
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn the_log_is_parsed_into_commits() {
        let output = "aaa\u{1f}bbb\u{1f}Ada Lovelace\u{1f}ada@example.org\u{1f}one\n\nbody\n\u{1e}\nccc\u{1f}ddd eee\u{1f}Grace Hopper\u{1f}grace@example.org\u{1f}two\n\u{1e}\n";
        let commits = parse_log(output).unwrap();
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].id, "aaa");
        assert_eq!(commits[0].parents, 1);
        assert_eq!(commits[0].author_name, "Ada Lovelace");
        assert_eq!(commits[0].message, "one\n\nbody\n");
        assert_eq!(commits[1].id, "ccc");
        assert_eq!(commits[1].parents, 2);
        assert_eq!(commits[1].author_email, "grace@example.org");
    }

    #[test]
    fn a_root_commit_has_no_parents() {
        let commits = parse_log("aaa\u{1f}\u{1f}Ada\u{1f}a@b.c\u{1f}m\n\u{1e}\n").unwrap();
        assert_eq!(commits[0].parents, 0);
    }

    #[test]
    fn a_message_may_contain_the_field_separator() {
        let commits =
            parse_log("aaa\u{1f}\u{1f}Ada\u{1f}a@b.c\u{1f}one\u{1f}two\n\u{1e}\n").unwrap();
        assert_eq!(commits[0].message, "one\u{1f}two\n");
    }

    #[test]
    fn a_real_git_log_is_parsed_and_only_the_unsigned_commit_fails() {
        let scratch = Scratch::new();
        let git = |args: &[&str]| {
            let (success, output) = scratch.git(args);
            assert!(success, "git {args:?}");
            output
        };
        git(&["init", "--quiet"]);
        git(&["commit", "--quiet", "--allow-empty", "-m", "base"]);
        git(&[
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "unsigned change",
        ]);
        git(&[
            "commit",
            "--quiet",
            "--allow-empty",
            "-s",
            "-m",
            "signed change",
        ]);
        let commits = parse_log(&git(&["log", LOG_FORMAT, "HEAD~2..HEAD"])).unwrap();
        assert_eq!(commits.len(), 2);
        assert!(commits.iter().all(|commit| commit.parents == 1));
        let found = violations(&commits);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("(unsigned change)"), "{found:?}");
    }

    #[test]
    fn an_empty_log_has_no_commits() {
        assert_eq!(parse_log("").unwrap(), Vec::<Commit>::new());
    }

    #[test]
    fn a_record_with_too_few_fields_is_an_error() {
        assert!(parse_log("aaa\u{1f}bbb\u{1e}").is_err());
    }
}
