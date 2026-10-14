// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Tests of the pure parts of the spike: sample data, navigation, candidate filter and cell looks.

use tallyline_review_table_spike::filter::filter_candidates;
use tallyline_review_table_spike::model::{Column, Confidence, Deviation, Field, sample_rows};
use tallyline_review_table_spike::nav::{CellPos, Direction, Move, next_problem, step};
use tallyline_review_table_spike::style::{BorderStyle, Icon, Palette, look};

fn field(confidence: Confidence, deviation: Deviation) -> Field {
    Field {
        value: "1234".to_owned(),
        confidence,
        deviation,
    }
}

#[test]
fn sample_has_the_requested_row_count_and_every_cell_state() {
    let rows = sample_rows(200);
    assert_eq!(rows.len(), 200);
    let fields: Vec<&Field> = rows
        .iter()
        .flat_map(|row| Column::ALL.map(|column| row.field(column)))
        .collect();
    for confidence in [
        Confidence::Ok,
        Confidence::Confirm,
        Confidence::Select,
        Confidence::Correct,
        Confidence::UserEntered,
        Confidence::Confirmed,
        Confidence::DoubleConfirmed,
    ] {
        assert!(
            fields.iter().any(|f| f.confidence == confidence),
            "missing {confidence:?}"
        );
    }
    assert!(fields.iter().any(|f| f.deviation == Deviation::Equal));
    assert!(
        fields
            .iter()
            .any(|f| matches!(f.deviation, Deviation::Minor { .. }))
    );
    assert!(
        fields
            .iter()
            .any(|f| matches!(f.deviation, Deviation::Major { .. }))
    );
    assert!(fields.iter().any(|f| f.deviation == Deviation::NoReference));
}

#[test]
fn sample_is_deterministic() {
    assert_eq!(sample_rows(50), sample_rows(50));
}

#[test]
fn tab_moves_in_reading_order_and_stops_at_the_ends() {
    let start = CellPos::new(0, Column::Commodity);
    assert_eq!(step(start, Move::Next, 3), CellPos::new(0, Column::Price));
    let last_in_row = CellPos::new(0, Column::ContainerSizes);
    assert_eq!(
        step(last_in_row, Move::Next, 3),
        CellPos::new(1, Column::Commodity)
    );
    assert_eq!(step(start, Move::Previous, 3), start);
    let last = CellPos::new(2, Column::ContainerSizes);
    assert_eq!(step(last, Move::Next, 3), last);
    assert_eq!(
        step(CellPos::new(1, Column::Commodity), Move::Previous, 3),
        CellPos::new(0, Column::ContainerSizes)
    );
}

#[test]
fn arrows_move_within_the_grid_and_clamp_at_its_edges() {
    let middle = CellPos::new(1, Column::Scu);
    assert_eq!(step(middle, Move::Up, 3), CellPos::new(0, Column::Scu));
    assert_eq!(step(middle, Move::Down, 3), CellPos::new(2, Column::Scu));
    assert_eq!(step(middle, Move::Left, 3), CellPos::new(1, Column::Price));
    assert_eq!(
        step(middle, Move::Right, 3),
        CellPos::new(1, Column::Status)
    );
    let corner = CellPos::new(2, Column::ContainerSizes);
    assert_eq!(step(corner, Move::Down, 3), corner);
    assert_eq!(step(corner, Move::Right, 3), corner);
    let origin = CellPos::new(0, Column::Commodity);
    assert_eq!(step(origin, Move::Up, 3), origin);
    assert_eq!(step(origin, Move::Left, 3), origin);
}

#[test]
fn attention_covers_open_decisions_and_deviations_only() {
    assert!(field(Confidence::Confirm, Deviation::Equal).needs_attention());
    assert!(field(Confidence::Select, Deviation::Equal).needs_attention());
    assert!(field(Confidence::Correct, Deviation::Equal).needs_attention());
    assert!(field(Confidence::Ok, Deviation::Minor { delta_percent: 4 }).needs_attention());
    assert!(field(Confidence::Ok, Deviation::Major { delta_percent: 40 }).needs_attention());
    assert!(!field(Confidence::Ok, Deviation::Equal).needs_attention());
    assert!(!field(Confidence::UserEntered, Deviation::NoReference).needs_attention());
    assert!(!field(Confidence::Confirmed, Deviation::Equal).needs_attention());
}

#[test]
fn only_a_user_confirmation_settles_a_deviation() {
    let major = Deviation::Major { delta_percent: 40 };
    let minor = Deviation::Minor { delta_percent: 4 };
    assert!(!field(Confidence::Confirmed, major.clone()).needs_attention());
    assert!(!field(Confidence::Confirmed, minor).needs_attention());
    assert!(field(Confidence::DoubleConfirmed, major.clone()).needs_attention());
    assert!(field(Confidence::UserEntered, major).needs_attention());
}

#[test]
fn f8_finds_the_next_problem_and_wraps() {
    let mut rows = sample_rows(3);
    for row in &mut rows {
        for column in Column::ALL {
            *row.field_mut(column) = field(Confidence::Ok, Deviation::Equal);
        }
    }
    *rows[0].field_mut(Column::Status) = field(Confidence::Confirm, Deviation::Equal);
    *rows[2].field_mut(Column::Price) =
        field(Confidence::Ok, Deviation::Major { delta_percent: 30 });

    let start = CellPos::new(0, Column::Commodity);
    let first = next_problem(&rows, start, Direction::Forward);
    assert_eq!(first, Some(CellPos::new(0, Column::Status)));
    let second = next_problem(&rows, CellPos::new(0, Column::Status), Direction::Forward);
    assert_eq!(second, Some(CellPos::new(2, Column::Price)));
    let wrapped = next_problem(&rows, CellPos::new(2, Column::Price), Direction::Forward);
    assert_eq!(wrapped, Some(CellPos::new(0, Column::Status)));
    let back = next_problem(&rows, CellPos::new(0, Column::Status), Direction::Backward);
    assert_eq!(back, Some(CellPos::new(2, Column::Price)));
}

#[test]
fn f8_returns_none_without_problems_and_the_only_problem_from_itself() {
    let mut rows = sample_rows(2);
    for row in &mut rows {
        for column in Column::ALL {
            *row.field_mut(column) = field(Confidence::Ok, Deviation::Equal);
        }
    }
    let start = CellPos::new(0, Column::Commodity);
    assert_eq!(next_problem(&rows, start, Direction::Forward), None);
    *rows[1].field_mut(Column::Scu) = field(Confidence::Correct, Deviation::Equal);
    let only = CellPos::new(1, Column::Scu);
    assert_eq!(next_problem(&rows, only, Direction::Forward), Some(only));
    assert_eq!(next_problem(&rows, only, Direction::Backward), Some(only));
}

#[test]
fn filter_returns_everything_for_an_empty_query() {
    let items = ["Agricium", "Laranite", "Quantanium"];
    assert_eq!(filter_candidates(&items, ""), vec![0, 1, 2]);
    assert_eq!(filter_candidates(&items, "   "), vec![0, 1, 2]);
}

#[test]
fn filter_ranks_prefix_before_word_start_before_substring() {
    let items = ["Raw Tin", "Tin", "Titanium", "Distilled Spirits", "Stims"];
    assert_eq!(filter_candidates(&items, "ti"), vec![1, 2, 0, 3, 4]);
}

#[test]
fn filter_is_case_insensitive_and_drops_non_matches() {
    let items = ["Agricium", "Laranite", "Quantanium"];
    assert_eq!(filter_candidates(&items, "LARA"), vec![1]);
    assert!(filter_candidates(&items, "xyz").is_empty());
}

#[test]
fn confidence_draws_icon_and_border_but_no_background() {
    let palette = Palette::DARK;
    let cell = look(&field(Confidence::Confirm, Deviation::Equal), &palette);
    assert_eq!(cell.background, None);
    assert_eq!(cell.badge, None);
    let border = cell.border.as_ref().map(|b| (b.colour, b.style));
    assert_eq!(
        border,
        Some((palette.confidence_confirm, BorderStyle::Solid))
    );
    assert_eq!(cell.confidence_icon.map(|i| i.icon), Some(Icon::Help));
}

#[test]
fn deviation_draws_background_and_badge_but_keeps_the_confidence_border() {
    let palette = Palette::DARK;
    let major = look(
        &field(Confidence::Ok, Deviation::Major { delta_percent: 35 }),
        &palette,
    );
    assert_eq!(major.background, Some(palette.deviation_major_bg));
    assert_eq!(major.border, None);
    let badge = major.badge.as_ref().map(|b| (b.text.as_str(), b.fill));
    assert_eq!(
        badge,
        Some(("Δ +35 %", Some(palette.deviation_major_badge_bg)))
    );

    let minor = look(
        &field(Confidence::Ok, Deviation::Minor { delta_percent: -4 }),
        &palette,
    );
    assert_eq!(minor.background, Some(palette.deviation_minor_bg));
    let badge = minor.badge.as_ref().map(|b| (b.text.as_str(), b.fill));
    assert_eq!(badge, Some(("Δ −4 %", None)));
}

#[test]
fn confidence_and_deviation_are_independent_dimensions() {
    let palette = Palette::DARK;
    let confidences = [
        Confidence::Ok,
        Confidence::Confirm,
        Confidence::Select,
        Confidence::Correct,
        Confidence::UserEntered,
        Confidence::Confirmed,
        Confidence::DoubleConfirmed,
    ];
    let deviations = [
        Deviation::Equal,
        Deviation::Minor { delta_percent: 3 },
        Deviation::Major { delta_percent: 50 },
        Deviation::NoReference,
    ];
    for confidence in confidences {
        let borders: Vec<_> = deviations
            .iter()
            .map(|d| look(&field(confidence, d.clone()), &palette).border)
            .collect();
        assert!(borders.windows(2).all(|w| w[0] == w[1]) || confidence == Confidence::Ok);
    }
    for deviation in deviations {
        let backgrounds: Vec<_> = confidences
            .iter()
            .map(|c| look(&field(*c, deviation.clone()), &palette).background)
            .collect();
        assert!(backgrounds.windows(2).all(|w| w[0] == w[1]));
    }
}

#[test]
fn no_reference_uses_a_dashed_border_only_when_confidence_has_none() {
    let palette = Palette::DARK;
    let cell = look(&field(Confidence::Ok, Deviation::NoReference), &palette);
    assert_eq!(cell.background, Some(palette.no_reference_bg));
    let border = cell.border.as_ref().map(|b| (b.colour, b.style));
    assert_eq!(
        border,
        Some((palette.no_reference_border, BorderStyle::Dashed))
    );
    assert_eq!(cell.badge, None);
}

#[test]
fn every_marked_cell_has_a_text_reason() {
    let palette = Palette::DARK;
    let cell = look(
        &field(Confidence::Select, Deviation::Major { delta_percent: 22 }),
        &palette,
    );
    assert_eq!(
        cell.reason,
        "Select: n candidates · Major deviation – confirmation required"
    );
    let plain = look(&field(Confidence::Ok, Deviation::Equal), &palette);
    assert_eq!(plain.reason, "");
}
