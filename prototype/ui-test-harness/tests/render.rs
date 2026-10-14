// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pixel rendering through the wgpu renderer of `egui_kittest`; built only with the `wgpu` feature.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "spike test code outside main"
)]
#![cfg(feature = "wgpu")]

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use tallyline_ui_harness_spike::{
    FAN_KIT_LOGO_ALT_TEXT, FAN_KIT_LOGO_PNG, FanKitUnit, about_view, decode_logo,
};

struct App {
    unit: Option<FanKitUnit>,
}

fn mean_rgb(pixels: impl Iterator<Item = [u8; 4]>) -> [f32; 3] {
    let mut sum = [0.0_f32; 3];
    let mut count = 0.0_f32;
    for [r, g, b, a] in pixels {
        let alpha = f32::from(a) / 255.0;
        sum[0] += f32::from(r) * alpha;
        sum[1] += f32::from(g) * alpha;
        sum[2] += f32::from(b) * alpha;
        count += 1.0;
    }
    sum.map(|channel| channel / count)
}

#[test]
fn the_logo_is_rendered_into_pixels() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 900.0))
        .wgpu()
        .build_ui_state(
            |ui, app: &mut App| {
                let unit = app
                    .unit
                    .get_or_insert_with(|| FanKitUnit::load(ui.ctx()).unwrap());
                about_view(ui, unit);
            },
            App { unit: None },
        );
    harness.run();
    let image = harness.render().unwrap();
    let ppp = harness.ctx.pixels_per_point();
    let rect = harness.get_by_label(FAN_KIT_LOGO_ALT_TEXT).rect();

    let x0 = (rect.min.x * ppp).ceil() as u32;
    let y0 = (rect.min.y * ppp).ceil() as u32;
    let x1 = (rect.max.x * ppp).floor() as u32;
    let y1 = (rect.max.y * ppp).floor() as u32;
    let rendered = mean_rgb(
        (y0..y1)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .map(|(x, y)| {
                let p = image.get_pixel(x, y).0;
                [p[0], p[1], p[2], 255]
            }),
    );

    let background = image
        .get_pixel(
            ((rect.max.x + 40.0) * ppp) as u32,
            (rect.center().y * ppp) as u32,
        )
        .0;
    assert_eq!(
        background[3], 255,
        "the sample beside the logo is not the opaque group fill"
    );
    let logo = decode_logo(FAN_KIT_LOGO_PNG).unwrap();
    let bg = [
        f32::from(background[0]),
        f32::from(background[1]),
        f32::from(background[2]),
    ];
    let expected = {
        let mut sum = [0.0_f32; 3];
        for pixel in &logo.pixels {
            let [r, g, b, a] = pixel.to_srgba_unmultiplied();
            let alpha = f32::from(a) / 255.0;
            sum[0] += f32::from(r) * alpha + bg[0] * (1.0 - alpha);
            sum[1] += f32::from(g) * alpha + bg[1] * (1.0 - alpha);
            sum[2] += f32::from(b) * alpha + bg[2] * (1.0 - alpha);
        }
        sum.map(|channel| channel / logo.pixels.len() as f32)
    };

    let distance = rendered
        .iter()
        .zip(expected.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    assert!(
        distance < 12.0,
        "rendered logo mean {rendered:?}, expected {expected:?}, background {background:?}"
    );
}

#[test]
fn about_view_snapshot() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 900.0))
        .wgpu()
        .build_ui_state(
            |ui, app: &mut App| {
                let unit = app
                    .unit
                    .get_or_insert_with(|| FanKitUnit::load(ui.ctx()).unwrap());
                about_view(ui, unit);
            },
            App { unit: None },
        );
    harness.run();
    harness.snapshot("about_view");
}
