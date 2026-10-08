# ADR-002: Cut the core modules by bounded context, keep adapters by technology

- **Status:** Accepted (2026-10-08)
- **Supersedes:** the layer-based core modules (`domain`, `pipeline`, `application`) in `docs/plan/02-architecture.md` §2
- **Related:** [ADR-001 language decision](../plan/03-language-decision.md), [11-ddd-and-tdd.md](../plan/11-ddd-and-tdd.md)

## Context

The plan uses DDD with six bounded contexts (Capture, Recognition, Reporting, Submission, Reference Data, Game Environment). The Gradle/JPMS modules, however, were cut **by technical layer**:

- `domain` held all entities, value objects and ports of all contexts;
- `pipeline` held the recognition logic;
- `application` held the use cases of all contexts;
- adapters were split by technology.

The bounded contexts existed only as **packages**, protected by ArchUnit rules. Question from the project owner: can the modules be designed along DDD lines, or would that bring disadvantages?

## Options

| | A: Layer modules (status quo) | B: Pure context modules | C: Hybrid (chosen) |
|---|---|---|---|
| Core | `domain`, `pipeline`, `application` | one module per context, **including its own adapters** | one module per context (domain + application, pure); `shared-kernel`; `workflows` for cross-context processes |
| Adapters | by technology | per context (e.g. `reporting-storage`, `submission-uex`, `reference-uex`) | by technology, internally packaged per context |
| Module count | ~12 | ~25+ | ~17 |

## Evaluation

**Advantages of cutting the core by context (B and C):**

1. **Boundaries enforced by the compiler.** JPMS refuses access to non-exported packages of another context. With A this rested only on ArchUnit rules.
2. **High cohesion.** Everything about the reporting rules (Report aggregate, invariants I1–I5, submission gate, deviation assessment, review use cases) is in **one** module. With A it was spread over `domain` and `application`.
3. **Change locality.**
   - A Star Citizen UI patch changes `recognition` (and `adapter-ocr`).
   - A UEX API change changes `adapter-uex` (and at most `reference-data`).
   - Neither touches a shared "god" `domain` module.
4. **Ubiquitous language per context.** A term like "Terminal" can mean the full UEX master record in Reference Data and just a `TerminalId` plus display name in Reporting, without one class having to serve both.
5. **Tests and build per context.** Gradle rebuilds and re-tests only affected modules. PIT and JaCoCo run per context. `tools/ocr-eval` depends on `recognition` and the adapters it needs instead of the whole domain; it is a second composition root.

**Disadvantages and how we handle them (honest):**

| # | Disadvantage | Severity | Handling |
|---|---|---|---|
| D1 | More modules (`build.gradle.kts`, `module-info.java` each) | low | Convention plugins in `build-logic`: a module's build file is one or two lines |
| D2 | Cross-context flows (capture → recognition → reporting → submission; AI re-read) need **explicit orchestration** and **translation** between context models (e.g. `CardReading` → `ReportRow`) – extra code | medium | Small `workflows` module (process managers). The translation code is deliberate: it is where context boundaries become visible and testable. |
| D3 | A wrong cut is more expensive to fix than with packages (moving types between JPMS modules touches `module-info`, exports, build files) | medium | The flow is linear and the contexts follow it, so the cut risk is low. Review the cut after M3 with real code; merging two context modules is allowed if they turn out to change together. |
| D4 | The shared kernel tends to grow into a new "god module" | medium | Rule: only value objects used by **at least two** contexts. Changes need the maintainer's review (CODEOWNERS). ArchUnit: no aggregates, services or ports in `shared-kernel`. |
| D5 | Over-engineering risk for a desktop app with one or few developers | medium | Only six contexts, no microservices, no messaging; in-process synchronous events. Option B (adapters per context) is rejected for exactly this reason. |
| D6 | Cyclic dependencies between contexts (e.g. Submission reports success back to Reporting) | high if ignored | Context map with **one direction only** (below). Feedback flows are handled in `workflows`, not by a back-reference. |
| D7 | Adapters serving several contexts depend on several context modules | low | Accepted; adapters are leaves (nothing depends on them except the composition roots `app` and `tools/ocr-eval`). |

**Why not option B (adapters per context as well)?** It would double the module count and duplicate infrastructure: three HTTP clients for UEX, several SQLite connection setups and migration runners. The benefit (independent deployability of contexts) is worthless for a single desktop application.

## Decision

Option **C**:

```
── Core (pure: no JavaFX, HTTP, SQL, ONNX, file system) ──
shared-kernel     IDs, money/quantity value objects, Outcome, domain-event base, DDD marker annotations
game              Game Environment context: environment, version, game state; port GameStateProbe
reference-data    Reference Data context: ReferenceSnapshot, vocabulary indexes, fuzzy matcher; port ReferenceDataSource
capture           Capture context: Capture, WatchedFolder, import use cases; ports CaptureSource, CaptureRepository
recognition       Recognition context (formerly "pipeline"): locate, layout, parse, resolve, fuse, validate, stitch; ports Reader, TextDetector
reporting         Reporting context: Report aggregate, submission gate, deviation assessment, grouping, review use cases; port ReportRepository
submission        Submission context: SubmissionJob, Cooldown, queue; ports SubmissionGateway, repositories
workflows         Process managers across contexts: capture→recognition→reporting, reporting→submission, AI re-read (RecognitionPolicy, AI queue)

── Adapters (by technology, internally packaged per context) ──
adapter-uex, adapter-storage, adapter-ocr, adapter-vlm, adapter-files, adapter-platform

── Presentation and startup ──
ui, app, tools/ocr-eval
```

**Context map (dependencies, acyclic):**

```
shared-kernel ← every module
game           → shared-kernel
reference-data → shared-kernel
capture        → game
recognition    → reference-data, game
reporting      → recognition (published language: Scan/CardReading), reference-data, game
submission     → reference-data, game            (does NOT know reporting)
workflows      → capture, recognition, reporting, submission, game, reference-data
ui             → workflows and the public application API of the context modules
adapter-*      → the context modules whose ports they implement
app            → everything (composition root, no logic)
```

- `submission` does not depend on `reporting`. `workflows` turns a released `Report` into a `SubmissionRequest` (a type of `submission`).
- Results from `submission` (`ReportSubmitted`, `SubmissionRejected`) are applied to the report by `workflows`. This avoids the cycle D6.

## Consequences

- `docs/plan/02-architecture.md` §2, `09-engineering-principles.md`, `11-ddd-and-tdd.md`, `04-roadmap.md` and `CLAUDE.md` use the new module names.
- `adapter-refdata` disappears:
  - vocabulary indexes and fuzzy matching are domain logic and move into `reference-data`;
  - fetching moves into `adapter-uex`, caching into `adapter-storage`;
  - the `global.ini` parser moves into `adapter-files`.
- `adapter-capture` is renamed `adapter-files`. It handles folder watching, the stable-file gate and reading game files (`global.ini`, `user.cfg`, RSI launcher log).
- The core does not use AWT/ImageIO or `java.nio.file` (added in the 2026-10-08 review): images are passed as an own `ImageRaster`, folders as `FolderLocation`, so that `recognition`, `capture` and `reporting` stay free of I/O and do not need the `java.desktop` module.
- Each context module exports `api` (application services, commands, read models, events, ports) and `api.model` (aggregates and value objects that ports and other contexts must name); `internal` (domain services, policies) stays hidden. Exported aggregates are protected by their compact constructor plus an ArchUnit rule on constructor calls (corrected in the third review: hiding the whole model made repository ports unimplementable).
- **Revisit** after M3: if two contexts consistently change together, merge them; if `shared-kernel` grows beyond value objects, split or push types back into contexts.
