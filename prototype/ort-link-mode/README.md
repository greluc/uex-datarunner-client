# ONNX Runtime link-mode spike (issue #25)

Spike code for the M0 ONNX Runtime link-mode item ([04](../../docs/plan/04-roadmap.md) M0; ADR-0005 D2; 10 S-1, S-22, S-24; R-SEC-8; O-108). It lives on the branch `prototype/ort-link-mode` and never merges into `main`; only the result is carried into the documents that own the facts.

## What it is

A standalone crate (its own `[workspace]` and `Cargo.lock`) on `ort` `=2.0.0-rc.14` with `default-features = false` and the features `load-dynamic` and `api-31`, plus `sha2` for the hash check. It:

- holds the pinned library of each shipped target (file name, SHA-256, release commit);
- refuses a directory that is not absolute, a missing file and a file with another SHA-256, before anything is loaded;
- on Windows restricts the process DLL search path with `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_DEFAULT_DIRS)` (application directory, `System32`, added directories; no current directory, no `PATH`);
- loads the verified file with `ort::init_from(<absolute path>)`, checks that the module the process mapped is the verified file, and then builds the one ONNX Runtime environment of the process with telemetry events switched off (`with_telemetry(false)`); sessions are created from that environment;
- runs a hand-encoded ONNX model (one `Identity` node, float `[3]`) to prove the loaded runtime works.

## Commands

```powershell
cargo test --locked --manifest-path prototype/ort-link-mode/Cargo.toml
$env:TALLYLINE_ORT_DIR = '<absolute directory holding the pinned library>'
cargo test --locked --manifest-path prototype/ort-link-mode/Cargo.toml -- --include-ignored --nocapture
```

The workflow `.github/workflows/ort-link-mode-spike.yml` runs the same on `windows-latest` and `ubuntu-latest` on a push to this branch: it downloads the official archive, checks it against the pinned archive SHA-256 before extracting, copies only the library into an empty directory and runs every test.

## Tests

| File | What it pins |
|---|---|
| `tests/refuse.rs` | a relative directory, a missing file and a file under the pinned name whose SHA-256 differs are refused, and no module of that name is mapped into the process afterwards |
| `tests/preloaded.rs` (ignored without `TALLYLINE_ORT_DIR`) | a runtime that `ort` loaded from another path before the verified load makes the load fail with `LoadedElsewhere` instead of being accepted |
| `tests/load.rs` (ignored without `TALLYLINE_ORT_DIR`) | the pinned file loads from its absolute path; the module the process mapped is that file; `ort::info()` names the release commit; the `Identity` model returns its input; no ONNX Runtime file or other `.dll`/`.so` appears in the temp directory |

## The pinned library

ONNX Runtime **1.31.0** (release `v1.31.0` of `microsoft/onnxruntime`, published 2026-10-09, not a pre-release, tag commit `402979ac1bef89dcd55add41eaa7277c407599eb`). `ort` 2.0.0-rc.14 binds the 1.31 C API. The spike first ran on `ort` 2.0.0-rc.13 with ONNX Runtime 1.28.3; on 2026-10-14 @greluc decided to move to rc.14 at once, and the spike was repeated with the pins below.

| Target | Release asset | Asset SHA-256 | Library in the asset | Library SHA-256 | Size |
|---|---|---|---|---|---|
| `x86_64-pc-windows-msvc` | `onnxruntime-win-x64-1.31.0.zip` | `3958d8a44984160692c1b4ffd0de3cb48f8c39f9d34bb31e433db06e30b82195` | `lib/onnxruntime.dll` | `0c42dc4e278f0968fdf9ebf1b1e54a74c657a9942e70803e453c91993451cf4c` | 17,253,176 bytes |
| `x86_64-unknown-linux-gnu` | `onnxruntime-linux-x64-1.31.0.tgz` | `cc5c72baf5ae5c8238a6841f0897227be2d02826b9cf98eaf02fdefaeeb03a57` | `lib/libonnxruntime.so.1.31.0` | `469849977cb60744ea160927e1a3a0643fd4c55c0aff743251c6d293402e334e` | 29,345,384 bytes |

How the hashes were established (2026-10-14):

- Both archives were downloaded from the release URLs; their SHA-256 matches the `digest` GitHub reports for each asset. That is one source: the download host.
- **Windows, second source:** `onnxruntime.dll` carries a valid Authenticode signature by `CN=Microsoft Corporation`, issued by `Microsoft Code Signing PCA 2024` (`Get-AuthenticodeSignature`: `Valid`).
- **Both targets, second source:** the NuGet package `Microsoft.ML.OnnxRuntime` 1.31.0 from nuget.org holds byte-identical copies (`runtimes/win-x64/native/onnxruntime.dll`, `runtimes/linux-x64/native/libonnxruntime.so`); `dotnet nuget verify --all` accepts its author signature (Microsoft Corporation) and its nuget.org repository signature (exit code 0). For 1.28.3 the Linux library had no second source (no NuGet or PyPI package of that version, no artifact attestation), which raised O-127.

The archives also hold `LICENSE` (MIT), `ThirdPartyNotices.txt` and `Privacy.md`; those differ in bytes between the Windows and the Linux archive, so the notices step takes them from the archive of each target.

## Result

### `load-dynamic` works and needs no build-time download

- With `load-dynamic`, `ort-sys` gets `disable-linking`, and its build script returns before linking or downloading anything. The compiled graph (`cargo tree --target all -e normal`) is `ort`, `ort-sys`, `libloading` 0.9.0, `smallvec`, `windows-link` on Windows and `libc` on Linux; `Cargo.lock` also lists the optional dependencies of `ort` (`ndarray`, `tracing` and theirs), which these features do not compile. `ort-sys` keeps `links = "onnxruntime"`, so the 09 §3b architecture check sees it.
- On Windows, the official `onnxruntime.dll` loaded from an absolute path in a directory that held nothing else, after `SetDefaultDllDirectories`, and ran the model. `onnxruntime_providers_shared.dll` was not present and not needed for the CPU execution provider. The dependent `MSVCP140*.dll`/`VCRUNTIME140*.dll` were found in `System32`, because the Visual C++ redistributable is installed on the test machine; whether they ship is the import-analysis item (O-12).
- `api-31` makes `ort` refuse a library older than 1.31 (`LoadDynamicError::BadVersion`) and log, not refuse, a newer one. The test also showed that the 1.28.3 library of the first run is refused on its hash.
- Linux did not run locally (no Rust toolchain in the available WSL distribution). In CI the workflow passed on `windows-latest` and `ubuntu-latest` on 2026-10-14 (run 38585549568): on both, all five tests passed with the official 1.31.0 library.

### Behaviour of `ort` that the adapter must respect

- In rc.14 the environment is an owned value (`EnvironmentBuilder::build`), at most one per process, and `Session::builder` takes it as a parameter; telemetry events are on unless `with_telemetry(false)` is set (O-13 decides whether that is enough).`n- `ort` keeps the loaded library in a process-wide `OnceLock`: once a library is loaded, a later `init_from` with another path returns `Ok` without loading anything. The verified load therefore checks afterwards which file the process mapped (`tests/preloaded.rs`).
- If any `ort` API runs before `init_from`, `ort` loads the library itself from the environment variable `ORT_DYLIB_PATH` or, failing that, from the bare name `onnxruntime.dll` / `libonnxruntime.so` (resolved next to the executable if it exists there, otherwise through the system search path), **without a hash check**, and panics if that fails. `adapter-ocr` must therefore call the verified load before any other `ort` call, and be the only crate that names `ort`.
- `init_from` resolves a relative path against the executable's directory; the adapter passes an absolute path anyway.
- `libloading` calls `LoadLibraryExW(path, NULL, 0)`, so the library's own imports follow the process's default DLL search order; `SetDefaultDllDirectories` is what removes the current directory and `PATH` from it. It is an `unsafe` FFI call and belongs to `adapter-platform` (09 §3a).
- The hash is checked on the bytes read, and the same path is then loaded. The window between the two needs write access to the installation directory, which already needs administrator rights (10 S-24).

### `download-binaries` (the default) is rejected

Read in the published `ort-sys` 2.0.0-rc.13 (`build/main.rs`, `build/download/*`) and compared with 2.0.0-rc.14, whose download code is unchanged apart from where `copy-dylibs` puts files:

- It downloads **pyke's own builds** of ONNX Runtime (1.28.0 for rc.13, 1.31.0 for rc.14) from `cdn.pyke.io`, not Microsoft's release asset, so it cannot meet "taken from the official release asset" (10 S-22).
- The SHA-256 of each archive is pinned in the crate (`build/download/dist.tsv`, covered by the `Cargo.lock` checksum), but the archive is **extracted before the hash is compared**, into a shared per-user cache (`ort.pyke.io/dfbin/<target>/<hash>` in the user's cache directory); a mismatch fails the build and leaves the extracted files there for inspection.
- It is a build-time network fetch: it needs a TLS feature (`ureq` with `native-tls` by default), adds `ureq`, `lzma-rust2` and `hmac-sha256` as build dependencies, and fails to link in an offline or `--frozen` build (`CARGO_NET_OFFLINE` skips the download).
- The archives hold static libraries, so ONNX Runtime would be linked into our binary. For `x86_64-pc-windows-msvc` without execution-provider features the resolver picks the first listed build, which is the DirectML one.
- The default feature set also enables `copy-dylibs` and `tls-native`.

Linking at load time against an official build (`ORT_LIB_PATH` with `ORT_PREFER_DYNAMIC_LINK=1`) is not used either: the OS loader would find `onnxruntime.dll` through its search order before any of our code could check its hash (10 S-24).

### Other findings

- `ort` 2.0.0-rc.14 was published on 2026-10-12 and ONNX Runtime 1.31.0 on 2026-10-09, both inside the 7-day cooldown that S-7 sets for Dependabot; the move was decided by @greluc anyway.
