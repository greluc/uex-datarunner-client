// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The mapping from the outcome of a subcommand to the process exit code.

use std::io::{self, Write};
use std::process::ExitCode;

/// Returns the exit code for `outcome`, reporting an error on `stderr` before it fails.
///
/// A failure to write the report is ignored, because there is no other channel left.
pub fn exit_code(outcome: io::Result<ExitCode>, stderr: &mut impl Write) -> ExitCode {
    match outcome {
        Ok(code) => code,
        Err(error) => {
            writeln!(stderr, "xtask: {error}").ok();
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, ErrorKind, Write};
    use std::process::ExitCode;

    use super::exit_code;

    struct BrokenPipe;

    impl Write for BrokenPipe {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_successful_outcome_keeps_its_code_and_writes_nothing() {
        let mut stderr = Vec::new();
        let code = exit_code(Ok(ExitCode::SUCCESS), &mut stderr);
        assert_eq!(code, ExitCode::SUCCESS);
        assert!(stderr.is_empty());
    }

    #[test]
    fn a_failed_outcome_keeps_its_code() {
        let mut stderr = Vec::new();
        let code = exit_code(Ok(ExitCode::FAILURE), &mut stderr);
        assert_eq!(code, ExitCode::FAILURE);
    }

    #[test]
    fn an_error_is_reported_on_stderr_and_fails() {
        let mut stderr = Vec::new();
        let error = io::Error::new(
            ErrorKind::NotFound,
            "docs/plan/01-requirements.md is missing",
        );
        let code = exit_code(Err(error), &mut stderr);
        assert_eq!(code, ExitCode::FAILURE);
        let report = String::from_utf8_lossy(&stderr);
        assert!(
            report.contains("docs/plan/01-requirements.md is missing"),
            "{report}"
        );
    }

    #[test]
    fn an_error_still_fails_when_the_report_cannot_be_written() {
        let code = exit_code(Err(io::Error::from(ErrorKind::NotFound)), &mut BrokenPipe);
        assert_eq!(code, ExitCode::FAILURE);
    }
}
