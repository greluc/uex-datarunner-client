// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The spike window: tool bar with the display scale, the review table, a status line, and the scripted measurement mode.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui::{Event, RichText, Ui, ViewportCommand};

use crate::measure::{Phase, Plan, Sample, report, summarise};
use crate::model::{Column, sample_rows};
use crate::nav::{CellPos, Direction};
use crate::style::Palette;
use crate::table::{Command, ReviewTable};

/// The display scales the spike offers and measures, in percent.
pub const SCALES: [u32; 5] = [100, 125, 150, 200, 250];

/// The number of rows of the prototype.
pub const ROW_COUNT: usize = 200;

const DEFAULT_SCALE_PERCENT: u32 = 100;
const DEFAULT_IDLE_SECONDS: u64 = 20;
const SCROLL_STEP: f32 = 13.0;
const SCROLL_RANGE: f32 = 5_000.0;
const EDIT_CYCLE: usize = 30;
const EDIT_QUERY: &str = "quanti";
const EDIT_ROW_STRIDE: usize = 7;

/// The command-line options of the spike.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The effective display scale in percent (physical pixels per point × 100), reached through egui zoom whatever the OS scale.
    pub scale_percent: u32,
    /// Where the measurement mode writes its CSV report; `None` opens the interactive window.
    pub measure: Option<PathBuf>,
    /// How long the window stays idle after the scripted run before it closes.
    pub idle_seconds: u64,
    /// A file created on the first frame, so that a script can time the start.
    pub first_frame_marker: Option<PathBuf>,
}

/// Why the options could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionsError {
    /// An option that needs a value came last.
    MissingValue(String),
    /// A value could not be parsed.
    InvalidValue(String),
    /// An unknown argument.
    Unknown(String),
}

impl Options {
    /// Parses the arguments after the program name.
    ///
    /// # Errors
    /// Returns the [`OptionsError`] of the first argument that cannot be read.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, OptionsError> {
        let mut options = Self {
            scale_percent: DEFAULT_SCALE_PERCENT,
            measure: None,
            idle_seconds: DEFAULT_IDLE_SECONDS,
            first_frame_marker: None,
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let mut value = || {
                args.next()
                    .ok_or_else(|| OptionsError::MissingValue(arg.clone()))
            };
            match arg.as_str() {
                "--scale" => {
                    let raw = value()?;
                    options.scale_percent = raw
                        .parse()
                        .ok()
                        .filter(|scale| SCALES.contains(scale))
                        .ok_or(OptionsError::InvalidValue(raw))?;
                }
                "--measure" => options.measure = Some(PathBuf::from(value()?)),
                "--idle-seconds" => {
                    let raw = value()?;
                    options.idle_seconds =
                        raw.parse().map_err(|_| OptionsError::InvalidValue(raw))?;
                }
                "--first-frame-marker" => {
                    options.first_frame_marker = Some(PathBuf::from(value()?))
                }
                _ => return Err(OptionsError::Unknown(arg)),
            }
        }
        Ok(options)
    }
}

/// Sets the egui zoom so that the effective pixels per point equal `scale_percent`, whatever the OS scale of the monitor.
fn apply_scale(ui: &Ui, scale_percent: u32) {
    let native = ui.ctx().native_pixels_per_point().unwrap_or(1.0);
    ui.ctx()
        .set_zoom_factor(scale_percent as f32 / 100.0 / native);
}

struct Measurement {
    out: PathBuf,
    plan: Plan,
    frame: usize,
    samples: Vec<Sample>,
    previous: Option<(Phase, Instant)>,
    finished: Option<Instant>,
    idle_frames: usize,
    idle_frames_with_input: usize,
}

/// The spike application.
pub struct SpikeApp {
    table: ReviewTable,
    options: Options,
    scale_percent: u32,
    measurement: Option<Measurement>,
    started: bool,
    error: Option<String>,
}

impl SpikeApp {
    /// The app with [`ROW_COUNT`] sample rows.
    #[must_use]
    pub fn new(options: Options) -> Self {
        let measurement = options.measure.clone().map(|out| Measurement {
            out,
            plan: Plan::STANDARD,
            frame: 0,
            samples: Vec::new(),
            previous: None,
            finished: None,
            idle_frames: 0,
            idle_frames_with_input: 0,
        });
        Self {
            table: ReviewTable::new(sample_rows(ROW_COUNT)),
            scale_percent: options.scale_percent,
            options,
            measurement,
            started: false,
            error: None,
        }
    }

    fn start(&mut self, ui: &Ui) {
        if self.started {
            return;
        }
        self.started = true;
        apply_scale(ui, self.scale_percent);
        ui.ctx().set_visuals(egui::Visuals::dark());
        if let Some(marker) = &self.options.first_frame_marker
            && let Err(error) = std::fs::write(marker, b"")
        {
            self.error = Some(format!("first-frame marker: {error}"));
        }
    }

    fn drive_measurement(&mut self, ui: &Ui, cpu_seconds: Option<f32>) {
        let Some(measurement) = &mut self.measurement else {
            return;
        };
        let now = Instant::now();
        if let Some(finished) = measurement.finished {
            measurement.idle_frames += 1;
            if ui.input(|input| !input.events.is_empty() || input.pointer.is_moving()) {
                measurement.idle_frames_with_input += 1;
            }
            let idle = Duration::from_secs(self.options.idle_seconds);
            if now.duration_since(finished) >= idle {
                let mut path = measurement.out.clone().into_os_string();
                path.push(".idle-frames");
                let counts = format!(
                    "{} ({} with input)",
                    measurement.idle_frames, measurement.idle_frames_with_input
                );
                if let Err(error) = std::fs::write(&path, counts) {
                    self.error = Some(format!("idle frames: {error}"));
                }
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            } else {
                ui.ctx()
                    .request_repaint_after(idle.saturating_sub(now.duration_since(finished)));
            }
            return;
        }
        if let (Some((phase, last)), Some(cpu)) = (measurement.previous, cpu_seconds) {
            measurement.samples.push(Sample {
                phase,
                cpu_ms: cpu * 1_000.0,
                interval_ms: now.duration_since(last).as_secs_f32() * 1_000.0,
            });
        }
        let Some((phase, index)) = measurement.plan.phase_at(measurement.frame) else {
            self.table.set_scroll_offset(None);
            self.table.apply(Command::Cancel);
            let ppp = ui.ctx().pixels_per_point();
            let csv = report(self.scale_percent, ppp, &summarise(&measurement.samples));
            if let Err(error) = std::fs::write(&measurement.out, csv) {
                self.error = Some(format!("report: {error}"));
            }
            measurement.finished = Some(now);
            ui.ctx()
                .request_repaint_after(Duration::from_secs(self.options.idle_seconds));
            return;
        };
        match phase {
            Phase::Warmup => {}
            Phase::Scroll => self
                .table
                .set_scroll_offset(Some((index as f32 * SCROLL_STEP) % SCROLL_RANGE)),
            Phase::Navigate => {
                self.table.set_scroll_offset(None);
                self.table.apply(Command::Problem(Direction::Forward));
            }
            Phase::Edit => {
                let step = index % EDIT_CYCLE;
                if step == 0 {
                    let row = (index / EDIT_CYCLE * EDIT_ROW_STRIDE) % ROW_COUNT;
                    self.table
                        .apply(Command::Focus(CellPos::new(row, Column::Commodity)));
                    self.table.apply(Command::BeginEdit);
                } else if step == EDIT_CYCLE - 1 {
                    self.table.apply(Command::Cancel);
                } else if let Some(c) = EDIT_QUERY.chars().nth(step - 1) {
                    ui.ctx()
                        .input_mut(|input| input.events.push(Event::Text(c.to_string())));
                }
            }
        }
        measurement.previous = Some((phase, now));
        measurement.frame += 1;
        ui.ctx().request_repaint();
    }

    fn tool_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Review table spike").strong());
            ui.separator();
            ui.label("Scale");
            for scale in SCALES {
                if ui
                    .selectable_label(self.scale_percent == scale, format!("{scale} %"))
                    .clicked()
                {
                    self.scale_percent = scale;
                    apply_scale(ui, scale);
                }
            }
            ui.separator();
            ui.label(format!(
                "{} fields need attention",
                self.table.problem_count()
            ));
        });
        ui.label(
            RichText::new(
                "Tab / Shift+Tab move · arrows move · Enter or F2 edit · Enter commit · Esc cancel · Space confirm · F8 / Shift+F8 next / previous problem",
            )
            .color(Palette::DARK.text_secondary),
        );
    }
}

impl eframe::App for SpikeApp {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        self.start(ui);
        self.drive_measurement(ui, frame.info().cpu_usage);
        self.table.handle_input(ui);

        egui::Panel::top("tool-bar").show(ui, |ui| self.tool_bar(ui));
        egui::Panel::bottom("status-line").show(ui, |ui| {
            let text = self
                .error
                .clone()
                .unwrap_or_else(|| self.table.focus_description());
            ui.label(text);
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).fill(Palette::DARK.surface))
            .show(ui, |ui| self.table.draw(ui));
    }
}
