// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Fan Kit pinning tests of R-UI-19 expressed with `egui_kittest`, and probes that a broken unit fails them.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "spike test code outside main"
)]

mod support;

use egui::Ui;
use egui_kittest::Harness;
use support::{
    AGREEMENT_NOTICE_SHA256, LOGO_FILE_SHA256, RecordedTextures, RecordingRenderer,
    TRADEMARK_LINE_SHA256, check_fan_kit_unit, logo_file_on_disk, sha256_hex,
};
use tallyline_ui_harness_spike::{
    FAN_KIT_AGREEMENT_NOTICE, FAN_KIT_LOGO_PNG, FAN_KIT_TRADEMARK_LINE, FanKitUnit,
    OnboardingState, about_view, about_view_folded, about_view_logo_only, onboarding_start_view,
};

type View = fn(&mut Ui, &mut OnboardingState, &FanKitUnit);

struct App {
    onboarding: OnboardingState,
    unit: Option<FanKitUnit>,
    view: View,
    loader: fn(&egui::Context) -> FanKitUnit,
}

fn pinned_unit(ctx: &egui::Context) -> FanKitUnit {
    FanKitUnit::load(ctx).unwrap()
}

fn harness_for(
    view: View,
    loader: fn(&egui::Context) -> FanKitUnit,
) -> (Harness<'static, App>, RecordedTextures) {
    let textures = RecordedTextures::default();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 900.0))
        .renderer(RecordingRenderer(textures.clone()))
        .build_ui_state(
            |ui, app: &mut App| {
                let unit = app.unit.get_or_insert_with(|| (app.loader)(ui.ctx()));
                (app.view)(ui, &mut app.onboarding, unit);
            },
            App {
                onboarding: OnboardingState::default(),
                unit: None,
                view,
                loader,
            },
        );
    harness.run();
    (harness, textures)
}

fn about(ui: &mut Ui, _: &mut OnboardingState, unit: &FanKitUnit) {
    about_view(ui, unit);
}

fn onboarding(ui: &mut Ui, state: &mut OnboardingState, unit: &FanKitUnit) {
    onboarding_start_view(ui, state, unit);
}

#[test]
fn r_ui_19_pinned_constants_have_their_sha256() {
    assert_eq!(
        sha256_hex(FAN_KIT_TRADEMARK_LINE.as_bytes()),
        TRADEMARK_LINE_SHA256
    );
    assert_eq!(
        sha256_hex(FAN_KIT_AGREEMENT_NOTICE.as_bytes()),
        AGREEMENT_NOTICE_SHA256
    );
    assert_eq!(sha256_hex(FAN_KIT_LOGO_PNG), LOGO_FILE_SHA256);
    assert_eq!(FAN_KIT_LOGO_PNG, logo_file_on_disk().as_slice());
}

#[test]
fn r_ui_19_about_dialog_shows_the_unit() {
    let (harness, textures) = harness_for(about, pinned_unit);
    check_fan_kit_unit(&harness, &textures).unwrap();
}

#[test]
fn r_ui_19_onboarding_start_screen_shows_the_unit() {
    let (harness, textures) = harness_for(onboarding, pinned_unit);
    check_fan_kit_unit(&harness, &textures).unwrap();
}

#[test]
fn probe_logo_without_notices_fails() {
    fn view(ui: &mut Ui, _: &mut OnboardingState, unit: &FanKitUnit) {
        about_view_logo_only(ui, unit);
    }
    let (harness, textures) = harness_for(view, pinned_unit);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("trademark line appears 0 times"), "{error}");
}

#[test]
fn probe_unit_folded_behind_a_disclosure_fails() {
    fn view(ui: &mut Ui, _: &mut OnboardingState, unit: &FanKitUnit) {
        about_view_folded(ui, unit);
    }
    let (harness, textures) = harness_for(view, pinned_unit);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("trademark line appears 0 times"), "{error}");
}

#[test]
fn probe_tidied_notice_fails() {
    fn tidied(ctx: &egui::Context) -> FanKitUnit {
        FanKitUnit::load_with(
            ctx,
            FAN_KIT_LOGO_PNG,
            "Star Citizen®, Roberts Space Industries® and Cloud Imperium® are registered trademarks of Cloud Imperium Rights LLC",
            FAN_KIT_AGREEMENT_NOTICE,
        )
        .unwrap()
    }
    let (harness, textures) = harness_for(about, tidied);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("trademark line appears 0 times"), "{error}");
}

#[test]
fn probe_other_logo_fails() {
    fn other_logo(ctx: &egui::Context) -> FanKitUnit {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(256, 256, image::Rgba([0, 0, 0, 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        FanKitUnit::load_with(ctx, &png, FAN_KIT_TRADEMARK_LINE, FAN_KIT_AGREEMENT_NOTICE).unwrap()
    }
    let (harness, textures) = harness_for(about, other_logo);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("uploaded logo pixels"), "{error}");
}

#[test]
fn probe_unit_scrolled_off_screen_fails() {
    fn view(ui: &mut Ui, state: &mut OnboardingState, unit: &FanKitUnit) {
        ui.add_space(2000.0);
        onboarding_start_view(ui, state, unit);
    }
    let (harness, textures) = harness_for(view, pinned_unit);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("is not inside the screen"), "{error}");
}

#[test]
fn probe_parts_shown_apart_fails() {
    fn view(ui: &mut Ui, _: &mut OnboardingState, unit: &FanKitUnit) {
        about_view_logo_only(ui, unit);
        ui.label(FAN_KIT_TRADEMARK_LINE);
        ui.label(FAN_KIT_AGREEMENT_NOTICE);
    }
    let (harness, textures) = harness_for(view, pinned_unit);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("not only the unit"), "{error}");
}

#[test]
fn probe_small_notice_fails() {
    fn small(ctx: &egui::Context) -> FanKitUnit {
        FanKitUnit::load(ctx).unwrap().with_notice_font_size(8.0)
    }
    let (harness, textures) = harness_for(about, small);
    let error = check_fan_kit_unit(&harness, &textures).unwrap_err();
    assert!(error.contains("font size"), "{error}");
}

#[test]
fn dump_accessibility_tree() {
    let (harness, _) = harness_for(about, pinned_unit);
    let tree = format!("{:#?}", harness.root());
    assert!(tree.contains("Image"), "{tree}");
}
