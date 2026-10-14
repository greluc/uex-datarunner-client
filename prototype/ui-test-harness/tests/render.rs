// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pixel rendering through the wgpu renderer of `egui_kittest`; built only with the `wgpu` feature.

#![cfg(feature = "wgpu")]
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "spike test code outside main"
)]

use egui::Ui;
use egui_kittest::kittest::Queryable;
use egui_kittest::wgpu::{create_render_state, default_wgpu_setup};
use egui_kittest::{Harness, HarnessBuilder};
use tallyline_ui_harness_spike::{
    FAN_KIT_LOGO_ALT_TEXT, FAN_KIT_LOGO_PNG, FAN_KIT_LOGO_SIZE, FanKitUnit, about_view, decode_logo,
};

const MAX_CHANNEL_DIFFERENCE: u8 = 8;

type View = fn(&mut Ui, &FanKitUnit);

struct App {
    unit: Option<FanKitUnit>,
    view: View,
}

fn builder() -> HarnessBuilder<App> {
    let state = create_render_state(default_wgpu_setup(), Default::default());
    let info = state.adapter.get_info();
    eprintln!(
        "wgpu adapter: {} ({:?}, {:?}, driver {} {})",
        info.name, info.backend, info.device_type, info.driver, info.driver_info
    );
    Harness::builder()
        .with_size(egui::vec2(800.0, 900.0))
        .wgpu()
}

fn harness(view: View, pixels_per_point: f32) -> Harness<'static, App> {
    let mut harness = builder()
        .with_pixels_per_point(pixels_per_point)
        .build_ui_state(
            |ui, app: &mut App| {
                let unit = app
                    .unit
                    .get_or_insert_with(|| FanKitUnit::load(ui.ctx()).unwrap());
                (app.view)(ui, unit);
            },
            App { unit: None, view },
        );
    harness.run();
    harness
}

fn worst_logo_pixel(harness: &mut Harness<'static, App>) -> (u8, usize) {
    let image = harness.render().unwrap();
    let ppp = harness.ctx.pixels_per_point();
    let rect = harness.get_by_label(FAN_KIT_LOGO_ALT_TEXT).rect();
    let logo = decode_logo(FAN_KIT_LOGO_PNG).unwrap();
    let [width, height] = logo.size;
    let x0 = (rect.min.x * ppp).round() as u32;
    let y0 = (rect.min.y * ppp).round() as u32;
    let background = image
        .get_pixel(
            ((rect.max.x + 40.0) * ppp) as u32,
            (rect.center().y * ppp) as u32,
        )
        .0;
    assert_eq!(
        background[3], 255,
        "the sample beside the logo is not opaque"
    );

    let mut worst = 0_u8;
    let mut differing = 0_usize;
    for y in 0..height {
        for x in 0..width {
            let [r, g, b, a] = logo.pixels[y * width + x].to_srgba_unmultiplied();
            let alpha = f32::from(a) / 255.0;
            let expected = [r, g, b]
                .iter()
                .zip(background.iter())
                .map(|(&f, &bg)| {
                    (f32::from(f) * alpha + f32::from(bg) * (1.0 - alpha)).round() as u8
                })
                .collect::<Vec<_>>();
            let seen = image.get_pixel(x0 + x as u32, y0 + y as u32).0;
            let difference = expected
                .iter()
                .zip(seen.iter())
                .map(|(&e, &s)| e.abs_diff(s))
                .max()
                .unwrap_or(0);
            worst = worst.max(difference);
            if difference > MAX_CHANNEL_DIFFERENCE {
                differing += 1;
            }
        }
    }
    (worst, differing)
}

fn about(ui: &mut Ui, unit: &FanKitUnit) {
    about_view(ui, unit);
}

fn mirrored(ui: &mut Ui, unit: &FanKitUnit) {
    ui.heading("About Tallyline");
    ui.label("Tallyline is free software under GPL-3.0-or-later and comes with no warranty.");
    ui.separator();
    ui.group(|ui| {
        ui.add(
            egui::Image::new((
                unit.logo_texture().id(),
                egui::vec2(FAN_KIT_LOGO_SIZE, FAN_KIT_LOGO_SIZE),
            ))
            .uv(egui::Rect::from_min_max(
                egui::pos2(1.0, 0.0),
                egui::pos2(0.0, 1.0),
            ))
            .alt_text(FAN_KIT_LOGO_ALT_TEXT),
        );
    });
}

fn one_texel_per_pixel() -> f32 {
    let logo = decode_logo(FAN_KIT_LOGO_PNG).unwrap();
    logo.size[0] as f32 / FAN_KIT_LOGO_SIZE
}

#[test]
fn the_logo_is_rendered_pixel_for_pixel() {
    let mut harness = harness(about, one_texel_per_pixel());
    let (worst, differing) = worst_logo_pixel(&mut harness);
    eprintln!(
        "worst channel difference {worst}, pixels above {MAX_CHANNEL_DIFFERENCE}: {differing}"
    );
    assert_eq!(differing, 0, "worst channel difference {worst}");
}

#[test]
fn probe_a_mirrored_logo_fails_the_pixel_comparison() {
    let mut harness = harness(mirrored, one_texel_per_pixel());
    let (worst, differing) = worst_logo_pixel(&mut harness);
    eprintln!(
        "mirrored: worst channel difference {worst}, pixels above {MAX_CHANNEL_DIFFERENCE}: {differing}"
    );
    assert!(differing > 1000, "only {differing} pixels differ");
}

#[test]
fn about_view_snapshot() {
    let mut harness = harness(about, 1.0);
    harness.snapshot("about_view");
}
