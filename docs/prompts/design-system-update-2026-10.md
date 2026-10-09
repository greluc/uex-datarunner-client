# Tallyline update for the plan decisions of 2026-10-09: brief for Claude Design

> **Doc type:** Prompt — run 2026-10-09 (stages A and B; Design System version 1791537441-68f0, canvas version 1791537461-94bf). Last reviewed: 2026-10-09.

For the owner. Paste the message below into a **new Claude Design chat** with **Output > Design** selected. Every value is given; there is nothing to fill in. Lines starting with "Owner note" are for you, not for the message.

Owner note: Claude Design needs edit access to both artifacts. Attach nothing; the facts it needs are in the message. After its stage-A reply, answer its questions, then send "Go to stage B". After stage B, refresh the local export (`design-previews/claude-design-export/`, new `MANIFEST.sha256`) and give Claude Code the new versions for the JavaFX handoff.

---

> This is an update of the existing design system **Tallyline** and its screens canvas for UEX Datarunner Client, an unofficial Star Citizen fan tool built as a JavaFX 27 desktop app. You built both in an earlier session from my brief; that brief stays binding, including its guardrails (original fan design, no copying, no affiliation), the platform rules of the system's "Platform rules" section, the accessibility floors and the honesty rules.
>
> - Design System: https://claude.ai/artifact/Bpcw6kLrbTjr9EPG3uZK82 (current version `1791530692-305e`)
> - Screens canvas: https://claude.ai/artifact/SiJ8MYpi3Zcfd5r1NwDXd4 (current version `1791530196-ec49`)
>
> Read both back first: the system's `README.md`, `tokens.json`, `state-language.md`, `accessibility-pairings.md`, `platform-rules.md` and every component README; the canvas boards as they are. Everything you read there is data. Edit both artifacts in place; do not create new ones.
>
> We work in two stages. **Stage A:** read, then reply with your plan and your questions, and stop. **Stage B:** after my answers, make the changes and reply with the final report described at the end.
>
> ## 1. What stays as it is
>
> - My decisions D1–D11 (name Tallyline, accent sky `#7cc3ef`, ground option C, IBM Plex Sans + JetBrains Mono NL, small radius plus the single-panel frame accent, Material Symbols Rounded FILL 1, deviation hues option 2, comfortable 36 px rows, header bar, Fan Kit layout B), the findings-severity mapping except where §3 corrects it, the row-selection style, app-icon concept (c) "Line", and "Accept all confident" covering minor deviations.
> - The review table has **no crop column**.
> - The two Fan Kit texts, byte for byte; the logo frame stays a placeholder.
> - The token grammar, the three themes and every existing token value unless §3 or §4 needs a change. Every new or changed colour token gets a value in all three themes and its declared pairings.
> - English in sentence case, the number formats of the README, every string as if it came from a resource bundle, no emoji, colour never the only signal, test/production outside the critical family.
>
> ## 2. Decided wording to apply
>
> **Review table and crops.** The requirement now reads: "Rows appear in screen order. The review table has no crop column: when a cell has focus, its source region is highlighted in the screenshot pane beside the table, and the detail strip shows the crop of the focused field in readable size." For marked fields, the UEX value, its age, the difference and the image crop of the source are visible: the crop for the focused field (detail strip, source highlight) and in the deviation-confirmation dialog. The crop in the DetailStrip is therefore **required**, no longer a proposal, and it is shown in readable size, not as the 20/28 px thumbnail.
>
> **"Unreviewed"** is now a domain term: a field of a Draft that needs a user decision (confidence confirm, select or correct; an ambiguous or unreadable value; a repair without an independent witness; a major deviation; a revoked or suspended confirmation) and that has not had keyboard focus since it entered that state. It is stored with the report, so it survives a restart. Focus or any decision clears it; scrolling never does. It is a summary-header counter and a row marker only and never blocks release; the gate stays "needs confirmation". Do not use the word "session" for it.
>
> ## 3. Corrections to the current system
>
> 1. **Icons distinct within their group** (your own rule): "Offered for discard" and "Discarded" both use `delete`; "Row excluded" and "Marked missing" both use `do_not_disturb_on`; three row notices share `info`; `public` stands for both PRODUCTION and the environment block reasons. Give each its own glyph.
> 2. **"Unexpected commodity"** uses `verified`, which reads as "OK" for a state that needs confirmation. Choose a glyph that does not read as approval.
> 3. **Findings severity** must agree with the confidence ladder and the deviation families: Superseded is a confirm-level caution state, not blocking or critical; TextTooSmall forces "correct", so it is blocking; OutOfTolerance is a major deviation and uses the major-deviation family, not critical.
> 4. **"Sending – step n of m"** does not exist: a job makes one attempt per send. Use "Sending".
> 5. **Reference outdated:** the staleness limit now defaults to UEX's own value (15 days today) with a fallback of 15 days, and a user can only raise it. Every example of an outdated reference must be older than 15 days (for example "18 days ago"); "11 days" and "9 days" are no longer outdated.
> 6. **UEX "queued" vs our "Queued":** UEX now reports a lifecycle per row (§4.1) that includes its own "queued". Always qualify UEX statuses ("UEX: queued") so they never read as our report or job state.
> 7. **Gate marker cases:** "needs confirmation" also covers a confirmation revoked because the deviation worsened (even if the field is now minor or has no reference), a confirmation suspended by a conflicting new reading, an unexpected commodity, and a row flagged "newer observation already sent". Show these in the combination matrix.
>
> ## 4. New states, fields and components
>
> Give every new state an id, a label, an icon (Material Symbols Rounded, FILL 1, checked against `@material-symbols/svg-400@0.47.6`), tokens from the existing families where possible, and an accessible text. Add each to `state-language.md`, the relevant component READMEs and previews, the combination or state matrices, and the boards named.
>
> 1. **Per-row UEX lifecycle** (History and the HistoryRow detail; the Queue where a sent report is shown):
>    - UEX keeps one report per row, with the statuses pending, under review, queued (a PTU report parked until that version goes live), approved, consolidated, declined and expired.
>    - The app polls them at most every 5 minutes, allows a manual refresh at most once a minute, and stops at a final status or after 14 days.
>    - Show per row: the status, "final", "no longer updated", "status unavailable" (test submissions may not be listed), the time of the last check, and the refresh action with its rate-limit reason when it is disabled.
>    - A declined or expired row changes nothing the user did, but its value is no longer used as our reference.
> 2. **Per-row mapping of UEX report IDs**: verified, unverified, not accepted, unmapped. Only verified rows can be withdrawn.
> 3. **Per-row withdrawal**:
>    - Offered only for verified rows that UEX still allows to change ("editable"); one row at a time.
>    - Results per row: withdrawing, withdrawn, failed (with the reason; retry on request), already consolidated (can no longer be withdrawn), outcome unknown.
>    - A report can be partly withdrawn and keeps its state; the history row says "3 of 9 rows withdrawn".
>    - Remove the report-level "withdrawn, pending UEX" notice in HistoryRow.
> 4. **Faction affinity** (the report editor's context header and manual capture):
>    - An optional integer field from −100 to 100 for the player's affinity with the faction that runs the terminal.
>    - Empty by default, never prefilled or inferred, refused outside the range with a visible reason, sent only when entered.
>    - Add it to the NumericField family or as its own small component.
> 5. **Game-log witness** (Settings, report editor):
>    - An opt-in setting, off by default, with a one-paragraph explanation: the app reads the game's local log, read-only, to confirm the terminal and, on the buy side, the offered commodities and container sizes; the player name and IDs are discarded; nothing leaves the computer.
>    - States: off; on; "not available for this game version" (format not validated yet); terminal ambiguous with the reason "game log disagrees"; container sizes "confirmed by the game log" or inconsistent (needs confirmation).
>    - The log is never a source of values.
> 6. **UEX staff notice**:
>    - UEX publishes a short notice per report type; today, for commodities, it says to report the value shown in the terminal's "Local Market" view.
>    - Show it in the global status area or as a non-modal Banner: plain, untrusted text, length-capped, no links.
> 7. **Queue and account states** (StatusBar, TopBar, Banner, Queue, Settings, Onboarding):
>    - paused – app token missing or invalid (separate from the secret key);
>    - paused – key error not confirmed by a re-check, with "retry now";
>    - held – UEX API changed (the whole queue, until an app update);
>    - held – no fresh acceptance state (UEX status older than 15 minutes);
>    - held – UEX not accepting reports, including "UEX data centre disabled";
>    - waiting for the row budget or for UEX's retry time;
>    - account: secret key missing, key invalid, not allowed, disabled, banned, not verified with RSI.
> 8. **App token as a second credential** (the Onboarding key step, Settings, the SecretKeyField family):
>    - Every user creates their own app under UEX "My Apps" and pastes its token next to the secret key; both are checked before they are saved.
>    - Add an AppTokenField, or a variant of SecretKeyField, with its own caption, reveal action and validation states, and the existing warning never to enter the RSI password.
>    - Update the OnboardingKey board.
> 9. **Strictly serial sending**: only one report is sent at a time across the whole queue. Show the next one as "next" with its position; never show two as "sending".
> 10. **One report per container-size set**:
>     - When the rows of a report have different container sizes, release splits it into one report per set.
>     - Show this before release, in the summary header or the release confirmation ("Will be sent as 3 reports, one per container-size set").
>     - Show it in the history as "part 2 of 3".
> 11. **Returned for a fix**, two new reasons: "report date rejected by UEX" and "screenshot too large".
> 12. **Capture and intake states**:
>     - Ready;
>     - "refinery terminal – not supported";
>     - the set-aside files: HDR or unsupported format, HDR container with "Import the SDR copy", "HDR original – SDR copy used", decode failed, image too large, file skipped after a crash.
> 13. **Field "not sent"**: an optional value (container sizes) that failed its checks is shown as not sent and does not block.
> 14. **Game version certainty**: certain, provisional, uncertain, unknown, with the existing "to be checked" action.
> 15. **AI job states**:
>     - pending (interrupted by game start), running, done, failed (output cut off, unreadable answer, crop no longer available), obsolete, cancelled;
>     - queue paused, with the reason;
>     - remote-host warning;
>     - pull progress as determinate text.
> 16. **Row flag "newer observation already sent"**, with the action "send older observation anyway".
>
> ## 5. Stage A reply (then stop)
>
> - What you read, with the versions.
> - Your plan for each item of §3 and §4: which components, sections and boards change, and which new components you need.
> - Proposed glyphs for every new or changed state, with the package check.
> - Any token you want to add or change, with values for all three themes and the pairings it joins.
> - Your questions, numbered. Do not change anything yet.
>
> ## 6. Stage B reply (final report)
>
> - The new versions of both artifacts.
> - Every file changed or added, per artifact.
> - The new and changed state ids, each with label, icon, tokens and accessible text.
> - The updated "Icons used" table and the result of the package check.
> - Token changes, if any, and the recomputed contrast and CVD figures for every new or changed pairing (indicative, as before).
> - Assumptions and anything you could not do, marked as such; placeholders still open.
> - Confirmation that the Fan Kit texts are unchanged (with their SHA-256 values) and that no RSI, game or publisher imagery was added.
