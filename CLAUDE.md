# CLAUDE.md – UEX Datarunner Client

Desktop client (Windows + Linux) for capturing Star Citizen commodity terminal data – **manually or via screenshot OCR** – and submitting it to the **UEX API 2.0** (`POST /data_submit`). Current UEX data (terminals, commodities, latest prices, status levels, tolerances) are **constraints and default values** for recognition and selection.

**Status:** planning phase. No code exists yet. Starting point is milestone M0 in `docs/plan/04-roadmap.md`.

## Language

**Everything in the repository is English:** UI texts, code, comments, identifiers, docs, commit messages, PRs, issues. (The project owner may chat with Claude in German; that does not change the repository language.)

## Required reading before changes

| File | Contents |
|---|---|
| `docs/plan/01-requirements.md` | Requirements with IDs (`R-…`) and **assumptions A1–A23** (unverified!) |
| `docs/plan/02-architecture.md` | Modules (bounded contexts + adapters), context map, domain model, threading |
| `docs/adr/` | Architecture decision records (ADR-002: modules by bounded context) |
| `docs/release-process.md` | Versioning, release checklist, upgrade/downgrade, SC patch-day and UEX API-change procedures |
| `docs/plan/03-language-decision.md` | ADR Java vs. Rust, including licence note on basetool (GPL-3.0) |
| `docs/plan/04-roadmap.md` | Milestones M0–M5 with acceptance criteria, **requirement coverage** (R-ID → milestone, qualified tags, checks outside JUnit; drives the traceability build check in 11 §B4), risks |
| `docs/plan/05-datarunner-bug-analysis.md` | Known bugs of the predecessor (F1–F30) and our fixes |
| `docs/plan/06-uex-api.md` | API notes; **unverified points are marked** |
| `docs/plan/07-ocr-concept.md` | OCR pipeline, confidence and send threshold, AI fusion, screenshot observations |
| `docs/plan/08-review.md` | Plan review: errors and gaps found, what changed, what is open |
| `docs/plan/09-engineering-principles.md` | **Binding** rules on modularisation, clean code, tests, definition of done – including enforcement (ArchUnit, Error Prone, CI) |
| `docs/plan/10-supply-chain-security.md` | Supply chain: threat model and measures S-1…S-31 (locking, verification, pinned actions, SBOM, attestation) |
| `docs/plan/11-ddd-and-tdd.md` | **Ubiquitous language**, bounded contexts, aggregates/invariants, events; TDD rules and exceptions |

If a change contradicts a requirement or architecture decision, update the document in the same commit or ask.

A new or moved requirement ID also updates the "Requirement coverage" table in `docs/plan/04-roadmap.md` in the same commit.

## Working rules

- **Do not guess.** Verify versions, API fields and library APIs at the source before use (Maven Central, official docs, source code). If uncertain: mark it as an assumption and ask.
- **UEX API:** New or changed fields and error codes are first verified against the live API with `is_production=0`, then `docs/plan/06-uex-api.md` is updated. **Never** test against the API with `is_production=1`.
- **basetool-sc-extractor is GPL-3.0.** Do not copy or translate its code while the project licence is undecided; only re-implement concepts. The PP-OCR models are Apache-2.0 and may be bundled (with `NOTICE` and SHA-256).
- SC-Datarunner-UEX is closed source: **do not decompile**.
- No secrets (UEX secret key, app token) in code, tests, logs, fixtures or commits.
- Corpus screenshots: redact the balance ("CURRENT BALANCE") before they enter the repo. Public corpus in `corpus/public/` (rules in `corpus/README.md`); do not count entries with `"verified": false` as golden metrics.

## Tech stack (as of 2026-10-08, verified)

| Area | Choice | Version |
|---|---|---|
| Language/JDK | Java (toolchain via Foojay) | **27** (not LTS; support ends when 28 ships in March 2027, so the move to 28 is released before the April 2027 JDK security update – `docs/release-process.md`) |
| UI | JavaFX (`org.openjfx:javafx-controls`/`-fxml`, platform classifier `win`/`linux`) | 27 |
| Build | Gradle, Kotlin DSL, version catalog `gradle/libs.versions.toml` | 9.8.1 |
| OCR runtime | `com.microsoft.onnxruntime:onnxruntime` (natives win-x64/linux-x64 inside the JAR) | 1.30.0 |
| OCR models | PaddleOCR PP-OCRv6 small det + rec (ONNX, Hugging Face `PaddlePaddle/…`) | – |
| Optional AI | Local VLM via the Ollama HTTP API (not bundled, installed by the user); endpoints verified against the official Ollama docs (`docs/api.md`) | v0.40.1 (current) |
| JSON | Jackson 3: `tools.jackson.core:jackson-databind` (packages `tools.jackson.*`; **annotations stay** `com.fasterxml.jackson.annotation`) | 3.2.3 |
| HTTP | `java.net.http.HttpClient` (JDK) | – |
| DB | `org.xerial:sqlite-jdbc` | 3.53.4.0 |
| Logging | SLF4J API + Logback | 2.0.20 / 1.6.5 |
| Nullness | JSpecify + NullAway (via Error Prone) | 1.0.1 / 0.14.2 / 2.50.0 |
| Format | Spotless with google-java-format (Google Java Style, default style, 2-space indent, 100 columns) | Spotless plugin 8.10.3 / google-java-format 1.37.0 |
| Tests | JUnit Jupiter, AssertJ, Mockito, WireMock, jqwik, TestFX | 6.1.3 / 3.27.7 / 5.24.0 / 3.13.2 / 1.10.1 / 4.0.18 |
| Mutation testing | PIT, Gradle plugin `info.solidsoft.pitest`, `pitest-junit5-plugin` (verify compatibility with JUnit 6 and JDK 27 in M0) | 1.30.0 / 1.19.0 / 1.2.3 |
| Architecture/coverage | ArchUnit (`archunit-junit5`), JaCoCo | 1.5.1 / 0.8.15 (both read Java 27 class files – verified; JaCoCo's Java 27 support is experimental) |
| Supply chain | CycloneDX Gradle plugin `org.cyclonedx.bom`; OSV-Scanner CLI (release binary, SHA-256 pinned, 10 S-19); GitHub Actions `gradle/actions` (wrapper-validation, dependency-submission), `actions/attest`, `actions/setup-java`, `actions/checkout`, `actions/upload-artifact`, `actions/download-artifact` | 3.5.0; v2.6.0; v6.4.0, v4.2.2, v6.0.1, v7.0.1, v7.0.2, v8.0.2 (pin by SHA) |
| Gradle plugins | `net.ltgt.errorprone` 5.1.1, `org.beryx.jlink` 4.1.1, `com.github.ben-manes.versions` 0.65.0, `org.gradle.toolchains.foojay-resolver-convention` 1.0.0 | |

Supply-chain rules (binding, details in `10-supply-chain-security.md`):

- Only Maven Central (libraries) and the Gradle Plugin Portal (plugins); no `mavenLocal()`, no JitPack, no snapshots, no dynamic versions.
- Every dependency change updates `gradle.lockfile` (`--write-locks`) **and** `gradle/verification-metadata.xml` in the same PR, in two runs so that every artifact keeps a SHA-256 entry: `--write-verification-metadata sha256 help check`, then `--write-verification-metadata pgp,sha256 --export-keys help check` (10 S-5). New signing keys are never accepted unchecked; fingerprint and source go into the PR.
- Verification failures are never "fixed" by disabling verification; CI passes `--dependency-verification strict` and fails on any override (10 S-4).
- GitHub Actions only pinned by full commit SHA with a version comment; minimal `permissions`.
- No runtime downloads of code; OCR assets (models and dictionary) only with a fixed SHA-256 (10 S-22/S-23); optional Ollama models per 10 S-27 (digest compared with the evaluated-model list; unevaluated models capped below the send threshold); no auto-update.

Stack rules:

- **Stable** releases only; no alpha/beta/RC/milestone versions. Exceptions only with a justification in the commit.
- New dependency = check the version on Maven Central, add it to the catalog, check the licence (Apache/MIT/BSD/EPL/LGPL fine; GPL only after the licence decision).
- No heavyweight frameworks (no Spring, no ORM, no DI container); constructor injection.
- No OpenCV/JavaCV; image operations are implemented on our own immutable `ImageRaster` (ARGB `int[]`) in the core. `BufferedImage`/ImageIO only in adapters (decoding, encoding).

## Commands

Valid from M0 once the build exists:

```bash
./gradlew check                 # format check, Error Prone/NullAway, all tests
./gradlew spotlessApply         # format
./gradlew :app:run              # start the app
./gradlew test --tests '*ParserTest'          # targeted tests
./gradlew :tools:ocr-eval:run --args="--corpus $UEXDR_CORPUS_DIR"   # OCR evaluation
./gradlew dependencyUpdates     # available updates (adopt stable ones only)
./gradlew :app:jpackage         # installer for the current OS
```

`./gradlew check` must be green before every commit (including ArchUnit and the JaCoCo gate). Definition of done and review checklist: `09-engineering-principles.md` §11.

## Java conventions (modern Java, final features only)

- **No preview or incubator features** (no `--enable-preview`; e.g. structured concurrency is still preview in JDK 27).
- **Data types:** records for values and DTOs; `sealed` interfaces plus records for result and error types.
- **Control flow:** `switch` with pattern matching / record patterns instead of `instanceof` chains; exhaustive switches without `default` over sealed types; unnamed variables `_` for unused bindings.
- **Nullness:** explicit: `@NullMarked` in every package's `package-info.java`, `@Nullable` only where needed. No `Optional` fields or parameters; `Optional` only as a return type.
- **Concurrency:**
  - **Virtual threads** for I/O (`Executors.newVirtualThreadPerTaskExecutor()`).
  - CPU-heavy work runs only on **bounded** platform executors: OCR (ONNX Runtime with the typed `OcrRuntimeSettings`, never its defaults; CPU ceiling = pool size × intra-op threads) and image decoding/encoding and content hashing (intake executor). These workers run below normal OS priority (R-NF-3, 02 §7).
  - `ScopedValue` instead of `ThreadLocal` for context. Bindings do not cross executor boundaries without `StructuredTaskScope` (preview, banned), so `app` wraps every injected executor in a context-propagating decorator that re-binds the `LogContext` key in the task and sets and clears the logging MDC (the one accepted `ThreadLocal`); only `app` creates executors and threads (09 §4, §5).
  - UI updates only via an injected UI executor (`Platform::runLater` in production), so ViewModels are testable without a toolkit.
  - JNI calls (SQLite, ONNX, ImageIO JPEG decoding, ICC colour conversion) pin virtual threads; run them, and all image decoding, encoding and content hashing, on bounded platform executors.
- **Collections and streams:** sequenced collections (`getFirst()`/`getLast()`/`reversed()`), stream gatherers where they make code clearer. Immutable collections (`List.of`, `Stream.toList()`).
- **Native access:** **FFM API** (`java.lang.foreign`) for our own native calls (Credential Manager, libsecret – prefer the non-variadic `secret_password_storev_sync`/`lookupv_sync`); no JNA or JNI in our own code. Libraries (ONNX Runtime, sqlite-jdbc, JavaFX) use JNI; all native-using modules get `--enable-native-access` (JEP 472).
- **Money and units:**
  - Money as `BigDecimal`, never `double`.
  - Time as `java.time` (`Instant` internally, `Duration` for TTL and cooldown).
- **JPMS:** every module has a `module-info.java`; export API packages only.
- **Error handling:**
  - Expected failures (API status, parse errors) as return types (sealed `Result` types), not exceptions.
  - Exceptions only for programming errors and genuine I/O failures.
  - Never swallow exceptions.
- **Logging:** SLF4J with placeholders; secrets masked by a filter; image data (`screenshot`, Ollama `images`) and complete request/response bodies are never logged at any level (DEBUG: summaries only, 09 §4).
- **UI texts:** English, only via ResourceBundles (`messages.properties`); no strings in code. The bundle structure allows adding languages later without code changes.

## Architecture guardrails

- **Modules by bounded context + ports & adapters** (ADR-002): core = `shared-kernel`, the context modules `game`, `reference-data`, `capture`, `recognition`, `reporting`, `submission`, plus `workflows` for cross-context processes (no JavaFX, HTTP, SQL, ONNX, file system; time via `Clock`). Adapters (`adapter-uex`, `-storage`, `-ocr`, `-vlm`, `-files`, `-platform`) are cut by technology and implement the ports of the context modules. `ui` talks only to the context modules' application APIs and `workflows`; `app` is a pure composition root. Dependencies only along the context map in `02-architecture.md` §2, acyclic, enforced by Gradle, JPMS and ArchUnit. `submission` never depends on `reporting`; feedback goes through `workflows`. `shared-kernel` holds only IDs, value objects (incl. `ImageRaster` and `FolderLocation`), `Outcome`, the event base, the `LogContext` scoped-value key and marker annotations used by at least two contexts. Context modules export `api` and `api.model`; aggregate constructors may only be called by the aggregate and the persistence mapper (ArchUnit).
- Modules export only `api` and `api.model` (types that ports and other contexts must name, 09 §2); implementation lives in `internal`. No technology types (DTOs, `ResultSet`, `OrtSession`, JavaFX) across module boundaries.
- Package-by-feature inside a module; no service locators, no static singletons, no global mutable state.
- The core uses no `java.awt`, `javax.imageio`, file-system, network, process or native APIs (`java.nio.file`, `java.io.File*`, `java.net` except `URI`, `ProcessBuilder`, `java.lang.foreign`) and reads time only via `Clock` (ArchUnit, 09 §2); images enter as `ImageRaster`, folders as `FolderLocation`.
- Thresholds live in typed settings records with documented defaults and ranges; no magic numbers; no boolean control parameters in public APIs. **Safety thresholds can only be made stricter** than their default (send and digit threshold, staleness limit, deviation tolerances, `maxObservationAge`); the confidence ladder, the R-OCR-17 limits, the R-OCR-18 tone thresholds and the R-VAL-6 hard limit are not user settings; config values that cannot be parsed or are out of range are replaced by the default with a visible error and never used (R-NF-5).
- `sealed` only within one JPMS module; interfaces implemented by other modules (e.g. `Reader`) are not `sealed`.
- Pipeline stages are **pure functions** on immutable records, without UI, network or file-system access, so they are golden-testable.
- **Never guess silently:**
  - Every uncertain value carries a `Finding` with a reason code.
  - The submission gate blocks mandatory fields below the send threshold (starting value 0.80) unless they are confirmed or have origin `USER` (entered or corrected by the user), as well as unresolved terminals, sides and commodities. Fields with `Ambiguous` or `Unreadable` findings and prior-only repairs need a user decision whatever their numeric confidence (11 §A1, §A3 I2).
  - Repairs that rely only on the UEX prior are never applied automatically; an independent witness is required (R-VAL-2b).
  - In manual entry, UEX values are shown as reference, not pre-filled (R-MAN-2).
  - **Deviations from the previous UEX value** are highlighted in the UI by colour (plus icon and Δ text). Major deviations always require confirmation, even when recognition is confident (R-UI-10..12). Recognition confidence and deviation are separate dimensions (`FieldAssessment`); colours only via CSS theme variables.
  - If there are several candidates, the app never preselects one silently.
- **Do not hard-code UEX values:** game version and status levels come from the API or the cache, never from constants (see F14); deviation tolerances follow the effective-tolerance rule of R-VAL-2 (the UEX value, which the user can only tighten; a documented conservative settings default only while the UEX value is unverified or missing, A15) – never scattered constants.
- Free OCR strings never reach the API; only resolved IDs from the UEX vocabulary.
- **Resolution and HDR independence:**
  - No absolute pixel thresholds or absolute RGB colour thresholds in recognition code. The only exception is the R-OCR-17 legibility limits on the price-digit cap height in source pixels (typed settings record).
  - Geometry relative to the normalized panel; colour decisions relative within the panel (comparisons, hue/relative saturation after per-panel tone analysis) (R-OCR-17, R-OCR-18).
  - Formats are detected by magic bytes; HDR-encoded files (PQ/HLG `cICP`, JPEG XR, AVIF, EXR) are never read as sRGB (R-CAP-8).
  - Every new recognition rule is evaluated per corpus class (resolution, text-height band, HDR, capture method) and must keep the metamorphic gate green (07 §4).
- **Image intake:**
  - Images come from **user-defined folders**, by default only when the user clicks "Import". Automatic import when new files appear is opt-in per folder (WatchService plus polling fallback).
  - A newly added folder imports only captures newer than its cutoff (default: time of adding minus the R-VAL-6 hard limit); importing an older backlog is an explicit user choice (R-CAP-1).
  - Read files only once they are completely written (stable-file gate).
  - The capture queue holds persisted file references (Captures not yet scanned: `Imported`, `EnvironmentPending`, `Ready`), never decoded rasters; the bounded OCR job takes only `Ready` captures and decodes inside the job.
  - Every file is processed only once (processed-file register with hash).
  - Never add or remove folders silently.
- **Crops never contain the balance:** the shop-panel and location-field crops end at the panel top edges, and everything above them (the terminal header with the balance) is excluded geometrically, never by searching for the balance text, before the working copy is stored. Review, VLM, upload, misread export and diagnostics use only these redacted crops (test mandatory for each path). If the crop geometry was not established from located anchors (manual corners or crop, layout-profile fallback, unvalidated game version), and for manual attachments before automatic locate exists (M1), a confirmed full-size preview is required before release (R-SUB-7).
- **Submission lifecycle:** automatic retries only for requests that provably were not processed (R-SUB-9: connect failure, 429, `requests_limit_reached`). Every other failure after sending began – including shutdown, crash or kill while sending (a user cancel is refused once the job is sending) – is an unknown outcome: never retried automatically and never sent to the fallback host. Partial `ids_reports` are never treated as success and are mapped to rows by position only when the counts match. Error classes per R-SUB-11; observation age limits per R-VAL-6.
- **Do not compete with the game:** background events never open dialogs or windows, take focus or change the window state; they show as persistent state with an action (R-UI-16, 09 §7).
- **Optional AI recognition (VLM/Ollama):**
  - Classic OCR is always the primary path; the app must be fully functional without Ollama.
  - In "Automatic" mode the VLM runs **only while the game is closed**. When the game starts, the request is cancelled and the model unloaded immediately (`keep_alive: 0`); jobs are never lost.
  - Only a loopback host (definition in R-VLM-6) without asking; no remote (cloud) models without explicit consent, classified by Ollama's `remote_host`/`remote_model` metadata, not by name alone.
  - VLM results go through the same resolution and validation as OCR. Fusion follows the rules in `07-ocr-concept.md` §2.7.
  - The AI never overwrites a field the user confirmed or one with origin `USER` (typed or corrected); neither does a new capture, a re-stitch or a merge (11 §A3 I7).
  - CI does not start Ollama; the parser is tested with recorded responses; live runs are opt-in via `UEXDR_VLM_HOST`.

## Tests

**Way of working: TDD.** In the core modules no production code is written without a previously failing test (red → green → refactor). New use cases start outside-in with an acceptance test at the application-service level of the context module (or `workflows`), tagged with the requirement ID (`@Tag("R-…")`; qualified, e.g. `@Tag("R-VAL-1:ocr")`, where 04 splits a requirement – 11 §B4). Port fakes from `java-test-fixtures` take precedence over Mockito. For OCR heuristics a spike in the eval harness is allowed; before merging, the behaviour is pinned down with failing golden and unit tests (11 §B1). Mutation testing (PIT) checks that the tests actually catch defects.

**Modelling: DDD.** Domain terms exactly as in 11 §A1. Contexts as modules; other aggregates only by ID; aggregates are immutable records, commands return an `Outcome` (new state + events, or a refusal reason). The invariants of `Report` (I1–I7) belong in the aggregate, not in the UI or services. UEX, game and Ollama formats are translated only in the adapters (anti-corruption layer).

- **Unit:** parsers (price, SCU, status, cargo sizes) with jqwik property tests; fuzzy matcher; validation rules; stitching.
- **API:** WireMock with recorded (anonymised) responses. A live test runs only opt-in and always with `is_production=0`: locally via `UEXDR_LIVE_TEST=1`, or in the weekly scheduled API-drift job (04 M1), which runs the prebuilt API-spike CLI, without Gradle, with the key of a dedicated UEX test account from a protected GitHub environment (10 S-14) and never runs on pull requests. Without the variable the JUnit live test is skipped (`Assumptions.assumeTrue`).
- **OCR:**
  - Synthetic images for locate and layout; synthetic class variants (scale, tone, canvas, JPEG) and the metamorphic gate (07 §4).
  - The golden corpus via `UEXDR_CORPUS_DIR`; without the variable the test is skipped, never falsely green (`Assumptions.assumeTrue`).
  - Digest test for model and runtime updates.
- **"Silently wrong" metric** (wrong and marked as confident): regressions are blockers.
- **UI:** TestFX only for critical flows (onboarding, submission block).
- Bug fixes always come with a test that reproduces the bug first.

## Git

- Small, focused commits; messages in English, imperative mood (`Add price parser for currency glyph`).
- Never commit generated artefacts, models > 50 MB, private screenshots or secrets. OCR assets (models, dictionary) with their hash in `NOTICE`.
