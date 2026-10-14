// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spike for O-114: the Fan Kit unit and its two placements drawn with egui, driven by `egui_kittest`.

use egui::{Color32, ColorImage, FontId, Label, RichText, TextureHandle, TextureOptions, Ui};

/// The trademark line of the Fan Kit Guidelines §2b.
pub const FAN_KIT_TRADEMARK_LINE: &str = "Star Citizen®, Roberts Space Industries® and Cloud Imperium ® are registered trademarks of Cloud Imperium Rights LLC";

/// The notice of the Fankit Agreement clause 2(g).
pub const FAN_KIT_AGREEMENT_NOTICE: &str = "This site is not endorsed by or affiliated with the Cloud Imperium or Roberts Space Industries group of companies. All game content and materials are copyright Cloud Imperium Rights LLC and Cloud Imperium Rights Ltd.. Star Citizen®, Squadron 42®, Roberts Space Industries®, and Cloud Imperium® are registered trademarks of Cloud Imperium Rights LLC. All rights reserved.";

/// The "Made By The Community" logo file, byte for byte.
pub const FAN_KIT_LOGO_PNG: &[u8] =
    include_bytes!("../../../docs/images/fankit/made-by-the-community.png");

/// The accessible name of the logo image.
pub const FAN_KIT_LOGO_ALT_TEXT: &str = "Made By The Community";

/// The name of the logo texture in the egui texture manager.
pub const FAN_KIT_LOGO_TEXTURE_NAME: &str = "fan-kit-logo";

/// The edge length of the drawn logo in egui points.
pub const FAN_KIT_LOGO_SIZE: f32 = 96.0;

/// The font size of both notices in egui points.
pub const FAN_KIT_NOTICE_FONT_SIZE: f32 = 14.0;

/// Decodes the logo file into egui pixels.
///
/// # Errors
/// Returns the decoder error if the embedded file is not a readable PNG.
pub fn decode_logo(png: &[u8]) -> Result<ColorImage, image::ImageError> {
    let rgba = image::load_from_memory_with_format(png, image::ImageFormat::Png)?.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Ok(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

/// The Fan Kit unit: logo, trademark line and agreement notice, always drawn together.
pub struct FanKitUnit {
    logo: TextureHandle,
    trademark_line: &'static str,
    agreement_notice: &'static str,
    notice_font_size: f32,
}

impl FanKitUnit {
    /// Uploads the logo texture and returns the unit with the pinned notices.
    ///
    /// # Errors
    /// Returns the decoder error if the logo file cannot be decoded.
    pub fn load(ctx: &egui::Context) -> Result<Self, image::ImageError> {
        Self::load_with(
            ctx,
            FAN_KIT_LOGO_PNG,
            FAN_KIT_TRADEMARK_LINE,
            FAN_KIT_AGREEMENT_NOTICE,
        )
    }

    /// Uploads the given logo and returns a unit with the given texts; the probes of the spike use it to build a broken unit.
    ///
    /// # Errors
    /// Returns the decoder error if the logo bytes cannot be decoded.
    pub fn load_with(
        ctx: &egui::Context,
        png: &[u8],
        trademark_line: &'static str,
        agreement_notice: &'static str,
    ) -> Result<Self, image::ImageError> {
        let logo = ctx.load_texture(
            FAN_KIT_LOGO_TEXTURE_NAME,
            decode_logo(png)?,
            TextureOptions::LINEAR,
        );
        Ok(Self {
            logo,
            trademark_line,
            agreement_notice,
            notice_font_size: FAN_KIT_NOTICE_FONT_SIZE,
        })
    }

    /// Returns the unit with another notice font size; the probes of the spike use it to build a broken unit.
    #[must_use]
    pub fn with_notice_font_size(mut self, size: f32) -> Self {
        self.notice_font_size = size;
        self
    }

    /// Draws the three parts in one vertical group.
    pub fn show(&self, ui: &mut Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.add(
                    egui::Image::new((
                        self.logo.id(),
                        egui::vec2(FAN_KIT_LOGO_SIZE, FAN_KIT_LOGO_SIZE),
                    ))
                    .alt_text(FAN_KIT_LOGO_ALT_TEXT),
                );
                ui.add(Label::new(notice_text(self.trademark_line, self.notice_font_size)).wrap());
                ui.add(
                    Label::new(notice_text(self.agreement_notice, self.notice_font_size)).wrap(),
                );
            });
        })
        .response
    }
}

fn notice_text(text: &str, size: f32) -> RichText {
    RichText::new(text)
        .font(FontId::proportional(size))
        .color(Color32::WHITE)
}

/// The state of the onboarding start screen.
#[derive(Debug, Default)]
pub struct OnboardingState {
    /// The secret key typed so far.
    pub secret_key: String,
    /// The number of times "Continue" was activated.
    pub continue_activations: u32,
}

/// The label of the secret-key field.
pub const SECRET_KEY_LABEL: &str = "UEX secret key";

/// The label of the continue button.
pub const CONTINUE_LABEL: &str = "Continue";

/// Draws the onboarding start screen with the Fan Kit unit below the form.
pub fn onboarding_start_view(ui: &mut Ui, state: &mut OnboardingState, unit: &FanKitUnit) {
    ui.heading("Welcome to Tallyline");
    let label = ui.label(SECRET_KEY_LABEL);
    ui.add(egui::TextEdit::singleline(&mut state.secret_key).password(true))
        .labelled_by(label.id);
    if ui.button(CONTINUE_LABEL).clicked() {
        state.continue_activations += 1;
    }
    ui.separator();
    unit.show(ui);
}

/// Draws the About dialog content with the Fan Kit unit.
pub fn about_view(ui: &mut Ui, unit: &FanKitUnit) {
    ui.heading("About Tallyline");
    ui.label("Tallyline is free software under GPL-3.0-or-later and comes with no warranty.");
    ui.separator();
    unit.show(ui);
}

/// Draws the About dialog content with the logo but without the notices; a broken placement for the probes.
pub fn about_view_logo_only(ui: &mut Ui, unit: &FanKitUnit) {
    ui.heading("About Tallyline");
    ui.add(
        egui::Image::new((
            unit.logo.id(),
            egui::vec2(FAN_KIT_LOGO_SIZE, FAN_KIT_LOGO_SIZE),
        ))
        .alt_text(FAN_KIT_LOGO_ALT_TEXT),
    );
}

/// Draws the About dialog content with the unit folded behind a collapsing header; a broken placement for the probes.
pub fn about_view_folded(ui: &mut Ui, unit: &FanKitUnit) {
    ui.heading("About Tallyline");
    ui.collapsing("Legal", |ui| {
        unit.show(ui);
    });
}
