// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spike for R-CAP-8: decode PNG, JPEG and BMP to 8-bit sRGB with pure-Rust decoders and set HDR encodings aside.

mod bmp_input;
mod colour;
pub mod decode;
pub mod format;
pub mod intake;
mod jpeg_input;
mod png_input;
