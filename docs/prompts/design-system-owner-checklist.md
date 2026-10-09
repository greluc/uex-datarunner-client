# Owner checklist: running the design brief in Claude Design

> **Doc type:** Prompt — not yet run. Last reviewed: 2026-10-08.

For the owner only. **Never attach this file to Claude Design.** It replaces the "How to use" part that used to sit at the top of the brief.

## The files

- `design-system.md`: the brief for Claude Design. Attach it once, with the stage-1 message. Attach it again only if you start a new chat.
- `design-system-kickoff.md`: ready-to-paste messages for every stage, each with a decision block to fill in.
- `design-system-javafx-handoff.md`: the task for Claude Code after the last stage.

## Where to run it

- Use claude.ai on the web or in the desktop app. In the mobile apps you can ask for a design and view it, but "editing on the canvas and changing sharing settings need Claude on web or desktop" (Help Center 14604416).
- Before sending the stage-1 message, select **Output > Design** in the message box, or start from the Artifacts tab and pick the Design template (Help Center 14604416).
- The brief targets the artifact-based Claude Design (the Design System and Design types; their format rules were read on 2026-10-08). The standalone app at claude.ai/design has its own design-system setting (Help Center 14604397). Whether it follows the same rules is unverified; your SCTradersMate system was made there and used a different file layout.
- Plans: the Help Center says beta on Pro, Max, Team and Enterprise; the release notes say artifacts, including Claude Design, are on every plan, including Free. Artifacts need **"Cloud code execution and file creation"** switched on, in Settings > Capabilities (Free, Pro, Max) or Organization settings > Capabilities (Team, Enterprise) (Help Center 17153992).
- Usage draws from the same pool as Claude Code, and complex projects use more (Help Center 14604416). That is why every stage is its own message.

## Why "lean strongly" becomes genre-level closeness

You asked for a design that leans strongly on the newer RSI and Spectrum look. The brief implements that as genre-level closeness: dark blue-black, one light-blue accent, flat panels, linear lists. The reason is that the Design type's own instructions say "no other company's proprietary design" (Design type SKILL.md, read on 2026-10-08; not a public document), and the Usage Policy forbids infringing third-party intellectual property. Closer resemblance is not something Claude Design will do. This is our interpretation; how Claude Design reacts (dilution, refusal, partial matching) is unverified.

## Before stage 1

1. **Design systems.** Open Settings > Design systems. Check that none of your four systems is marked as default: ACL; DAS KARTELL – Profit Basetool Design System; Home Inventory Design System; SCTradersMate Design System. On 2026-10-08 none was. A default would be applied to new designs.
2. **No screenshots at stage 1.** Section 4.1 of the brief describes the look in the project's own words. If you still want mood images, attach them only after stage 1 has produced an original direction, 2–4 at most, each labelled "mood only – do not match layout, components or values": 100 % zoom, dark theme, PNG or JPEG at least 1000 px wide, with handles, avatars, e-mail addresses, pledges, store credit and balances cropped or redacted. An image sent in a chat stays in its context, so if Claude Design starts copying layouts or values, start a new chat without them (the "Restart" message in `design-system-kickoff.md`).
3. **Fonts.** After checkpoint 1, download the **static TTF files** of the chosen families, unmodified, at most 1 MB each, from google/fonts or upstream. At checkpoint 2a, drop them onto the page of the empty design system that stage 2a creates. The type documents an empty system's page as "one drop target" that files fonts in `fonts/` (format.md). Attaching TTFs to the chat is unverified: the upload help page (Help Center 8241126) lists images, PDF, DOCX, CSV, TXT, HTML, ODT, RTF, EPUB, JSON and XLSX, and no font types. If the drop fails, say so in the stage-2b message; Claude Design then uses the identical Google Fonts families as flagged stand-ins.
4. **Icons.** The recommended Material Symbols load from Google Fonts. Phosphor or IBM Carbon are compared only if you attach their SVGs, or a JSON file with each icon's name, viewBox and path data.

## Do not attach or connect

- DevTools dumps, RSI CSS, or any value lists measured from RSI pages;
- logos, the Fan Kit logo files, game screenshots or ship art (the brief draws the Fan Kit logo as a placeholder frame; see below);
- the old Claude Code prompt or the plan documents (the facts Claude Design needs are in Appendix A of the brief);
- this checklist.
- **Do not connect the GitHub repository.** It is a planning repository without UI code, and the brief is self-contained.

## Why the Fan Kit logo is a placeholder in Claude Design

The plan puts the Star Citizen Fan Kit unit into the About dialog and the onboarding start screen (R-UI-19). The brief gives Claude Design both notice texts verbatim, but lets it draw the logo only as a labelled placeholder frame. The real file is inserted by the app. The reasons:

- The logo is fixed artwork. Nothing about it is designable: it is used unmodified, and which variant goes on which theme is your check against the kit (O-27), not a design decision.
- Whether the Fankit Agreement allows uploading the logo to a third-party design tool was not checked; the Agreement was not re-read for this plan.
- The Design System type's own rules say to copy marks as files and never approximate them. A labelled placeholder satisfies that rule without the file.

If you want the real logo in the previews anyway, that is your decision; tell Claude Design in the stage message and upload the file yourself, and the brief's placeholder rule no longer applies to that file.

## During the stages

- Send one stage per message, using `design-system-kickoff.md`. Fill in the decision block: Claude Design knows only the decisions you write down.
- Give decisions in the chat; use inline comments only for small tweaks (known issue: comments "can disappear before Claude reads them", Help Center 14604416).
- There is no version history yet: use **Export > Download as .zip** at every checkpoint (Help Center 14604416). To explore, say "Save what we have and try a completely different approach."
- Stages 2a–3c work on the design system, stages 1 and 4 on canvases. Whether a still-selected Output > Design interferes with the system stages is unverified; if Claude Design starts a canvas instead, reply "Use the Design System type for this stage."
- Keep sharing at "Only you" until the name is checked (D1). Do not turn on "Published" and do not set the system as a default.
- After every stage that touched the FanKitUnit, the About board or the onboarding start board, check the two Fan Kit texts: copy each from the component's README or preview and compare it with `CLAUDE.md` "Star Citizen Fan Kit unit", for example with `printf '%s' '<text>' | sha256sum` against the SHA-256 there. Any changed character (a quote, a space, a full stop, ®) is a defect to report back.

## After the last stage

Give Claude Code the link to the design system and the canvases, the final Claude Design reply, and `design-system-javafx-handoff.md`. Claude Code reads the system's `README.md` and `tokens.json` with the Artifact tool. A .zip export or "Handoff to Claude Code" (Send to local coding agent / Send to Claude Code Web, Help Center 14604416) also works.

## Unverified in this setup

- whether the chat accepts `.md` attachments (the upload page lists TXT, not MD; if refused, rename the brief to `.txt` or paste the text), and TTF or SVG files;
- whether fonts can also be dropped onto a system page that already has files (the type's import guides mention "a drop on the page"; format.md documents the drop target only for the empty system);
- whether a line in the brief overrides a design system marked as default (step 1 avoids the question);
- whether Claude Design honours the brief's overrides of its defaults: 24 px targets instead of 44 px, no phone layouts, fill-based Material Symbols instead of stroke SVGs, the full inventory instead of a small first system;
- whether `data-theme` on a board root pins the theme of the light and high-contrast boards on the canvas;
- how Claude Design reacts to the RSI/Spectrum mood reference;
- whether the web-capture tool from the launch still exists (the brief forbids it for RSI, Spectrum and UEX pages anyway).

## Decisions the brief leaves to you

The repository tracks these as open point O-83 (and the handoff's D10 as O-84) in `docs/adr/0000-open-points.md`; record each decision there when you make it.

- D1 name (plus approval of `uex` or `datarunner` as name segments), D2–D9, the findings severity mapping and the row-selection style (checkpoint 1);
- D11, the layout of the Fan Kit unit and the non-affiliation statement in About and on the onboarding start screen (checkpoint 4; the Fan Kit texts are fixed, and the non-affiliation statement changes only if you change it);
- the app, window and tray icon (R-UI-9): ask for concepts or leave it to a later step;
- the state-language proposals (checkpoint 2): how a confirmed major deviation looks, the compact-cell icon slots, Δ formats for status and container sizes, the definition of "unreviewed", and whether "Accept all confident" covers minor deviations;
- the proposals in Appendix A (Unknown game state, Session overview as a sidebar item, dark as the default theme, density, reduce motion, notification bell, detail strip, row markers, saved sort order, and the others marked *(proposal)*); the high-contrast theme and the text-size setting are required (R-UI-17), not proposals;
- placeholders you may supply, or leave as placeholders: [UEX KEY PAGE URL], [GAME VERSION], [IDS], [MODEL NAME], [FOLDER].

Orbitron and Electrolize were removed from the candidates. Legacy and fan sources of low confidence associate them with the older RSI look (the 2013 Fan Site Kit, the HangarXPLOR CSS, a 2019 fan-forum post; Appendix A O12 and O13 of the previous prompt, commit `07f7356`), and the brief keeps a visible distance from the official brand. Share Tech Mono, named only in that 2019 fan-forum post, stays as an accent-only candidate; drop it too if you prefer more distance.
