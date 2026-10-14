// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Magic-byte detection of the formats R-CAP-8 names, independent of the file name.

#![cfg(test)]

use tallyline_decoder_check_spike::format::{Format, detect};
use tallyline_decoder_check_spike::intake::HdrContainer;

fn iso_bmff(major: &[u8; 4], compatible: &[&[u8; 4]]) -> Vec<u8> {
    let size = u32::try_from(16 + 4 * compatible.len()).unwrap_or(u32::MAX);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&size.to_be_bytes());
    bytes.extend_from_slice(b"ftyp");
    bytes.extend_from_slice(major);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    for brand in compatible {
        bytes.extend_from_slice(*brand);
    }
    bytes.extend_from_slice(&[0, 0, 0, 8, b'm', b'e', b't', b'a']);
    bytes
}

#[test]
fn detects_the_supported_formats() {
    assert_eq!(detect(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR"), Format::Png);
    assert_eq!(detect(b"\xff\xd8\xff\xe0\0\x10JFIF\0"), Format::Jpeg);
    assert_eq!(detect(b"BM\x46\0\0\0\0\0\0\0\x36\0\0\0"), Format::Bmp);
}

#[test]
fn detects_jpeg_xr_by_its_tiff_like_header() {
    assert_eq!(
        detect(b"II\xbc\x01\x08\0\0\0"),
        Format::Hdr(HdrContainer::JpegXr)
    );
    assert_eq!(
        detect(b"II\xbc\x00\x08\0\0\0"),
        Format::Hdr(HdrContainer::JpegXr)
    );
}

#[test]
fn detects_avif_by_major_or_compatible_brand() {
    assert_eq!(
        detect(&iso_bmff(b"avif", &[b"mif1", b"miaf"])),
        Format::Hdr(HdrContainer::Avif)
    );
    assert_eq!(
        detect(&iso_bmff(b"mif1", &[b"avif", b"miaf"])),
        Format::Hdr(HdrContainer::Avif)
    );
    assert_eq!(
        detect(&iso_bmff(b"avis", &[])),
        Format::Hdr(HdrContainer::Avif)
    );
}

#[test]
fn detects_heif_brands_that_are_not_avif() {
    assert_eq!(
        detect(&iso_bmff(b"heic", &[b"mif1"])),
        Format::Hdr(HdrContainer::Heif)
    );
    assert_eq!(
        detect(&iso_bmff(b"mif1", &[b"heic"])),
        Format::Hdr(HdrContainer::Heif)
    );
}

#[test]
fn detects_openexr_by_its_magic_number() {
    assert_eq!(
        detect(b"\x76\x2f\x31\x01\x02\0\0\0"),
        Format::Hdr(HdrContainer::OpenExr)
    );
}

#[test]
fn reports_unknown_for_other_content() {
    assert_eq!(detect(b""), Format::Unknown);
    assert_eq!(detect(b"\x89PN"), Format::Unknown);
    assert_eq!(detect(b"GIF89a"), Format::Unknown);
    assert_eq!(detect(&iso_bmff(b"isom", &[b"mp41"])), Format::Unknown);
    assert_eq!(detect(b"\xff\x0a"), Format::Unknown);
}
