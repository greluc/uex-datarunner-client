// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keyboard events and assertions on the accessibility tree of the onboarding start screen.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "spike test code outside main"
)]

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use tallyline_ui_harness_spike::{
    CONTINUE_LABEL, FanKitUnit, OnboardingState, SECRET_KEY_LABEL, onboarding_start_view,
};

struct App {
    onboarding: OnboardingState,
    unit: Option<FanKitUnit>,
}

fn harness() -> Harness<'static, App> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 900.0))
        .build_ui_state(
            |ui, app: &mut App| {
                let unit = app
                    .unit
                    .get_or_insert_with(|| FanKitUnit::load(ui.ctx()).unwrap());
                onboarding_start_view(ui, &mut app.onboarding, unit);
            },
            App {
                onboarding: OnboardingState::default(),
                unit: None,
            },
        );
    harness.run();
    harness
}

fn secret_field<'h>(harness: &'h Harness<'_, App>) -> egui_kittest::Node<'h> {
    harness.get_by(|node| node.role() == Role::PasswordInput || node.role() == Role::TextInput)
}

#[test]
fn tab_types_and_enter_activate_the_form() {
    let mut harness = harness();

    harness.key_press(Key::Tab);
    harness.run();
    assert!(secret_field(&harness).is_focused(), "{harness:#?}");

    harness.event(egui::Event::Text("abc-123".to_owned()));
    harness.run();
    assert_eq!(harness.state().onboarding.secret_key, "abc-123");

    harness.key_press(Key::Backspace);
    harness.run();
    assert_eq!(harness.state().onboarding.secret_key, "abc-12");

    harness.key_press(Key::Tab);
    harness.run();
    assert!(harness.get_by_label(CONTINUE_LABEL).is_focused());

    harness.key_press(Key::Enter);
    harness.run();
    assert_eq!(harness.state().onboarding.continue_activations, 1);

    harness.key_press_modifiers(egui::Modifiers::SHIFT, Key::Tab);
    harness.run();
    assert!(secret_field(&harness).is_focused());
}

#[test]
fn the_password_field_is_labelled_and_does_not_expose_the_secret() {
    let mut harness = harness();
    secret_field(&harness).focus();
    harness.run();
    secret_field(&harness).type_text("top-secret");
    harness.run();

    assert_eq!(harness.state().onboarding.secret_key, "top-secret");
    let field = secret_field(&harness);
    assert_eq!(
        field.accesskit_node().role(),
        Role::PasswordInput,
        "{harness:#?}"
    );
    assert_eq!(
        field.accesskit_node().label().as_deref(),
        Some(SECRET_KEY_LABEL),
        "{harness:#?}"
    );
    let value = field.value().unwrap_or_default();
    assert!(
        !value.contains("top-secret"),
        "the accessibility value leaks the secret: {value}"
    );
}

#[test]
fn text_typed_by_key_events_reaches_the_rendered_output() {
    let mut harness = harness();
    harness.key_press(Key::Tab);
    harness.event(egui::Event::Text("x".to_owned()));
    harness.run();
    let painted_texts: Vec<String> = harness
        .output()
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect();
    assert!(
        painted_texts.iter().any(|text| text == "\u{2022}"),
        "{painted_texts:?}"
    );
}
