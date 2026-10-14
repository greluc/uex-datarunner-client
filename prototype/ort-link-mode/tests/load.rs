// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pinned official ONNX Runtime library loads by absolute path and runs a model.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use ort::session::Session;
use ort::value::Tensor;
use tallyline_ort_link_mode_spike::{identity_model, load, loaded_runtime_path, pinned_for_target};

fn native_files_in_temp() -> Result<BTreeSet<PathBuf>, std::io::Error> {
    let mut found = BTreeSet::new();
    for entry in fs::read_dir(std::env::temp_dir())? {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name.contains("onnxruntime") || name.ends_with(".dll") || name.contains(".so") {
            found.insert(path);
        }
    }
    Ok(found)
}

#[test]
#[ignore = "needs TALLYLINE_ORT_DIR, the directory holding the pinned official ONNX Runtime library"]
fn the_pinned_runtime_loads_from_its_absolute_path_and_runs_a_model()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::var("TALLYLINE_ORT_DIR")?);
    let pinned = pinned_for_target().ok_or("no pinned runtime for this target")?;
    let before = native_files_in_temp()?;

    let path = load(&dir, pinned)?;

    assert_eq!(path, dir.join(pinned.file_name));
    let loaded = loaded_runtime_path(pinned).ok_or("the runtime is not mapped into the process")?;
    assert_eq!(fs::canonicalize(loaded)?, fs::canonicalize(&path)?);
    assert!(
        ort::info().contains(&format!("git-commit-id={}", pinned.git_commit)),
        "{}",
        ort::info()
    );

    let mut session = Session::builder()?.commit_from_memory(&identity_model())?;
    let input = Tensor::from_array(([3_usize], vec![1.5_f32, -2.0, 3.25]))?;
    let outputs = session.run(ort::inputs!["x" => input])?;
    let (shape, values) = outputs["y"].try_extract_tensor::<f32>()?;
    assert_eq!(shape.to_vec(), vec![3_i64]);
    assert_eq!(values, [1.5_f32, -2.0, 3.25]);

    assert_eq!(native_files_in_temp()?, before);
    Ok(())
}
