// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rows of the suppression register (09 §3a).

const REGISTER_HEADING: &str = "## 3a.";

/// One table row of the suppression register.
#[derive(Clone, Copy, Debug)]
pub struct Row<'a> {
    /// The whole line of the row.
    pub text: &'a str,
    /// The last cell of the row, which holds the revisit date.
    pub revisit: &'a str,
}

impl Row<'_> {
    /// Whether the last cell of the row holds an ISO date.
    #[must_use]
    pub fn dated(&self) -> bool {
        contains_iso_date(self.revisit)
    }
}

/// The table rows of `register`.
///
/// When the text has a section headed `## 3a.`, only that section is read; otherwise every line
/// is.
#[must_use]
pub fn rows(register: &str) -> Vec<Row<'_>> {
    let section: Vec<&str> = if register
        .lines()
        .any(|line| line.starts_with(REGISTER_HEADING))
    {
        register
            .lines()
            .skip_while(|line| !line.starts_with(REGISTER_HEADING))
            .enumerate()
            .take_while(|(index, line)| *index == 0 || !line.starts_with("## "))
            .map(|(_, line)| line)
            .collect()
    } else {
        register.lines().collect()
    };
    section
        .into_iter()
        .filter(|line| line.trim_start().starts_with('|'))
        .map(|line| Row {
            text: line,
            revisit: line
                .trim()
                .trim_matches('|')
                .rsplit('|')
                .next()
                .unwrap_or_default(),
        })
        .collect()
}

fn contains_iso_date(text: &str) -> bool {
    text.as_bytes().windows(10).any(|window| {
        window.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            _ => byte.is_ascii_digit(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::rows;

    #[test]
    fn a_register_without_a_section_heading_is_read_whole() {
        let found = rows("| a | b | 2026-10-14 |\n| c | d | soon |\n");
        assert_eq!(found.len(), 2);
        assert!(found[0].dated());
        assert!(!found[1].dated());
    }

    #[test]
    fn only_the_register_section_is_read_when_it_exists() {
        let text = "| outside | 2026-10-14 |\n## 3a. Register\n| inside | 2027-04-13 |\n## 3b. Next\n| after | 2027-01-01 |\n";
        let found = rows(text);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].revisit.trim(), "2027-04-13");
    }

    #[test]
    fn a_date_in_an_earlier_cell_does_not_make_a_row_dated() {
        let found = rows("| x | 2026-10-14 | when syn 2 lands |\n");
        assert!(!found[0].dated());
    }
}
