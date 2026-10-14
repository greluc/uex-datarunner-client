// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The probe runs both sessions in every variant, and only the `env` variant sets the opt-out variable.

use std::process::Command;

const ORT_DIR: &str = "TALLYLINE_ORT_DIR";

fn run_probe(variant: &str) -> Result<String, Box<dyn std::error::Error>> {
    let dir = std::env::var(ORT_DIR)?;
    let output = Command::new(env!("CARGO_BIN_EXE_ort-telemetry-probe"))
        .args([
            "--variant",
            variant,
            "--ort-dir",
            &dir,
            "--hold",
            "0",
            "--linger",
            "0",
        ])
        .env_remove("ORT_DISABLE_TELEMETRY")
        .output()?;
    let stdout = String::from_utf8(output.stdout)?;
    if !output.status.success() {
        return Err(format!(
            "probe exited with {}: {stdout}{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(stdout)
}

fn assert_both_sessions_ran(stdout: &str) {
    assert!(stdout.contains("session=1 inference=ok"), "{stdout}");
    assert!(stdout.contains("session=2 inference=ok"), "{stdout}");
    assert!(stdout.lines().any(|line| line == "done"), "{stdout}");
}

#[test]
#[ignore = "needs TALLYLINE_ORT_DIR; loads ONNX Runtime with its telemetry on"]
fn no_mitigation_runs_and_leaves_the_variable_unset() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = run_probe("none")?;
    assert_both_sessions_ran(&stdout);
    assert!(stdout.contains("ORT_DISABLE_TELEMETRY=<unset>"), "{stdout}");
    Ok(())
}

#[test]
#[ignore = "needs TALLYLINE_ORT_DIR; loads ONNX Runtime with its telemetry events off"]
fn api_off_runs_and_leaves_the_variable_unset() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = run_probe("api")?;
    assert_both_sessions_ran(&stdout);
    assert!(stdout.contains("ORT_DISABLE_TELEMETRY=<unset>"), "{stdout}");
    Ok(())
}

#[test]
#[ignore = "needs TALLYLINE_ORT_DIR, the directory holding the pinned official ONNX Runtime library"]
fn environment_and_api_off_runs_with_the_variable_set() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = run_probe("env")?;
    assert_both_sessions_ran(&stdout);
    assert!(stdout.contains("ORT_DISABLE_TELEMETRY=1"), "{stdout}");
    Ok(())
}
