// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The loader refuses a library it cannot verify, and nothing is loaded afterwards.

use std::fs;
use std::path::PathBuf;

use tallyline_ort_link_mode_spike::{
    LoadError, PinnedRuntime, load, loaded_runtime_path, pinned_for_target,
};

const FAKE: PinnedRuntime = PinnedRuntime {
    file_name: "tallyline-spike-runtime.bin",
    sha256: "0000000000000000000000000000000000000000000000000000000000000000",
    git_commit: "0000000",
};

fn scratch_dir(name: &str) -> Result<PathBuf, std::io::Error> {
    let dir =
        std::env::temp_dir().join(format!("tallyline-ort-spike-{name}-{}", std::process::id()));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn a_relative_directory_is_refused() {
    let outcome = load(std::path::Path::new("lib"), &FAKE);
    assert!(
        matches!(outcome, Err(LoadError::NotAbsolute(_))),
        "{outcome:?}"
    );
}

#[test]
fn a_missing_file_is_refused() -> Result<(), std::io::Error> {
    let dir = scratch_dir("missing")?;
    let outcome = load(&dir, &FAKE);
    assert!(
        matches!(outcome, Err(LoadError::Read { .. })),
        "{outcome:?}"
    );
    fs::remove_dir_all(dir)
}

#[test]
fn a_file_with_another_hash_is_refused_before_it_is_loaded() -> Result<(), std::io::Error> {
    let pinned = pinned_for_target()
        .ok_or_else(|| std::io::Error::other("no pinned runtime for this target"))?;
    let dir = scratch_dir("tampered")?;
    fs::write(dir.join(pinned.file_name), b"not the pinned runtime")?;
    let outcome = load(&dir, pinned);
    let Err(LoadError::HashMismatch { actual, .. }) = outcome else {
        fs::remove_dir_all(&dir)?;
        return Err(std::io::Error::other(format!(
            "expected a hash mismatch, got {outcome:?}"
        )));
    };
    assert_eq!(actual.len(), 64);
    assert_ne!(actual, pinned.sha256);
    assert!(loaded_runtime_path(pinned).is_none());
    fs::remove_dir_all(dir)
}
