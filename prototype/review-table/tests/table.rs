// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keyboard and editing flows of the review table, driven through `egui_kittest` without a renderer.

use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use tallyline_review_table_spike::model::{Column, Confidence, sample_rows};
use tallyline_review_table_spike::nav::{CellPos, Direction, next_problem};
use tallyline_review_table_spike::table::{Editor, ReviewTable};

const STEPS: usize = 4;

fn harness() -> Harness<'static, ReviewTable> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1100.0, 640.0))
        .build_ui_state(
            |ui, table: &mut ReviewTable| table.ui(ui),
            ReviewTable::new(sample_rows(200)),
        );
    harness.run_steps(STEPS);
    harness
}

fn press(harness: &mut Harness<'static, ReviewTable>, key: Key) {
    harness.key_press(key);
    harness.run_steps(STEPS);
}

fn press_shift(harness: &mut Harness<'static, ReviewTable>, key: Key) {
    harness.key_press_modifiers(Modifiers::SHIFT, key);
    harness.run_steps(STEPS);
}

fn type_text(harness: &mut Harness<'static, ReviewTable>, text: &str) {
    harness.event(Event::Text(text.to_owned()));
    harness.run_steps(STEPS);
}

fn value(harness: &Harness<'static, ReviewTable>, pos: CellPos) -> String {
    harness
        .state()
        .rows()
        .get(pos.row)
        .map(|row| row.field(pos.column).value.clone())
        .unwrap_or_default()
}

#[test]
fn tab_and_shift_tab_move_the_focus() {
    let mut harness = harness();
    assert_eq!(harness.state().focus(), CellPos::new(0, Column::Commodity));
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Tab);
    assert_eq!(harness.state().focus(), CellPos::new(0, Column::Scu));
    press_shift(&mut harness, Key::Tab);
    assert_eq!(harness.state().focus(), CellPos::new(0, Column::Price));
    press(&mut harness, Key::ArrowDown);
    assert_eq!(harness.state().focus(), CellPos::new(1, Column::Price));
}

#[test]
fn enter_opens_the_editor_and_enter_commits_a_typed_value() {
    let mut harness = harness();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Enter);
    assert!(matches!(
        harness.state().editor(),
        Some(Editor::Text { .. })
    ));
    type_text(&mut harness, "999");
    press(&mut harness, Key::Enter);
    let pos = CellPos::new(0, Column::Price);
    assert_eq!(harness.state().editor(), None);
    assert_eq!(value(&harness, pos), "999");
    assert_eq!(
        harness.state().rows()[0].field(Column::Price).confidence,
        Confidence::UserEntered
    );
    assert_eq!(harness.state().focus(), pos);
}

#[test]
fn escape_discards_the_edit() {
    let mut harness = harness();
    let pos = CellPos::new(0, Column::Price);
    let before = value(&harness, pos);
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Enter);
    type_text(&mut harness, "1");
    press(&mut harness, Key::Escape);
    assert_eq!(harness.state().editor(), None);
    assert_eq!(value(&harness, pos), before);
    assert_eq!(harness.state().focus(), pos);
}

#[test]
fn tab_in_the_editor_commits_and_moves_on() {
    let mut harness = harness();
    press(&mut harness, Key::Tab);
    press(&mut harness, Key::Enter);
    type_text(&mut harness, "4711");
    press(&mut harness, Key::Tab);
    assert_eq!(value(&harness, CellPos::new(0, Column::Price)), "4711");
    assert_eq!(harness.state().focus(), CellPos::new(0, Column::Scu));
    assert_eq!(harness.state().editor(), None);
}

#[test]
fn f8_and_shift_f8_walk_the_problems_and_scroll_them_into_view() {
    let mut harness = harness();
    let mut expected = CellPos::new(0, Column::Commodity);
    for _ in 0..200 {
        expected = next_problem(harness.state().rows(), expected, Direction::Forward)
            .expect("the sample has problems");
        press(&mut harness, Key::F8);
        assert_eq!(harness.state().focus(), expected);
        assert!(
            harness.state().drawn_rows().contains(&expected.row),
            "row {} not drawn, drawn {:?}",
            expected.row,
            harness.state().drawn_rows()
        );
    }
    assert!(expected.row > 60, "F8 should have left the first screens");
    let back = next_problem(harness.state().rows(), expected, Direction::Backward)
        .expect("the sample has problems");
    press_shift(&mut harness, Key::F8);
    assert_eq!(harness.state().focus(), back);
}

#[test]
fn choice_editor_filters_as_you_type_and_enter_picks() {
    let mut harness = harness();
    press(&mut harness, Key::Enter);
    assert!(matches!(
        harness.state().editor(),
        Some(Editor::Choice { .. })
    ));
    type_text(&mut harness, "lara");
    assert_eq!(harness.state().candidates(), vec!["Laranite"]);
    press(&mut harness, Key::Enter);
    assert_eq!(
        value(&harness, CellPos::new(0, Column::Commodity)),
        "Laranite"
    );
    assert_eq!(harness.state().editor(), None);
}

#[test]
fn an_untouched_combo_box_never_picks_a_candidate() {
    let mut harness = harness();
    let pos = CellPos::new(0, Column::Commodity);
    let before = harness.state().rows()[0].field(Column::Commodity).clone();
    press(&mut harness, Key::Enter);
    press(&mut harness, Key::Enter);
    assert_eq!(harness.state().editor(), None);
    assert_eq!(harness.state().rows()[0].field(Column::Commodity), &before);
    press(&mut harness, Key::Enter);
    press(&mut harness, Key::Tab);
    assert_eq!(harness.state().rows()[0].field(Column::Commodity), &before);
    assert_eq!(
        harness.state().focus(),
        CellPos::new(pos.row, Column::Price)
    );
}

#[test]
fn an_arrow_key_picks_from_the_unfiltered_list() {
    let mut harness = harness();
    press(&mut harness, Key::Enter);
    press(&mut harness, Key::ArrowDown);
    press(&mut harness, Key::ArrowDown);
    press(&mut harness, Key::Enter);
    assert_eq!(
        value(&harness, CellPos::new(0, Column::Commodity)),
        "Agricultural Supplies"
    );
}

#[test]
fn arrow_keys_move_the_highlight_in_the_choice_list() {
    let mut harness = harness();
    press(&mut harness, Key::Enter);
    type_text(&mut harness, "ti");
    assert_eq!(harness.state().candidates()[..2], ["Tin", "Titanium"]);
    press(&mut harness, Key::ArrowDown);
    press(&mut harness, Key::ArrowDown);
    press(&mut harness, Key::ArrowUp);
    press(&mut harness, Key::Enter);
    assert_eq!(
        value(&harness, CellPos::new(0, Column::Commodity)),
        "Titanium"
    );
}

#[test]
fn the_status_column_uses_the_choice_editor_too() {
    let mut harness = harness();
    for _ in 0..3 {
        press(&mut harness, Key::Tab);
    }
    press(&mut harness, Key::Enter);
    type_text(&mut harness, "very h");
    assert_eq!(harness.state().candidates(), vec!["Very high inventory"]);
    press(&mut harness, Key::Enter);
    assert_eq!(
        value(&harness, CellPos::new(0, Column::Status)),
        "Very high inventory"
    );
}

#[test]
fn space_confirms_the_focused_field() {
    let mut harness = harness();
    press(&mut harness, Key::Space);
    assert_eq!(
        harness.state().rows()[0]
            .field(Column::Commodity)
            .confidence,
        Confidence::Confirmed
    );
}

#[test]
fn the_table_draws_only_the_visible_rows() {
    let harness = harness();
    let drawn = harness.state().drawn_rows();
    assert!(drawn.start == 0 && drawn.len() < 40, "drawn {drawn:?}");
}
