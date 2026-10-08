# Messages for Claude Design, one per stage

> **Doc type:** Prompt — not yet run. Last reviewed: 2026-10-08.

For the owner. Paste one message per stage, fill in the `<…>` fields, and wait for the reply and your checkpoint before the next. Lines starting with "Owner note" are for you, not for the message.

## Stage 1 (new chat)

Owner note: select **Output > Design** in the message box before sending. Attach the brief (`design-system.md`, or `design-system.txt` if `.md` is refused). Attach no screenshots.

> Attached is my brief for a new design system for UEX Datarunner Client, an unofficial Star Citizen fan tool that will be built as a JavaFX desktop app. Read all of it; it is binding, and we work through its stages one message at a time.
>
> Stage 1 only: the direction study of section 9. At stage 1, create only the direction-study Design canvas with options A, B and C as defined there; do not create the design system yet. Do not use, mount or derive from any of my existing design systems (ACL, DAS KARTELL, Home Inventory, SCTradersMate). I attach no screenshots; build the look from section 4.
>
> Reply with the items of the stage-1 list in section 9, then stop and wait for my decisions.

## Stage 2a (after checkpoint 1)

> Stage 2a only. My decisions from checkpoint 1 (provisional where marked):
>
> D1: <name> (provisional) · D2: <hue / option> · D3: <option> · D4: <pairing> · D5: <square / small radius / small radius + single-panel accent> · D6: Material Symbols <Sharp / Rounded>, FILL <0 / 1> · D7: option <1 / 2> · D8: density <compact / comfortable>, crop height <…>, long names <middle ellipsis / wrap>, reduce motion <…>, forced while the game runs <yes / no>, follow OS <yes / no>, percent <"+12%" / "+12 %"> · D9: <native title bar / header bar> · Findings severity: <your mapping / changes> · Row selection: <tint + leading bar / tint + inset outline / icon slot> · App icon: <later / show concepts>
>
> Now create the design system from the Design System type with the title "<D1 name>" and no files. Give me its link and stop. I will drop the static font files onto its empty page.

## Stage 2b (after dropping the fonts)

> Stage 2b only (section 9). Fonts: <I dropped these files onto the empty system page: …> / <the drop failed; use the identical Google Fonts families as flagged stand-ins>.
>
> Read the system back first, and list every file in fonts/ in type.fonts with its exact family, weight and style. Then write tokens.json with the three themes, the type styles, the README, the sections State language, Accessibility pairings and Platform rules, and the cover. Include the state-language proposals of section 4.3, each marked provisional. Reply with the contrast table per theme, the CVD review and the remaining inventory, then stop.

## Stage 3a (after checkpoint 2)

> Stage 3a only: the review-table family, chips and badges, buttons, fields and the shell (section 6.5); previews follow section 5. Decisions from checkpoint 2: confirmed major deviation: <…> · compact-cell icon slots: <…> · Δ for status: <…> · Δ for container sizes: <…> · "unreviewed" means: <…> · "Accept all confident" covers minor deviations: <yes / no> · token changes: <…>. Do this stage only, then stop.

## Stage 3b

> Stage 3b only: the rest of the component inventory of section 6.5. Fixes from checkpoint 3a: <none / …>. Do this stage only, then stop.

## Stage 3c

> Stage 3c only: the ContrastMatrix, the CombinationMatrix, the hand-written bundle (section 6.5) and the cover if the name, palette, display face or scales changed. Fixes from checkpoint 3b: <none / …>. Do this stage only, then stop.

## Stage 4a (after checkpoint 3)

Owner note: if Claude Design does not create a Design canvas, reply "Use the Design type for this canvas."

> Stage 4a only: create the screens canvas of section 7 with the new system installed, and build boards 1–4: the report editor in dark, light and high contrast, and the two fixed 1280×720 boards. Fixes from checkpoint 3: <none / …>. Do this stage only, then stop.

## Stage 4b

> Stage 4b only: boards 5–15 of section 7 on the same canvas, with two D11 layouts for the Fan Kit unit and the non-affiliation statement on the About and onboarding start boards (section 3.3; the logo stays a placeholder frame, the texts stay exactly as in the brief). Fixes from checkpoint 4a: <none / …>. Do this stage only, then stop.

## Stage 5

> Stage 5 only. D11: layout <A / B> for About and <A / B> for the onboarding start screen. Apply it, then render checks of <none / the cover and boards …>, then the final reply of section 11.

## When a reply stops early

> Continue from the remaining-inventory list in your last reply. Same stage, same decisions.

## Optional: mood images (only after stage 1)

Owner note: 2–4 images at most, redacted as described in `design-system-owner-checklist.md`.

> Mood only – do not match layout, components or values. These images show the genre of the mood reference (section 3.6). Use them only to judge lightness steps, accent loudness, list density and panel flatness. Do not change the current stage.

## Restart in a new chat

Owner note: attach the brief again and no screenshots.

> Attached is my brief (design-system.md). We worked through stage <n> in another chat; continue with stage <next>, as listed in section 9. The design system is <link>; the canvas is <link>. Current decisions: <paste the last decision blocks>. Use no images from the earlier chat. Do this stage only, then stop.
