// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The intake entry point: detect the format by content, then decode or set aside (R-CAP-8).

use crate::format::{Format, detect};
use crate::intake::{Intake, IntakeLimits, SetAsideReason, SrgbRaster};
use crate::{bmp_input, jpeg_input, png_input};

/// Reads one file's bytes into an 8-bit sRGB raster or sets it aside; the file name is never consulted.
#[must_use]
pub fn read(bytes: &[u8], limits: &IntakeLimits) -> Intake {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limits.max_file_bytes {
        return Intake::SetAside(SetAsideReason::FileTooLarge);
    }
    let decoded = match detect(bytes) {
        Format::Png => png_input::read(bytes, limits),
        Format::Jpeg => jpeg_input::read(bytes, limits),
        Format::Bmp => bmp_input::read(bytes, limits),
        Format::Hdr(container) => Err(SetAsideReason::HdrFormat(container)),
        Format::Unknown => Err(SetAsideReason::UnsupportedFormat),
    };
    match decoded {
        Ok((width, height, rgb)) => Intake::Decoded(SrgbRaster { width, height, rgb }),
        Err(reason) => Intake::SetAside(reason),
    }
}
