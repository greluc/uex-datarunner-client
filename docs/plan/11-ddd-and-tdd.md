# Domain-Driven Design and Test-Driven Development

How we model the domain (DDD) and how we let code come into being (TDD). Both complement [02-architecture.md](02-architecture.md) (Ports & Adapters) and [09-engineering-principles.md](09-engineering-principles.md) (enforcement).

**Principle "as far as possible":**

- **DDD** pays off where real domain rules live: Report, submission gate, deviations, recognition, cooldown. In technical peripheral areas (file watching, OS integration) it would be ceremony.
- **TDD** applies to the core without exception. For OCR thresholds, UI layout and native OS integration it is only of limited use; for these there are explicitly different rules below, instead of pretending to do TDD.

## Part A – Domain-Driven Design

### A1. Ubiquitous Language

The domain terms are binding: the same terms in docs, UI and code (all English). One term = one meaning. New terms come here first, then into the code.

| Term | Meaning |
|---|---|
| Capture (`Capture`) | an imported screenshot (file identity, source, environment, capture time, game version at capture time) |
| Scan (`Scan`) | result of the recognition of a Capture: read cards with findings |
| Card reading (`CardReading`) | a read commodity card (raw values per reader) |
| Reader (`Reader`, `ReaderKind`) | method that reads a panel (OCR, VLM) |
| Report (`Report`) | report for **one** terminal, **one** side, **one** environment; unit for `data_submit` |
| Row (`ReportRow`) | a commodity in the Report with price, SCU, status, container sizes |
| Side (`TradeSide`) | BUY ("Buy" tab) / SELL ("Local Market Value" tab) |
| Prior (`PricePrior`) | previous UEX value for (terminal, commodity, side) including its age |
| Finding (`Finding`) | justified indication of uncertainty (reason code) |
| Confidence (`Confidence`) | how reliably something was **read** |
| Deviation (`Deviation`) | how strongly a value deviates from the prior (`EQUAL`, `MINOR`, `MAJOR`, `NO_REFERENCE`) |
| Field assessment (`FieldAssessment`) | confidence + deviation + reference of a field |
| Confirmation (`Confirmation`) | explicit approval of a field by the user, bound to exactly this value |
| Send threshold (`SendThreshold`) | minimum confidence for sending without confirmation |
| Submission gate (`SubmissionGate`) | set of rules deciding whether a Report may be sent |
| Cooldown (`Cooldown`) | lock period after a successful report for (terminal, commodity, environment) |
| Environment (`GameEnvironment`) | LIVE, PTU, EPTU, HOTFIX, TECH-PREVIEW |
| Reference snapshot (`ReferenceSnapshot`) | immutable state of the UEX data for one environment and one decision |
| Session (`Session`) | consecutive captures without a gap longer than 60 min (configurable); context for terminal disambiguation and the session overview (R-UI-14) |
| Stitched scan (`StitchedScan`) | the merged rows of all scans of one group, produced by Recognition and consumed by Reporting |
| Observation time (`observedAt`) | time of the oldest capture of a Report; basis for the age rules (R-VAL-6) |

### A2. Bounded Contexts

| Context | Responsibility | Core model | Relationship |
|---|---|---|---|
| **Capture** | Take in images from folders/clipboard, avoid duplicates, record environment and version at capture time | `Capture`, `WatchedFolder` | delivers `CaptureImported` to Recognition |
| **Recognition** | Locate, read, resolve, fuse and validate the panel | `Scan`, `CardReading`, `Finding`, domain services | uses Reference Data (read-only); delivers `CaptureScanned` |
| **Reporting** | Build, check, confirm and release Reports – **core of the domain rules** | Aggregate `Report` | uses Recognition and Reference Data; delivers `ReportReleased` |
| **Submission** | Sending, queue, cooldown, history, withdrawal | Aggregates `SubmissionJob`, `Cooldown` | Customer of the UEX API via an **Anti-Corruption-Layer** |
| **Reference Data** | UEX master data and prices as a read model | `ReferenceSnapshot` and value objects | **Conformist** to UEX (we adopt their model), shielded by the ACL in `adapter-uex` |
| **Game Environment** | Environment, version, "game running", game localization | `GameEnvironment`, `GameVersion`, `GameState` | Upstream of Capture, Recognition, Reporting and Submission (published language: `GameEnvironment`, `GameVersion`) |

**Implementation in code:**

- Each context is its **own Gradle/JPMS module** (`capture`, `recognition`, `reporting`, `submission`, `reference-data`, `game`) containing its domain model and application services; cross-context processes live in `workflows`. Decision and trade-offs: [ADR-002](../adr/0002-modules-by-bounded-context.md); context map: [02 §2](02-architecture.md).
- Shared types (IDs, `PricePerScu`, `ScuQuantity`, `Outcome`, event base, marker annotations) live in the **Shared Kernel** module `shared-kernel`. `GameEnvironment` and `GameVersion` belong to the `game` context and are used by others as its published language. It is kept small; changes there require particular care.
- **Rule:** Contexts reference other aggregates only by ID and communicate via domain events or application services, never via direct object references. This is enforced by JPMS (only `api` packages are exported) and ArchUnit.

### A3. Aggregates and invariants

**`Report`** (aggregate root, context Reporting) – this is where the most important domain rules live:

| Invariant | Rule |
|---|---|
| I1 | Each commodity appears at most once per Report. |
| I2 | Release (`release()`) is only possible if: the terminal is resolved; every mandatory field is ≥ send threshold **or** confirmed; every `MAJOR` deviation is confirmed; environment and game version match the UEX acceptance state. |
| I3 | A confirmation applies to exactly one value. `correct(field, newValue)` revokes the confirmation. |
| I4 | A submitted Report is immutable; only `withdraw()` is possible (leads to `data_remove`). Corrections create a **new** Draft (`duplicateAsDraft()`), never an edit in place (unless `data_edit` is verified, R-SUB-10). |
| I6 | A Report is released only if its observation age is within the limits (R-VAL-6) and its rows are evaluated against the current reference snapshot (R-VAL-7). |
| I5 | The game version is the one valid at capture time; a version change marks the Report as "to be checked". |

The `Report` carries a `version` number for optimistic concurrency: every command is applied to a specific version, and the repository rejects writes based on a stale version (e.g. user edit and AI re-read at the same time).

States as sealed types: `Draft → Released → Queued → (WaitingForCooldown →) Submitted | PartiallyAccepted | OutcomeUnknown | Rejected`; `Released/Queued → Draft` (cancel); `OutcomeUnknown → Submitted | Draft` (after resolution via `data_info` or by the user); `Submitted/PartiallyAccepted → Withdrawn`; `Rejected/Withdrawn` can be duplicated as a new Draft. Invalid transitions return `Outcome.Refused` with a reason code; there are no status strings.

Records are not fully encapsulated: their canonical constructor is as visible as the record. The compact constructor therefore validates all state-local invariants (I1, value-bound confirmations), and an ArchUnit rule allows `new Report(...)` only inside the aggregate and in the persistence mapper (reconstitution). Transition invariants (I3, I4) are enforced by the command methods.

Further aggregates:

- **`Capture`** (Capture): identity = content hash; state `Imported → Scanned | Failed`; environment and version immutable after import.
- **`SubmissionJob`** (Submission): one submission attempt of a released Report; retry counter, last error; persisted.
- **`Cooldown`** (Submission): key (terminal, commodity, environment), `until` timestamp; invariant: a job for a key in cooldown is not sent.

**Size:** Aggregates stay small. A Report knows Captures only by ID, terminals and commodities only by ID.

### A4. Value Objects

Immutable records with validation in the compact constructor, equality by value, no primitive types in domain signatures (against "primitive obsession"):

`TerminalId`, `CommodityId`, `CaptureId`, `ReportId`, `PricePerScu` (BigDecimal ≥ 0), `ScuQuantity` (≥ 0), `InventoryStatus`, `ContainerSizes` (subset of the allowed sizes, sorted), `GameVersion`, `Confidence` (0..1), `Deviation`, `Percent`, `FileFingerprint`, `Confirmation`.

### A5. Domain services

Stateless domain logic that belongs to no single aggregate (pure functions):

- **Recognition** (module `recognition`): `CommodityResolver`, `TerminalResolver`, `PriceCandidateEvaluator`, `ReaderFusion`, `Stitcher`
- **Reporting**: `DeviationAssessor` (→ `FieldAssessment`), `ReportGrouper`, `SubmissionGate`

### A6. Domain events

`CaptureImported`, `CaptureScanned`, `ReportDraftCreated`, `ReportReleased`, `ReportSubmitted`, `ReportWithdrawn`, `SubmissionSucceeded`, `SubmissionPartiallyAccepted`, `SubmissionOutcomeUnknown`, `SubmissionRejected`, `GameStateChanged`, `ReferenceDataRefreshed`.

- Events are immutable records in the `api` package of the context that publishes them (base type in `shared-kernel`). Submission results are published by `submission` (`SubmissionSucceeded`, `SubmissionPartiallyAccepted`, `SubmissionOutcomeUnknown`, `SubmissionRejected`); `workflows` translates them into Report state changes, which publish `ReportSubmitted` etc. in `reporting`.
- They are delivered **in-process** via a small dispatcher in `workflows`, without a framework. To survive crashes, events are written to an **outbox table in the same SQLite transaction** as the aggregate state and dispatched after commit; on startup, undelivered outbox entries are dispatched again (handlers are idempotent).
- **Aggregates are immutable** (records): a command such as `report.confirm(…)` returns `Outcome.Ok(newState, events)` or `Outcome.Refused(reason)`. The application service stores the new state via the repository and then publishes the events. This makes aggregates testable without mocks and thread-safe.
- Events make the flows testable: "Given events / When command / Then events".

### A7. Repositories and Anti-Corruption-Layer

- **One repository per aggregate** (port in the owning context module, implementation in `adapter-storage`). There are no repositories for entities within an aggregate.
- **ACL to UEX** (`adapter-uex`):
  - UEX DTOs (snake_case, numbers as strings, 0/1 flags, status strings) are translated into value objects at the boundary.
  - UEX error codes become sealed `SubmissionError` types.
  - No UEX term "leaks" into the core.
- **ACL to the game:** `global.ini` and in-game texts are translated into domain terms via mapping tables (`InventoryStatus`, `TradeSide`).
- **ACL to Ollama:** the model response becomes a `CardReading`.

### A8. Marking and enforcement

- Our own dependency-free marker annotations in `shared-kernel` (`@AggregateRoot`, `@ValueObject`, `@DomainEvent`, `@DomainService`).
- **No jMolecules:** that would be an additional dependency in the core (checked: `org.jmolecules:jmolecules-ddd` 2.0.1). Four annotations of our own do the same job here.
- **ArchUnit rules:**
  - `@ValueObject` and `@DomainEvent` are records.
  - `@AggregateRoot` classes are loaded and stored only via their repository.
  - Aggregates reference other aggregates only via `*Id` types.
  - Domain types have no public setters.
  - Context modules do not access the `internal` packages of other contexts (also enforced by JPMS exports).

## Part B – Test-Driven Development

### B1. Where TDD applies

| Area | Approach |
|---|---|
| Core modules (context modules, `shared-kernel`, `workflows`) | **TDD mandatory:** Red → Green → Refactor. No production code without a previously failing test. |
| Adapters (`adapter-uex`, `adapter-storage`, `adapter-files`) | **Test-first, as far as possible:** contract and integration tests (WireMock, real SQLite file, real temp directory) first, then the implementation |
| `adapter-ocr`, `adapter-vlm` and OCR thresholds | **Spike & Stabilize:** exploration in the eval harness is allowed (result: measurements, no merge). Before the merge, the desired behavior is pinned down with golden and unit tests that fail without the change. |
| `ui` | ViewModels via TDD (without a window); views (layout, CSS) without TDD, critical flows via TestFX after implementation |
| `adapter-platform` (FFM, process list) | Thin wrappers behind ports; the port is tested via a fake, the native side via integration tests on the target OS (CI matrix) |
| Bugfix (everywhere) | First a test that reproduces the bug |

### B2. Outside-in per use case

1. **Acceptance test** at the application-service level of the context module (or `workflows` for cross-context use cases) (domain language, fakes for all ports), derived from the requirement. The test name or tag refers to the requirement ID, e.g. `@Tag("R-UI-10")`.
2. Unit tests for aggregates, value objects and domain services follow from it.
3. Finally come the adapters with their contract tests.

Example (structure, not final code):

```java
@Test @Tag("R-UI-10") @Tag("R-VAL-2")
void majorPriceDeviationBlocksReleaseUntilConfirmed() {
  var report = aDraftReport().at(PYRO_GATEWAY_STANTON).side(BUY)
      .row(HYDROGEN_FUEL, price("782"), prior("520", ageDays(1)))   // > price_variation
      .build();

  assertThat(report.release(gate)).isEqualTo(refused(UNCONFIRMED_MAJOR_DEVIATION));

  var confirmed = report.confirm(HYDROGEN_FUEL, PRICE).orThrow();
  assertThat(confirmed.release(gate)).satisfies(isOkWithState(Released.class));
}
```

### B3. Tools and rules

- **Fakes before mocks:** for every port there is a handwritten fake implementation in the Gradle `java-test-fixtures` source set of the respective module. All tests share these fixtures. (How `java-test-fixtures` behaves with JPMS modules is not verified – M0 spike; the GradleX `java-module-testing` plugin is the fallback.) Mockito only for interaction checks that a fake cannot sensibly represent.
- **Test data builders** (`aDraftReport()`, `aCapture()`, `aSnapshot()`) in the same test fixtures, instead of copying test data.
- **Property-based tests** (jqwik) for parser, fuzzy matcher, fusion, `DeviationAssessor` and the invariants of the `Report` (e.g. "after `correct`, a field is never confirmed").
- **Golden tests** for the `recognition` pipeline against the corpus (`corpus/`).
- **Mutation testing** (PIT) for the core modules, to check whether the tests really find bugs – with TDD, the honest check against "tests that assert nothing":
  - Initial value for the mutation score ≥ 70 %; it is fixed after the first measurement and may only rise.
  - Runs in the PR for changed classes and fully nightly.
  - Versions checked: PIT 1.30.0, Gradle plugin `info.solidsoft.pitest` 1.19.0, `pitest-junit5-plugin` 1.2.3. **That the JUnit 5 plugin works together with JUnit 6 and JDK 27 has not been checked** → M0.
- **Never red on `main`:** red/green happens locally or in the feature branch; every commit on `main` is green.
- **The refactor step is mandatory:** after green, clean up (names according to the Ubiquitous Language, remove duplicates) while the tests stay green.

### B4. Traceability

- Every M requirement from 01 has at least one acceptance test with `@Tag("<R-ID>")`.
- A small report in the build lists R-IDs without a test (Gradle task over JUnit tags). Each requirement group is assigned to a milestone in [04-roadmap.md](04-roadmap.md) ("Requirement coverage"). Missing tests are a warning for open milestones and a **build error** for M requirements of milestones that are already completed.
