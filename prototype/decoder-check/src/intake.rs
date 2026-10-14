// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Intake outcomes of R-CAP-8: an 8-bit sRGB raster or a file set aside with its reason.

/// An 8-bit sRGB raster, three bytes per pixel, rows top to bottom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SrgbRaster {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Interleaved R, G, B samples, `width * height * 3` bytes.
    pub rgb: Vec<u8>,
}

/// The result of reading one file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Intake {
    /// The file decoded to sRGB.
    Decoded(SrgbRaster),
    /// The file is not read and is set aside with this reason.
    SetAside(SetAsideReason),
}

/// HDR container formats that are recognised but never decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdrContainer {
    /// JPEG XR (`.jxr`).
    JpegXr,
    /// AVIF (`.avif`).
    Avif,
    /// HEIF other than AVIF (`.heic`, `.heif`).
    Heif,
    /// OpenEXR (`.exr`).
    OpenExr,
}

/// Why a file is set aside.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetAsideReason {
    /// "HDR screenshot format (JPEG XR/AVIF/EXR) – not supported".
    HdrFormat(HdrContainer),
    /// "HDR or unsupported colour encoding – use an SDR copy".
    HdrOrUnsupportedEncoding(Encoding),
    /// The content is none of the supported or recognised formats.
    UnsupportedFormat,
    /// The file is larger than the file-size limit.
    FileTooLarge,
    /// The header declares more pixels than the hard limit.
    TooManyPixels,
    /// An embedded colour profile is larger than the profile limit.
    ProfileTooLarge,
    /// The decoder failed, for example on a truncated file.
    DecodeFailed(String),
}

/// The colour encoding that made a file unreadable as sRGB.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// A PNG `cICP` with a PQ (16) or HLG (18) transfer.
    CicpHdr {
        /// Colour primaries code point.
        primaries: u8,
        /// Transfer characteristics code point.
        transfer: u8,
    },
    /// A PNG `cICP` other than BT.709 primaries with sRGB transfer at full range.
    CicpOther {
        /// Colour primaries code point.
        primaries: u8,
        /// Transfer characteristics code point.
        transfer: u8,
        /// Whether the image is full range.
        full_range: bool,
    },
    /// An ICC profile whose `cicp` tag names a PQ or HLG transfer.
    IccHdr,
    /// An ICC profile the colour library cannot read or convert.
    IccUnusable(String),
    /// An ICC profile of a colour space other than RGB.
    IccNotRgb,
    /// A PNG `cHRM` whose primaries are not BT.709.
    ChromaticitiesNotBt709,
    /// A colour chunk or marker is present that the decoder dropped as malformed.
    MalformedColourMetadata(String),
    /// PNG `mDCV` or `cLLI` HDR metadata without a `cICP`.
    HdrMetadataWithoutCicp,
    /// A JPEG in CMYK or YCCK.
    Cmyk,
    /// A BMP colour space other than sRGB or the Windows default.
    BmpColourSpace(u32),
    /// A greyscale image with an embedded ICC profile.
    GreyWithIcc,
}

/// Hard limits checked before pixel data is decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntakeLimits {
    /// Largest accepted file in bytes.
    pub max_file_bytes: u64,
    /// Largest accepted declared pixel count.
    pub max_pixels: u64,
    /// Largest accepted embedded ICC profile in bytes, compressed or not.
    pub max_profile_bytes: usize,
}

impl Default for IntakeLimits {
    fn default() -> Self {
        Self {
            max_file_bytes: 200 * 1024 * 1024,
            max_pixels: 100_000_000,
            max_profile_bytes: 1024 * 1024,
        }
    }
}
