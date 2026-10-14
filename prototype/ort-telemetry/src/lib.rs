// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spike for O-13: measure the telemetry of the official ONNX Runtime library with and without the R-NF-12 mitigations.

pub mod model;
pub mod runtime;
pub mod strace;
pub mod variant;
