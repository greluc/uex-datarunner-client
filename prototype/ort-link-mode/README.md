# ONNX Runtime link-mode spike (issue #25)

Spike code for the M0 ONNX Runtime link-mode item ([04](../../docs/plan/04-roadmap.md) M0; ADR-0005 D2; 10 S-1, S-22, S-24; R-SEC-8; O-108). It lives on the branch `prototype/ort-link-mode` and never merges into `main`; only the result is carried into the documents that own the facts.

## What it is

A standalone crate (its own `[workspace]` and `Cargo.lock`) on `ort` `=2.0.0-rc.13` with `default-features = false` and the features `load-dynamic` and `api-28`, plus `sha2` for the hash check. It:

- holds the pinned library of each shipped target (file name, SHA-256, ONNX Runtime version, release commit);
- refuses a directory that is not absolute, a missing file and a file with another SHA-256, before anything is loaded;
- on Windows restricts the process DLL search path with `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_DEFAULT_DIRS)` (application directory, `System32`, added directories; no current directory, no `PATH`);
- loads the verified file with `ort::init_from(<absolute path>)` and commits the environment;
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
| `tests/refuse.rs` | a relative directory, a missing file and a file whose SHA-256 differs are refused, and the library is not mapped into the process afterwards |
| `tests/load.rs` (ignored without `TALLYLINE_ORT_DIR`) | the pinned file loads from its absolute path; the module the process mapped is that file; `ort::info()` names the release commit; the `Identity` model returns its input; no ONNX Runtime file or other `.dll`/`.so` appears in the temp directory |

## The pinned library

ONNX Runtime **1.28.3** (release `v1.28.3` of `microsoft/onnxruntime`, published 2026-10-02, tag commit `0d68ff6b3b72b04aac578decd6c4c45d322bb962`). `ort` 2.0.0-rc.13 binds the 1.28 C API; 1.28.3 is the latest patch of that line and carries memory-safety fixes in shape inference and graph validation that 1.28.0 lacks.

| Target | Release asset | Asset SHA-256 | Library in the asset | Library SHA-256 | Size |
|---|---|---|---|---|---|
| `x86_64-pc-windows-msvc` | `onnxruntime-win-x64-1.28.3.zip` | `1d6fab48e85f948436af7c8c971d2c145cf224e2c444755dec894f8b0de11a83` | `lib/onnxruntime.dll` | `4d2774a5f64e4a230b16b74b167171e92f65b0683c19680c1d04fb284b92a6e2` | 15,828,832 bytes |
| `x86_64-unknown-linux-gnu` | `onnxruntime-linux-x64-1.28.3.tgz` | `db14e4863bd37893fc59729d986ab2a0d043d10b7d44da1913c4982b7e3d009c` | `lib/libonnxruntime.so.1.28.3` | `5a1ce74e56e8c4b278d5a9e9d12b7192d9f70811084dde9616da4fa1710efc5a` | 24,301,616 bytes |

How the hashes were established (2026-10-14):

- Both archives were downloaded from the release URLs; their SHA-256 matches the `digest` GitHub reports for each asset. That is one source: the download host.
- **Windows, second source:** `onnxruntime.dll` and `onnxruntime_providers_shared.dll` carry a valid Authenticode signature by `CN=Microsoft Corporation`, issued by `Microsoft Code Signing PCA 2024` (`Get-AuthenticodeSignature`: `Valid`).
- **Linux, no second source:** the `.so` is not signed; NuGet `Microsoft.ML.OnnxRuntime` has 1.28.0 and then 1.29.0, not 1.28.3; PyPI has no 1.28.3 of `onnxruntime`; the release has no GitHub artifact attestation (`gh attestation verify`: 404). The Linux hash rests on the release host alone.

The archives also hold `LICENSE` (MIT), `ThirdPartyNotices.txt` and `Privacy.md`; those differ in bytes between the Windows and the Linux archive, so the notices step takes them from the archive of each target.

## Result

### `load-dynamic` works and needs no build-time download

- With `load-dynamic`, `ort-sys` gets `disable-linking`, and its build script returns before linking or downloading anything. The resolved graph is `ort`, `ort-sys`, `libloading` 0.9.0, `smallvec` and (Windows) `windows-link`. `ort-sys` keeps `links = "onnxruntime"`, so the 09 §3b architecture check sees it.
- On Windows, the official `onnxruntime.dll` loaded from an absolute path in a directory that held nothing else, after `SetDefaultDllDirectories`, and ran the model. `onnxruntime_providers_shared.dll` was not present and not needed for the CPU execution provider. The dependent `MSVCP140*.dll`/`VCRUNTIME140*.dll` were found in `System32`, because the Visual C++ redistributable is installed on the test machine; whether they ship is the import-analysis item (O-12).
- `api-28` makes `ort` refuse a library older than 1.28 (`LoadDynamicError::BadVersion`) and log, not refuse, a newer one.
- Linux has not run locally (no Rust toolchain in the available WSL distribution); the workflow covers it once the branch is pushed.

### Behaviour of `ort` that the adapter must respect

- If any `ort` API runs before `init_from`, `ort` loads the library itself from the environment variable `ORT_DYLIB_PATH` or, failing that, from the bare name `onnxruntime.dll` / `libonnxruntime.so` (resolved next to the executable if it exists there, otherwise through the system search path), **without a hash check**, and panics if that fails. `adapter-ocr` must therefore call the verified load before any other `ort` call, and be the only crate that names `ort`.
- `init_from` resolves a relative path against the executable's directory; the adapter passes an absolute path anyway.
- `libloading` calls `LoadLibraryExW(path, NULL, 0)`, so the library's own imports follow the process's default DLL search order; `SetDefaultDllDirectories` is what removes the current directory and `PATH` from it. It is an `unsafe` FFI call and belongs to `adapter-platform` (09 §3a).
- The hash is checked on the bytes read, and the same path is then loaded. The window between the two needs write access to the installation directory, which already needs administrator rights (10 S-24).

### `download-binaries` (the default) is rejected

Read in the published `ort-sys` 2.0.0-rc.13 (`build/main.rs`, `build/download/*`):

- It downloads **pyke's own builds** of ONNX Runtime **1.28.0** from `cdn.pyke.io`, not Microsoft's release asset, so it cannot meet "taken from the official release asset" (10 S-22) and misses the 1.28.3 fixes.
- The SHA-256 of each archive is pinned in the crate (`build/download/dist.tsv`, covered by the `Cargo.lock` checksum), but the archive is **extracted before the hash is compared**, into a shared per-user cache (`ort.pyke.io/dfbin/<target>/<hash>` in the user's cache directory); a mismatch fails the build and leaves the extracted files there for inspection.
- It is a build-time network fetch: it needs a TLS feature (`ureq` with `native-tls` by default), adds `ureq`, `lzma-rust2` and `hmac-sha256` as build dependencies, and fails to link in an offline or `--frozen` build (`CARGO_NET_OFFLINE` skips the download).
- The archives hold static libraries, so ONNX Runtime would be linked into our binary. For `x86_64-pc-windows-msvc` without execution-provider features the resolver picks the first listed build, which is the DirectML one.
- The default feature set also enables `copy-dylibs` and `tls-native`.

Linking at load time against an official build (`ORT_LIB_PATH` with `ORT_PREFER_DYNAMIC_LINK=1`) is not used either: the OS loader would find `onnxruntime.dll` through its search order before any of our code could check its hash (10 S-24).

### Other findings

- `ort` 2.0.0-rc.14 was published on 2026-10-12 and binds ONNX Runtime 1.31. Moving to it is its own pull request (04 risk table); this spike stays on the pinned rc.13.
