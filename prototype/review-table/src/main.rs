// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Starts the review-table spike window.

use std::process::ExitCode;

use tallyline_review_table_spike::app::{Options, SpikeApp};

const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 800.0;

#[expect(
    clippy::print_stderr,
    reason = "the spike binary reports its start errors on the console"
)]
fn main() -> ExitCode {
    let options = match Options::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("review-table-spike: {error:?}");
            return ExitCode::FAILURE;
        }
    };
    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tallyline review table spike")
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "tallyline-review-table-spike",
        native,
        Box::new(|_| Ok(Box::new(SpikeApp::new(options)))),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("review-table-spike: {error}");
            ExitCode::FAILURE
        }
    }
}
