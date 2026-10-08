# Review of the plan (2026-10-08)

All documents under `docs/plan/` were reviewed, plus `CLAUDE.md` and `README.md`. The criteria were completeness, factual errors, logic gaps and contradictions between the documents. The findings have been corrected directly in the documents. This list records **what** was wrong or incomplete, so that the changes remain traceable.

## A. Factual errors (corrected)

| # | Finding | Correction |
|---|---|---|
| A-1 | `sealed interface Reader permits OcrReader, VlmReader` in `domain`, but the implementations in the modules `ocr` and `vlm`. **Does not compile:** in named JPMS modules, permitted subtypes must be in the same module. | `Reader` is a normal interface; `ReaderKind` as an enum (02 §4b, CLAUDE.md rule) |
| A-2 | The price regex in 07 §2.4 rejected exactly the case it was intended for (`96705/SCU`), and all numbers without thousands separators | Procedural parser with candidate generation (07 §2.4) |
| A-3 | ADR: "iced has no mature table widget" – **wrong**. `iced_widget` 0.14 contains `table.rs` (checked in the crate source code). | Statement corrected; the assessments of egui and slint are marked as unverified. The decision does not change. |
| A-4 | ADR: basetool "proves" that the OCR approach works – **overstated**. There, PP-OCR is only a digit second reader; the primary reader is a VLM. | Toned down; suitability as the primary reader is measured in M2 |
| A-5 | `InventoryStatus` checked hard for 1–7. That contradicts our own rule "status levels from the API, not from constants". | Check against `ReferenceSnapshot` (02 §3) |
| A-6 | Thread pool `min(2, cores/2)` yields 0 threads on single-core systems | `max(1, min(2, cores / 2))` |
| A-7 | R-NF-1 claimed Wayland operation via XWayland as a given | Marked as an assumption or test point; native Wayland support of JavaFX is not proven |
| A-8 | Rust release date: the build date (2026-09-28) was given as the release date | Clarified |

## B. Logic gaps and contradictions (closed)

| # | Finding | Solution | Location |
|---|---|---|---|
| B-1 | `pipeline → refdata → uex-api` and `pipeline → ocr`: the "pure" pipeline depended on modules with network and file I/O, contradicting guiding principle 3 | `pipeline` now depends only on `domain`. It is passed `ReferenceSnapshot`, `TextDetector` and `ReaderResult`; `app` wires everything. | 02 §2, §4 |
| B-2 | Fusion order contradictory (diagram: fusion → validation; text: validation → fusion) | Defined: per reader parse and resolve → fusion → validate **once** → stitching | 02 §4b, 07 §2.7 |
| B-3 | **Send threshold** not defined anywhere; confidence values without meaning for the submission gate | Send threshold 0.80 as a term and in the confidence table | 01 Terms, 07 §2.6 |
| B-4 | Prior-based repair could **silently "repair" real price changes back to old values**, and that with 0.85, i.e. sendable | Repairs automatic only with an independent witness, otherwise 0.75 → confirm | R-VAL-2b, 07 §2.5 |
| B-5 | No behavior defined if **no prior** exists (new commodity at the terminal) | Fallback to the commodity average, otherwise no repair | R-VAL-2a |
| B-6 | Manual capture **prefilled UEX values**. This would allow outdated values to be sent as a fresh report by clicking through; a data quality problem for UEX. | Values are shown as a reference, not prefilled; only actively accepted rows are sent | R-MAN-2 |
| B-7 | Manual reports have no screenshot, but new DataRunners need one (`screenshot_required`) | Screenshot can be attached, the requirement is shown in advance | R-MAN-5, A11 |
| B-8 | A report consists of several scroll captures, but the API only accepts **one** screenshot | Compose the shop crops vertically (assumption A11) | R-SUB-7 |
| B-9 | "Time window" for reports was not defined | Grouping rule: same terminal, same side and environment, ≤ 10 min apart; splitting and merging in the UI | R-OCR-13 |
| B-10 | UEX knows only `live`/`ptu`; the mapping of HOTFIX, EPTU and TECH-PREVIEW was missing | Default mapping plus assumption A9 | R-CAP-3a |
| B-11 | **Patch day:** screenshot before the patch, sending afterwards → wrong game version | Record the version at capture time, warn on change | R-CAP-3b |
| B-12 | `data_parameters` was cached for 1 day, but the acceptance flags change on patch day | Before sending, a state at most 15 min old | R-SUB-5 |
| B-13 | The send queue was not persistent: offline reports were lost on exit | Persistent queue; after a restart send only after release | R-SUB-2, 02 §6 |
| B-14 | Cooldown key inconsistent (01: with side; 02: with side and environment); the UEX rule is unknown | Uniform and conservative (terminal, commodity, environment); `duplicated_report` is treated as cooldown; assumption A12 | R-SUB-4, 02 §6 |
| B-15 | If the original is deleted (cleanup function or user), the image was missing for review, AI and upload | Working copies of the panel crops with a retention period | R-CAP-7 |
| B-16 | "manual" as an environment value per folder was unclear | Options: automatic from path / fixed / ask for every image | R-CAP-1 |
| B-17 | Diagram: manual reports bypassed validation (contradiction to R-MAN-4) | Arrow added | 02 §1 |
| B-18 | Locate needs coarse OCR, but the pipeline must not know `ocr` | Interface `TextDetector` is injected | 02 §4 |
| B-19 | M5 acceptance criteria "≤ 5 s abort" could not be reliably reached with a 5 s check interval | 2 s interval during an AI run | 02 §4b, 04 M5 |
| B-20 | References to non-existent requirements (`R-SCOPE`, tray without a requirement) | `R-SCOPE-2`, new `R-UI-9` | 05 F28/F29 |
| B-21 | CLAUDE.md mentioned "assumptions A1–A6", meanwhile there are A1–A12 | Updated | CLAUDE.md |

## C. Missing parts (added)

| # | Was missing | Added in |
|---|---|---|
| C-1 | Risk register (among others app token as a possible project blocker, UEX terms of use, OCR accuracy, layout patches) | 04 "Risks" |
| C-2 | Check whether the toolchain (Error Prone, NullAway, google-java-format, jlink plugin, jpackage/WiX) runs with **JDK 27** | 04 M0, ADR §4 |
| C-3 | Open API questions: sell SCU field (`scu_sell` vs. `scu_sell_stock`), `container_sizes` per commodity vs. per report, `is_missing` mandatory fields, screenshot composite | 06 Open points 8–14, A10–A12 |
| C-4 | Processing status of own reports at UEX (`data_info`) in the history | R-SUB-4 |

## D. Deliberately left open (decision or verification needed)

- **To be decided by the project owner:** project and package name, license, JDK 27 vs. 25 LTS.
- **To be clarified against the live API or with UEX:** A1, A2, A9–A12 as well as the open points in 06. This applies especially to `container_sizes`: the sizes differ per commodity in the game. If the field exists only per report, we should rather omit it than falsify it.
- **To be measured on the corpus:** all thresholds (send threshold, confidence levels, tolerances, grouping window), runtime and RAM targets, VLM suitability (A8).
- **To be tested on real systems:** Linux/Wine paths (A3), game detection (A7), XWayland, Secret Service. For libsecret via FFM, note that the store API is variadic; the FFM API supports this, but it increases the effort. An alternative would be D-Bus directly.

## E. Overall assessment (honestly)

- The plan is complete enough to start M0. The biggest uncertainties lie **outside** our control: the app token and UEX's terms of use as well as the exact semantics of `data_submit`. These points should be clarified before any larger implementation, because in the worst case they block the project.
- The second-biggest uncertainty is the **real OCR accuracy** of PP-OCR on terminal screenshots. That is why M2 (measurement) comes before building the complete pipeline.
- The numbers from basetool (VLM speed, Markdown vs. JSON accuracy, recognition rates) come from a different domain (refinery). They are to be understood as starting values, not as a promise.

## F. Addendum: modularization, maintainability and deviation marking

A second review focusing on modularization, clean code and best practices yielded the following (module names in sections A–E still refer to the old split):

| # | Finding | Change |
|---|---|---|
| F-1 | `app` combined UI, secret store, AI control and wiring; `submission` and `capture` mixed domain logic (queue, cooldown, policy) with technology (HTTP, SQLite, file system). Use cases were therefore only testable with UI or technology. | New split following **Ports & Adapters**: core `domain`/`pipeline`/`application`, adapters `adapter-*`, presentation `ui`, composition root `app` (02 §2) |
| F-2 | Several modules would have had their own SQLite access (duplication, scattered SQL) | One module `adapter-storage` with repositories; SQL only there (09 §6) |
| F-3 | OS integration (secret store, process monitor, installation detection, truststore) was spread across `app` and `capture` | Bundled in `adapter-platform` |
| F-4 | Best practices were named but not **enforced** | 09: rules with enforcement (JPMS, ArchUnit, Error Prone/NullAway, JaCoCo gate, Dependabot, PR checklist, Definition of Done); M0 sets up the gates before the first domain code |
| F-5 | Time-dependent logic (cooldown, hysteresis, grouping) was not deterministically testable | `java.time.Clock` as an injected port |
| F-6 | Deviations from the previous UEX value were only indirectly visible via the confidence. A **cleanly read but wrong** value (transposed digits in the source, wrong row) would have remained unmarked. | Separate dimension "deviation" with color marking, Δ display, age of the reference and mandatory confirmation for strong deviation (R-UI-10..12, 07 §2.6, 02 §7) |

## G. Addendum: module cut by bounded context

The project owner asked whether the modules can follow DDD. Assessment and decision are in [ADR-002](../adr/0002-modules-by-bounded-context.md).

In short:

- The core is now cut by bounded context (`capture`, `recognition`, `reporting`, `submission`, `reference-data`, `game`) plus `shared-kernel` and `workflows`. Adapters stay cut by technology.
- `adapter-refdata` was dissolved: fuzzy matching and indexes are domain logic. `adapter-capture` became `adapter-files`.
- Module names in sections A–F above refer to the earlier cuts.

Disadvantages accepted, with handling documented in the ADR:

- more modules
- explicit translation between contexts
- more expensive re-cuts
- the risk that the shared kernel grows

## H. Second full review (2026-10-08)

All documents were re-read after the switch to English and the module re-cut. Findings and fixes:

### Contradictions (fixed)

| # | Finding | Fix |
|---|---|---|
| H-1 | R-VAL-2 said deviations beyond tolerance only warn ("not blocking"), while R-UI-10 requires confirmation for major deviations | R-VAL-2 now: never auto-rejected, but a major deviation must be confirmed |
| H-2 | The goal said manual input is "prefilled with current UEX data", contradicting R-MAN-2 (reference only) | Goal text corrected |
| H-3 | The core must not touch the file system, but `WatchedFolder` used `java.nio.file.Path` and the folder scanner/watcher were placed in `capture` | `FolderLocation` value object in the core; scanner, watcher and stable-file gate live in `adapter-files` |
| H-4 | `recognition` worked on `BufferedImage` (AWT, module `java.desktop`) although the core is supposed to be technology-free | Own `ImageRaster` type in the core; ImageIO only in adapters; ArchUnit forbids `java.awt..`/`javax.imageio..` in the core |
| H-5 | The game process monitor published a "JavaFX property" from an adapter | Publishes via the `GameStateProbe` port; only `ui` maps it to JavaFX |
| H-6 | Retries were assigned to both `submission` and `adapter-uex` | `submission` decides retries; the gateway makes one attempt per call |
| H-7 | `GameEnvironment` was listed in `shared-kernel` (11) and in `game` (ADR-002) | Owned by `game`, used as published language |
| H-8 | Confidence was also lowered for out-of-tolerance values, i.e. the deviation was counted twice | Confidence is independent of the prior band; deviation is its own dimension |
| H-9 | The traceability rule required tests for all M requirements from M3 on, although M requirements are implemented in M1–M5 | Requirement-to-milestone table in 04; the rule applies per completed milestone |

### Logic gaps (closed)

| # | Finding | Fix |
|---|---|---|
| H-10 | The AI re-read could change reports that were already released or queued, and could collide with a concurrent user edit | Only `Draft` reports; `Report.version` for optimistic concurrency |
| H-11 | Catch-up import after a patch: the "version at capture time" would have been the new version | Local history of observed version changes; uncertain versions must be confirmed (R-CAP-3b) |
| H-12 | Manual reports had no defined environment or game version | R-MAN-6 |
| H-13 | Screenshot attachment for manual reports in M1 would have uploaded the full screenshot including the balance, because automatic locate only arrives in M3 | In M1 the user selects the shop-panel region; only that region is uploaded |
| H-14 | M2 should measure raw OCR accuracy per field, but layout and field parsers were only planned for M3 | `recognition` is split into a first slice (M2) and a second slice (M3) |
| H-15 | UEX counts every price row as a report (1000 per 30 min) – not enforced | Rate budget in `submission`; open point 16 in 06 |
| H-16 | Non-terminal screenshots had no defined handling (the predecessor's "failed to process" error) | R-OCR-16 |
| H-17 | Test-mode default unclear for end users | R-SUB-8: on in dev/CI, off in release builds after onboarding, always visible |
| H-18 | No defined home for user settings, upload encoding and update check | Ports `*SettingsStore`, `ImageEncoder`, `UpdateCheck` (02 §2) |
| H-19 | Units of `price_variation`/`scu_variation` unconfirmed but used as thresholds | Assumption A15; configurable conservative defaults |
| H-20 | `data_remove`/`data_info` parameters unknown although withdrawal (I4) depends on them | Open point 15 in 06 |
| H-21 | No "delete all local data" action for a tool that stores secrets and screenshots | R-NF-9 |
| H-22 | Smaller errors: corpus structure in 07 did not match the real corpus; prompt path pointed to an old module name; requirement order within tables; German mermaid ID | Fixed |

### Still open (unchanged, outside the plan's control)

- UEX app token and terms of use (A2) remain the biggest risk.
- The exact `data_submit` semantics remain unverified: header, `container_sizes`, sell-side SCU, duplicate lock, environments (A1, A9–A12, A15).
- All thresholds are start values until measured on the corpus.

## I. Third review (2026-10-08) – three independent reviewers

Three reviewers ran in parallel, without knowledge of the earlier reviews: cross-document consistency, technical correctness (claims checked against JDK, Gradle, GitHub, ONNX Runtime, sqlite-jdbc and Ollama sources), and completeness along end-to-end scenarios. They produced about 75 findings. A sample of the technical claims was re-checked against primary sources before changing anything:

- `actions/attest` README: `artifact-metadata: write`, and public repositories only on small plans.
- `gradle/actions` dependency-submission docs: `contents: write`.
- ONNX Runtime 1.30.0 source: `onnxruntime.native.path`.

The most important fixes:

| Area | Finding | Fix |
|---|---|---|
| JPMS | Repository ports in `api` named aggregates hidden in `internal`, so adapters could not implement them | Export `api` + `api.model`; aggregates protected by the compact constructor plus an ArchUnit constructor rule (02 §2, ADR-002, 09, 11) |
| Ownership | Recognition produced a Reporting aggregate (`Report`) although the context map forbids it | `Stitcher` → `StitchedScan` (recognition); `ReportGrouper` builds the `Report` (reporting) |
| Fusion | The prior alone could decide between OCR and VLM, contradicting R-VAL-2b | The prior alone gives at most 0.75 (confirm); glyph-topology role defined once (witness when agreeing, veto when contradicting) |
| Submission lifecycle | Unknown outcome → possible double submission; partial `ids_reports`; rejected and withdrawn reports were dead ends; no error classes; cooldown vs. multi-row reports; no correction path | R-SUB-9…12, extended R-SUB-4, report states and transitions in 11 §A3 |
| Data quality for UEX | Old captures sent as fresh data; PTU compared against LIVE priors; patch-day layout drift could produce confident wrong reads; stale snapshot at release | R-VAL-6, R-API-6, R-OCR-19, R-VAL-7 |
| Security/platform | Linux jlink runtime ignores the system CA store; `Windows-ROOT` missing from a minimal jlink image; composite trust manager could silently skip hostname verification; native libraries extracted into shared temp (incl. JavaFX); JEP 472 native-access warnings; locking did not cover buildscript/settings/build-logic; plugins mostly unsigned; dependency-submission and attestation permissions | 02 §8, S-3, S-4, S-14, S-16, S-24, S-26 |
| Robustness | Game detection on Linux via `ProcessHandle` misses Wine (argv[0] dropped); HttpClient cancel is best effort and Ollama unloads only after the request ends; in-process events lost on crash; MDC vs. `ScopedValue`; pyramid scaling wrong | `/proc` primary; cancel + verify via `/api/ps`; transactional outbox; MDC exception; target-height downscale |
| Operations | No release, upgrade/downgrade or patch-day process; no user documentation; no single instance; "portable" undefined; diagnostics redaction unspecified; no path for real misreads into the corpus | `docs/release-process.md`, R-NF-10/11, R-NF-2, R-NF-6, R-DOC-1/2, R-QA-5 |
| Consistency | About 20 smaller mismatches, among them: `Deviation` naming and stale rule; "same PR" vs "separate PR"; R-VLM-3 vs mode "Always"; requirement-to-milestone table gaps; wrong citations (R-UI-4, `data_parameters`, R-SUB-3 tag); annotation count; A6 scope; §-reference; `tools/ocr-eval` dependencies | Fixed in place |

Verified as correct by the technical reviewer:

- ArchUnit 1.5.1 and JaCoCo 0.8.15 read Java 27 class files.
- ScopedValue is final in JDK 25.
- The Gradle option names used are real.
- Dependabot `cooldown`.
- OSV-Scanner supports Gradle lockfiles.
- The ONNX Runtime and sqlite-jdbc artifacts are as described.

Still unverified, and listed in M0: jqwik with JUnit Platform 6; TestFX headless with JavaFX 27; `java-test-fixtures` with JPMS; Gradle 9.8.1 toolchain 27; Wine argv[0] in practice; EAC and `ProcessHandle` on Windows.
