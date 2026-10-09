# States: from the domain to the screen

> **Doc type:** Living spec — draft (JavaFX handoff stage 2, ADR-0004 Proposed). Last reviewed: 2026-10-09.

This document maps every user-visible state of the plan to the Tallyline state ids, and each id to what the JavaFX theme uses: a pseudo-class or a style class. The id list with labels, icons, tokens and accessible texts is generated into [`ui/design-tokens/states.json`](../../ui/design-tokens/states.json) from the system's "State language" section (Claude Design system Tallyline, version `1791537441-68f0`, 171 ids). The domain terms are those of [11](../plan/11-ddd-and-tdd.md) §A1.

## Rules

- Every state is shown by colour, icon and text together (R-UI-4, R-UI-11, R-UI-17). Colour comes only from the tokens; the ViewModel sets pseudo-classes and style classes and holds no colours or thresholds (02 §7, 09 §7).
- A state id is the stable key. The label and the accessible text come from bundle keys `state.<id>.label` and `state.<id>.accessible` (R-L10N-1); the icon is fixed per id.
- **Reference outdated** is a modifier, not a level: with it, a minor deviation is shown like equal, a major deviation is never lowered and still needs a confirmation, and "no reference" stays ([07](../plan/07-ocr-concept.md) §2.6). The default staleness limit is UEX's `commodity.ttl` (15 days today) with a 15-day fallback (R-UI-11).
- **Unreviewed** follows its definition in 11 §A1: a counter and a row marker, never a gate.
- Test and production are shown in the shell of every view and on report and history rows; their tokens (`env-test-*`, `env-production-*`) stay outside the critical family.

## Field level: the pseudo-class set of 02 §7

| State id | JavaFX | Plan source |
|---|---|---|
| `confidence-ok` | none (level OK has no marking in cells) | 07 §2.6 |
| `confidence-confirm` | `:confidence-confirm` | 07 §2.6, R-UI-4 |
| `confidence-select` | `:confidence-select` | 07 §2.6, I2 |
| `confidence-correct` | `:confidence-correct` | 07 §2.6, I2 |
| `user-entered` | `:user-entered` | 11 §A1 "Origin" |
| `confirmed` | `:confirmed` | I3 |
| `needs-confirmation` | `:needs-confirmation` | 02 §7, I2 |
| `deviation-equal` | none | 07 §2.6 |
| `deviation-minor` | `:deviation-minor` | R-UI-10 |
| `deviation-major` | `:deviation-major` | R-UI-10, R-UI-12 |
| `no-reference` | `:no-reference` | R-UI-10 |
| `reference-outdated` | `:reference-outdated` | R-UI-11, 07 §2.6 |

**Additions to 02 §7** (decided 2026-10-09 by @greluc, checkpoint B):

| State id | Pseudo-class | Why |
|---|---|---|
| `double-confirmed` | `:double-confirmed` | OCR and AI agree (07 §2.7); a subtle border and icon distinct from `:confirmed` |
| `deviation-worsened` | `:deviation-worsened` | I6: a confirmation was revoked because the deviation worsened |
| `superseded-conflict` | `:confirmation-suspended` | I7: a confirmation is suspended by a conflicting reading |
| `ai-hint` | `:ai-hint` | 07 §2.7: a differing AI reading that never overwrites |
| `field-not-sent` | `:not-sent` | 11 §A1 "Mandatory field": an optional value that failed its checks is not sent and does not block |
| `superseded-later-reading` | none; `:confidence-confirm` with the reason as text | 07 §2.5b |

**Selection and focus in the review table (SP-9):** in cell-selection mode the focused cell gets `:focused` (never `:focus-visible`), but rows never get `:selected`; in row-selection mode the row gets `:selected`, but no cell gets `:focused`. The design needs both a focused cell and a selected row, so the table runs in cell-selection mode and the row receives the pseudo-class `:row-selected` from the selection model (02 §7, decided 2026-10-09).

**`:needs-confirmation`** covers the I2 cases, a confirmation revoked by a worsened deviation (I6), a confirmation suspended by a conflicting reading (I7), an unexpected commodity (R-UI-12) and a row flagged "newer observation already sent" (R-VAL-8); all of them already block release (02 §7, decided 2026-10-09).

## Row level

| State id | JavaFX | Plan source |
|---|---|---|
| `unreviewed` | row pseudo-class `:unreviewed` | 11 §A1 "Unreviewed" |
| `row-excluded` | row style class `tl-state-row-excluded` | 02 §3 |
| `not-observed` | row style class (NotObservedRow) | 11 §A1, R-VAL-5 |
| `marked-missing` | row style class (MissingRow) | 11 §A1, R-VAL-5 |
| `unexpected-commodity` | row style class plus the finding | 11 §A1, R-UI-12 |
| `possibly-incomplete` | report notice | 07 §2.5b |
| `section-not-visible`, `section-unsupported` | report notice | 07 §2.5b |
| `newer-observation-sent` | row pseudo-class `:newer-observation-sent`, plus `:needs-confirmation` | R-VAL-8 |

## Everything else: style classes

Report, job, capture, intake, folder, game, game version, game log, faction affinity, staff notice, AI, AI job, connection, account, queue, UEX row lifecycle, report-ID mapping, per-row withdrawal, mode and generic-feedback states are not field states. Each is drawn by a component (StateChip, QueueRow, HistoryRow, UexRowLifecycle, RowWithdrawControl, QueueHoldNotice, AccountStateLine, IntakeSetAsideLine, AiJobChip, Banner, StatusBar) with the style class `tl-state-<id>`; no change to 02 §7 is needed. The groups, their ids and their plan sources:

| Group (`states.json`) | Ids | Plan source |
|---|---|---|
| Report state | 18 (`report-*`) | 02 §3 `ReportState`, Draft sub-states R-UI-1, `report-partly-withdrawn` I4, `report-split` 02 §3 |
| Job | 13 (`job-*`), plus `job-next`, `job-waiting-row-budget`, `job-waiting-retry-time` | 02 §6, R-SUB-2 |
| Queue | `queue-paused-app-token`, `queue-paused-key-unconfirmed`, `queue-held-api-changed`, `queue-held-stale-acceptance`, `queue-held-not-accepting` | R-SUB-11, R-SUB-5 |
| Account | `account-*` (6), `key-*` (5) | R-API-4, R-SUB-11 |
| UEX row lifecycle | `uex-*` (10) | R-SUB-4, 06 "Reading, correcting and withdrawing reports" |
| Report-ID mapping and withdrawal | `mapping-*` (4), `row-withdraw*`, `row-withdrawn`, `row-already-consolidated` | R-SUB-9, I4 |
| Capture and intake | `capture-*` (9), `intake-*` (6) | R-CAP-1c, R-CAP-1d, R-CAP-8, R-OCR-8, R-OCR-16 |
| Folder | `folder-*` (6) | R-CAP-1e |
| Release split and returns | `release-split`, `returned-*` (2) | 02 §3, R-SUB-11 |
| Game, game version, game log | `game-*` (3), `game-version-*` (4), `gamelog-*` (6) | 02 §4b, R-CAP-3b, R-OCR-20 |
| Faction affinity, staff notice | `affinity-*` (2), `staff-notice` | R-SUB-13, R-UI-8 |
| AI and AI jobs | `ai-*` (11), `ai-job-*` (6), `ai-queue-paused`, `ai-remote-host`, `ai-pull-progress` | R-VLM-1…8, 02 §4b |
| Connection and mode | 11 connection ids, `env-test`, `env-production`, `env-other-channel` | R-UI-8, R-SUB-8, R-CAP-10 |
| Generic feedback | `feedback-*` (6) | — |

## Open design defects (version `1791537441-68f0`)

Found when the update was checked against its brief and the plan on 2026-10-09. They go back to Claude Design or to @greluc; the repository does not patch the design silently.

| # | Defect | Plan | Needs |
|---|---|---|---|
| 1 | "UEX: approved" is shown as final and "not editable" | Final UEX statuses are consolidated, declined and expired; editability comes from `is_editable` (R-SUB-4, 02 §6) | Claude Design |
| 2 | Combination case 13 (confirmation revoked, now "no reference") cannot occur: a move to no reference is never "worse" | 07 §2.6 `isWorseThan`; the case came from the update brief, not from the plan | Claude Design (remove the case) |
| 3 | The staff notice is a banner only in the report editor and manual capture, and "Show full notice" opens the uncapped text | R-UI-8, 02 §7, O-100: in the global status area, length-capped | decided 2026-10-09 (short form in the status area plus a banner in the report editor and manual capture, cut after 280 characters, never shown uncut; R-UI-8); Claude Design |
| 4 | `gamelog-sizes-inconsistent` sets the gate marker | Container sizes are optional: a failing set is "not sent" and does not block (11 §A1); R-OCR-20 makes a contradiction `Ambiguous` | Claude Design |
| 5 | The release-split sample counts 11 rows in three parts and ignores the row without a size set | A row without a sent set goes into its own part without the field (02 §3) | Claude Design |
| 6 | Reports whose job is sending, next or waiting for cooldown are labelled "Released" | Those jobs belong to a `Queued` or `WaitingForCooldown` report (02 §6) | Claude Design |
| 7 | "Queue held – UEX API changed" has no "retry now" | R-SUB-11: until an app update or the user's "retry now" | Claude Design |
| 8 | A failed AI job uses the critical tone | 02 §4b: the report keeps its OCR result and the queue row shows an info note | Claude Design |
| 9 | One "Reference outdated after 7 days" example remains (Panel preview) | 15-day default (R-UI-11) | Claude Design |
| 10 | Glyphs still repeat inside a group: `help` six times in the block-reason list, `schedule` four times, `visibility_off` twice; `swap_horiz` for both superseded states; `undo` for the withdraw action and its result; "newer observation already sent" uses `schedule` as a block reason and `upcoming` as a row flag | The design's own rule: distinct within each group | Claude Design |
| 11 | Findings severity still files Ambiguous, a repair without a witness, UnvalidatedGameVersion and UnexpectedCommodity as blocking/critical | The confidence ladder makes them select- or confirm-level (07 §2.6) | Claude Design |
| 12 | The detail-strip crop is bounded by box height (32–96 px), not by digit size; under 1:1 a small-text capture shows digits at or below body-text size | R-UI-3 "readable size"; R-OCR-17 | decided 2026-10-09 (whole-number nearest-neighbour upscaling until the digit cap height is at least twice the body text's; R-UI-3); Claude Design, then a JavaFX spike |
| 13 | StatusBar and TopBar previews do not show the queue-wide holds, the app-token pause or the staff notice; the Settings board has no "UEX account" section with the app-token field | 02 §7 status area, R-API-4 | Claude Design |
| 14 | `PossiblyAlreadyReceived` (R-SUB-9), the capture label "waiting for the first UEX version data" (R-UI-13), the folder status "polling active" (R-CAP-1e) and specific labels for the return reasons `TOO_MANY_ROWS`, `TERMINAL_NOT_FOUND` and `RESTART` are missing | — | Claude Design |
| 15 | The capture label "Ready" collides with the report label "Ready" in the same queue list | — | Claude Design ("Ready to scan") |
| 16 | `README.md` of the system still calls "unreviewed" a proposal | Decided 2026-10-09 (11 §A1) | Claude Design |
