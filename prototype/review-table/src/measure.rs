// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The scripted frame plan of the measurement mode and the per-phase frame-time statistics.

/// A phase of the scripted run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Frames that are not counted: font atlas, pipelines and first layouts.
    Warmup,
    /// The table scrolls by a fixed step every frame.
    Scroll,
    /// F8 every frame: focus jump plus scroll to the row.
    Navigate,
    /// The combo box opens, filters on a typed character and closes again.
    Edit,
}

impl Phase {
    /// The phase name in the CSV output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::Scroll => "scroll",
            Self::Navigate => "navigate",
            Self::Edit => "edit",
        }
    }
}

/// The number of frames of each phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Frames of [`Phase::Warmup`].
    pub warmup: usize,
    /// Frames of [`Phase::Scroll`].
    pub scroll: usize,
    /// Frames of [`Phase::Navigate`].
    pub navigate: usize,
    /// Frames of [`Phase::Edit`].
    pub edit: usize,
}

impl Plan {
    /// The plan the measurement script runs.
    pub const STANDARD: Self = Self {
        warmup: 60,
        scroll: 600,
        navigate: 300,
        edit: 300,
    };

    /// The phase of frame `frame` and the frame's index inside it; `None` once the plan is done.
    #[must_use]
    pub fn phase_at(&self, frame: usize) -> Option<(Phase, usize)> {
        let mut start = 0;
        for (phase, length) in [
            (Phase::Warmup, self.warmup),
            (Phase::Scroll, self.scroll),
            (Phase::Navigate, self.navigate),
            (Phase::Edit, self.edit),
        ] {
            if frame < start + length {
                return Some((phase, frame - start));
            }
            start += length;
        }
        None
    }
}

/// The cost of one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// The phase the frame belongs to.
    pub phase: Phase,
    /// eframe's CPU time of the frame (UI pass and rendering, without the vsync wait) in milliseconds.
    pub cpu_ms: f32,
    /// The wall-clock time since the previous frame in milliseconds.
    pub interval_ms: f32,
}

/// The statistics of one phase.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhaseSummary {
    /// The phase.
    pub phase: Phase,
    /// The number of frames.
    pub frames: usize,
    /// The median CPU time per frame.
    pub cpu_median_ms: f32,
    /// The 95th percentile of the CPU time per frame.
    pub cpu_p95_ms: f32,
    /// The longest CPU time of a frame.
    pub cpu_max_ms: f32,
    /// The median frame interval.
    pub interval_median_ms: f32,
    /// The 95th percentile of the frame interval.
    pub interval_p95_ms: f32,
}

/// The nearest-rank percentile `p` (0 to 100) of ascending `sorted`; `None` for an empty slice.
#[must_use]
pub fn percentile(sorted: &[f32], p: f32) -> Option<f32> {
    let rank = (p / 100.0 * sorted.len() as f32).ceil();
    let index = (rank as usize)
        .saturating_sub(1)
        .min(sorted.len().saturating_sub(1));
    sorted.get(index).copied()
}

fn sorted(values: impl Iterator<Item = f32>) -> Vec<f32> {
    let mut values: Vec<f32> = values.collect();
    values.sort_by(f32::total_cmp);
    values
}

/// The statistics per phase in plan order; the warm-up is left out.
#[must_use]
pub fn summarise(samples: &[Sample]) -> Vec<PhaseSummary> {
    [Phase::Scroll, Phase::Navigate, Phase::Edit]
        .into_iter()
        .filter_map(|phase| {
            let of_phase: Vec<&Sample> = samples.iter().filter(|s| s.phase == phase).collect();
            let cpu = sorted(of_phase.iter().map(|s| s.cpu_ms));
            let interval = sorted(of_phase.iter().map(|s| s.interval_ms));
            Some(PhaseSummary {
                phase,
                frames: of_phase.len(),
                cpu_median_ms: percentile(&cpu, 50.0)?,
                cpu_p95_ms: percentile(&cpu, 95.0)?,
                cpu_max_ms: percentile(&cpu, 100.0)?,
                interval_median_ms: percentile(&interval, 50.0)?,
                interval_p95_ms: percentile(&interval, 95.0)?,
            })
        })
        .collect()
}

/// The CSV report: one line per phase, with the scale and the pixel density of the run.
#[must_use]
pub fn report(scale_percent: u32, pixels_per_point: f32, summaries: &[PhaseSummary]) -> String {
    let mut out = String::from(
        "scale_percent,pixels_per_point,phase,frames,cpu_median_ms,cpu_p95_ms,cpu_max_ms,interval_median_ms,interval_p95_ms\n",
    );
    for s in summaries {
        out.push_str(&format!(
            "{scale_percent},{pixels_per_point:.2},{},{},{:.3},{:.3},{:.3},{:.3},{:.3}\n",
            s.phase.name(),
            s.frames,
            s.cpu_median_ms,
            s.cpu_p95_ms,
            s.cpu_max_ms,
            s.interval_median_ms,
            s.interval_p95_ms
        ));
    }
    out
}
