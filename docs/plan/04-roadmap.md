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

- [ ] `domain`: value objects, ports; `application`: use cases submission (queue, cooldown, send threshold), deviation assessment (R-UI-10), manual capture – tested with fakes of the ports
- [ ] `adapter-uex`: client, DTO mapping, envelope, error codes, rate limiter, host fallback; contract tests with recorded responses
- [ ] `adapter-storage`: SQLite, migrations, repositories (queue, history, cache)
- [ ] `adapter-refdata`: cache, TTL refresh, vocabulary indexes, fuzzy matcher (property-tested)
- [ ] `adapter-platform`: secret store (FFM: Credential Manager / libsecret), paths, truststore
- [ ] `ui`: onboarding (key → `/user` check), settings, **manual capture** (R-MAN-*) with deviation marking (R-UI-10..12), history, diagnostics
- [ ] `app`: composition root, configuration
- [ ] Packaging: MSI / `.deb` / archives from CI, with SHA256SUMS, SBOM and build attestation (R-SEC-6)

**Acceptance criteria:** A user captures a terminal manually in < 1 min (target value), submits it and sees the report IDs. A cooldown survives a restart of the app. All M requirements of this milestone have acceptance tests (`@Tag`) that were written **before** the implementation; mutation score of the core measured and frozen as a lower bound.

## M2 – OCR core and eval

- [ ] `adapter-ocr`: ORT sessions, DB postprocessing, CTC decode, image operations, homography
- [ ] `tools/ocr-eval`: corpus runner, metrics, crop dumps, digest
- [ ] Baseline measurement on the corpus: raw accuracy per field type, runtime, RAM

**Acceptance criteria:** Reproducible eval report; the raw accuracy is documented (still without a target value).

## M3 – Pipeline and review UI (release 0.5)

- [ ] `pipeline`: locate, layout, field parser, resolution, validation, repair, stitching, confidence
- [ ] `adapter-capture` + `application`: user-defined folders, button "Import" (manual, default), auto-watch opt-in per folder (WatchService plus polling fallback, catch-up scan), stable-file gate, processed-file register, drag & drop, Ctrl+V, capture time
- [ ] `ui`: queue view, report editor with image crops and source highlight, deviation marking against UEX (R-UI-10..12), submission block, "Accept all confident"
- [ ] Upload screenshot: cropping, redact balance (test!)

**Acceptance criteria:** On the corpus, "silently wrong" is ≈ 0. The field accuracy after validation is measured, and the target values are set **based on the measurement** and frozen in CI.

## M4 – Robustness and platform (release 1.0)

- [ ] Further themes/layouts (blue standard, Nyx, gateways, red scrapyard terminals), ultrawide/1080p/4K
- [ ] Game localization (`global.ini`) and SC installation detection including Wine/Proton (assumptions A3/A4 verified)
- [ ] Second reader for status (bar/color) and cargo sizes
- [ ] HiDPI, accessibility (keyboard, contrast), UI texts in English via ResourceBundles
- [ ] Resource measurement next to the running game; set heap and thread limits

## M5 – Optional AI recognition with the game closed (release 1.1, can be brought forward)

Requires M3 (shared resolution, validation and stitching). Can run in parallel to M4 if there is capacity.

- [ ] `adapter-platform`: `GameProcessMonitor` (Windows and Linux/Wine, hysteresis); verify assumption A7 on real systems
- [ ] `adapter-vlm`: `OllamaClient` (`/api/version`, `/api/tags`, `/api/ps`, `/api/pull`, `/api/chat`), host allowlist, cancellation and unloading (`keep_alive: 0`)
- [ ] Prompt v1 plus deterministic answer parser (golden tests with recorded answers, without Ollama in CI)
- [ ] `pipeline`: fusion rules (07 §2.7), property-tested
- [ ] `application`: AI queue (persisted), `RecognitionPolicy`; `ui`: settings (Off / Automatic / Always), UI: status "AI pending / running / done", display of both candidates on contradiction, model management with pull progress
- [ ] **Bake-off** on the corpus: models (including `qwen3-vl:8b-instruct`, `qwen3-vl:4b-instruct`), Markdown vs. `format` output, image size; metrics "OCR only" / "VLM only" / "fusion", runtime, VRAM

**Acceptance criteria:**

- Game start aborts an AI run within ≤ 5 s (2 s interval during a run plus abort) and unloads the model (test with simulated process).
- Without Ollama there is no error message.
- The fusion measurably lowers "silently wrong" and "flagged" on the corpus compared with "OCR only". **If that is not the case, the feature is not shipped** (assumption A8).

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
