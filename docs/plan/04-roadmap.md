# Roadmap

Order by risk and benefit: first the API uncertainties are clarified. Then follows **manual capture plus submission**; that is already a usable product without OCR. After that comes the OCR. Effort is deliberately not estimated in days, because the team's availability and experience are unknown.

## M0 – Foundation and verification

- [ ] Project owner decides: project and package name, license (GPL-3.0 if code is ported from basetool), JDK 27 vs. 25 LTS
- [ ] **Toolchain check JDK 27:** Do Gradle 9.8.1 (toolchain 27), Error Prone 2.50.0, NullAway, google-java-format, `org.beryx.jlink` and jpackage (WiX ≥ 4 on Windows) run with JDK 27? If not: JDK 25 LTS (see ADR)
- [ ] **Sharpen the domain model** ([11](11-ddd-and-tdd.md)): go through Ubiquitous Language, Bounded Contexts, report invariants I1–I5 together with the project owner (short event storming pass); create marker annotations and ArchUnit DDD rules
- [ ] TDD infrastructure: `java-test-fixtures` per module (fakes of the ports, test data builders), jqwik, PIT (check compatibility with JUnit 6 and JDK 27), tag report for requirement IDs
- [ ] Gradle multi-project according to [02-architecture.md](02-architecture.md), `build-logic`, version catalog, Spotless, Error Prone/NullAway, JUnit 6
- [ ] Quality gates according to [09-engineering-principles.md](09-engineering-principles.md): ArchUnit rules (layers, cycles, forbidden dependencies), JaCoCo thresholds, Dependabot, PR template with checklist – **before** the first domain code, so that the rules apply from the start
- [ ] CI: GitHub Actions, matrix Windows/Linux, `./gradlew check`
- [ ] **Secure the supply chain before the first dependency is checked in** ([10](10-supply-chain-security.md)): repository restriction, locking, `verification-metadata.xml` (SHA-256 + PGP) with keyring, wrapper validation, actions pinned by SHA, minimal `permissions`, Dependabot with cooldown period, OSV scan, CODEOWNERS, branch protection, `SECURITY.md`
- [ ] **API spike** (small CLI in the module `adapter-uex`) against the live API with `is_production=0`; clarifies all open points from [06-uex-api.md](06-uex-api.md) (header, app token, `status_*`, `container_sizes`, `/user`)
- [ ] Clarify with UEX: app token for open-source clients, terms of use, assumptions A9–A12 (environment mapping, `scu_sell` vs. `scu_sell_stock`, composed screenshot, duplicate lock)
- [ ] Expand the corpus: first entry `corpus/public/pyro-gateway-stanton-01` available (transcription **to be verified by a human**); supply the Patch City screenshots as files; preferably **original screenshots** (lossless, original resolution) instead of chat uploads

**Acceptance criteria:** `./gradlew check` is green on both OSes; a test report with `is_production=0` was submitted successfully and the response documented. **Negative tests of the supply chain:** a tampered artifact (changed hash) and a dynamic version demonstrably make the build fail.

## M1 – Reference data, manual capture, submission (first usable release 0.1)

- [ ] `shared-kernel`, `game`, `reference-data` (snapshot, vocabulary indexes, fuzzy matcher – property-tested); `reporting` (Report aggregate, submission gate, deviation assessment R-UI-10, manual capture use cases); `submission` (queue, cooldown); `workflows` (reporting → submission) – all tested with fakes of the ports
- [ ] `adapter-uex`: client, DTO mapping, envelope, error codes, rate limiter, host fallback; contract tests with recorded responses
- [ ] `adapter-storage`: SQLite, migrations, repositories (reports, queue, cooldown, history), reference-data cache with TTL refresh
- [ ] `adapter-platform`: secret store (FFM: Credential Manager / libsecret), paths, truststore
- [ ] `ui`: onboarding (key → `/user` check, test submission), settings, **manual capture** (R-MAN-*) with deviation marking (R-UI-10..12), history, diagnostics
- [ ] Screenshot attachment for manual reports (R-MAN-5): the user **selects the shop-panel region** (no automatic locate before M3); only that region is uploaded, so the balance is never included; test with the corpus images
- [ ] `app`: composition root, configuration
- [ ] Packaging: MSI / `.deb` / archives from CI, with SHA256SUMS, SBOM and build attestation (R-SEC-6)

**Acceptance criteria:** A user captures a terminal manually in < 1 min (target value), submits it and sees the report IDs. A cooldown survives a restart of the app. All M requirements of this milestone have acceptance tests (`@Tag`) that were written **before** the implementation; mutation score of the core measured and frozen as a lower bound.

## M2 – OCR core and eval

- [ ] `adapter-ocr`: ORT sessions, DB postprocessing, CTC decode
- [ ] `recognition` (first slice, needed for measuring): `ImageRaster` operations, locate, homography, layout, field parsers – without resolution, fusion, repair and stitching
- [ ] `tools/ocr-eval`: corpus runner, metrics, crop dumps, digest
- [ ] Corpus extension: resolutions 1080p/1440p/4K/21:9/32:9/windowed, **paired HDR on/off captures** per capture method (verifies A16)
- [ ] Tone normalization and scale handling in the first `recognition` slice (R-OCR-17, R-OCR-18)
- [ ] Baseline measurement on the corpus: raw accuracy per field type **and per corpus class** (resolution, HDR, theme), runtime, RAM

**Acceptance criteria:** Reproducible eval report; the raw accuracy is documented (still without a target value).

## M3 – Pipeline and review UI (release 0.5)

- [ ] `recognition` (second slice): resolution against the vocabulary, validation, repair, stitching without overlap, confidence, not-a-terminal detection (R-OCR-16)
- [ ] `capture` + `adapter-files` + `workflows` (capture → recognition → reporting): report grouping (R-OCR-13), game version at capture time (R-CAP-3b), working copies (R-CAP-7), user-defined folders, button "Import" (manual, default), auto-watch opt-in per folder (WatchService plus polling fallback, catch-up scan), stable-file gate, processed-file register, drag & drop, Ctrl+V, capture time
- [ ] `ui`: queue view, report editor with image crops and source highlight, deviation marking against UEX (R-UI-10..12), submission block, "Accept all confident"
- [ ] Upload screenshot: cropping, redact balance (test!)

**Acceptance criteria:** On the corpus, "silently wrong" is ≈ 0. The field accuracy after validation is measured, and the target values are set **based on the measurement** and frozen in CI.

## M4 – Robustness and platform (release 1.0)

- [ ] Further themes/layouts (blue standard, Nyx, gateways, red scrapyard terminals), ultrawide/1080p/4K
- [ ] Game localization (`global.ini`) and SC installation detection including Wine/Proton (assumptions A3/A4 verified)
- [ ] Second reader for status (bar/color) and cargo sizes
- [ ] HiDPI, accessibility (keyboard, contrast)
- [ ] Update notice (R-NF-7), "delete all local data" (R-NF-9)
- [ ] Resource measurement next to the running game; set heap and thread limits

## M5 – Optional AI recognition with the game closed (release 1.1, can be brought forward)

Requires M3 (shared resolution, validation and stitching). Can run in parallel to M4 if there is capacity.

- [ ] `adapter-platform`: `GameProcessMonitor` (Windows and Linux/Wine, hysteresis); verify assumption A7 on real systems
- [ ] `adapter-vlm`: `OllamaClient` (`/api/version`, `/api/tags`, `/api/ps`, `/api/pull`, `/api/chat`), host allowlist, cancellation and unloading (`keep_alive: 0`)
- [ ] Prompt v1 plus deterministic answer parser (golden tests with recorded answers, without Ollama in CI)
- [ ] `recognition`: fusion rules (07 §2.7), property-tested
- [ ] `workflows`: AI queue (persisted), `RecognitionPolicy`; `ui`: settings (Off / Automatic / Always), UI: status "AI pending / running / done", display of both candidates on contradiction, model management with pull progress
- [ ] **Bake-off** on the corpus: models (including `qwen3-vl:8b-instruct`, `qwen3-vl:4b-instruct`), Markdown vs. `format` output, image size; metrics "OCR only" / "VLM only" / "fusion", runtime, VRAM

**Acceptance criteria:**

- Game start aborts an AI run within ≤ 5 s (2 s interval during a run plus abort) and unloads the model (test with simulated process).
- Without Ollama there is no error message.
- The fusion measurably lowers "silently wrong" and "flagged" on the corpus compared with "OCR only". **If that is not the case, the feature is not shipped** (assumption A8).

## Requirement coverage

Every requirement group from [01-requirements.md](01-requirements.md) is assigned to the milestone that implements it. The traceability report (11 §B4) uses this table: M requirements of completed milestones without a tagged test fail the build.

| Milestone | Requirements |
|---|---|
| M0 | R-SEC-1…5, R-QA-1 (corpus structure) |
| M1 | R-MAN-*, R-VAL-1…4 (manual path), R-UI-2, R-UI-4, R-UI-7, R-UI-8, R-UI-10…12 (manual path), R-SUB-*, R-API-*, R-CAP-3/3a (manual environment), R-L10N-1, R-NF-2, R-NF-4…6, R-NF-8, R-SEC-6…8 |
| M2 | R-OCR-1, R-OCR-2 (locate), R-OCR-5, R-OCR-17, R-OCR-18, R-CAP-8, R-QA-3 (eval CLI), R-QA-4 |
| M3 | R-CAP-1…1e, R-CAP-2, R-CAP-3b, R-CAP-5, R-CAP-7, R-OCR-3, R-OCR-4, R-OCR-6…8, R-OCR-11…14, R-OCR-16, R-VAL-1…5 (OCR path), R-UI-1, R-UI-3, R-UI-5, R-UI-6, R-UI-10…12 (OCR path), R-QA-2 |
| M4 | R-CAP-4, R-CAP-6, R-OCR-9, R-OCR-10, R-OCR-15, R-L10N-2, R-NF-1 (Linux/XWayland tests), R-NF-3, R-NF-7, R-NF-9 |
| M5 | R-VLM-* |
| later | R-UI-9, R-L10N-3, R-SCOPE-2 (UI), R-SEC-9 |

## Risks

| Risk | Impact | Countermeasure / check |
|---|---|---|
| UEX requires an app token that an open-source client cannot distribute securely (A2) | **Project blocker** for submission | M0: clarify early with UEX; option "user enters their own app token" |
| UEX terms of use exclude third-party clients or automated capture | Project blocker | M0: read the terms and obtain approval if necessary |
| PP-OCR is too inaccurate as the primary reader for terminals | Lots of manual correction; core benefit drops | M2 measures early; countermeasures: fine-tuning the recognition model, VLM second reader (M5) |
| Game patch changes the terminal layout | Recognition breaks | Data-driven layout profiles, crop dumps, corpus regression test; manual cropping as fallback |
| Toolchain does not support JDK 27 | Build blocked | M0 check; fallback JDK 25 LTS |
| Linux specifics (Wine paths, game detection, Secret Service, XWayland) deviate from the assumptions | Linux functions restricted | Manual configuration as fallback everywhere; tests on real systems in M4/M5 |
| Compromised dependency, CI action or compromised build artifact | Malicious code at users, theft of the secret key | Measures S-1…S-31 in [10](10-supply-chain-security.md); residual risk "malicious, correctly signed new version" remains medium |
| UEX API changes (fields, error codes) | Submission fails | Defensive parsing, contract tests with recorded responses, understandable error messages |

## Afterwards (1.x / later)

- Tray mode with notification (#36)
- Items and vehicle purchase/rental (#20)
- Further OCR writing systems (Cyrillic/Korean)
- Optionally a fine-tuned recognition model for the SC HUD font
- Trade routes are deliberately **not** planned; specialized tools exist for that
