# Engineering Principles: Modularization, Maintainability, Clean Code

This document makes "best practices" **verifiable**: every rule states *how* it is enforced. Where possible this happens automatically (compiler, JPMS, ArchUnit, Error Prone, CI), otherwise via the review checklist and the Definition of Done.

Rules without enforcement are wishful thinking. That is why there are deliberately few rules, and they are binding.

## 1. Quality goals (descending priority)

1. **Correctness of the submitted data:** no silently wrong values sent to UEX.
2. **Maintainability:** SC patches change the terminal layout, UEX changes the API. Such changes should stay local to one module.
3. **Testability:** domain logic testable without UI, network, database and game.
4. **Security and privacy:** secret key, balance.
5. **Portability:** Windows and Linux.
6. **Resource efficiency** alongside the running game.

## 2. Modularization

| Rule | Enforcement |
|---|---|
| Ports & Adapters: the core modules (`shared-kernel`, the context modules, `workflows`) know no technology | JPMS: core modules `requires` only other core modules (along the context map) and JSpecify; this blocks every module outside `java.base` (`javafx.*`, `java.desktop` with `java.awt`/`javax.imageio`, `java.net.http`, `java.sql`, `java.logging`, ONNX Runtime, `tools.jackson.*`, `com.fasterxml.jackson.annotation`). **ArchUnit** covers what JPMS cannot hide in `java.base`: no class in the core depends on `java.nio.file..`, `java.nio.channels..`, `java.io.File`, `FileInputStream`, `FileOutputStream`, `FileReader`, `FileWriter`, `RandomAccessFile`, `java.net..` (except `java.net.URI`), `ProcessBuilder`, `ProcessHandle` or `java.lang.foreign..`, or calls `Runtime.exec`, `System.load`/`System.loadLibrary`, the no-argument `now()` of `java.time` types or `System.currentTimeMillis` (time only via the injected `Clock`). As a second check it also lists the external packages (`javafx..`, `java.awt..`, `javax.imageio..`, `java.net.http..`, `java.sql..`, `ai.onnxruntime..`, `tools.jackson..`, `com.fasterxml.jackson..`). Images enter as `ImageRaster`, locations as `FolderLocation`. |
| Modules cut by bounded context ([ADR-002](../adr/0002-modules-by-bounded-context.md)); dependency direction as in the context map in [02 §2](02-architecture.md), free of cycles | Gradle project dependencies plus JPMS (cycles do not compile); ArchUnit `slices().matching("..<module>.(*)..").should().beFreeOfCycles()` also for packages **within** a module |
| Every module has a narrow public API | `module-info.java` exports only `…<module>.api` and `…<module>.api.model` (types that ports and other contexts must name); the implementation lives in `…<module>.internal`; ArchUnit: no access to other modules' `internal` packages |
| Adapters do not depend on each other; they are leaves (only `app` and the second composition root `tools/ocr-eval` may depend on adapters) | Gradle dependencies allowed centrally in `build-logic`; ArchUnit rule per adapter |
| `shared-kernel` contains only value objects, IDs, `Outcome`, event base, the `LogContext` scoped-value key (§4) and marker annotations used by at least two contexts | ArchUnit: no aggregates, services or ports in `shared-kernel`; CODEOWNERS review for every change |
| Cross-context processes only in `workflows`; context modules never call each other's application services backwards against the context map | Gradle/JPMS (missing dependency = does not compile) plus ArchUnit |
| No passing of technology types across module boundaries (DTOs, `ResultSet`, `OrtSession`, JavaFX types) | Port signatures use only the exported types (`api`/`api.model`) of their own module, of `shared-kernel`, or of upstream context modules along the context map in [02 §2](02-architecture.md) (published language, e.g. `GameEnvironment`, `GameVersion`, `Finding`), plus non-technology JDK types (`java.lang`, `java.util`, `java.time`, `java.math`); adapters map at the boundary (ArchUnit: port signatures only with these types) |
| Package-by-feature within a module (e.g. `recognition.locate`, `recognition.parse`, `recognition.stitch`) instead of package-by-layer | Review checklist |
| `app` contains only wiring and startup | ArchUnit: no class in `app` except `Main`, `*Wiring`/`*Module`, the `ContextPropagatingExecutor` (§4) and the bootstrap loader (storage locations per 02 §8, portable marker; user settings go through the `*SettingsStore` ports in `adapter-files`, and `*Wiring` passes setting values to adapters as `Supplier`s backed by the stores, 02 §2); line budget as a review hint |
| No service locator, no static singletons, no global mutable state | ArchUnit: no non-final `static` fields; `static final` only for loggers and immutable constants |

## 3. Clean Code

| Rule | Enforcement |
|---|---|
| **Meaningful names** in the ubiquitous language of [11 §A1](11-ddd-and-tdd.md) (e.g. `Capture`, `Scan`, `Report`, `PricePrior`, `Finding`) – the same everywhere | Review; 11 §A1 is binding |
| **One responsibility** per class and method. Guideline values: methods ≤ ~30 lines, classes ≤ ~300 lines. Exceeding them requires a reason. | Review checklist (guideline, not dogma) |
| **No magic numbers:** thresholds (send threshold, tolerances, quiet periods, hysteresis, grouping windows) live in typed settings records with documented defaults and ranges, not scattered across the code. UEX values come from `ReferenceSnapshot`; deviation tolerances follow the effective-tolerance rule (R-VAL-2), the only place where a UEX value has a settings default. **Safety thresholds** (send and digit threshold, staleness limit, deviation tolerances, `maxObservationAge`) accept only values at least as strict as their default; release-calibrated values (confidence ladder, R-OCR-17 limits, R-OCR-18 tone thresholds, R-VAL-6 hard limit) are not user settings (R-NF-5). | Review; Error Prone; tests check the defaults in one place; the compact constructor of each settings record checks its range; a jqwik property test shows that no accepted settings value makes a prior-only repair, an `Ambiguous` or an `Unreadable` field sendable without a user decision, or removes a confirmation that the defaults require |
| **Immutable by default:** records, `List.copyOf`, no setters in the core; aggregates are records too (commands return new state + events, 11 §A6) | ArchUnit: domain-model classes in the core modules are records, enums, sealed interfaces, ports (interfaces) or annotations (documented exemption: `ImageRaster`); the canonical constructor of an aggregate may only be called by the aggregate itself and the persistence mapper in `adapter-storage`; DDD rules from 11 §A8 |
| **No boolean control parameters** in public APIs (`process(x, true)`) → enums or separate methods | Review |
| **Null-free:** JSpecify `@NullMarked`, NullAway at error level | The build fails on violations |
| **Errors are values:** expected errors as sealed `Result`/`Finding` types, no exceptions for control flow; never swallow exceptions | Error Prone (`CatchAndPrintStackTrace`, unused return values via `@CheckReturnValue`), review |
| **Comments explain the why**, not the what. Public ports and modules have Javadoc. | Javadoc lint for exported packages (`-Xdoclint` on `api` packages) |
| **No dead code, no commented-out blocks**; `TODO` only with an issue number | Error Prone (`UnusedVariable`, `UnusedMethod`); a CI step checks for `TODO` without `#<Issue>` |
| **Uniform formatting** | Spotless with google-java-format (Google Java Style) in `check`; formatting is never discussed by hand |
| **Small, testable pure functions** in `recognition`; side effects only in adapters | Module split plus ArchUnit (see §2) |

## 4. Error handling, logging, observability

- **Error categories:**
  - domain (`Finding`, UEX status codes) → explained to the user
  - technically expected (network down, file locked) → retry or notice
  - programming errors → exception, log at ERROR, diagnostics export
- **User texts** only via keys in ResourceBundles, with English texts; error codes are mapped to keys centrally (one table, covered by tests: every known UEX code has a text).
- **Logging:** SLF4J; ONNX Runtime logs via `java.util.logging`, bridged with `jul-to-slf4j`.
  - **Context** via MDC (`captureId`, `reportId`) – the one accepted `ThreadLocal`. The core binds the log context as one `ScopedValue<LogContext>` (`LogContext`: `@Nullable CaptureId`, `@Nullable ReportId`; record and key in `shared-kernel`). Neither the MDC nor `ScopedValue` bindings cross executor boundaries (`ScopedValue` inheritance needs `StructuredTaskScope`, which is preview in JDK 27 and banned), so `app` wraps **every** injected executor (virtual-thread I/O, bounded OCR, bounded JNI/SQLite, UI executor, the `HttpClient` executor) in one `ContextPropagatingExecutor`. At submit time it captures `KEY.isBound() ? KEY.get() : null`; in the task it re-binds with `ScopedValue.where(KEY, ctx).call(…)`/`.run(…)`, sets the MDC and clears it in `finally`; without a context the task runs unbound with an empty MDC. One parameterized test (platform pool, virtual-thread executor, direct executor) proves that the context is visible inside the task and the MDC is empty afterwards.
  - **Content:** secrets, image data (the `/data_submit` `screenshot` field, Ollama `images`, raster bytes) and complete request or response bodies are never logged, at any level. At DEBUG, payloads are summarised (row count, byte sizes, SHA-256). `GameProcessMonitor` logs only matched/not matched plus the matched executable path, never other processes' command lines. Tests prove that the masking filter takes effect and that a DEBUG-level submit and VLM call log no Base64 image data.

## 5. Concurrency

- Shared state is immutable (records). Mutable state has **exactly one owner** (e.g. the queue service) and is changed only via its API.
- Executors are injected and closed in an orderly way on shutdown (`AutoCloseable`, `try-with-resources` in `app`). There are no self-created threads in domain code. Only `app` creates executors and threads (and the second composition root `tools/ocr-eval` for its own runs): ArchUnit forbids `java.util.concurrent.Executors`, `new Thread`, `Thread.ofPlatform`/`Thread.ofVirtual`/`Thread.startVirtualThread` and `HttpClient.Builder.executor` outside them. Adapters receive injected, wrapped executors (WatchService loop, game monitor) or an `HttpClient` that `app` builds with a wrapped executor (`adapter-uex`, `adapter-vlm`; §4).
- Cancellation is a regular path: AI run, import and submission are cancellable, and this is tested. A user cancel is refused once a submission job is `sending`; a shutdown, crash or kill after that leads to an unknown outcome, never to a re-send (R-SUB-9).
- Time always comes via `java.time.Clock` (injected): this makes cooldown, hysteresis and grouping deterministically testable.

## 6. Persistence

- `adapter-storage` owns the connection. Every domain table belongs to exactly one repository.
- **No SQL outside the repositories** (ArchUnit: `java.sql..` only in `adapter-storage`).
- Schema migrations are versioned, forward-only and each has a test (empty DB → current version; previous version with test data → current version).
- The configuration file has one schema version, one migration and one writer: all `*SettingsStore` implementations share the `ConfigFile` component in `adapter-files` (02 §2; F17).

## 7. UI (MVVM)

- **Views** are passive (layout, binding, CSS) and contain no logic.
- **ViewModels** hold the UI state and call use cases; they are unit-testable without a window.
- Formatting (numbers, Δ %, "3 days ago") lives in **one** place (formatter class) and is localized.
- The display of deviation and confidence is derived via the CSS pseudo-classes listed in 02 §7 from `Field` (assessment, confirmation, origin); the ViewModel contains no thresholds, and there are no color values in Java code.
- **Dialogs only from user actions** (R-UI-16): dialogs and windows are opened only through one `ui` `DialogPresenter`. ArchUnit forbids constructing `Alert`/`Dialog`/`Stage` and calling the window-level `Stage.toFront`, `Window.requestFocus`, `Stage.setIconified`, `Stage.setMaximized` or `Stage.setAlwaysOnTop` outside it; `Node.requestFocus` only moves the focus owner within its scene and stays allowed for keyboard navigation (R-UI-17). ViewModel tests feed every background event type in and assert that no dialog request comes out, only state or notice changes. One TestFX test checks that a background event while the window is unfocused or minimised creates no new `Stage` and changes neither the focus nor the iconified state.
- **Rendering load next to the game** (R-NF-3): no continuous animations and no indeterminate progress. ArchUnit forbids in `ui` the no-argument constructors of `ProgressBar` and `ProgressIndicator` (they create indeterminate controls) and calls to `Animation.setCycleCount` (repetition needs it; timers use the injected UI executor instead); ViewModel tests assert that progress values are never negative.

## 8. Tests (test pyramid)

Approach: **Test-Driven Development** and outside-in; rules, exceptions and tools (fakes in `java-test-fixtures`, PIT mutation testing, traceability via requirement tags) in [11-ddd-and-tdd.md](11-ddd-and-tdd.md) Part B.


| Level | What | Tool |
|---|---|---|
| Unit (base, the majority) | core modules (context modules, `shared-kernel`, `workflows`) with fakes of the ports | JUnit 6, AssertJ, jqwik (parser, fuzzy matcher, fusion, deviation assessment) |
| Architecture | Rules from §2/§3/§5/§6/§7 | ArchUnit |
| Adapter integration | SQLite (real file DB in the temp folder), UEX client against WireMock, folder watcher against a real temp directory | JUnit, WireMock |
| Contract | UEX responses (recorded, anonymized) against our DTOs | JUnit |
| Live API drift (opt-in) | field names and error codes against the recorded contracts, always `is_production=0` | weekly scheduled CI job (04 M1, 10 S-14) or locally via `UEXDR_LIVE_TEST=1`; skipped otherwise |
| Golden/corpus | OCR and pipeline result against expected values, metric "silently wrong" | `tools/ocr-eval`, opt-in via `UEXDR_CORPUS_DIR` |
| UI (few) | Onboarding, submission lock, deviation confirmation, no dialog or focus change on background events (§7) | TestFX |

Further test rules:

- **Test names** describe the behavior (`rejectsSubmissionWhenMajorDeviationUnconfirmed`); structure Given/When/Then; test data via builders instead of copy-paste.
- **Flaky tests** are not tolerated: fix the cause, do not disable them.
- **Coverage gate (JaCoCo):** initial value ≥ 85 % lines for the core modules. There is no quota for adapters; integration tests instead. The quota serves to find gaps, not as an end in itself.

## 9. Dependencies and build

- Versions only in the version catalog; convention plugins in `build-logic` instead of copy-paste in `build.gradle.kts`.
- **Dependabot** (GitHub-native) for Gradle and GitHub Actions; updates only to stable versions, each with green CI.
- A new dependency only with a justification in the PR. License, maintenance status (last release, maintainers) and size must be checked. Better 50 lines of our own code than a heavy library for one function.
- Reproducible and verified builds: Gradle wrapper with checksum, dependency locking, dependency verification (SHA-256 for every artifact, plus PGP where signed), SHA-pinned actions. The details and the threat model are in [10-supply-chain-security.md](10-supply-chain-security.md).

## 10. Documentation and decisions

- **ADRs** for architecture decisions under `docs/adr/NNNN-title.md`. ADR-001 is the language decision (historically located at [03](03-language-decision.md)); ADR-002 is [modules by bounded context](../adr/0002-modules-by-bounded-context.md); more will follow, e.g. for the deviation model and the license.
- `docs/plan/` remains the domain source of truth. Code that deviates from it changes the document in the same PR.
- `CHANGELOG.md` following "Keep a Changelog"; versioning following **SemVer**. Persisted data (DB, config) counts as a public interface: if the format breaks, there is a migration.

## 11. Process: Definition of Done and review checklist

**Definition of Done** for every change:

- [ ] `./gradlew check` green on Windows and Linux (CI matrix)
- [ ] Developed via TDD in the core: test first (red), then implementation (green), then refactoring; bugfix with a reproducing test
- [ ] Domain terms match the Ubiquitous Language (11 §A1); aggregate invariants are covered by tests
- [ ] No new ArchUnit, Error Prone or NullAway violations; no suppression without a comment giving the reason
- [ ] UI texts in English, via ResourceBundles (no hard-coded strings)
- [ ] Affected plan or ADR documents updated
- [ ] No secrets, private screenshots or large binary files in the diff
- [ ] For changed dependencies: lockfile and `verification-metadata.xml` updated in the same PR; new signing keys checked and documented in the PR

**Review checklist** (PR template `.github/pull_request_template.md`):

- Is the code in the right module and package? Is a new module edge needed – and allowed?
- Is the domain logic free of technology?
- Are there magic numbers, boolean parameters, null returns or swallowed exceptions?
- Do names conform to the glossary?
- Are error cases and cancellation handled and tested?
- Is the change as small as possible (one thing per PR)?

## 12. Tool versions (checked on 2026-10-08)

| Tool | Version | Note |
|---|---|---|
| ArchUnit (`com.tngtech.archunit:archunit-junit5`) | 1.5.1 | Reads Java 27 class files (verified in the third review, 08 §I) |
| JaCoCo | 0.8.15 (June 2026) | Experimental support for Java 27 class files per the 0.8.15 changelog (verified, 08 §I); if instrumentation fails in CI, suspend the coverage gate until the update (do not switch the JDK) |
| Others | see [CLAUDE.md](../../CLAUDE.md) | |
