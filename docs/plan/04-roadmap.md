# Roadmap

Order by risk and benefit: first the API uncertainties are clarified. Then follows **manual capture plus submission**; that is already a usable product without OCR. After that comes the OCR. Effort is deliberately not estimated in days, because the team's availability and experience are unknown.

## M0 – Foundation and verification

- [ ] Project owner decides: project and package name, license (GPL-3.0 if code is ported from basetool), JDK 27 vs. 25 LTS
- [ ] **Toolchain check JDK 27:** Do Error Prone 2.50.0, NullAway, google-java-format, `org.beryx.jlink` and jpackage run with JDK 27? If not: JDK 25 LTS (see ADR). MSI: JDK 27 jpackage accepts WiX 3 (`candle`/`light` ≥ 3.0) or `wix` ≥ 4.0.4 (JDK source). The default is the WiX Toolset 3.14.1 preinstalled on `windows-latest` ([10](10-supply-chain-security.md) S-11); WiX ≥ 4 only after an owner decision
- [ ] **Sharpen the domain model** ([11](11-ddd-and-tdd.md)): go through Ubiquitous Language, Bounded Contexts, report invariants I1–I7 together with the project owner (short event storming pass); create marker annotations and ArchUnit DDD rules
- [ ] TDD infrastructure: `java-test-fixtures` per module (fakes of the ports, test data builders; **spike: behaviour with JPMS**, fallback GradleX `java-module-testing`), jqwik (**check compatibility with JUnit Platform 6** – jqwik 1.10.1 is built against Platform 1.x), PIT (check compatibility with JUnit 6 and JDK 27), TestFX headless on CI (Monocle availability for JavaFX 27, else Xvfb), tag report for requirement IDs (qualified tags and the list "Verified outside JUnit", 11 §B4)
- [ ] Already verified: ArchUnit 1.5.1 reads Java 27 class files, and JaCoCo 0.8.15 supports them experimentally (third review); Gradle 9.8.1 runs on Java 27 and provides Java 27 toolchains (Gradle compatibility matrix: both since 9.8.0). Still open: whether Gradle detects the `setup-java` JDK with auto-download off (`org.gradle.java.installations.fromEnv`)
- [ ] Gradle multi-project according to [02-architecture.md](02-architecture.md), `build-logic`, version catalog, Spotless, Error Prone/NullAway, JUnit 6
- [ ] Quality gates according to [09-engineering-principles.md](09-engineering-principles.md): ArchUnit rules (layers, cycles, forbidden dependencies), JaCoCo thresholds, Dependabot, PR template with checklist – **before** the first domain code, so that the rules apply from the start
- [ ] CI: GitHub Actions, matrix Windows/Linux, `./gradlew check`
- [ ] **Secure the supply chain before the first dependency is checked in** ([10](10-supply-chain-security.md)): repository restriction, locking, `verification-metadata.xml` (SHA-256 for every artifact, plus PGP where signed) with armored keyring and CI guard, wrapper validation, actions pinned by SHA, minimal `permissions`, Dependabot with cooldown period, OSV scan, CODEOWNERS, branch protection, `SECURITY.md`; the CI checks named in "Verified outside JUnit" (Requirement coverage)
- [ ] **API spike** (small CLI in the module `adapter-uex`) against the live API with `is_production=0`; clarifies all open points from [06-uex-api.md](06-uex-api.md) (header, app token, `status_*`, `container_sizes`, `/user`)
- [ ] Clarify with UEX: app token for open-source clients, terms of use, assumptions A9–A12 (environment mapping, `scu_sell` vs. `scu_sell_stock`, composed screenshot, duplicate lock and its scope: side, user, test mode, live/PTU), which hosts UEX operates and which of them accept POSTs (06 open point 4), and a dedicated test account for the scheduled live-API job (M1)
- [ ] Expand the corpus: first entry `corpus/public/pyro-gateway-stanton-01` available (transcription **to be verified by a human**); supply the Patch City screenshots as files; preferably **original screenshots** (lossless, original resolution) instead of chat uploads; a schema test (tagged R-QA-1) validates every `corpus/public/*/expected.json`

**Acceptance criteria:** `./gradlew check` is green on both OSes; a test report with `is_production=0` was submitted successfully and the response documented. **Negative tests of the supply chain:** a tampered artifact (changed hash) and a dynamic version demonstrably make the build fail. A clean `./gradlew check` (empty Gradle caches) passes dependency verification with the committed metadata, so no task resolves an artifact that is missing from it (detached tool configurations, S-3).

## M1 – Reference data, manual capture, submission (first usable release 0.1)

- [ ] `shared-kernel`, `game`, `reference-data` (snapshot, vocabulary indexes, fuzzy matcher – property-tested); `reporting` (Report aggregate, submission gate, deviation assessment R-UI-10, manual capture use cases); `submission` (queue, cooldown); `workflows` (reporting → submission) – all tested with fakes of the ports
- [ ] `adapter-uex`: client, DTO mapping, envelope, error codes, rate limiter, host fallback; contract tests with recorded responses; WireMock fault tests for R-SUB-9 (connection refused → retried; a timeout or 504 after the request was received → exactly one request and `OutcomeUnknown`)
- [ ] `adapter-storage`: SQLite, migrations, repositories (reports, queue, cooldown, history), reference-data cache with TTL refresh
- [ ] `adapter-platform`: secret store (FFM: Credential Manager / libsecret), paths, truststore
- [ ] `adapter-files` (first slice): the `ConfigFile` component and the `*SettingsStore` implementations M1 needs (`ReportingSettingsStore`, `GameSettingsStore`, `AppSettingsStore`; settings catalogue in 02 §2), with atomic writes, schema version and visible errors (R-NF-5); PNG/JPEG decoding to `ImageRaster` for the manual attachment (R-MAN-5; completed per R-CAP-8 in M2); `ImageEncoder` and `WorkingCopyStore` for the user-selected upload region (R-SUB-7, manual path); contract tests against a real temp directory (11 §B1)
- [ ] `ui`: app shell with the global status area (R-UI-8), onboarding (key → `/user` check, test submission), settings, **manual capture** (R-MAN-*) with deviation marking (R-UI-10..12), history, diagnostics, **submission queue view** with the report and job states and their actions (R-UI-1 `:reports`): WaitingForCooldown with ETA and "send the others now"; held and paused with reason, "retry now" and re-authentication; PartiallyAccepted with the per-row result and its resolution; OutcomeUnknown resolution via `data_info` or by the user; Rejected and Withdrawn with "duplicate as new draft" (not for `invalid_game_version`, R-SUB-11); Discarded; renewed release after a restart (R-SUB-2); prompt for queued jobs after a key change (R-SUB-12). No dialog from background events (R-UI-16)
- [ ] Screenshot attachment for manual reports (R-MAN-5): the user **selects the shop-panel region** (no automatic locate before M3); only that region is uploaded, so the balance is never included; test with the corpus images
- [ ] `app`: composition root (wiring), bootstrap (storage locations per 02 §8, portable marker); user settings only through the `*SettingsStore` ports (09 §2)
- [ ] Packaging: MSI / ZIP / `.deb` / `tar.gz` from read-only CI build jobs, with SHA256SUMS, SBOM and build attestation from a separate publish job (R-SEC-6, [10](10-supply-chain-security.md) S-14). jlink image incl. `jdk.crypto.mscapi` (Windows) and `--enable-native-access` for the modules that load natives, with an explicit `mergedModuleName`, because ONNX Runtime is merged (02 §8). JavaFX natives copied from the verified Maven Central platform JARs into the runtime image (`lib/` on Linux, `bin/javafx/` on Windows), so there is no runtime extraction (S-24). The Linux `tar.gz` keeps the execute bits (S-30). Portable marker (R-NF-2)
- [ ] **Packaged-app smoke test** in the release build jobs on `windows-latest` (ZIP) and `ubuntu-latest` (`tar.gz`, under `xvfb-run`). Unpack the archive and check (Linux) that `<App>/bin/<App>` and `lib/runtime/lib/jspawnhelper` are executable. Then start the launcher with `--illegal-native-access=deny` added to its Java options, `HOME`/`USERPROFILE` and `java.io.tmpdir` pointing to empty directories, in a headless self-test mode that the app defines: print the version, start the JavaFX toolkit, open SQLite, initialize the FFM secret-store adapter (an unavailable OS store is acceptable), from M2 on create an ORT session from the bundled models, and exit with code 0. The test fails on a non-zero exit, on any 'restricted method' or 'Unknown module' warning on stderr, and if a `.openjfx` or `.openjfx_<user>` directory was created.
- [ ] Single instance (R-NF-10), DB backup before migrations and refusal of newer schemas (R-NF-11), `docs/release-process.md` followed for 0.1
- [ ] Submission lifecycle completely implemented: error classes (R-SUB-11), unknown outcome and partial acceptance (R-SUB-9), cooldown waiting (R-SUB-4), observation age (R-VAL-6), no older observation after a newer one, oldest-first dispatch per key (R-VAL-8), fresh reference at release (R-VAL-7), acceptance and game-version re-check before every send attempt (R-SUB-5, R-CAP-3b manual path), account binding (R-SUB-12), first start without cache (R-UI-13), connection test (R-UI-15)
- [ ] `docs/user/` skeleton (R-DOC-1) and issue templates (R-DOC-2)
- [ ] Opt-in scheduled CI job (weekly; `schedule`/`workflow_dispatch` on `main` only, never on pull requests) against the live UEX API with `is_production=0`. A keyless job builds the API-spike CLI. A second job runs the prebuilt CLI with the key of a **dedicated UEX test account**, never a maintainer's personal key: `is_production` is only a request flag, so a leaked key could also write production data. The key is stored in a protected GitHub environment restricted to `main` and exposed only to that step ([10](10-supply-chain-security.md) S-14). The job only reports changes in field names and error codes, never response values such as `username` (detects API drift before users do)

**Acceptance criteria:** A user captures a terminal manually in < 1 min (target value), submits it and sees the report IDs. A cooldown and the 30-minute row budget survive a restart of the app (R-SUB-2, R-SUB-4). A job interrupted while sending (shutdown, crash, kill) ends as `OutcomeUnknown` and is never sent again automatically, and a user cancel is refused once the job is sending; after a restart, reports with an unfinished job are back in Draft (R-SUB-9, R-SUB-2). All M requirements of this milestone have acceptance tests (`@Tag`, qualified where the coverage table qualifies them) that were written **before** the implementation, or an entry under "Verified outside JUnit"; mutation score of the core measured and frozen as a lower bound.

## M2 – OCR core and eval

- [ ] `adapter-ocr`: ORT sessions, DB postprocessing, CTC decode
- [ ] `recognition` (first slice, needed for measuring): `ImageRaster` operations, locate, homography, layout, field parsers – without resolution, fusion, repair and stitching
- [ ] `tools/ocr-eval`: corpus runner, metrics, crop dumps, digest
- [ ] Capture protocol `docs/user/corpus-capture.md` and `expected.json` schema version 2 with `conditions`/`measured`, capture times, UEX IDs and the frozen `reference/` snapshot, and `tools/ocr-eval annotate`; migrate `pyro-gateway-stanton-01` (07 §4)
- [ ] Corpus extension along the R-QA-2 ladders: resolution (1080p/1440p/4K/21:9/32:9/windowed; DSR/VSR allowed and recorded), distance, **paired HDR captures** per capture method **and renderer** (DX11/Vulkan, where selectable) and at two Windows SDR-brightness values, with tool name and version recorded (verifies A16); an expanded SELLABLE CARGO section and an expanded OUT OF STOCK section (verifies A21); a card cut at the top edge, e.g. after mouse-wheel scrolling (07 §2.3); Linux captures if a tester is available (low priority); keep the original file names of every tool (they verify A19)
- [ ] Check A14 and A22 on the corpus. Until they are confirmed, the dependent rules stay off (A14) or produce findings only (07 §2.4, §2.5 item 4).
- [ ] `adapter-files` decoding extended from the M1 slice to full R-CAP-8 (magic bytes, colour metadata, HDR set-aside, pixel budget) with the decode fixtures (07 §4)
- [ ] Scale handling and per-panel tone analysis in the first `recognition` slice (R-OCR-17, R-OCR-18; 07 §2.1 items 1–7)
- [ ] **Synthetic class variants** in `tools/ocr-eval` (07 §4), including the **scale sweep** (box and nearest) down to 4.5 px price cap height: per corpus class and model export, the reading-limit curve per field type (raw reads). It runs again on every model or runtime update (digest test); from M3 on it also reports "silently wrong".
- [ ] Synthetic tone fixtures (07 §4): expected R-CAP-8 outcome, tone class, and no increase in wrong reads
- [ ] Run the sweeps with the official Hugging Face PP-OCRv6 export (pinned by SHA-256); the planning figures came from the third-party onnxocr 4.0.0 export
- [ ] Calibrate the text-size gate (R-OCR-17) per theme and the tone-finding thresholds (R-OCR-18) on the sweeps and the paired captures
- [ ] Check every capture tip and every format message (R-CAP-8, R-DOC-1) against the paired captures and the current game build; drop or reword tips that do not hold
- [ ] Baseline measurement on the corpus on the **reference machine** (chosen by the project owner, e.g. the main tester's PC; CPU model, core count and RAM recorded in the eval report): raw accuracy per field type **and per corpus class** with class status (R-QA-3), runtime (p95 per resolution class) with the default OCR pool and `OcrRuntimeSettings`, peak heap and peak process memory (R-NF-3)

**Acceptance criteria:**

- Reproducible eval report including the class-coverage matrix with a status per class (R-QA-3) and the raw accuracy per class (still without a target value).
- Synthetic sweep (07 §4) on all public entries with the **official, SHA-pinned** model export: a reading-limit curve per field type (price, SCU, status, name, label; raw reads) over the price cap height, for `scale-box` and `scale-nearest`. The R-OCR-17 lower and confirm limits are set from it as settings values, or the start values are kept with a documented reason. The `TextTooSmall` finding below the lower limit is pinned by tests that failed first; the confirm cap follows in M3 with confidence.
- Metamorphic gate, M2 stage (07 §4: locate-quad and layout invariance, raw-read comparison reported), green in CI on Windows and Linux.
- Paired captures for the SC screenshot key (DX11 and Vulkan) and the Game Bar PNG exist, and A16 is updated with the classification per capture method (07 §4). If no HDR display is available, the classes are marked `missing`, A16 stays open and the risk row "no HDR/ultrawide hardware" applies. The washed-out detector raises no finding on any normal-contrast capture with Windows HDR off, and raises it on every `tone-*` variant whose expected label is "washed out" (07 §4). The R-OCR-18 thresholds are measured values or kept with a documented reason.
- A real resolution ladder (1080p/1440p/2160p, same view) exists and the synthetic-to-real validity check (07 §4) is reported, or the ladder is marked `missing` under the same risk row.
- R-CAP-8 decode fixtures (07 §4) green.

## M3 – Pipeline and review UI (release 0.5)

- [ ] `recognition` (second slice): resolution against the vocabulary, validation, repair, stitching without overlap, confidence, locate outcome `WrongScreen`/`NotLocated` (R-OCR-16, 07 §2.1 item 3), pinned first by failing synthetic tests: non-terminal images such as a desktop or other SC UI → `WrongScreen`; crops with anchors but a broken frame → `NotLocated`
- [ ] `capture` + `adapter-files` + `workflows` (capture → recognition → reporting): report grouping (R-OCR-13), game version at capture time with the persisted per-environment version history (`GameObservationRepository` in `adapter-storage`), the extra `game_versions` fetches and one test per classification case (certain, provisional, uncertain, unknown), including a catch-up import after a patch (R-CAP-3b), working copies (R-CAP-7; `WorkingCopyStore` in `adapter-files`, retention in `workflows`), user-defined folders, button "Import" (manual, default), auto-watch opt-in per folder (WatchService plus polling fallback, catch-up scan), stable-file gate, processed-file register, drag & drop, Ctrl+V, clipboard monitoring (opt-in; verify A23), capture time with its source and the file-name patterns per tool (R-CAP-5, A19)
- [ ] Back-pressure test: an Import of a backlog (200 files at 3840×2160 and at the pixel budget) stays within the heap limit (R-NF-3) and never holds more decoded rasters than the OCR pool and intake executor bounds allow
- [ ] `ui`: queue view extended with the capture states and their actions (R-UI-1 `:captures`: imported, scanned, environment pending, not located / not a terminal, failed, Draft with findings), session overview with "release all ready" (R-UI-14), report editor with image crops and source highlight, manual crop/corner correction (R-UI-6), deviation marking against UEX (R-UI-10..12), submission block, "Accept all confident"; background processing with bounded parallelism (R-OCR-12)
- [ ] Layout profiles with validated game versions (initial lists only from verified entries with a non-null `gameVersion`, R-OCR-19) and the `UnvalidatedGameVersion` cap (R-OCR-19); "report a misread" export (R-QA-5)
- [ ] Crop extent and header-band redaction when the working copies are created (R-SUB-7, R-CAP-7). Mandatory tests: a synthetic frame with a balance marker just above the shop panel, also with a skewed or enlarged quad, a manual crop and shifted or localized balance text, leaves no marker pixel in the working copy, the VLM request image, the upload image or the R-QA-5/R-NF-6 exports; reports with unconfirmed crop geometry cannot be released without the confirmed preview; on the corpus captures the crop top stays below the redaction box
- [ ] Confidence caps from capture findings (07 §2.6); decide per tone finding whether the "confirm" cap stays (R-OCR-18, based on "silently wrong" per class); per-digit certainty rule (07 §2.6)
- [ ] Before release 0.5, from the M2 baseline: set the R-OCR-12 runtime target (p95 per resolution class on the reference machine) and the initial launcher limits (`-Xmx`, `-XX:MaxDirectMemorySize`: measured peak plus a stated margin, R-NF-3); record both in the 0.5 release notes

**Acceptance criteria:** On the corpus, "silently wrong" is ≈ 0. The field accuracy after validation is measured, and the target values are set **based on the measurement** and frozen (R-QA-3 gates with `tools/ocr-eval/baseline.json`, evaluated on the frozen per-entry reference snapshots: in CI for entries with a `reference/` directory, otherwise in the local pre-release run on the private corpus, 07 §4). The metamorphic gate (07 §4) is fully green, with the send-threshold rules and the R-OCR-17 confirm cap on every entry that has a reference snapshot. For every capture method in the corpus, the plan states "supported", "supported with confirmation" or "not supported", together with the message the user sees. The R-OCR-12 runtime target and the initial R-NF-3 limits are set and recorded in the 0.5 release notes.

## M4 – Robustness and platform (release 1.0)

- [ ] Further themes/layouts (blue standard, Nyx, gateways, red scrapyard terminals), ultrawide/1080p/4K
- [ ] Game localization (`global.ini`) and SC installation detection including Wine/Proton (assumptions A3/A4 verified)
- [ ] Game process and channel detection in `adapter-platform` (`GameProcessMonitor`, shared with R-CAP-4; Windows via Toolhelp snapshot, Linux/Wine via `/proc`) and the environment cross-check (R-CAP-10). Test written first: a fake enumerator that returns `StarCitizen.exe` without a path gives RUNNING with an unknown channel, which never counts as a mismatch. Verify assumption A7 on real systems: Windows with EAC active and with the game started elevated; Linux with Wine, Proton and Lutris
- [ ] Status-bar witness (bar/colour) and cargo-box count for cargo sizes; dimmed cards (R-OCR-15); optional cleanup of originals (R-CAP-6)
- [ ] HiDPI and accessibility (R-UI-17): tagged tests for contrast in every theme, target size and focus rules (`:automated`); manual checks in release checklist step 7 (`:manual`): keyboard-only walkthrough, 200 % text size at the minimum window size, Windows screen-reader check (Narrator or NVDA), HiDPI 100–250 % including mixed-scale monitors
- [ ] Update notice (R-NF-7), "delete all local data" (R-NF-9)
- [ ] Resource measurement next to the running game on the reference machine (R-NF-3): peak process memory, heap and direct memory, busy cores of the OCR and intake workers, the worker priority in effect, the app's VRAM where the OS reports it, and the game's frame time with the app closed, visible and idle, and importing (e.g. PresentMon, if a tester has it); set the final heap, direct-memory and process-memory limits and the `OcrRuntimeSettings` defaults (07 §2.2)
- [ ] Capture check in diagnostics and onboarding (R-CAP-9); capture tips in the user guide (R-DOC-1) with the classes that are not verified

**Acceptance criteria (release 1.0):** every *required* R-QA-2 class is `gated` with no silently wrong field, or is named "not verified" in the release notes and in R-DOC-1 together with the capture advice that avoids it. Runtime (p95), peak heap and peak process memory are measured per resolution class on the reference machine, including 5120×1440 and one input larger than 3840×2160; the R-OCR-12 target holds or has been revised per R-OCR-12. Next to the running game, an import of 10 captures with the default settings causes no out-of-memory error, stays below `-Xmx` and the R-NF-3 process-memory ceiling, and never has more busy OCR threads than pool size × intra-op threads; the frame-time comparison and the app's VRAM are recorded in the release notes, against a tolerance set by the project owner.

## M5 – Optional AI recognition with the game closed (release 1.1, can be brought forward)

Requires M3 (shared resolution, validation and stitching). Can run in parallel to M4 if there is capacity.

- [ ] `adapter-platform`: `GameProcessMonitor` hysteresis and AI-run interval (R-VLM-2); process and channel detection come with M4, or are built here if M5 runs before M4 (then with the test-first case and the A7 verification listed in M4)
- [ ] `adapter-vlm`: `OllamaClient` (`/api/version`, `/api/tags`, `/api/show`, `/api/ps`, `/api/pull`, `/api/chat`), loopback-host check and remote-model classification with consent (R-VLM-6; `HttpClient.Builder.NO_PROXY` for loopback), digest comparison with the evaluated-model list (S-27), cancellation and unloading (`keep_alive: 0`). Verify assumption A20 and set the minimum Ollama version from it. Tests:
  - recorded `/api/tags` and `/api/show` responses: a local model, `name:cloud`, `name:120b-cloud`, and a custom-named model with `remote_host`;
  - host cases: `localhost`, `LOCALHOST.`, `127.1.2.3` and `[::1]` are used without asking; `localhost.example.com`, `0.0.0.0` and a LAN address only after confirmation.
- [ ] Prompt v1 plus deterministic answer parser (golden tests with recorded answers, without Ollama in CI; the example answer in 07 §2.7 yields quantity 333 and price 3237; a literal `?` yields `Unreadable`)
- [ ] `recognition`: fusion rules over the per-reader candidate sets (07 §2.7), property-tested. Named tests:
  - F1: OCR {96705, 6705} + VLM 6705 → 6705, `Repaired`, 0.85;
  - a confusable substitution with a glyph-topology veto → `Ambiguous`;
  - recorded VLM answers for the partial cards in `pyro-gateway-stanton-01` sell-1/buy-1/buy-2 → no row;
  - a model that is not evaluated never lifts a field above "confirm" through the VLM.
- [ ] `workflows`: AI queue (persisted), `RecognitionPolicy`; `ui`: settings (Off / Automatic / Always), AI job states (pending, incl. "interrupted – game started"; running; done; failed with reason; cancelled; obsolete) and the queue-paused reason (02 §4b), display of both candidates on contradiction, model management with pull progress; a test per AI job transition, including truncation after the retry and Ollama disappearing mid-run
- [ ] **Bake-off** on the corpus with local models only (R-VLM-6): models (including `qwen3-vl:8b-instruct`, `qwen3-vl:4b-instruct`), Markdown vs. `format` output, image size; metrics "OCR only" / "VLM only" / "fusion", runtime, VRAM. Output: the evaluated-model list resource (name, tag, manifest digest(s), approximate download size, metrics; R-VLM-11, S-27)

**Acceptance criteria:**

- Game start aborts an AI run within ≤ 5 s (2 s interval during a run plus abort) and unloads the model (test with simulated process).
- Without Ollama there is no error message.
- The fusion measurably lowers "silently wrong" and "flagged" on the corpus compared with "OCR only". **If that is not the case, the feature is not shipped** (assumption A8).

## Requirement coverage

Every requirement group from [01-requirements.md](01-requirements.md) is assigned to the milestone that implements it. The traceability report (11 §B4) uses this table and the list "Verified outside JUnit" below: an M requirement of a completed milestone that has neither a tagged test nor an entry in that list fails the build. A requirement that is split across rows or verification methods carries a qualifier in backticks in each place (e.g. R-VAL-1 with `:manual` in M1 and `:ocr` in M3); its tests use the qualified tag (`@Tag("R-VAL-1:ocr")`). A new or moved requirement ID updates this table in the same commit. R-SCOPE-3 is an exclusion (prio –) and is not assigned to a milestone.

| Milestone | Requirements |
|---|---|
| M0 | R-QA-1 (corpus structure: schema test), R-SEC-1, R-SEC-2, R-SEC-3, R-SEC-4, R-SEC-5 |
| M1 | R-SCOPE-1, R-SCOPE-2 `:model`, R-MAN-1…6, R-VAL-8, R-UI-1 `:reports` (report and job states), R-UI-2, R-UI-7 `:theme`, R-UI-8, R-UI-13, R-UI-15, R-UI-16, R-SUB-1…6, R-SUB-8…12, R-API-1…7, R-CAP-3a, R-L10N-1, R-VLM-12, R-NF-2 `:portable`, R-NF-2 `:installers`, R-NF-4…6, R-NF-8, R-NF-10, R-NF-11, R-DOC-2, R-SEC-6, R-SEC-8; manual path (`:manual`): R-VAL-1, R-VAL-2, R-VAL-2a, R-VAL-3, R-VAL-4, R-VAL-6, R-VAL-7, R-UI-10…12, R-SUB-7 (user-selected region + confirmed preview), R-CAP-3 (manual environment), R-CAP-3b (version re-check before sending) |
| M2 | R-OCR-1, R-OCR-2 (locate), R-OCR-5, R-OCR-17, R-OCR-18, R-CAP-8, R-QA-2 `:started` (ladders started), R-QA-3 (eval CLI), R-QA-4, R-SEC-7 |
| M3 | R-CAP-1, R-CAP-1a, R-CAP-1b, R-CAP-1c, R-CAP-1d, R-CAP-1e, R-CAP-2, R-CAP-5, R-CAP-7, R-OCR-3, R-OCR-4, R-OCR-6, R-OCR-7 `:uex-names`, R-OCR-8, R-OCR-11…14, R-OCR-16, R-OCR-19, R-VAL-2b, R-VAL-5, R-UI-1 `:captures` (capture states), R-UI-3, R-UI-4, R-UI-5, R-UI-6, R-UI-14, R-QA-5; OCR path (`:ocr`): R-VAL-1, R-VAL-2, R-VAL-2a, R-VAL-3, R-VAL-4, R-VAL-6, R-VAL-7, R-UI-10…12, R-SUB-7, R-CAP-3b (version history at capture time); R-CAP-3 `:folder` (environment from the folder path) |
| M4 | R-CAP-4, R-CAP-6, R-CAP-9, R-CAP-10, R-OCR-7 `:localized`, R-OCR-9, R-OCR-10, R-OCR-15, R-L10N-2, R-UI-7 `:hidpi`, R-UI-17 `:automated` (contrast in every theme including high contrast, colour not the only signal, target size, focus rules), R-UI-17 `:manual` (screen reader, keyboard walkthrough, 200 % text, HiDPI), R-NF-1 (Linux/XWayland tests), R-NF-3, R-NF-7, R-NF-9, R-DOC-1, R-QA-2 `:required` (required classes, 1.0 criteria) |
| M5 | R-VLM-1…11, R-UI-1 `:ai` (AI job states) |
| later | R-UI-9, R-L10N-3, R-SCOPE-2 `:ui`, R-SEC-9 |

**Verified outside JUnit** (read by the traceability report; each check must exist once its milestone is completed):

| Requirement | Verified by |
|---|---|
| R-SEC-1 | CI job with the M0 negative test: a dynamic version makes the build fail (10 S-1…S-3) |
| R-SEC-2 | CI job with the M0 negative test: a tampered artifact makes the build fail; `CODEOWNERS` covers the files of 10 S-5 |
| R-SEC-3 | CI wrapper-validation job; `distributionSha256Sum` set; toolchain auto-download off in CI (10 S-10, S-11) |
| R-SEC-4 | CI lint: every `uses:` is pinned by a 40-character commit SHA and every workflow declares `permissions`; release only from protected tags (10 S-13…S-15) |
| R-SEC-5 | `.github/dependabot.yml` with the 7-day cooldown; OSV-Scanner job (10 S-7, S-19) |
| R-SEC-6 | Release checklist steps 6–7 ([release process](../release-process.md)): SHA256SUMS, SBOM, `gh attestation verify` |
| R-NF-1 | CI matrix runs `./gradlew check` on Windows and Linux; release checklist step 7 starts the app on Windows and on Linux under X11 and XWayland |
| R-NF-2 `:installers` | Release checklist step 7: install on a clean Windows and a clean Linux machine |
| R-UI-17 `:manual` | Release checklist step 7: keyboard-only walkthrough, Narrator or NVDA on Windows, 200 % text size at the minimum window size, HiDPI 100–250 % including mixed-scale monitors |
| R-DOC-1 | Release checklist steps 2 and 5: user guide updated, unverified classes named |
| R-DOC-2 | CI check that the issue templates and `SECURITY.md` exist |
| R-QA-2 `:started`, `:required` | Class-coverage matrix of the eval (R-QA-3) and release checklist step 2 |

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
| No access to an HDR monitor, an ultrawide monitor or some capture tools | Classes stay `synthetic only`/`missing`; HDR and ultrawide independence cannot be shown | Publish the capture protocol (07 §4) so that DataRunners can contribute redacted full frames; synthetic variants as interim coverage; unverified classes named in the release notes |

## Afterwards (1.x / later)

- Tray mode with notification (#36)
- Items and vehicle purchase/rental (#20)
- Further OCR writing systems (Cyrillic/Korean)
- Optionally a fine-tuned recognition model for the SC HUD font
- Trade routes are deliberately **not** planned; specialized tools exist for that
