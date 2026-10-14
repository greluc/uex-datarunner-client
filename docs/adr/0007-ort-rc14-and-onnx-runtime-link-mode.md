# ADR-0007: `ort` 2.0.0-rc.14 with ONNX Runtime 1.31.0, loaded at run time

> **Doc type:** Decision record — Accepted. Last reviewed: 2026-10-14.

- **Status:** Accepted (2026-10-14)
- **Decider:** @greluc, in chat on 2026-10-14, after the M0 ONNX Runtime link-mode spike (issue #25) that [ADR-0005](0005-rust-and-egui.md) D2 and O-108 call for
- **Amends:** ADR-0005 D2, the `ort` version (`2.0.0-rc.13` becomes `2.0.0-rc.14`); the exception itself, its reason and the other rows of D2 are unchanged
- **Related:** O-108 and O-127 in [the register](0000-open-points.md), [09](../plan/09-engineering-principles.md) §3b, [10](../plan/10-supply-chain-security.md) S-1, S-2, S-7, S-22, S-24 and the spike result in §2.5, [02](../plan/02-architecture.md) §8, [01](../plan/01-requirements.md) R-SEC-8, [`docs/dependency-pins.md`](../dependency-pins.md)

## Context

ADR-0005 D2 registers `ort`, the Rust binding of ONNX Runtime, as a native-code exception and as the one exception to "stable releases only", pinned at `2.0.0-rc.13`, and leaves open how the ONNX Runtime library reaches the build and the installation. The library must be pinned by SHA-256 like the OCR assets (10 S-22), loaded only from the installation directory and never extracted to a shared temp directory (10 S-24, R-SEC-8).

The spike first ran on rc.13 with ONNX Runtime 1.28.3. During it, `ort` 2.0.0-rc.14 appeared (published 2026-10-12), which binds the ONNX Runtime 1.31 C API (1.31.0 published 2026-10-09).

## Method

The spike lives on the branch `prototype/ort-link-mode` (never merged; full record in its `README.md`). It read the published sources of `ort` and `ort-sys` rc.13 and rc.14, downloaded the official release archives of ONNX Runtime 1.28.3 and 1.31.0, hashed the archives and the libraries in them, checked them against independent sources, and ran a standalone crate on Windows 11 that verifies the library, loads it by absolute path and runs a one-node ONNX model.

## Findings

- With `load-dynamic`, `ort-sys` builds with `disable-linking`: no build step downloads or links ONNX Runtime. The same holds for rc.13 and rc.14.
- The default feature `download-binaries` fetches pyke's own builds from `cdn.pyke.io`, not Microsoft's release asset, and extracts the archive into a shared per-user cache before it compares the pinned hash; it also adds a TLS stack and three build dependencies and fails offline.
- `ort` keeps the first loaded library for the life of the process, and falls back to `ORT_DYLIB_PATH` or the bare library name without a hash check if any `ort` call precedes `init_from`.
- rc.14 makes the environment an owned value that `Session::builder` takes, one per process, with ONNX Runtime telemetry events on unless switched off.
- For 1.28.3 the Linux library had no second source of its hash (O-127). For 1.31.0 both libraries are byte-identical to the copies in the NuGet package `Microsoft.ML.OnnxRuntime` 1.31.0, whose Microsoft author and nuget.org repository signatures verify; the Windows library also carries a valid Microsoft Authenticode signature.

## Alternatives considered

- **Stay on rc.13 with ONNX Runtime 1.28.3** and move later in its own pull request: the recommendation of the spike, because rc.14 and 1.31.0 were inside the 7-day cooldown that S-7 sets for Dependabot. Not chosen.
- **`download-binaries`:** rejected for the reasons above.
- **Linking at load time against an official build** (`ORT_LIB_PATH`): rejected, because the OS loader resolves the library through its search order before our code can check its hash.

## Decision

1. **`ort` `=2.0.0-rc.14`** (with `ort-sys` `=2.0.0-rc.14`), `default-features = false`, features `load-dynamic` and `api-31`. It stays the one registered release-candidate exception (10 S-2); moving to a stable release is its own pull request.
2. **ONNX Runtime 1.31.0**, the official release asset of `microsoft/onnxruntime` (tag `v1.31.0`, commit `402979ac1bef89dcd55add41eaa7277c407599eb`), pinned by the SHA-256 of each archive and of each shipped library in `docs/dependency-pins.md` and `NOTICE`.
3. **Loaded at run time:** `onnxruntime.dll` (Windows x64) and `libonnxruntime.so.1.31.0` (Linux x64) sit next to the executable. `adapter-ocr`, the only crate that depends on `ort`, hashes the file, restricts the Windows DLL search path, loads it with `ort::init_from(<absolute path>)` before any other `ort` call, checks that the module the process mapped is the verified file, and only then builds the environment. `onnxruntime_providers_shared` is not shipped.
4. **Trust anchor (O-127):** the official GitHub release of `microsoft/onnxruntime` is accepted as the source of the library hashes, also where no second source exists; a second source is recorded where one exists.

## Consequences

- 09 §3b, 09 §12, 10 S-2 and §2.5, 02 §8, `NOTICE` 2.3, `docs/dependency-pins.md`, the 04 risk table and `CLAUDE.md` name rc.14 and 1.31.0.
- An `ort` or ONNX Runtime update changes the pins, `NOTICE` and the hashes in the code together, and repeats the load test.
- The Linux run of the spike, the import analysis of the Windows library (O-12) and the telemetry spike (O-13) are still open; `with_telemetry(false)` is set by the spike but is not yet shown to be enough.
