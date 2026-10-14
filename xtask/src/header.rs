// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The licence-header rule of 10 S-35 for Rust sources.

use crate::guard::SourceFile;

const COPYRIGHT_TAG: &str = concat!("// SPDX-File", "CopyrightText: ");
const LICENCE_TAG: &str = concat!("// SPDX-License-", "Identifier: ");
const YEAR_DIGITS: usize = 4;

/// Names the Rust files of `files` whose first lines are not an SPDX licence header.
///
/// Every file ending in `.rs` must start with one or more SPDX copyright-text lines
/// (`// <tag>: <year> <holder>`, `<year>` being `YYYY` or `YYYY-YYYY`), followed directly by one
/// SPDX licence-identifier line (`// <tag>: <expression>`). Other files are not looked at;
/// `reuse lint` and `REUSE.toml` cover them.
#[must_use]
pub fn violations(files: &[SourceFile]) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.path.ends_with(".rs"))
        .filter_map(|file| problem(&file.text).map(|problem| format!("{}: {problem}", file.path)))
        .collect()
}

fn problem(text: &str) -> Option<String> {
    let mut copyright_lines = 0_usize;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if let Some(rest) = line.strip_prefix(COPYRIGHT_TAG) {
            if !is_year_and_holder(rest) {
                return Some(format!(
                    "line {number} is not `{COPYRIGHT_TAG}<year> <holder>` with a year such as 2026 or 2026-2027"
                ));
            }
            copyright_lines += 1;
        } else if copyright_lines == 0 {
            return Some(format!(
                "line {number} is not `{COPYRIGHT_TAG}<year> <holder>`; a Rust file starts with its licence header"
            ));
        } else if let Some(expression) = line.strip_prefix(LICENCE_TAG) {
            return (!is_licence_expression(expression)).then(|| {
                format!(
                    "line {number} does not hold an SPDX licence expression after `{LICENCE_TAG}`"
                )
            });
        } else {
            return Some(missing_licence_line(number, copyright_lines));
        }
    }
    Some(if copyright_lines == 0 {
        format!("the file is empty; a Rust file starts with `{COPYRIGHT_TAG}<year> <holder>`")
    } else {
        missing_licence_line(copyright_lines + 1, copyright_lines)
    })
}

fn missing_licence_line(number: usize, copyright_lines: usize) -> String {
    format!(
        "line {number} should be `{LICENCE_TAG}<expression>` directly after the {copyright_lines} copyright line(s)"
    )
}

fn is_year_and_holder(rest: &str) -> bool {
    let Some((years, holder)) = rest.split_once(' ') else {
        return false;
    };
    !holder.trim().is_empty() && is_years(years)
}

fn is_years(years: &str) -> bool {
    let is_year =
        |text: &str| text.len() == YEAR_DIGITS && text.bytes().all(|b| b.is_ascii_digit());
    match years.split_once('-') {
        Some((first, last)) => is_year(first) && is_year(last),
        None => is_year(years),
    }
}

fn is_licence_expression(expression: &str) -> bool {
    let spaced = expression.replace('(', " ( ").replace(')', " ) ");
    let mut depth = 0_usize;
    let mut expect_operand = true;
    let mut after_with = false;
    for token in spaced.split_whitespace() {
        match token {
            "(" if expect_operand && !after_with => depth += 1,
            ")" if !expect_operand && depth > 0 => depth -= 1,
            "AND" | "OR" if !expect_operand => expect_operand = true,
            "WITH" if !expect_operand => {
                expect_operand = true;
                after_with = true;
            }
            identifier if expect_operand && is_identifier(identifier) => {
                expect_operand = false;
                after_with = false;
            }
            _ => return false,
        }
    }
    depth == 0 && !expect_operand
}

fn is_identifier(token: &str) -> bool {
    !matches!(token, "(" | ")" | "AND" | "OR" | "WITH")
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-' | ':'))
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::{COPYRIGHT_TAG, LICENCE_TAG, violations};
    use crate::guard::SourceFile;

    fn rust(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_owned(),
            text: text.to_owned(),
        }
    }

    fn copyright(rest: &str) -> String {
        format!("{COPYRIGHT_TAG}{rest}")
    }

    fn licence(expression: &str) -> String {
        format!("{LICENCE_TAG}{expression}")
    }

    fn header(copyright_lines: &[&str], expression: &str) -> String {
        let mut lines: Vec<String> = copyright_lines.iter().map(|rest| copyright(rest)).collect();
        lines.push(licence(expression));
        format!("{}\n", lines.join("\n"))
    }

    fn good() -> String {
        format!(
            "{}\n//! Docs.\n",
            header(&["2026 Lucas Greuloch"], "GPL-3.0-or-later")
        )
    }

    #[test]
    fn r_sec_10__a_rust_file_with_the_header_passes() {
        assert_eq!(
            violations(&[rust("a/src/lib.rs", &good())]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn r_sec_10__a_rust_file_without_the_header_fails_naming_the_file() {
        let found = violations(&[rust("a/src/lib.rs", "//! Docs.\n")]);
        assert_eq!(found.len(), 1);
        assert!(found[0].starts_with("a/src/lib.rs: line 1 "), "{found:?}");
    }

    #[test]
    fn r_sec_10__an_empty_rust_file_fails() {
        assert_eq!(violations(&[rust("build.rs", "")]).len(), 1);
    }

    #[test]
    fn r_sec_10__a_header_without_the_licence_line_fails() {
        let text = format!("{}\n\n//! Docs.\n", copyright("2026 Lucas Greuloch"));
        let found = violations(&[rust("x.rs", &text)]);
        assert!(found[0].contains("line 2 should be"), "{found:?}");
    }

    #[test]
    fn r_sec_10__a_header_that_ends_after_the_copyright_line_fails() {
        let found = violations(&[rust("x.rs", &copyright("2026 Lucas Greuloch"))]);
        assert!(found[0].contains("line 2 should be"), "{found:?}");
    }

    #[test]
    fn r_sec_10__the_licence_line_before_the_copyright_line_fails() {
        let text = format!(
            "{}\n{}\n",
            licence("GPL-3.0-or-later"),
            copyright("2026 Lucas Greuloch")
        );
        assert_eq!(violations(&[rust("x.rs", &text)]).len(), 1);
    }

    #[test]
    fn r_sec_10__several_copyright_lines_before_the_licence_line_pass() {
        let text = header(
            &["2026 Lucas Greuloch", "2026-2027 Another Person"],
            "GPL-3.0-or-later",
        );
        assert_eq!(violations(&[rust("x.rs", &text)]), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_copyright_line_without_a_year_or_a_holder_fails() {
        for rest in [
            "Lucas Greuloch",
            "2026",
            "2026 ",
            "26 Lucas Greuloch",
            "2026- Lucas Greuloch",
        ] {
            let text = header(&[rest], "GPL-3.0-or-later");
            assert_eq!(violations(&[rust("x.rs", &text)]).len(), 1, "{rest}");
        }
    }

    #[test]
    fn r_sec_10__a_copyright_tag_without_the_space_after_the_colon_fails() {
        let text = format!(
            "{}2026 Lucas Greuloch\n{}\n",
            COPYRIGHT_TAG.trim_end(),
            licence("GPL-3.0-or-later")
        );
        assert_eq!(violations(&[rust("x.rs", &text)]).len(), 1);
    }

    #[test]
    fn r_sec_10__an_empty_or_malformed_licence_expression_fails() {
        for expression in [
            "",
            "   ",
            "GPL-3.0-or-later; rm",
            "$$",
            "all rights reserved",
            "GPL-3.0-or-later MIT",
            "MIT OR",
            "OR MIT",
            "MIT AND OR Apache-2.0",
            "(MIT",
            "MIT)",
            "()",
            "(MIT) (Apache-2.0)",
            "MIT WITH",
            "MIT WITH (LLVM-exception)",
        ] {
            let text = header(&["2026 Lucas Greuloch"], expression);
            assert_eq!(
                violations(&[rust("x.rs", &text)]).len(),
                1,
                "{expression:?}"
            );
        }
    }

    #[test]
    fn r_sec_10__a_compound_licence_expression_passes() {
        let text = header(
            &["2026 Lucas Greuloch"],
            "(MIT OR Apache-2.0) AND LicenseRef-X",
        );
        assert_eq!(violations(&[rust("x.rs", &text)]), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_licence_expression_with_an_exception_or_a_plus_passes() {
        for expression in [
            "Apache-2.0 WITH LLVM-exception",
            "GPL-2.0+",
            "MIT",
            "((MIT OR Apache-2.0) AND (GPL-3.0-or-later OR LicenseRef-X))",
        ] {
            let text = header(&["2026 Lucas Greuloch"], expression);
            assert_eq!(
                violations(&[rust("x.rs", &text)]),
                Vec::<String>::new(),
                "{expression}"
            );
        }
    }

    #[test]
    fn r_sec_10__a_header_with_windows_line_endings_passes() {
        let text = good().replace('\n', "\r\n");
        assert_eq!(violations(&[rust("x.rs", &text)]), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__a_byte_order_mark_before_the_header_fails() {
        let text = format!("\u{feff}{}", good());
        assert_eq!(violations(&[rust("x.rs", &text)]).len(), 1);
    }

    #[test]
    fn r_sec_10__a_plain_comment_instead_of_the_tag_fails() {
        let text = format!(
            "// Copyright 2026 Lucas Greuloch\n{}\n",
            licence("GPL-3.0-or-later")
        );
        assert_eq!(violations(&[rust("x.rs", &text)]).len(), 1);
    }

    #[test]
    fn r_sec_10__files_that_are_not_rust_are_not_looked_at() {
        let files = [
            rust("README.md", "# readme\n"),
            rust("a/Cargo.toml", "[package]\n"),
            rust("a/rs", "no extension\n"),
        ];
        assert_eq!(violations(&files), Vec::<String>::new());
    }

    #[test]
    fn r_sec_10__every_failing_file_is_reported_in_input_order() {
        let found = violations(&[
            rust("b.rs", "fn main() {}\n"),
            rust("a.rs", &good()),
            rust("c.rs", "fn main() {}\n"),
        ]);
        assert_eq!(found.len(), 2);
        assert!(found[0].starts_with("b.rs: "));
        assert!(found[1].starts_with("c.rs: "));
    }
}
