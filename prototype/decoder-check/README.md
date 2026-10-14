# Decoder check spike (issue #28)

Spike code for the M0 item "HDR and format detection with Rust decoders" ([04](../../docs/plan/04-roadmap.md) M0; R-CAP-8; ADR-0005 D1; [07](../../docs/plan/07-ocr-concept.md) §4). It lives on the branch `prototype/decoder-check` and never merges into `main`; only the result is carried into the documents that own the facts (02, 07 §4, 04 M0, the open-points register). It gates M2.

## What it is

A standalone crate (its own `[workspace]` and `Cargo.lock`) with one entry point, `decode::read(bytes, limits) -> Intake`: it detects the format from the content (never from a file name), decodes PNG, JPEG and BMP to an 8-bit sRGB raster with the colour step of R-CAP-8, or sets the file aside with a reason. The fixtures are generated inside the tests (no committed binary files):

- `tests/format.rs`: magic bytes of PNG, JPEG, BMP, JPEG XR (`II BC 01` and `II BC 00`), AVIF and HEIF (ISO-BMFF `ftyp` major or compatible brand), OpenEXR (`76 2F 31 01`), and unknown content.
- `tests/png.rs`: 8 and 16 bit, `cICP` PQ, HLG, sRGB and others, `iCCP` (linear and Display P3 profiles, a profile with a PQ `cicp` tag), `sRGB`, `gAMA`, `cHRM`, the R-CAP-8 precedence, malformed colour chunks, `cLLI` without `cICP`, greyscale, truncation and the three hard limits.
- `tests/jpeg.rs`: baseline 4:4:4 and 4:2:0 and progressive, an APP2 ICC profile (Display P3, sRGB, PQ), a corrupt APP2 sequence, truncation and the pixel limit.
- `tests/bmp_and_containers.rs`: 24-bit BMP, V5 headers per colour space, the HDR container stubs, JPEG content that would sit in a `.png` file, and an unsupported format.

The ICC profiles of the fixtures are written by hand in `tests/common/mod.rs` (ICC v2 matrix/TRC: `desc`, `wtpt`, colorants, `curv` or `para` curves), so the colour library is not only tested against profiles it wrote itself; only the PQ profile comes from `moxcms`, because the hand-written profiles carry no `cicp` tag. The expected values are computed in `f64` in the tests.

## Chosen crates (read at source 2026-10-14)

| Role | Crate | Licence | Why |
|---|---|---|---|
| PNG | `png` `=0.18.1` (image-rs) | MIT OR Apache-2.0 | Pure Rust, `#![forbid(unsafe_code)]`. Parses `cICP`, `iCCP`, `sRGB`, `gAMA`, `cHRM`, `mDCV` and `cLLI` into `Info` and applies none of them to the pixels; 16-bit samples come out in full (big-endian). |
| JPEG | `zune-jpeg` `=0.5.15` (with `zune-core` `=0.5.3`) | MIT OR Apache-2.0 OR Zlib | Pure Rust; the JPEG decoder of `image` 0.25. Baseline, progressive and all tested subsamplings; reassembles a multi-segment APP2 ICC profile. 0.5.15 is the latest stable release (0.5.16-rc2 is a pre-release). |
| BMP | `image` `=0.25.10`, default features off, only `bmp` | MIT OR Apache-2.0 | Its own pure-Rust BMP codec with a header-only `dimensions()`. The only other BMP crate, `zune-bmp` 0.5.2, has about 98,000 downloads and does not count as mature. |
| ICC | `moxcms` `=0.8.1` | BSD-3-Clause OR Apache-2.0 | Pure Rust; the colour engine of `image` (also `typst`, `jxl-oxide`; 47 dependent crates). 8- and 16-bit transforms, parses the ICC v4.4 `cicp` tag. 0.9.1 is current, but `image` 0.25.10 requires `^0.8`, and 0.8.1 keeps one copy in the build. |

Rejected: `qcms` 0.3.0 (MIT, Firefox's colour engine) handles 8-bit data types only (`RGB8`, `RGBA8`, `BGRA8`, `Gray8`, `GrayA8`, `CMYK`), so a 16-bit `iCCP` PNG would have to be truncated before its conversion, against R-CAP-8; `lcms2` is a binding to Little CMS (C) and needs no exception when a pure-Rust engine exists (D1).

**Native code: none.** No crate in the normal dependency set has a `links` key or a `-sys` dependency. Two have a build script, both read: `crc32fast` 1.5.2 and `num-traits` 0.2.19 (through `autocfg`) only ask `rustc` for its version and set `cfg` flags; neither downloads or compiles anything.

**Licences of the shipped set** (all on the allowlist; for an `OR` expression the first allowed option): `adler2` 0BSD OR MIT OR Apache-2.0, `bitflags`, `cfg-if`, `crc32fast`, `fdeflate`, `flate2`, `image`, `num-traits`, `png` MIT OR Apache-2.0, `bytemuck` Zlib OR Apache-2.0 OR MIT, `byteorder-lite` Unlicense OR MIT, `miniz_oxide` 0.8.9 and 0.9.1 MIT OR Zlib OR Apache-2.0, `moxcms` and `pxfm` BSD-3-Clause OR Apache-2.0, `simd-adler32` MIT, `zune-core` and `zune-jpeg` MIT OR Apache-2.0 OR Zlib.

**`unsafe` surface:** `png` and `fdeflate` forbid `unsafe` code and `image`'s BMP codec contains none; `zune-jpeg` (85 occurrences, `forbid(unsafe_code)` only without its default `x86` and `neon` features), `zune-core` (10), `moxcms` (344) and its maths crate `pxfm` (224) use `unsafe` for SIMD intrinsics. That is Rust, not native code, so D1 needs no exception, but the `cargo-vet` review in M2 has to cover it. `zune-jpeg`'s SIMD can be switched off (`DecoderOptions::set_use_unsafe(false)` or without its `x86`/`neon` features); `moxcms`'s cannot, because `image` depends on it with default features.

**Test-only dependency:** `jpeg-encoder` `=0.7.1` writes the JPEG fixtures (progressive, 4:4:4 and 4:2:0, APP2 ICC). Its licence is `(MIT OR Apache-2.0) AND IJG`; the SPDX licence list marks IJG as FSF-free but not OSI-approved, so under the licence policy for build and test tools it needs @greluc's approval before M2 uses it on `main`. It is never shipped.

## Result (2026-10-14)

All 46 tests pass on Windows 11 (local, the pinned toolchain 1.99.0) with `cargo clippy --all-targets -- -D warnings` and `RUSTDOCFLAGS=-D warnings cargo doc` clean. The decoders are pure Rust, so the result does not depend on the OS; the workflow `.github/workflows/decoder-check-spike.yml` repeats it on `ubuntu-latest` and `windows-latest` on a push to this branch.

### 07 §4 decode fixtures

| 07 §4 item | Covered by | Result |
|---|---|---|
| 1. 8- and 16-bit PNG of one image differ by at most 1 | `eight_and_sixteen_bit_pngs_of_one_image_differ_by_at_most_one`, `sixteen_bit_samples_are_scaled_not_truncated_to_the_high_byte` | Pass. Sample 511 becomes 2, not the high byte 1. A synthetic image, not a corpus image. |
| 2. 16-bit PNG with a linear `iCCP` within 2 of the plain PNG | `a_16_bit_png_with_a_linear_icc_profile_decodes_to_the_srgb_source_within_two` | Pass, through `moxcms`'s 16-bit transform. |
| 3. `cICP` 9/16/0/1 (PQ) never read as sRGB | `a_cicp_pq_png_is_never_read_as_srgb`, also HLG 9/18/0/1, narrow range, P3 primaries and linear transfer | Pass; set aside. `cICP` 1/13/0/1 decodes unchanged. |
| 4. JPEG with APP2 ICC, progressive, 4:4:4 and 4:2:0 within 3 mean absolute error | `jpeg.rs` | Pass at quality 95. |
| 5. JPEG XR, AVIF, OpenEXR stubs by magic bytes; JPEG in `.png` decoded as JPEG | `format.rs`, `hdr_container_stubs_are_set_aside_with_the_hdr_format_reason`, `jpeg_data_in_a_file_named_png_is_decoded_as_jpeg` | Pass, HEIF included. `read` never sees a file name. |
| 6. Pixel budget: reduction at decode time, 7680×4320 within the heap limit | – | **Not run.** Needs the R-OCR-17 budget and the downscale path; stays for M2 (see "Not covered"). |
| 7. Truncated PNG and JPEG, too many pixels, oversized `iCCP` set aside, the oversized ones without decoding pixels | `a_truncated_png_is_set_aside`, `a_truncated_jpeg_is_set_aside`, `a_header_declaring_too_many_pixels_is_set_aside_without_decoding`, `an_oversized_iccp_chunk_is_set_aside_without_decoding`, `an_iccp_that_inflates_beyond_the_limit_is_set_aside`, `a_file_above_the_size_limit_is_set_aside`, `a_jpeg_above_the_pixel_limit_is_set_aside` | Pass. The "without decoding" fixtures cut the file inside `IDAT`, so any attempt to decode would fail with a different reason. |

Tone fixtures (07 §4) checked here: PNG with a Display P3 `iCCP` and with `gAMA` only. The others (PQ-as-sRGB, linear scaling, washout) are synthetic transforms of an SDR image, not decoder questions, and stay with the M2 eval harness.

### Gaps found, and how the spike closes them

Each guard was checked by switching it off: the fixtures named fail without it.

1. **`png` drops malformed colour chunks silently, and the file then reads as plain sRGB.** `parse_iccp` ignores the result of `parse_iccp_raw` (`let _ = …`), an ancillary chunk of the wrong length is rejected without an error, and a format error in an ancillary chunk (for example `gAMA` 0, a duplicate, `cICP` after `PLTE`) becomes `Decoded::BadAncillaryChunk` (`decoder/stream.rs`). With the guard off, `cICP` of 3 bytes, `iCCP` with a broken zlib stream, `gAMA` 0 and `cHRM` of 31 bytes all decoded unchanged. **Guard:** a chunk walk before `IDAT` lists every colour chunk (`cICP`, `iCCP`, `sRGB`, `gAMA`, `cHRM`, `mDCV`, `cLLI`); each one present must appear in `png::Info`, and none may appear twice, otherwise the file is set aside as malformed colour metadata (`colour_chunks_the_decoder_drops_as_malformed_are_set_aside`). The same walk reads the `IHDR` dimensions and the `iCCP` chunk length before the decoder runs.
2. **`zune-jpeg` decodes a truncated file without an error by default.** Its default options are error-tolerant (`decoder_error_tolerance_mode`); a JPEG cut at two thirds, baseline or progressive, returned a full-size buffer. **Guard:** `set_strict_mode(true)`, which returns "Premature end of buffer" (baseline) or "Exhausted data in the image" (progressive) (`a_truncated_jpeg_is_set_aside`).
3. **`zune-jpeg` reports a corrupt APP2 ICC profile as no profile.** `icc_profile()` returns `None` for a zero or duplicate sequence number or a missing segment, the same as for a file without a profile. **Guard:** a marker walk up to `SOS` counts the `ICC_PROFILE` APP2 segments; if there are any and no profile assembles, the file is set aside (`a_corrupt_app2_profile_sequence_is_set_aside`).
4. **`image`'s BMP decoder ignores the colour space of a V4/V5 header** (`bV5CSType`, calibration endpoints and gammas, a linked or embedded profile). **Guard in the spike:** `sRGB` and `Win ` decode; `LCS_CALIBRATED_RGB` with all endpoints and gammas zero counts as sRGB; every other colour space is set aside. The zero-calibration rule is an assumption about what common writers emit and is unverified; R-CAP-8 does not mention BMP colour spaces.
5. **A PQ or HLG ICC profile is recognised only by its `cicp` tag** (ICC v4.4). A profile that describes PQ through its curves or a LUT without that tag would be converted and clipped like an SDR profile. Not tested; it needs a detection rule (for example from the `lumi` tag or the curve shape) and a fixture in M2.
6. **R-CAP-8 does not say what `mDCV` or `cLLI` without `cICP` means.** Both are HDR mastering metadata; the spike sets such a file aside as "HDR or unsupported colour encoding" (`hdr_metadata_without_cicp_is_set_aside`). This is a requirement decision for @greluc.
7. **`png`'s `Limits` is one byte budget shared by the decompressed `iCCP`, text chunks and the image buffer** (default 64 MiB). The adapter has to set it from its own limits (the spike: pixel limit × 8 bytes plus the profile limit) and check the inflated profile length itself: a 4 MiB profile from a 4 KB chunk passes `png`'s budget and is caught by the profile limit after the header, before pixels are decoded. The spike's profile limit is 1 MiB; 02 §9 fixes the start value in M2.

### Findings for 07 §4

- **The Display P3 tone fixture must compare with the exact conversion of the stored samples, not with the source image.** sRGB content stored as 8-bit Display P3 and converted back exactly (in `f64`) differs from the source by up to 8 code values in dark saturated channels; that is the quantisation of the 8-bit P3 file. `moxcms` matches the exact conversion within 1 code value (8-bit fixed-point and the 16-bit path), within 2 with floating point.
- **A `gAMA` of 45455 is a pure 2.2 power curve, not the sRGB curve.** Applied as R-CAP-8 prescribes, a file that a tool tagged with `gAMA` alone while it meant sRGB comes out darker in the shadows (code value 10 becomes 3). The `gAMA`-only tone fixture has to generate its samples with the power curve, and the effect on real captures belongs in the M2 corpus check.
- **Malformed colour metadata needs fixtures of its own:** a `cICP` of the wrong length, an `iCCP` with a broken zlib stream, `gAMA` 0, a short `cHRM`, a duplicate colour chunk, and a JPEG whose APP2 ICC segments do not assemble; each is set aside, never read as sRGB.
- **An ICC profile with a PQ or HLG `cicp` tag**, in a PNG `iCCP` and in a JPEG APP2 segment, is set aside like a PQ `cICP`.
- Item 1 asks for a corpus image; the spike used a synthetic gradient. M2 uses the public corpus image as written.

### Not covered (M2)

- 07 §4 item 6 (pixel budget): whether `zune-jpeg` can decode at a reduced scale, and the peak memory of a 7680×4320 decode, were not examined.
- Greyscale with an ICC profile (set aside in the spike; a grey profile would need `moxcms`'s `Gray` layout), alpha (dropped, not composited), interlaced PNG, palette PNG with `iCCP`, 16-bit greyscale, CMYK JPEG (set aside, no fixture), JPEG with an Ultra HDR gain map (its base image is SDR and is read as such; the gain map is ignored).
- JPEG XL is not recognised and is set aside as an unsupported format with the generic message; R-CAP-8 names JPEG XR, AVIF, OpenEXR and HEIF only. Folder import lists `.jxr`, `.avif` and `.exr` but not `.heic`/`.heif`, while content detection recognises HEIF.

## Commands

```powershell
cargo test --locked --manifest-path prototype/decoder-check/Cargo.toml
cargo clippy --locked --manifest-path prototype/decoder-check/Cargo.toml --all-targets -- -D warnings
```
