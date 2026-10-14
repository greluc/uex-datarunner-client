// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The frame plan and the statistics of the measurement mode.

use tallyline_review_table_spike::measure::{Phase, Plan, Sample, percentile, summarise};

#[test]
fn the_plan_walks_the_phases_in_order() {
    let plan = Plan {
        warmup: 2,
        scroll: 3,
        navigate: 1,
        edit: 2,
    };
    let phases: Vec<_> = (0..9).map(|frame| plan.phase_at(frame)).collect();
    assert_eq!(
        phases,
        vec![
            Some((Phase::Warmup, 0)),
            Some((Phase::Warmup, 1)),
            Some((Phase::Scroll, 0)),
            Some((Phase::Scroll, 1)),
            Some((Phase::Scroll, 2)),
            Some((Phase::Navigate, 0)),
            Some((Phase::Edit, 0)),
            Some((Phase::Edit, 1)),
            None,
        ]
    );
}

#[test]
fn percentile_uses_the_nearest_rank() {
    let sorted = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    assert_eq!(percentile(&sorted, 50.0), Some(5.0));
    assert_eq!(percentile(&sorted, 95.0), Some(10.0));
    assert_eq!(percentile(&sorted, 100.0), Some(10.0));
    assert_eq!(percentile(&[], 50.0), None);
}

#[test]
fn the_summary_skips_the_warmup_and_groups_by_phase() {
    let sample = |phase, cpu_ms, interval_ms| Sample {
        phase,
        cpu_ms,
        interval_ms,
    };
    let samples = [
        sample(Phase::Warmup, 100.0, 100.0),
        sample(Phase::Scroll, 3.0, 16.0),
        sample(Phase::Scroll, 1.0, 17.0),
        sample(Phase::Scroll, 2.0, 18.0),
        sample(Phase::Edit, 4.0, 16.0),
    ];
    let summary = summarise(&samples);
    assert_eq!(summary.len(), 2);
    assert_eq!(summary[0].phase, Phase::Scroll);
    assert_eq!(summary[0].frames, 3);
    assert!((summary[0].cpu_median_ms - 2.0).abs() < f32::EPSILON);
    assert!((summary[0].cpu_max_ms - 3.0).abs() < f32::EPSILON);
    assert!((summary[0].interval_median_ms - 17.0).abs() < f32::EPSILON);
    assert_eq!(summary[1].phase, Phase::Edit);
}
