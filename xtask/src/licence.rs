// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The byte comparison of `LICENSE` and `LICENSES/GPL-3.0-or-later.txt` (10 S-35).

/// The path of the licence text at the repository root.
pub const LICENSE_PATH: &str = "LICENSE";

/// The path of the copy that REUSE reads.
pub const COPY_PATH: &str = "LICENSES/GPL-3.0-or-later.txt";

/// Names how `copy` differs from `licence`, or `None` when they are byte-identical.
///
/// An empty `licence` is a difference too, because two empty files would pass a bare comparison.
#[must_use]
pub fn difference(licence: &[u8], copy: &[u8]) -> Option<String> {
    if licence.is_empty() {
        return Some(format!("{LICENSE_PATH} is empty"));
    }
    if licence == copy {
        return None;
    }
    let offset = licence
        .iter()
        .zip(copy)
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| licence.len().min(copy.len()));
    Some(format!(
        "{COPY_PATH} is not byte-identical to {LICENSE_PATH}: {} bytes against {} bytes, first difference at byte {offset}",
        copy.len(),
        licence.len()
    ))
}

#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod tests {
    use super::difference;

    #[test]
    fn r_sec_10__identical_texts_pass() {
        assert_eq!(
            difference(
                b"GNU GENERAL PUBLIC LICENSE\n",
                b"GNU GENERAL PUBLIC LICENSE\n"
            ),
            None
        );
    }

    #[test]
    fn r_sec_10__a_changed_byte_fails_and_names_its_offset() {
        let found = difference(b"abcdef", b"abcXef").unwrap();
        assert!(found.contains("first difference at byte 3"), "{found}");
    }

    #[test]
    fn r_sec_10__a_missing_final_newline_fails() {
        let found = difference(b"text\n", b"text").unwrap();
        assert!(found.contains("first difference at byte 4"), "{found}");
    }

    #[test]
    fn r_sec_10__line_ending_differences_fail() {
        assert!(difference(b"a\nb\n", b"a\r\nb\r\n").is_some());
    }

    #[test]
    fn r_sec_10__an_extra_trailing_blank_line_fails() {
        assert!(difference(b"text\n", b"text\n\n").is_some());
    }

    #[test]
    fn r_sec_10__two_empty_files_fail() {
        assert!(difference(b"", b"").is_some());
    }

    #[test]
    fn r_sec_10__an_empty_copy_fails() {
        assert!(difference(b"text\n", b"").is_some());
    }
}
