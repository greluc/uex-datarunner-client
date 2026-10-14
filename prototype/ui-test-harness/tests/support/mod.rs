// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared helpers of the spike tests: a texture-recording renderer, hashing and the Fan Kit unit check.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use egui::accesskit::Role;
use egui::epaint::{ImageData, Shape, TextureId, textures::TexturesDelta};
use egui::{ColorImage, Rect};
use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::{Harness, TestRenderer};
use sha2::{Digest, Sha256};
use tallyline_ui_harness_spike::{
    FAN_KIT_AGREEMENT_NOTICE, FAN_KIT_LOGO_ALT_TEXT, FAN_KIT_LOGO_TEXTURE_NAME,
    FAN_KIT_TRADEMARK_LINE, decode_logo,
};

/// SHA-256 of `made-by-the-community.png` as pinned in `NOTICE` §4.
pub const LOGO_FILE_SHA256: &str =
    "b6869016da2220a61dc6306f8365dcce006ddb6d8b405545c6d4540166469b6b";

/// SHA-256 of the §2b trademark line.
pub const TRADEMARK_LINE_SHA256: &str =
    "154deedbb183aeec2ba7f05c3d16416e55ceee634a36ec1f9ee4785b26467fa1";

/// SHA-256 of the clause 2(g) notice.
pub const AGREEMENT_NOTICE_SHA256: &str =
    "703b53f047769fe1a5299b53454cc79a36e2b3a09129291baf2c6948ce740933";

/// The lower bound for the notice font size this spike checks against, in egui points.
pub const MIN_NOTICE_FONT_SIZE: f32 = 10.0;

/// Lower-case hex SHA-256 of the bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Lower-case hex SHA-256 of the RGBA bytes of an egui image.
pub fn pixels_sha256(image: &ColorImage) -> String {
    let bytes: Vec<u8> = image
        .pixels
        .iter()
        .flat_map(|pixel| pixel.to_array())
        .collect();
    format!("{}x{}:{}", image.size[0], image.size[1], sha256_hex(&bytes))
}

/// The logo file read from the repository at test time, independent of the embedded constant.
pub fn logo_file_on_disk() -> Vec<u8> {
    std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/images/fankit/made-by-the-community.png"
    ))
    .unwrap()
}

/// The full textures the harness uploaded, by texture id.
#[derive(Clone, Default)]
pub struct RecordedTextures(Arc<Mutex<HashMap<TextureId, ColorImage>>>);

impl RecordedTextures {
    /// The recorded image of a texture, if it was uploaded.
    pub fn get(&self, id: TextureId) -> Option<ColorImage> {
        self.0.lock().unwrap().get(&id).cloned()
    }
}

/// A headless renderer that keeps every texture upload instead of drawing.
pub struct RecordingRenderer(pub RecordedTextures);

impl TestRenderer for RecordingRenderer {
    fn handle_delta(&mut self, delta: &mut TexturesDelta) {
        let mut textures = self.0.0.lock().unwrap();
        for (id, image_deltas) in delta.set.drain() {
            for image_delta in image_deltas {
                let ImageData::Color(image) = image_delta.image;
                match image_delta.pos {
                    None => {
                        textures.insert(id, (*image).clone());
                    }
                    Some(_) => {
                        textures.remove(&id);
                    }
                }
            }
        }
        for id in delta.free.drain() {
            textures.remove(&id);
        }
    }

    #[cfg(feature = "wgpu")]
    fn render(
        &mut self,
        _ctx: &egui::Context,
        _output: &egui::FullOutput,
    ) -> Result<image::RgbaImage, String> {
        Err("the recording renderer does not draw".to_owned())
    }
}

fn flatten<'a>(shape: &'a Shape, out: &mut Vec<&'a Shape>) {
    if let Shape::Vec(shapes) = shape {
        for inner in shapes {
            flatten(inner, out);
        }
    } else {
        out.push(shape);
    }
}

fn painted_shapes<'h, State>(harness: &'h Harness<'_, State>) -> Vec<(Rect, &'h Shape)> {
    let mut out = Vec::new();
    for clipped in &harness.output().shapes {
        let mut flat = Vec::new();
        flatten(&clipped.shape, &mut flat);
        out.extend(flat.into_iter().map(|shape| (clipped.clip_rect, shape)));
    }
    out
}

/// Checks that the Fan Kit unit is shown completely, byte-exact and together in the current frame.
///
/// # Errors
/// Returns the first deviation found, as a message for the assertion.
pub fn check_fan_kit_unit<State>(
    harness: &Harness<'_, State>,
    textures: &RecordedTextures,
) -> Result<(), String> {
    let screen = harness.ctx.content_rect();

    let trademark = single_node(harness, FAN_KIT_TRADEMARK_LINE, "the trademark line")?;
    let agreement = single_node(harness, FAN_KIT_AGREEMENT_NOTICE, "the agreement notice")?;
    let logo = single_node(harness, FAN_KIT_LOGO_ALT_TEXT, "the logo")?;

    if logo.accesskit_node().role() != Role::Image {
        return Err(format!(
            "the logo node has role {:?}",
            logo.accesskit_node().role()
        ));
    }
    for (name, node) in [
        ("trademark line", &trademark),
        ("agreement notice", &agreement),
    ] {
        if node.accesskit_node().role() != Role::Label {
            return Err(format!(
                "the {name} node has role {:?}",
                node.accesskit_node().role()
            ));
        }
    }

    let parents: Vec<_> = [&logo, &trademark, &agreement]
        .iter()
        .map(|node| node.accesskit_node().parent_id())
        .collect();
    if parents.iter().any(|parent| *parent != parents[0]) {
        return Err(format!(
            "the three parts have different parents: {parents:?}"
        ));
    }
    let siblings = logo
        .accesskit_node()
        .parent()
        .map_or(0, |parent| parent.children().count());
    if siblings != 3 {
        return Err(format!(
            "the group of the three parts holds {siblings} nodes, not only the unit"
        ));
    }

    for (name, node) in [
        ("logo", &logo),
        ("trademark line", &trademark),
        ("agreement notice", &agreement),
    ] {
        let rect = node.rect();
        if !screen.contains_rect(rect) {
            return Err(format!(
                "the {name} at {rect:?} is not inside the screen {screen:?}"
            ));
        }
    }

    let shapes = painted_shapes(harness);

    for (name, text) in [
        ("trademark line", FAN_KIT_TRADEMARK_LINE),
        ("agreement notice", FAN_KIT_AGREEMENT_NOTICE),
    ] {
        let painted: Vec<_> = shapes
            .iter()
            .filter_map(|(clip, shape)| match shape {
                Shape::Text(text_shape) if text_shape.galley.job.text == text => {
                    Some((clip, text_shape))
                }
                _ => None,
            })
            .collect();
        let [(clip, text_shape)] = painted.as_slice() else {
            return Err(format!(
                "the {name} is painted {} times as one text",
                painted.len()
            ));
        };
        let bounds = text_shape.visual_bounding_rect();
        if !clip.contains_rect(bounds) {
            return Err(format!("the {name} at {bounds:?} is clipped by {clip:?}"));
        }
        for section in &text_shape.galley.job.sections {
            if section.format.font_id.size < MIN_NOTICE_FONT_SIZE {
                return Err(format!(
                    "the {name} uses font size {}",
                    section.format.font_id.size
                ));
            }
        }
    }

    let manager = harness.ctx.tex_manager();
    let is_logo = |id: TextureId| {
        manager
            .read()
            .meta(id)
            .is_some_and(|meta| meta.name == FAN_KIT_LOGO_TEXTURE_NAME)
    };
    let logo_paints: Vec<_> = shapes
        .iter()
        .filter_map(|(clip, shape)| match shape {
            Shape::Rect(rect) => rect.brush.as_ref().and_then(|brush| {
                is_logo(brush.fill_texture_id).then_some(LogoPaint {
                    clip: *clip,
                    texture: brush.fill_texture_id,
                    rect: rect.rect,
                    uv: brush.uv,
                    tint: rect.fill,
                })
            }),
            Shape::Mesh(mesh) if is_logo(mesh.texture_id) => Some(LogoPaint {
                clip: *clip,
                texture: mesh.texture_id,
                rect: mesh.calc_bounds(),
                uv: Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint: mesh
                    .vertices
                    .first()
                    .map_or(egui::Color32::WHITE, |vertex| vertex.color),
            }),
            _ => None,
        })
        .collect();
    let [paint] = logo_paints.as_slice() else {
        return Err(format!(
            "the logo texture is painted {} times",
            logo_paints.len()
        ));
    };
    if !paint.clip.contains_rect(paint.rect) || !screen.contains_rect(paint.rect) {
        return Err(format!(
            "the logo at {:?} is clipped or off screen",
            paint.rect
        ));
    }
    if !paint.rect.contains(logo.rect().center()) {
        return Err("the painted logo is not where the logo node is".to_owned());
    }
    if paint.tint != egui::Color32::WHITE {
        return Err(format!("the logo is tinted {:?}", paint.tint));
    }
    if paint.uv != Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)) {
        return Err(format!(
            "the logo is cropped or flipped by uv {:?}",
            paint.uv
        ));
    }
    let aspect = paint.rect.width() / paint.rect.height();
    if (aspect - 1.0).abs() > f32::EPSILON {
        return Err(format!("the logo is distorted to aspect ratio {aspect}"));
    }
    let Some(uploaded) = textures.get(paint.texture) else {
        return Err("the logo texture was never uploaded in full".to_owned());
    };
    let file = logo_file_on_disk();
    if sha256_hex(&file) != LOGO_FILE_SHA256 {
        return Err("the logo file on disk does not have the pinned SHA-256".to_owned());
    }
    let expected = decode_logo(&file).map_err(|error| error.to_string())?;
    if pixels_sha256(&uploaded) != pixels_sha256(&expected) {
        return Err(format!(
            "the uploaded logo pixels {} differ from the pinned file {}",
            pixels_sha256(&uploaded),
            pixels_sha256(&expected)
        ));
    }

    Ok(())
}

struct LogoPaint {
    clip: Rect,
    texture: TextureId,
    rect: Rect,
    uv: Rect,
    tint: egui::Color32,
}

fn single_node<'h, State>(
    harness: &'h Harness<'_, State>,
    label: &'h str,
    name: &str,
) -> Result<egui_kittest::Node<'h>, String> {
    let mut nodes: Vec<_> = harness.query_all_by_label(label).collect();
    match nodes.len() {
        1 => Ok(nodes.remove(0)),
        count => Err(format!(
            "{name} appears {count} times in the accessibility tree"
        )),
    }
}
