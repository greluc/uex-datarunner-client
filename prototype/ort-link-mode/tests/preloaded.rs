// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! A runtime that `ort` loaded from another path before the verified load is detected, not silently accepted.

use std::fs;
use std::path::PathBuf;

use tallyline_ort_link_mode_spike::{LoadError, load, pinned_for_target};

#[test]
#[ignore = "needs TALLYLINE_ORT_DIR, the directory holding the pinned official ONNX Runtime library"]
fn a_runtime_loaded_from_another_path_first_is_refused() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::var("TALLYLINE_ORT_DIR")?);
    let pinned = pinned_for_target().ok_or("no pinned runtime for this target")?;
    let elsewhere = std::env::temp_dir().join(format!(
        "tallyline-ort-spike-elsewhere-{}",
        std::process::id()
    ));
    fs::create_dir_all(&elsewhere)?;
    fs::copy(dir.join(pinned.file_name), elsewhere.join(pinned.file_name))?;

    let _ = ort::init_from(elsewhere.join(pinned.file_name))?.commit();
    let outcome = load(&dir, pinned);

    assert!(
        matches!(outcome, Err(LoadError::LoadedElsewhere { .. })),
        "{outcome:?}"
    );
    Ok(())
}
