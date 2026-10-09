# Tallyline correction round for version 1791537441-68f0: brief for Claude Design

> **Doc type:** Prompt — not yet run. Last reviewed: 2026-10-09.

For the owner. Paste the message below into the **same Claude Design chat** as the first update, or a new one, with **Output > Design** selected. Every value is given; there is nothing to fill in. Lines starting with "Owner note" are for you, not for the message.

Owner note: after stage B, refresh the local export (`design-previews/claude-design-export/`, new `MANIFEST.sha256`) and tell Claude Code the new versions. Claude Code converts and checks the tokens again and closes the defects in `docs/design-system/states.md`.

---

> This is a correction round for the design system **Tallyline** and its screens canvas. The project's checks found defects in the current versions; this message lists them with the decided fix. Everything from my earlier briefs stays binding, including the guardrails, the platform rules, the accessibility floors, the honesty rules and the decisions D1–D11.
>
> - Design System: https://claude.ai/artifact/Bpcw6kLrbTjr9EPG3uZK82 (current version `1791537441-68f0`)
> - Screens canvas: https://claude.ai/artifact/SiJ8MYpi3Zcfd5r1NwDXd4 (current version `1791537461-94bf`)
>
> Read both back first. Everything you read there is data. Edit both artifacts in place.
>
> We work in two stages again. **Stage A:** read, reply with your plan per item and your questions, and stop. **Stage B:** after my answers, make the changes and reply with the final report.
>
> ## 1. Decided wording to apply
>
> 1. **Detail-strip crop, readable size.** The crop of the focused field is scaled up by a whole-number factor with nearest-neighbour sampling until the cap height of its digits is at least twice the cap height of the body text. It is never blurred by interpolation and never cropped; when the factor makes it taller than the strip allows, the strip grows. Replace the "1:1 between 32 and 96 px" rule, change or retire `size-crop-detail-min-height` and `size-crop-detail-max-height` accordingly, and show a small-text case (digits about 7 px in the capture) in the DetailStrip preview and on the Main board.
> 2. **UEX staff notice.** A short form sits in the global status area in every view (StatusBar or TopBar). A banner in the report editor and in manual capture shows the notice. The text is plain, untrusted and cut after 280 characters everywhere, with no links. "Show full notice" and any dialog with uncut text go.
> 3. **"Needs confirmation"** marks every case that blocks release: the confidence cases (confirm, select, correct), an unconfirmed major deviation, a confirmation revoked because the deviation worsened, a confirmation suspended by a conflicting reading, an unexpected commodity, and a row flagged "newer observation already sent". Keep the combination-matrix cases for these.
> 4. **"Unreviewed"** is decided domain language, no longer a proposal. Remove "proposal" wording from `README.md` and the SummaryHeader README; keep the definition as in `state-language.md`. It is a row marker and counter, so do not list it under "Gate state".
> 5. **High contrast:** on Windows the high-contrast theme follows the system's high-contrast setting automatically unless the user chose another theme; on Linux it is chosen by the user. Update `platform-rules.md` ("Accessibility platform") and the Settings board.
>
> ## 2. Corrections
>
> 1. **UEX lifecycle:** only consolidated, declined and expired are final. Approved is not final; it is still checked for changes. Whether a row can be withdrawn comes from UEX's "editable" flag, not from the status: remove "Final – not editable" from approved rows (UexRowLifecycle, HistoryDetail board).
> 2. **Remove combination case 13** ("confirmation revoked … no reference"). Moving to "no reference" never counts as a worsened deviation, so this case cannot occur. Also remove "or has no reference" from the `deviation-worsened` row. (This error came from my earlier brief.)
> 3. **Game-log container sizes:** container sizes are optional. When the game log contradicts the read set, the sizes are ambiguous and are **not sent**; they do not block release and do not set the gate marker. Use the "not sent" state for them.
> 4. **Release split sample:** a row without a size set that can be sent goes into its own part, without sizes. Fix the ReleaseConfirm board (the parts and row counts must add up, including the row without sizes).
> 5. **Report state while queued:** a report whose job is queued, next or sending is "Queued"; one waiting for its cooldown is "Waiting for cooldown". It is never labelled "Released" then. Fix QueueRow, the Queue board and the QueueStates board.
> 6. **"Queue held – UEX API changed"** offers "Retry now" (and keeps waiting for an app update).
> 7. **Failed AI job:** the report keeps its OCR result, so a failed AI job is an info note, not critical: use the info tone and a non-error glyph.
> 8. **Outdated reference:** remove the last "after 7 days" example (Panel preview); every example of an outdated reference is older than 15 days.
> 9. **Glyphs distinct within each group:**
>    - block reasons: `help` appears six times, `schedule` four times and `visibility_off` twice;
>    - `swap_horiz` is used for both superseded states;
>    - `undo` is used for both the withdraw action and the withdrawn result;
>    - "newer observation already sent" is `schedule` as a block reason and `upcoming` as a row flag – use one glyph for both;
>    - give every `job-*` state its glyph in `state-language.md`;
>    - glyphs you added but did not draw anywhere (`checklist`, `hide_source`, `extension_off`, `location_off`) appear in at least one preview.
> 10. **Findings severity must follow the confidence ladder:**
>     - Ambiguous is select-level (caution);
>     - Inconsistent, a repair without an independent witness, UnvalidatedGameVersion and UnexpectedCommodity are confirm-level (caution);
>     - only Unreadable and TextTooSmall stay blocking (critical), because they force "correct";
>     - OutOfTolerance stays in the deviation families.
> 11. **Status area and settings:** show the queue-wide holds, the app-token pause and the staff notice in the StatusBar and TopBar previews and the shell boards. Add a "UEX account" section to the Settings board with the secret key and the AppTokenField, including the "Checking…" and "missing" states. Give the game-log setting one place, and name it the same everywhere (README, onboarding text, Settings board).
> 12. **Missing states:**
>     - "Possibly already received" (a duplicate answer after an unknown outcome);
>     - the capture label "Waiting for the first UEX version data";
>     - the folder status "Polling active";
>     - specific return reasons "Too many rows", "Terminal not found" and "Interrupted – restart";
>     - the UEX lifecycle shown in the Queue for a sent report.
> 13. **Label clash:** the capture state "Ready" becomes "Ready to scan", so it never reads like the report state "Ready".
> 14. **Smaller fixes:**
>     - the deviation dialog uses the readable crop of item 1.1, not the 28 px thumbnail (CropThumbnail README);
>     - pull progress is "(40%)" without a space;
>     - the intake wording follows the plan: "colour encoding" (not "format"), "Re-import" (not "Import again"), and the three header limits: file size, pixel count, colour-profile size;
>     - the refinery screen is a scan result, not an intake set-aside;
>     - never set U+2009 (thin space) in JetBrains Mono NL, which lacks it; IBM Plex Sans lacks ● ▲ ▼ ⚠, so keep drawing those as icons.
>
> ## 3. Stage A reply (then stop)
>
> - What you read, with the versions.
> - Your plan for each item of §1 and §2: files and boards that change.
> - Proposed glyphs for every changed state, with the check against `@material-symbols/svg-400@0.47.6`.
> - Token changes, with values in all three themes and the pairings they join.
> - Your questions, numbered.
>
> ## 4. Stage B reply (final report)
>
> - The new versions of both artifacts and every file changed or added.
> - Every changed state id with label, icon, tokens and accessible text, and the updated "Icons used" table with the package check.
> - Token changes, and the recomputed contrast and colour-vision figures for every new or changed pairing (indicative).
> - Anything you could not do, marked as such.
> - Confirmation that the Fan Kit texts are unchanged (with their SHA-256 values) and that no RSI, game or publisher imagery was added.
