# CLAUDE.md – UEX Datarunner Client

Desktop client (Windows + Linux) for capturing Star Citizen commodity terminal data – **manually or via screenshot OCR** – and submitting it to the **UEX API 2.0** (`POST /data_submit`). Current UEX data (terminals, commodities, latest prices, status levels, tolerances) are **constraints and default values** for recognition and selection.

**Status:** planning phase. No code exists yet. Starting point is milestone M0 in `docs/plan/04-roadmap.md`.

## Language

**Everything in the repository is English:** UI texts, code, comments, identifiers, docs, commit messages, PRs, issues. (The project owner may chat with Claude in German; that does not change the repository language.)

## Required reading before changes

| File | Contents |
|---|---|
| `docs/plan/01-requirements.md` | Requirements with IDs (`R-…`) and **assumptions A1–A14** (unverified!) |
| `docs/plan/02-architecture.md` | Modules, dependency direction, domain model, threading |
| `docs/plan/03-language-decision.md` | ADR Java vs. Rust, including licence note on basetool (GPL-3.0) |
| `docs/plan/05-datarunner-bug-analysis.md` | Known bugs of the predecessor (F1–F30) and our fixes |
| `docs/plan/06-uex-api.md` | API notes; **unverified points are marked** |
| `docs/plan/07-ocr-concept.md` | OCR pipeline, confidence and send threshold, AI fusion, screenshot observations |
| `docs/plan/08-review.md` | Plan review: errors and gaps found, what changed, what is open |
| `docs/plan/09-engineering-principles.md` | **Binding** rules on modularisation, clean code, tests, definition of done – including enforcement (ArchUnit, Error Prone, CI) |
| `docs/plan/10-supply-chain-security.md` | Supply chain: threat model and measures S-1…S-31 (locking, verification, pinned actions, SBOM, attestation) |
| `docs/plan/11-ddd-and-tdd.md` | **Ubiquitous language**, bounded contexts, aggregates/invariants, events; TDD rules and exceptions |

If a change contradicts a requirement or architecture decision, update the document in the same commit or ask.

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
| Language/JDK | Java (toolchain via Foojay) | **27** (not LTS; plan the move to 28 in March 2027) |
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
| Architecture/coverage | ArchUnit (`archunit-junit5`), JaCoCo | 1.5.1 / 0.8.15 (verify JDK 27 support in M0) |
| Supply chain | CycloneDX Gradle plugin `org.cyclonedx.bom`; OSV-Scanner; GitHub Actions `gradle/actions` (wrapper-validation, dependency-submission), `actions/attest-build-provenance`, `actions/setup-java`, `actions/checkout` | 3.5.0; v2.6.0; v6.4.0, v4.2.2, v6.0.1, v7.0.1 (pin by SHA) |
| Gradle plugins | `net.ltgt.errorprone` 5.1.1, `org.beryx.jlink` 4.1.1, `com.github.ben-manes.versions` 0.65.0, `org.gradle.toolchains.foojay-resolver-convention` 1.0.0 | |

Supply-chain rules (binding, details in `10-supply-chain-security.md`):

- Only Maven Central (libraries) and the Gradle Plugin Portal (plugins); no `mavenLocal()`, no JitPack, no snapshots, no dynamic versions.
- Every dependency change updates `gradle.lockfile` (`--write-locks`) **and** `gradle/verification-metadata.xml` (`--write-verification-metadata pgp,sha256 --export-keys`) in the same PR. New signing keys are never accepted unchecked; fingerprint and source go into the PR.
- Verification failures are never "fixed" by disabling verification.
- GitHub Actions only pinned by full commit SHA with a version comment; minimal `permissions`.
- No runtime downloads of code; models only with a fixed SHA-256; no auto-update.

Stack rules:

- **Stable** releases only; no alpha/beta/RC/milestone versions. Exceptions only with a justification in the commit.
- New dependency = check the version on Maven Central, add it to the catalog, check the licence (Apache/MIT/BSD/EPL/LGPL fine; GPL only after the licence decision).
- No heavyweight frameworks (no Spring, no ORM, no DI container); constructor injection.
- No OpenCV/JavaCV; image operations are implemented on `BufferedImage` or `int[]` rasters.

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
  - CPU-heavy OCR runs in a **bounded** platform thread pool.
  - `ScopedValue` instead of `ThreadLocal` for context.
  - UI updates only via `Platform.runLater`.
- **Collections and streams:** sequenced collections (`getFirst()`/`getLast()`/`reversed()`), stream gatherers where they make code clearer. Immutable collections (`List.of`, `Stream.toList()`).
- **Native access:** **FFM API** (`java.lang.foreign`) for native calls (Credential Manager, libsecret); no JNA, no JNI.
- **Money and units:**
  - Money as `BigDecimal`, never `double`.
  - Time as `java.time` (`Instant` internally, `Duration` for TTL and cooldown).
- **JPMS:** every module has a `module-info.java`; export API packages only.
- **Error handling:**
  - Expected failures (API status, parse errors) as return types (sealed `Result` types), not exceptions.
  - Exceptions only for programming errors and genuine I/O failures.
  - Never swallow exceptions.
- **Logging:** SLF4J with placeholders; secrets masked by a filter; no image or payload dumps at INFO.
- **UI texts:** English, only via ResourceBundles (`messages.properties`); no strings in code. The bundle structure allows adding languages later without code changes.

## Architecture guardrails

- **Ports & adapters:** core = `domain`, `pipeline`, `application` (no JavaFX, HTTP, SQL, ONNX, file system; time via `Clock`). Adapters (`adapter-*`) implement ports from `domain`. `ui` talks only to `application`; `app` is a pure composition root. Dependencies only as in `02-architecture.md` §2, acyclic, enforced by JPMS and ArchUnit.
- Modules export only their `api` package; implementation lives in `internal`. No technology types (DTOs, `ResultSet`, `OrtSession`, JavaFX) across module boundaries.
- Package-by-feature inside a module; no service locators, no static singletons, no global mutable state.
- Thresholds live in typed settings records with documented defaults; no magic numbers; no boolean control parameters in public APIs.
- `sealed` only within one JPMS module; interfaces implemented by other modules (e.g. `Reader`) are not `sealed`.
- Pipeline stages are **pure functions** on immutable records, without UI, network or file-system access, so they are golden-testable.
- **Never guess silently:**
  - Every uncertain value carries a `Finding` with a reason code.
  - The submission gate blocks fields below the send threshold (starting value 0.80) and unresolved terminals.
  - Repairs that rely only on the UEX prior are never applied automatically; an independent witness is required (R-VAL-2b).
  - In manual entry, UEX values are shown as reference, not pre-filled (R-MAN-2).
  - **Deviations from the previous UEX value** are highlighted in the UI by colour (plus icon and Δ text). Major deviations always require confirmation, even when recognition is confident (R-UI-10..12). Recognition confidence and deviation are separate dimensions (`FieldAssessment`); colours only via CSS theme variables.
  - If there are several candidates, the app never preselects one silently.
- **Do not hard-code UEX values:** game version, status levels and tolerances come from the API or the cache, never from constants (see F14).
- Free OCR strings never reach the API; only resolved IDs from the UEX vocabulary.
- **Image intake:**
  - Images come from **user-defined folders**, by default only when the user clicks "Import". Automatic import when new files appear is opt-in per folder (WatchService plus polling fallback).
  - Read files only once they are completely written (stable-file gate).
  - Every file is processed only once (processed-file register with hash).
  - Never add or remove folders silently.
- The upload screenshot contains only the shop panel and the location field; **the balance is always redacted** (test mandatory).
- **Optional AI recognition (VLM/Ollama):**
  - Classic OCR is always the primary path; the app must be fully functional without Ollama.
  - In "Automatic" mode the VLM runs **only while the game is closed**. When the game starts, the request is cancelled and the model unloaded immediately (`keep_alive: 0`); jobs are never lost.
  - Only `localhost` without asking; no cloud models without explicit consent.
  - VLM results go through the same resolution and validation as OCR. Fusion follows the rules in `07-ocr-concept.md` §2.7.
  - The AI never overwrites fields the user has confirmed.
  - CI does not start Ollama; the parser is tested with recorded responses; live runs are opt-in via `UEXDR_VLM_HOST`.

## Tests

**Way of working: TDD.** In the core (`domain`, `pipeline`, `application`) no production code is written without a previously failing test (red → green → refactor). New use cases start outside-in with an acceptance test at `application` level, tagged with the requirement ID (`@Tag("R-…")`). Port fakes from `java-test-fixtures` take precedence over Mockito. For OCR heuristics a spike in the eval harness is allowed; before merging, the behaviour is pinned down with failing golden and unit tests (11 §B1). Mutation testing (PIT) checks that the tests actually catch defects.

**Modelling: DDD.** Domain terms exactly as in 11 §A1. Contexts as packages; other aggregates only by ID; aggregates are immutable records, commands return an `Outcome` (new state + events, or a refusal reason). The invariants of `Report` (I1–I5) belong in the aggregate, not in the UI or services. UEX, game and Ollama formats are translated only in the adapters (anti-corruption layer).

- **Unit:** parsers (price, SCU, status, cargo sizes) with jqwik property tests; fuzzy matcher; validation rules; stitching.
- **API:** WireMock with recorded (anonymised) responses. A live test runs only manually, opt-in via `UEXDR_LIVE_TEST=1` and with `is_production=0`.
- **OCR:**
  - Synthetic images for locate and layout.
  - The golden corpus via `UEXDR_CORPUS_DIR`; without the variable the test is skipped, never falsely green (`Assumptions.assumeTrue`).
  - Digest test for model and runtime updates.
- **"Silently wrong" metric** (wrong and marked as confident): regressions are blockers.
- **UI:** TestFX only for critical flows (onboarding, submission block).
- Bug fixes always come with a test that reproduces the bug first.

## Git

- Small, focused commits; messages in English, imperative mood (`Add price parser for currency glyph`).
- Never commit generated artefacts, models > 50 MB, private screenshots or secrets. Models with their hash in `NOTICE`.
