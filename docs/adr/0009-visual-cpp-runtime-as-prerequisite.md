# ADR-0009: The Visual C++ runtime of ONNX Runtime is a prerequisite, not shipped

> **Doc type:** Decision record — Accepted. Last reviewed: 2026-10-14.

- **Status:** Accepted (2026-10-14)
- **Decider:** @greluc, in chat on 2026-10-14, after the M0 ONNX Runtime import analysis (issue #27) that O-12 and 04 M0 call for
- **Amends:** the default design of O-12 (amendment of 2026-10-13: ship the runtime DLLs next to the executable, subject to the legal review), which becomes the rejected alternative; ADR-0003 decision 5.6 stands, with a smaller subject
- **Related:** O-12 in [the register](0000-open-points.md), [ADR-0003](0003-licence-and-contributions.md) decision 5, [ADR-0005](0005-rust-and-egui.md) D7 and Consequences, [ADR-0007](0007-ort-rc14-and-onnx-runtime-link-mode.md), [01](../plan/01-requirements.md) R-NF-13, [02](../plan/02-architecture.md) §8, [10](../plan/10-supply-chain-security.md) S-24 and §2.5

## Context

Our own Rust binary links the C runtime statically, but the official ONNX Runtime library (`onnxruntime.dll`, 1.31.0, ADR-0007) is a C++ library built with MSVC and needs the Microsoft Visual C++ runtime. These DLLs are proprietary: Microsoft lets only licensed Visual Studio users redistribute them, under the "Distributable Code" terms of the Visual Studio licence, and recommends the central Visual C++ Redistributable over copies next to an application, which Windows Update does not service. O-12 had planned to ship them next to the executable, subject to a legal review before the first release.

## Method

`dumpbin /dependents` and `dumpbin /headers` over the pinned library, after its archive matched the pinned SHA-256: on the owner's Windows 11 machine and in CI on `windows-latest` (run 38599030126, commit `a2dc228` of the spike branch `prototype/ort-imports`, removed from the remote after the run; the result is in 10 §2.5).

## Findings

- Besides Windows libraries and the Universal C Runtime (part of Windows 10 and 11, R-NF-1), the library imports four Visual C++ runtime DLLs: `MSVCP140.dll`, `MSVCP140_1.dll`, `VCRUNTIME140.dll` and `VCRUNTIME140_1.dll`. O-12 had named only the last two.
- Its PE linker version is 14.44; Microsoft requires the runtime to be at least as new as the toolset that built a binary, so the floor is 14.44.
- The runner's own Visual Studio redistributable is 14.29: a machine can have all four DLLs and still be too old for the library.
- The Windows loader resolves an import by module name to a DLL of that name already mapped in the process, whatever its directory, so even app-local copies would not guarantee which version the library binds to.

## Alternatives considered

- **Ship the four DLLs next to the executable** (the O-12 default). It works without a prerequisite, but it redistributes proprietary files under terms that forbid making them subject to a copyleft licence. The Windows package would then rest entirely on the GPL-3.0 System Library definition, and the files would have to come from a licensed Visual Studio installation. The project would also have to release whenever Microsoft services the runtime, and a copy mapped earlier by another module would still win. Not chosen.
- **Bundle the redistributable installer in the MSI or a bootstrapper:** still a redistribution of Microsoft's files, with the same licence questions. Not chosen.
- **A pure-Rust ONNX path** (`tract`, `rten`; ADR-0005 "Alternatives considered"): removes the native library and its runtime, but needs its own spike on PP-OCRv6. Not chosen; it stays an optional fallback.

## Decision

1. **No Microsoft runtime file is shipped:** no MSI, ZIP or other artefact contains `msvcp140*.dll`, `vcruntime140*.dll`, `ucrtbase.dll`, `api-ms-win-*.dll` or Microsoft's redistributable installer.
2. **The Visual C++ Redistributable (x64), at the floor of the pinned ONNX Runtime release or newer, is a documented prerequisite of the Windows app:** the MSI checks the version the redistributable records in the registry and refuses to install below the floor with a message that names the redistributable and Microsoft's download page; the ZIP, the README and the user guide name it. The app downloads and starts nothing (10 S-23, S-25).
3. **The run-time check of R-NF-13 is authoritative:** after the verified load of the library, `adapter-ocr` reads the file version of the four runtime modules the process mapped and refuses the OCR runtime if one is missing or older than the floor, with a visible error at start; the registry check of the MSI is only a convenience.
4. **The floor belongs to the pinned ONNX Runtime release:** every ONNX Runtime update repeats the import analysis and moves the floor in the same pull request.
5. **The legal review before the first release stays (O-12)**, reduced to the question whether a GPL-3.0-or-later program that needs the separately installed Microsoft runtime is covered by the System Library definition, and the §7 question before the first external contribution.

## Consequences

- 01 R-NF-13, 02 §8, 04 M0 and the risk table, 10 S-24, §2.5 and the T13 row, `NOTICE` 2.10, `README.md`, `docs/release-process.md` and `CLAUDE.md` state this decision; O-12 is amended.
- The packaged-app smoke test fails if a Windows artefact contains a Microsoft runtime file.
- Windows users without a current redistributable must install it once; the MSI and the start-up error tell them so.
