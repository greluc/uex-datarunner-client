# ADR-0008: ONNX Runtime telemetry: switched off on Linux, ETW events accepted on Windows

> **Doc type:** Decision record — Accepted. Last reviewed: 2026-10-14.

- **Status:** Accepted (2026-10-14)
- **Decider:** @greluc, in chat on 2026-10-14, after the M0 ONNX Runtime telemetry spike (issue #26) that R-NF-12 and O-13 call for
- **Amends:** [01](../plan/01-requirements.md) R-NF-12, which required an ONNX Runtime built with `--no_telemetry` if any telemetry remained with the mitigation; for the Windows ETW events that remain, that build is not made and the events are accepted
- **Related:** O-13 in [the register](0000-open-points.md), [ADR-0005](0005-rust-and-egui.md) Consequences, [ADR-0007](0007-ort-rc14-and-onnx-runtime-link-mode.md), [02](../plan/02-architecture.md) §8 and §9 row X1, [09](../plan/09-engineering-principles.md) §3a, [10](../plan/10-supply-chain-security.md) §2.5

## Context

The official ONNX Runtime libraries collect telemetry by default. R-NF-12 prescribed two mitigations (`ORT_DISABLE_TELEMETRY=1` set by our own code before the first ONNX Runtime call, and the `ort` setting `with_telemetry(false)` before the first session), an M0 spike measuring what remains, and an ONNX Runtime built with `--no_telemetry` if anything did.

## Method

The spike lives on the branch `prototype/ort-telemetry` (never merged; full record in its `README.md`). It read the `v1.31.0` sources of ONNX Runtime and the published `ort` 2.0.0-rc.14, searched both pinned official libraries for the telemetry code, and ran a probe that loads the pinned library, runs two inference sessions and stays alive 120 s, in four variants: no mitigation, the `ort` setting only, the variable only, and both. On Linux every `connect`, `sendto`, `sendmsg` and `sendmmsg` call was recorded with `strace`, each variant twice with a shared isolated `HOME`, with no CI variable in the environment. On Windows an ETW session recorded ONNX Runtime's TraceLogging provider (`Microsoft.ML.ONNXRuntime`, `3a26b1ff-7484-7484-7484-15261f42614d`) and the Kernel-Network, DNS-Client and WinHttp providers. It ran in CI (run 38591511090) and on the owner's Windows 11 machine.

## Findings

- **Linux** (1DS uploader in the library): without the variable, each run opened one HTTPS connection to a Microsoft address and wrote a persistent device ID and an event cache under `$XDG_CACHE_HOME/Microsoft/DeveloperTools/.onnxruntime/`, also with `with_telemetry(false)`. With the variable, with or without the `ort` setting, there was no connection and no file.
- **Windows** (the official DLL is the ETW build; it contains no 1DS uploader): the process opened no connection and wrote no file in any variant. With `with_telemetry(false)` four events remain at environment creation (`ProcessInfo`, `DriverInfo`, `RegisterEpLibraryStart`, `RegisterEpLibraryEnd`), because `ort` switches the events off only after `CreateEnv`; any ETW session that enables the provider afterwards switches all events on again (a late session saw the full session, model-load and evaluation set). The variable has no effect on this library. On the owner's machine, with Windows' default diagnostic-data level (Required) and the `DiagTrack` service running, the `Diagtrack-Listener` session enables the provider, so the events reach Windows' diagnostic-data pipeline; whether Windows uploads them is Windows' decision and was not observed.
- **CI:** the Linux uploader stays silent when any of 13 CI variables (`CI`, `GITHUB_ACTIONS` and others) or `ORT_RUNNING_UNIT_TESTS` is set, so a network check run in GitHub Actions sees nothing unless the app runs without them.

## Alternatives considered

- **An ONNX Runtime built with `--no_telemetry`**, as R-NF-12 prescribed: a new supply-chain input (our own build of a large C++ project, reproducible and hash-pinned, 10 S-22), and still unverified whether the flag removes the Windows provider or only its telemetry option. Not chosen.
- **A pure-Rust ONNX path** (`tract`, `rten`; ADR-0005 "Alternatives considered"): removes the native library altogether, but needs its own spike on PP-OCRv6. Not chosen; it stays an optional fallback.

## Decision

1. **Both platforms:** before the first ONNX Runtime call, `adapter-platform` sets `ORT_DISABLE_TELEMETRY=1` in the process environment, and `adapter-ocr` builds the environment with `with_telemetry(false)`. On Linux the variable is what removes the telemetry; on Windows neither removes it, but the setting limits the events to environment creation while no further trace session enables the provider, and the variable covers a Windows library that carries the 1DS uploader.
2. **Windows:** the ETW events of the official library that remain with this mitigation are **accepted**. They are listed in the data-flow inventory (02 §9 row X1) and in the user documentation. The app itself opens no connection for them; whether they leave the machine is governed by the user's Windows diagnostic-data settings.
3. **No `--no_telemetry` build** is made for now.
4. **Re-measure on every ONNX Runtime update:** the update pull request repeats the telemetry measurement of the spike on both platforms; a library that sends from the process, or that ignores the variable on Linux, reopens this decision.
5. **Smoke test:** the network check of R-NF-12 starts the app with an environment cleared of the CI variables and `ORT_RUNNING_UNIT_TESTS`, and runs once more without `ORT_DISABLE_TELEMETRY` as a control that must show the uploader's connection, so a silent library cannot pass it falsely.

## Consequences

- 01 R-NF-12, 02 §8 and §9 row X1, 09 §3a, 10 §2.5, `NOTICE` 2.3, `README.md`, `.github/SECURITY.md`, `CLAUDE.md` and the 04 roadmap and risk table state this decision; O-13 is decided.
- The `unsafe` `set_var` call stays the planned site of 09 §3a, now required on Linux by measurement.
- Windows users with diagnostic data switched on may have ONNX Runtime's initialisation events collected by Windows; the user guide (R-DOC-1, from M1) says so.
