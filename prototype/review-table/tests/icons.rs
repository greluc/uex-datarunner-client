// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The icon rendering check: glyph availability and pixel-grid alignment of every text quad at 100, 125 and 150 %.

use egui::epaint::{ClippedShape, Primitive};
use egui::{Color32, FontId, Shape, Ui};
use egui_kittest::Harness;
use tallyline_review_table_spike::model::sample_rows;
use tallyline_review_table_spike::style::{ICON_SIZE, Icon, Palette, StateIcon};
use tallyline_review_table_spike::table::{ReviewTable, paint_icon};

const CHECKED_SCALES: [f32; 3] = [1.0, 1.25, 1.5];
const GRID_TOLERANCE: f32 = 0.01;
const MISSING_GLYPH: char = '\u{E000}';
const KNOWN_GAP: char = '⇄';

fn icon_sheet(ui: &mut Ui) {
    let palette = Palette::DARK;
    let colours = [
        palette.confidence_confirm,
        palette.confidence_correct,
        palette.confirmed,
        palette.deviation_major_fg,
    ];
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, palette.surface);
    let origin = ui.max_rect().min;
    for (row, colour) in colours.into_iter().enumerate() {
        for (column, icon) in Icon::ALL.into_iter().enumerate() {
            let left = origin.x + 7.3 + column as f32 * (ICON_SIZE + 4.0);
            let centre_y = origin.y + 15.0 + row as f32 * 28.0;
            paint_icon(ui, left, centre_y, StateIcon { icon, colour });
        }
    }
}

fn off_grid(value: f32) -> bool {
    (value - value.round()).abs() > GRID_TOLERANCE
}

fn text_quads(harness: &Harness<'_, impl Sized>) -> (usize, usize) {
    let ppp = harness.ctx.pixels_per_point();
    let text_only: Vec<ClippedShape> = harness
        .output()
        .shapes
        .iter()
        .filter(|clipped| matches!(clipped.shape, Shape::Text(_)))
        .cloned()
        .collect();
    let primitives = harness.ctx.tessellate(text_only, ppp);
    let mut total = 0;
    let mut misaligned = 0;
    for primitive in primitives {
        let Primitive::Mesh(mesh) = primitive.primitive else {
            continue;
        };
        for quad in mesh.vertices.chunks(4) {
            total += 1;
            if quad
                .iter()
                .any(|v| off_grid(v.pos.x * ppp) || off_grid(v.pos.y * ppp))
            {
                misaligned += 1;
            }
        }
    }
    (total, misaligned)
}

fn glyph_texture(ui: &Ui, c: char) -> Option<([u16; 2], [u16; 2])> {
    let galley = ui.painter().layout_no_wrap(
        c.to_string(),
        FontId::proportional(ICON_SIZE),
        Color32::WHITE,
    );
    galley
        .rows
        .first()
        .and_then(|row| row.row.glyphs.first())
        .map(|glyph| (glyph.uv_rect.min, glyph.uv_rect.max))
}

#[test]
fn no_stand_in_icon_falls_back_to_the_missing_glyph_box() {
    let mut harness = Harness::new_ui_state(
        |ui, missing: &mut Vec<&'static str>| {
            let missing_box = glyph_texture(ui, MISSING_GLYPH);
            *missing = Icon::ALL
                .into_iter()
                .filter(|icon| glyph_texture(ui, icon.glyph()) == missing_box)
                .map(Icon::name)
                .collect();
        },
        vec!["not checked"],
    );
    harness.run();
    assert!(
        harness.state().is_empty(),
        "drawn as the missing-glyph box: {:?}",
        harness.state()
    );
}

#[test]
fn the_missing_glyph_probe_detects_a_known_gap() {
    let mut harness = Harness::new_ui_state(
        |ui, same: &mut bool| {
            *same = glyph_texture(ui, KNOWN_GAP) == glyph_texture(ui, MISSING_GLYPH);
        },
        false,
    );
    harness.run();
    assert!(
        harness.state(),
        "the probe does not see the gap of {KNOWN_GAP:?}"
    );
}

#[test]
fn icon_glyph_quads_sit_on_the_physical_pixel_grid() {
    for ppp in CHECKED_SCALES {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(260.0, 130.0))
            .with_pixels_per_point(ppp)
            .build_ui(icon_sheet);
        harness.run();
        let (total, misaligned) = text_quads(&harness);
        assert!(total >= Icon::ALL.len() * 4, "only {total} quads at {ppp}");
        assert_eq!(
            misaligned, 0,
            "{misaligned} of {total} icon quads off the grid at {ppp}"
        );
    }
}

#[test]
fn table_text_quads_sit_on_the_physical_pixel_grid() {
    for ppp in CHECKED_SCALES {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(1100.0, 400.0))
            .with_pixels_per_point(ppp)
            .build_ui_state(
                |ui, table: &mut ReviewTable| table.ui(ui),
                ReviewTable::new(sample_rows(200)),
            );
        harness.run_steps(4);
        let (total, misaligned) = text_quads(&harness);
        assert!(total > 500, "only {total} quads at {ppp}");
        assert_eq!(
            misaligned, 0,
            "{misaligned} of {total} text quads off the grid at {ppp}"
        );
    }
}

#[cfg(feature = "wgpu-tests")]
mod rendered {
    use super::{CHECKED_SCALES, icon_sheet};
    use egui_kittest::Harness;
    use std::path::PathBuf;
    use tallyline_review_table_spike::model::sample_rows;
    use tallyline_review_table_spike::table::ReviewTable;

    const MAGNIFICATION: u32 = 8;

    fn out_dir() -> std::io::Result<PathBuf> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/icon-check");
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn magnify(image: &image::RgbaImage) -> image::RgbaImage {
        image::imageops::resize(
            image,
            image.width() * MAGNIFICATION,
            image.height() * MAGNIFICATION,
            image::imageops::FilterType::Nearest,
        )
    }

    #[test]
    fn render_the_icon_sheet_and_the_table_at_each_checked_scale() {
        let dir = out_dir().unwrap();
        for ppp in CHECKED_SCALES {
            let percent = (ppp * 100.0).round() as u32;
            let mut sheet = Harness::builder()
                .with_size(egui::vec2(200.0, 130.0))
                .with_pixels_per_point(ppp)
                .wgpu()
                .build_ui(icon_sheet);
            sheet.run();
            let image = sheet.render().unwrap();
            image
                .save(dir.join(format!("icons-{percent}.png")))
                .unwrap();
            magnify(&image)
                .save(dir.join(format!("icons-{percent}-x{MAGNIFICATION}.png")))
                .unwrap();

            let mut table = Harness::builder()
                .with_size(egui::vec2(1000.0, 360.0))
                .with_pixels_per_point(ppp)
                .wgpu()
                .build_ui_state(
                    |ui, table: &mut ReviewTable| table.ui(ui),
                    ReviewTable::new(sample_rows(200)),
                );
            table.run_steps(4);
            table
                .render()
                .unwrap()
                .save(dir.join(format!("table-{percent}.png")))
                .unwrap();
        }
    }
}
