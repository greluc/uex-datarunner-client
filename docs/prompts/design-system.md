# Task: Design the UEX Datarunner Client design system (inspired by the current RSI website and Spectrum)

> **Doc type:** Prompt — not yet run; rewrite pending (08 §K "Deferred"). Last reviewed: 2026-10-08.

> **Superseded in part by [ADR-0003](../adr/0003-licence-and-contributions.md) (2026-10-08); these rules win over the text below until the rewrite:**
>
> - The licence is GPL-3.0-or-later. The licence policy is CLAUDE.md "Stack rules", which already allows OFL-1.1 fonts and ISC icons. The §9 statements on the licence list and "GPL is not allowed" no longer apply.
> - The Star Citizen Fan Kit unit is adopted (R-UI-19; CLAUDE.md "Star Citizen Fan Kit unit"): the logo plus both notices, byte-exact, in the About dialog and on the onboarding start screen.
>   - Superseded: the bans on Fankit files and assets, the "Trademark line: UNVERIFIED" section and its neutral wording, D11, and the matching self-check items.
>   - Fan Kit and other proprietary fonts stay excluded.
> - The design-system ADR takes the next free number at push time (ADR-0004 as of 2026-10-08), never ADR-0003. `docs/adr/0003-design-system.md` becomes `docs/adr/NNNN-design-system.md`.
> - The README disclaimer is in its "Star Citizen fan content" and "Disclaimer" sections, not at line 29.
> - Assumptions run to A23 (new ones from A24), and the invariants are I1–I7.

You are Claude Code, working in the repository `uex-datarunner-client`. Design and document the app's **design system**: design tokens, themes, a semantic state language, component specifications, JavaFX CSS theme files, check scripts, a preview gallery and an ADR.

The visual language must **lean strongly on the newer (2025/2026) look of the Roberts Space Industries (RSI) website and its community platform Spectrum**, not on the older RSI look. The app is an **unofficial fan tool**. It must *evoke* that look, never copy protected brand assets, and never suggest that it is affiliated with or endorsed by Cloud Imperium Games (CIG) or RSI.

This prompt is self-contained. The appendices hold research notes from an earlier session (2026-10-08), each with its source and confidence. Treat them as leads, not facts, and re-verify them before you rely on them.

---

## Execution plan (read this first)

The work runs in stages. **Each stage ends with a STOP: send the owner the message described, end your turn and wait for the answer.** Do not merge stages.

| Stage | Work | Ends with |
|---|---|---|
| 1 – Read and verify | §0; §3.3 (record blocked hosts and never route around them); rebuild the state inventory from the plan (§6); run the spikes your environment can run (§4) | **Checkpoint A1** (§11): findings, the decisions that need no reference evidence, and the evidence request (Appendix E). **STOP.** |
| 2 – Foundations | Token file and `tokens.md` (structure and semantic layer, with palette values marked PROVISIONAL); `states.md`; contrast script; ADR-0003 draft (status Proposed) | Once the owner's evidence has arrived: **Checkpoint A2** (D2–D5 with previews). **STOP.** |
| 3 – Build | Token sheets with the decided palette; component CSS; icons and fonts; lint and font scripts; gallery; synthetic sample imagery | **Checkpoint B** (gallery, confusion review, app icon). **STOP.** |
| 4 – Verify and finish | CVD script; `accessibility-report.md` including the manual protocol; plan-document updates; self-check (§13); final report | Commit only after the owner's go-ahead. |

**Minimum viable deliverable**, in case later stages never happen. Stage 2 produces exactly this:

- `docs/design-system/README.md`: principles, guardrails, open points;
- `states.md`: the complete state inventory, with tone family, icon, label key and pseudo-class for each state;
- the token file plus `tokens.md`, with provisional values;
- the contrast script with its self-test and a first report;
- ADR-0003 (Proposed).

If the owner explicitly tells you to continue without reference evidence, you may build D2–D5 from the hypothesis in §3.5, labelled "inspired-by, not measured". Never make that call yourself.

## 0. Before you start

1. **Read `CLAUDE.md` in full.** Its rules are binding. The ones that matter most here:
   - Everything in the repository is English.
   - **Do not guess.** Check versions, licences and APIs at their source. If you cannot, mark the point as an assumption and ask.
   - Follow the supply-chain and licence rules.
   - No secrets in the repository.
   - Make small, focused commits.
2. **Read these plan parts.** They define what the design system must express.
   - `docs/plan/01-requirements.md`:
     - R-UI-1 … R-UI-15, especially R-UI-3 (crop in readable size), R-UI-4 (confidence display), R-UI-7 (theme, HiDPI), R-UI-9 (tray) and R-UI-10 … R-UI-12 (deviation marking).
     - R-SUB-3, R-SUB-7 (upload preview), R-SUB-8 (test mode is always visible), R-SUB-9, R-SUB-11 (outcomes, error classes), R-SUB-12.
     - R-CAP-1, R-CAP-1c, R-CAP-1e, R-CAP-8 (HDR and colour-encoding messages), **R-CAP-9 (capture check, "each with icon and text")**.
     - R-OCR-16, R-OCR-17, R-OCR-18 (tone findings and non-modal per-source notices).
     - R-MAN-1 … R-MAN-3, R-VAL-5, R-VAL-6, R-VLM-1 … R-VLM-8, R-API-7.
     - R-L10N-1, R-L10N-2, R-NF-1, R-NF-7, R-NF-8, R-NF-9, R-QA-5, R-DOC-1.
     - Assumptions A1–A16. Number new assumptions from A17.
   - `docs/plan/02-architecture.md`:
     - §2: module tree, package root placeholder `space.uexdatarunner.<module>`.
     - §3: `Field`, `FieldAssessment`, the `Finding` permits list, `ReportState`.
     - §4b: AI and game state.
     - §6: submission queue states.
     - §8: no extraction into the shared temp directory.
     - **§7: UI.** MVVM, the view list, theming ("independent of the OS", colours "replaceable in the theme"), and the pseudo-classes `:deviation-minor`, `:deviation-major`, `:no-reference`, `:needs-confirmation`.
   - `docs/plan/05-datarunner-bug-analysis.md`:
     - F13: low-confidence values were sent anyway, which is why the submission gate exists.
     - F26: white background and poor readability, which is why the app has its own theme.
   - `docs/plan/07-ocr-concept.md`:
     - §2.5b: stitching gaps.
     - §2.6: the confidence levels ok / confirm / select / correct, the send threshold 0.80, and the stale-reference rule.
   - `docs/plan/09-engineering-principles.md`:
     - §1: quality goals, including resource efficiency next to the running game.
     - §7: no colour values in Java code; one formatter class for numbers, Δ % and "3 days ago".
     - §9: dependencies.
     - §11: definition of done.
   - `docs/plan/10-supply-chain-security.md`, especially:
     - S-22 (pinned sources and SHA-256);
     - S-23 (no runtime downloads);
     - S-24 (no extraction into the shared temp directory);
     - S-25 (no auto-update);
     - S-29 (few dependencies).
   - `docs/plan/11-ddd-and-tdd.md`:
     - §A1: the ubiquitous language. Use these terms in UI copy and in the docs.
     - §A3: report states, transitions and invariants I1–I6.
   - `docs/adr/0002-modules-by-bounded-context.md` for the ADR format, and `README.md` for the current disclaimer (line 29).
3. **Check the repository state and concurrent work.**
   - Run `git status` and `git log -10`.
   - Other sessions may be editing the plan.
     - If plan files have uncommitted changes that you did not make, do not stage them, reformat them or commit on top of them. Ask the owner whether to wait or to work on a branch.
     - Stage only the files or hunks you changed, and do not use interactive git flags.
     - If you are on the default branch, create a branch first.
   - Does the Gradle build exist yet (`settings.gradle.kts`, `gradle/libs.versions.toml`, a `ui/` module)?
     - **If not** (planning phase, before M0), deliver only docs, the token file, CSS sheets and the `tools/design-system/` scripts and spikes.
     - Custom controls (e.g. a chamfer pane) stay spike code under `tools/design-system/spikes/<SP-n>/`. They do not go into `ui` until the M0 build exists.
     - The CLAUDE.md rule "`./gradlew check` green before every commit" applies only once the build exists.
   - CSS resource path: `ui/src/main/resources/space/uexdatarunner/ui/theme/`. This follows the 02 §2 placeholder. Record the later rename, once the owner decides the base package, as an open point.
   - Is there a `NOTICE` file? On 2026-10-08 there was none.
4. **Check your tools and record what works.**
   - **JDK:** run `java -version`. The session that wrote this prompt had JDK 21.0.12, Python 3.13, gpg and Xvfb, but no JDK 25+ and no JavaFX.
     - JavaFX 27 needs JDK 25 or newer. If none is available, ask the owner at Checkpoint A1 whether you may fetch one into a scratch directory outside the repository, with a verified checksum.
     - Until then, mark JavaFX spikes **"open – not run"**.
   - **Scripts and spikes:** self-contained, JDK-only programs, run with `java File.java`, with no preview features. They must run on the JDK of your session. On JDK 21 that means no implicit classes: write an explicit class with `main`.
   - **JavaFX jars for spikes:** fetch `org.openjfx` jars at a pinned version from Maven Central into a scratch directory outside the repository. Verify them against the published `.sha1` file, and against the `.asc` signature if gpg is available. A container without a display may need Xvfb.
   - **Web access:** check whether WebFetch, WebSearch or curl can reach the hosts in §3.3.
5. **Language.** The project owner may write in German. Reply in the owner's language, but write all repository content in English.

## 1. Goal, audience and non-goals

### Goal

A coherent, documented and testable design system for the JavaFX 27 desktop client (Windows and Linux). It must:

- make dense commodity data quick to scan and safe to review, so that nothing wrong is ever submitted silently;
- show every domain state that the plan defines unambiguously, using colour **and** icon **and** text;
- feel at home next to the current RSI website and Spectrum (see §3.2 for what "current" means and how it is established);
- work within the real limits of JavaFX 27 CSS (§4, Appendix B);
- pass scripted WCAG 2.2 contrast checks, colour-vision-deficiency (CVD) checks and a documented manual accessibility protocol in every theme.

### Audience and usage context

- **Who:** DataRunners. These are Star Citizen players who capture commodity-terminal data and submit it to UEX.
- **What they review:** dense tables. Each row holds:
  - commodity name, status, SCU, price/SCU and container sizes;
  - an image crop of the source;
  - the UEX reference value, its age, and Δ;
  - per-field markings for recognition confidence and deviation.
- **Where and when:** long sessions in dark rooms, often on a **second monitor next to the running game**, or right after playing.
- **How:** mostly by keyboard.
  - F8 / Shift+F8 jump from problem to problem (R-UI-5).
  - Number keys set the status (R-MAN-3).
  - Tab and Enter move through fields.
- **Platforms:**
  - Windows 10/11 and Linux (X11, XWayland).
  - HiDPI from 100 % to 250 %, including setups with several monitors at different DPI.
  - Users with colour vision deficiency.
  - Screen-reader users on Windows.

### Design principles

Refine these and document them, but keep their intent:

1. Legibility and scannability come before decoration. This applies above all to digits, which users compare against image crops.
2. Never use colour alone (R-UI-4, R-UI-11).
3. **Calm and low in luminance.** This must be measurable (§5, luminance band). Key status is always visible: test mode, game state, connection and data age.
4. **Do not compete with the game.**
   - The app never opens a dialog by itself while the game may be running (R-CAP-1), and never steals focus.
   - No continuous animations.
   - Expensive effects only where needed (09 §1).
5. Evoke, do not copy (§2).

### Non-goals

- **No brand assets:**
  - no RSI, Star Citizen, Squadron 42, Cloud Imperium, Spectrum or UEX logos, emblems or wordmarks;
  - no ship renders, concept art or game screenshots used as decoration;
  - no Fankit files;
  - no RSI CSS, SVGs, images or fonts.
- **No proprietary or Fankit fonts.** This includes the Banu and Xi'an fonts, Agency FB and Univia Pro.
- **No overlays or motion backgrounds:** no in-game overlay mode (R-NF-8), and no video or animated backgrounds.
- **No new dependencies or downloads:**
  - no new runtime dependency unless it is justified and the owner approves it (§9);
  - no runtime downloads of fonts, icons or stylesheets.
- **No real app code:** no real views or ViewModels beyond what the preview gallery and the spikes need.

## 2. Brand and legal guardrails (binding)

This is not legal advice. Flag legal open points to the owner instead of deciding them yourself.

### Naming

- **Original name.** Give the design system an original name. Propose 2–3 names; the owner decides (D1).
  - For each name, record a quick conflict search (EUIPO/USPTO trademark search and GitHub, if reachable). If a search is not reachable, list it as an owner task.
- **Token prefix.** Derive the token prefix from the name, e.g. `-xx-…`. Never use `-fx-` for our own tokens.
- **Forbidden name parts.** The following must not appear in the design-system name or in theme, token, file or style-class names:
  - CIG and RSI marks: Star Citizen, Squadron 42/SQ42, Roberts Space Industries, Cloud Imperium, Turbulent, CitizenCon, Arena Commander, Spectrum, Comm-Link;
  - in-game proper nouns: manufacturers, corporations, locations, and in-game UI terms such as mobiGlas;
  - the abbreviations `sc`, `rsi`, `cig`, `sq42`;
  - `uex` and `datarunner`, unless the owner explicitly approves them (they are a third-party brand and the name of the closed-source predecessor).
- **Lint.** The lint script (§7) rejects these as whole name segments, delimited by `-`, `_`, `.` or a camel-case boundary. It must not match raw substrings: `scroll` and `screen` must not trip the check.

### Disclaimer

- **Where:** a visible English notice from ResourceBundle keys (R-L10N-1), in the About dialog and in onboarding; later also in the README and the installer.
- **Legibility:** it uses a text token that meets 4.5:1 at 1.0 em or larger, so it is never muted fine print. The RSI link gets an accessible name from the bundle.
- **Draft text,** adapted for a desktop app from RSI's "Star Citizen Fankit and Fandom FAQ" (so far seen only in search-engine excerpts):
  > "UEX Datarunner Client is an unofficial Star Citizen fan project. It is not affiliated with or endorsed by the Cloud Imperium group of companies or by UEX Corp. All content not created by the project's contributors is the property of its respective owners."
- **Original wording.** Keep the FAQ's original fan-site wording in `references.md` for comparison:
  > "This is an unofficial Star Citizen fan site, not affiliated with the Cloud Imperium group of companies. All content on this site not authored by its host or users are property of their respective owners."

  This was quoted from search excerpts; verify it.
- **Link** to https://robertsspaceindustries.com.
- **Verification:** mark the wording as an assumption until a human has checked the live FAQ (https://support.robertsspaceindustries.com/hc/en-us/articles/360006895793-Star-Citizen-Fankit-and-Fandom-FAQ). Record the date of that check.
  - Whether the FAQ's fan-site notice applies to a desktop app at all is a reasonable analogy, not a confirmed rule.
- **README:** do not change the README disclaimer until the owner approves the wording (D11).

### Trademark line: UNVERIFIED

- A Star Citizen Wiki template (seen as a search excerpt) uses:
  > "Star Citizen®, Squadron 42®, Roberts Space Industries® and Cloud Imperium® are registered trademarks of Cloud Imperium Rights LLC."
- Sources disagree on the owning entity (Cloud Imperium Rights LLC, Rights Ltd, Games Corp or RSI Corp).
- The only registry evidence found (trademark.justia.com) covers STAR CITIZEN, SQUADRON 42, SQ42, CITIZENCON, ARENA COMMANDER and BAR CITIZEN. It does not cover "Roberts Space Industries" or "Cloud Imperium".
- Putting ® on an unregistered mark, or naming the wrong owner, is a factual and legal risk.
- **Until a human verifies each mark** in the USPTO, EUIPO or WIPO registers, or against the trademark notice in the official Fankit, use only neutral wording without ®, for example:
  > "Star Citizen, Squadron 42, Roberts Space Industries and Cloud Imperium are trademarks of the Cloud Imperium group of companies."

  An alternative is "… are trademarks of their respective owners".
- Record the check, its date and its source in `references.md` and as an assumption (A17+).

### Fan-content rules (search excerpts, medium confidence)

- Fan works must say that they are unofficial. Never call the project "official", "licensed" or "endorsed".
- No paywalls. The FAQ states this for video and streaming content; it is applied here by analogy.
- Domains and URLs must not contain "Star Citizen", "Roberts Space Industries", "Cloud Imperium", "Turbulent", "Squadron 42" or in-game entity names.
- Fankit assets require a login-gated Fankit Agreement and may not be redistributed. Do not use Fankit assets at all.

### No extraction

- Never extract fonts, icons or images from the game, the RSI Launcher or the website.
  - The basis is the RSI ToS, which prohibits reverse engineering (search excerpts, medium confidence), and this project's evoke-don't-copy policy.
  - CLAUDE.md itself only forbids decompiling the closed-source predecessor SC-Datarunner-UEX.
- **Inspiration only.** Colours, layout ideas and general patterns may be *inspired by* what you observe. Token values are our own decisions, adjusted for contrast and CVD.
  - No observed RSI or legacy hex value is used verbatim as a token. Record the difference to the observed value.
  - Never label values "RSI colours".

### No confusion with official products (testable checklist; part of §12)

- The app's own name and icon are visible in the shell on every view and in the window title.
- There is no RSI-style brand header, wordmark or logo-like emblem.
- The navigation structure is not a 1:1 copy of RSI's site header or of the RSI Launcher.
- Orbitron and Electrolize, the fonts most associated with RSI, are at most accents, never the primary UI identity.
- The disclaimer appears in About and in onboarding.
- Palette, typographic mood, density and frame style may be close to RSI. That closeness is the point.
- At Checkpoint B, a **confusion review** compares the gallery side by side with the owner's reference screenshots and lists the differentiators. The owner signs it off as part of D11.

### No credential confusion

- An RSI-like look combined with a masked key field could lead users to type their RSI password.
  - The key field and the onboarding key step name **UEX** explicitly and link to the page where UEX issues the key. Take the URL from `docs/plan/06-uex-api.md`, or ask.
  - Add bundle text such as: "This is your UEX secret key – never enter your RSI account password here."
- The key step must not use RSI login-form visuals. The app never shows fields for an RSI username, password or e-mail address.

### Imagery

- The gallery, the screenshot-pane examples and all committed previews use **synthetic sample imagery**, generated by a script:
  - procedurally drawn terminal-like panels in orange/red and in blue hues;
  - HDR-washed-out variants;
  - invented digits and names.
- Never commit game or RSI imagery in `docs/design-system/`.
- Corpus images may be used only locally, and only from the redacted `corpus/public/`.
- **Owner open point:** whether in-game imagery in the repository (the corpus) needs the FAQ's "Made by the Community" logo and trademark notice.

### UEX branding

- Not researched. Do not use the UEX logo, and ask the owner whether UEX has brand guidelines.

## 3. Design direction

### 3.1 Research status

The earlier session could **not** open:

- robertsspaceindustries.com, Spectrum or the RSI support site;
- starcitizen.tools, web.archive.org, brandfetch.com;
- openjfx.io, bugs.openjdk.org, w3.org.

The egress proxy refused them by organisation policy (403), or DNS failed. **No hex value, font or measurement in this prompt comes from the live 2025/2026 RSI or Spectrum CSS.** The sources are search-result snippets, third-party code on GitHub (userstyles, extensions) and legacy CSS. The observations are in **Appendix A**.

When you search, note that "Spectrum" is also the name of Adobe's design system. Exclude Adobe results.

### 3.2 Newer design, not the legacy look (hard rule)

- The owner asked for the **newer** design.
- Evidence of the 2025/2026 design is a **hard input for D2–D5**. That evidence is the owner's screenshots and DevTools values (§3.3, Appendix E), or sources you can reach directly.
- **Do not decide D2–D5,** and do not start the chamfer spike (SP-6), until that evidence exists, or until the owner explicitly waives it (Execution plan).
- In Appendix A, the observations tagged **LEGACY** describe the pre-2025 look. Use them only if the new design keeps them. This covers the cyan focus glow, Electrolize labels, Orbitron headlines and translucent navy panels.
- No source shows **chamfered corners or HUD corner brackets** on the current site. They are LEGACY or speculative.
- If the new design turns out flat, rounded and minimal, the design system follows it. Chamfers and HUD frames then drop to an optional accent.

### 3.3 Check the sources again (Stage 1)

1. **Try to open the sources.** Use WebFetch first, then curl for the HTML and the linked CSS bundles:
   - the RSI home page;
   - the pledge store and one ship page;
   - the Spectrum Star Citizen community page with its sidebar, a forum thread list, and one thread;
   - the Phase 1 and Phase 2 announcement threads and the DevTracker;
   - the Fankit and Fandom FAQ and the ToS (for §2).
2. **Egress policy.** If a host is blocked by the proxy or by organisation policy, **do not route around it**: no third-party fetchers, proxies, caches, mirrors or archive services used to obtain blocked content. Record the host as blocked and continue. Directly reachable public sources such as GitHub, npm, Maven Central and google/fonts are fine.
3. **If a source is reachable,** record what you see in `references.md`, with the URL and the access date:
   - colours of surfaces, text, accents and borders;
   - font families, sizes, weights, case and letter-spacing;
   - radii and frames;
   - shadows and glows;
   - motion.

   Reading CSS to learn values is fine. Copying CSS, SVGs, images or fonts into the repository is not.
4. **If sources are blocked,** do not stop. Put the evidence request from **Appendix E** into the single Checkpoint A1 message. It contains a screenshot checklist and a ready-to-paste DevTools console snippet that dumps computed styles and all CSS custom properties, so one paste can yield a whole palette.
   - Ask the owner to crop or redact handles, e-mail addresses, pledges, store credit and balances before sharing.
   - Reference screenshots and DevTools dumps stay **outside the repository**. If you keep them in a local folder, add it to `.gitignore` in the same commit.
5. **Tag unverified values.** Until a value is verified, every RSI-derived value carries a tag in the docs, for example "Brandfetch #0A1D29 – third-party scrape, unverified".

### 3.4 Patterns to adopt, translated to our domain

Appendix A gives the sources, by observation ID.

- **Navigation (O2, O5):**
  - A left sidebar for primary navigation that can collapse to an icon rail.
  - The rail expands as an overlay on hover **and on keyboard focus**, and also by a shortcut. Esc collapses it. It stays open while hovered or focused, does not reflow the table, and never covers the focused table cell.
  - Sidebar items are the views from 02 §7 (Input/queue, Manual capture, History, Settings, Diagnostics), plus a **Session overview** (R-UI-14). 02 §7 does not list the Session overview yet; add it to 02 §7 in the same commit.
  - The Report editor is a context view, opened from a queue or session row; it is not a sidebar item. The Onboarding wizard is not a sidebar item either.
  - Rarely used areas live in a menu.
- **Top bar and status (O2):**
  - A compact top bar opens slide-in panels instead of modal dialogs, for example notifications.
  - Slide-in panels move focus into the panel when they open and close with Esc. They return focus to the control that opened them and do not trap focus.
  - The top bar shows the global status: environment and test mode, game state, AI state, and connection with data age.
- **Linear, scannable lists (O3, O6):** for the queue, history and report rows.
  - Put the subject on the left: the commodity, or the report's terminal.
  - Put compact meta on the right, right-aligned with tabular digits: price, SCU, status, Δ.
- **Row markers and jump navigation (O6):**
  - Rows that hold fields needing confirmation or major deviations get a marker at the start of the row.
  - An "unreviewed" marker works like Spectrum's unread marker.
  - F8 and Shift+F8 jump between flagged fields.
- **Context header:** the equivalent of Spectrum's pinned threads. It shows the terminal, side, environment and game version at the top of the report.
- **Chips:** at most one category chip per row.
- **Detail layout (O1):** like the new ship pages, the report editor leads with the key facts and the gate status ("ready" or "blocked: reason"), with tabs or sections below.
- **Lessons from O4 (CIG's post-redesign feedback):**
  - Keep counters visible: deviating fields, strongly deviating fields, unreviewed fields, unexpected and missing commodities (R-UI-12).
  - No overly bright or bold titles in dense lists.
  - Save sort order and view mode per user.
- **Notifications and settings (O9):**
  - Toasts plus a persistent notification panel with "Clear all".
  - Settings grouped by topic.
- **Motion (O14):** a reduce-motion setting; no video or animated backgrounds.
- **Theme (O7):** dark as the primary theme, plus a real light theme.

### 3.5 Starting hypothesis for the look

This is unverified and partly LEGACY. It is a design starting point, not a measurement. It gets replaced by the evidence from §3.3.

- **Surfaces:** deep blue-black/navy. Leads: around #0A1D29 (Brandfetch, low confidence), and #0b2031/#162a3f (LEGACY). Show elevation as lighter surface levels.
- **Text:** cool blue-grey secondary text (around #A9B3BD) and **off-white, never pure white,** primary text (O4).
- **Accent:** a light-blue/cyan interactive accent. Leads: around #54ADF7 (Brandfetch) and #00B7CA/#25e4ff (LEGACY).
- **Accent and states stay separate.** The accent never doubles as a state colour; blue must not mean both "interactive" and "no reference".
- **Light theme:** cool greys with the same accent family, designed as a full variant in its own right (not an inversion).

## 4. JavaFX 27 CSS: the real constraints

The facts, with their sources, are in **Appendix B (J1–J25)**. Re-verify them before you build on them. What they mean for the design system:

1. **Colour tokens are looked-up colours on `.root` (J2).**
   - Define them in the token stylesheets. Component CSS refers only to token names.
   - Every variant (hover, pressed, translucent) is its own token, because there is no `alpha()` or `color-mix()`.
   - **`derive()` is non-linear** (J2). Do not use `derive()` or `ladder()` for any colour that matters for contrast, unless the check scripts reproduce `Utils.deriveColor` and `ladder()` exactly.
2. **Modena.** Decide whether to keep Modena as the user-agent stylesheet and map our tokens onto its lookups, or to replace it.
   - Modena computes most of its palette with `derive()` and `ladder()`, e.g. `-fx-background: derive(-fx-base,26.4%)` and `-fx-control-inner-background: derive(-fx-base,80%)`. Mapping only `-fx-base` or `-fx-accent` therefore produces colours the contrast script never sees.
   - If you map, list every Modena lookup in `.root` of `modena.css` at 27-ga that touches text, borders, fills, marks or focus. Override each one explicitly with a token value, or have the contrast script reproduce the derivations and test the resulting pairs.
   - Do not claim that stock controls follow the tokens automatically.
   - Either way, our sheets override Modena's Windows high-contrast sheets (J18), so we need our own high-contrast theme (§5).
3. **Non-colour tokens** (spacing, radii, chamfers, border widths, font sizes, durations): do not rely on lookups until SP-1 proves they work (J3). Possible routes:
   - em values relative to the root font size;
   - generation from the token file;
   - styleable properties on custom controls (J10).

   Decide in the ADR.
4. **Root font size and text size.**
   - The default font size differs per OS (J9). Set the root size from code: `max(platform default from Font.getDefault().getSize(), design base) × the user's text-size factor`, applied as an inline style on the root.
   - The in-app text-size setting is relative (100–200 %), not an absolute px value.
   - SP-4 checks whether the Windows "Text size" accessibility slider changes `Font.getDefault()`.
5. **Theme switching.** Choose after SP-2, from:
   - (A) one stylesheet per theme, swapped by the app;
   - (B) `@media (prefers-color-scheme: dark)`, driven by `Scene.getPreferences().setColorScheme(...)`;
   - (C) conditional `@import` (J17).

   Scene preferences are **per Scene and fall back to the platform, not to the owner window** (J16). Dialogs, Alerts, Tooltips, ContextMenus, ComboBox popups and Popup-based panels have their own scenes.
   - The chosen mechanism must apply the theme, high contrast and reduced motion to **every** new window, for example through a `Window.getWindows()` listener or a central stage/popup factory.
   - A half-dark app (a dark main window with light menus) is a defect.
6. **High contrast.** High contrast is not a colour scheme, so it gets its own sheet or a root style class (§5).
7. **Motion.**
   - Wrap every CSS transition in `@media not (prefers-reduced-motion)`. JavaFX does not disable transitions by itself (J15).
   - The in-app setting is a **tri-state: Follow system (default) / Reduce / Allow**.
     - "Follow system" sets the Scene override to `null`.
     - "Reduce" sets it to `true`.
     - `false` is set only when the user explicitly chooses "Allow", so the app never overrides an OS reduce-motion request by default.
   - Keep durations short and use no infinite animations.
   - Indeterminate progress (ProgressBar, ProgressIndicator) does not honour reduced motion (J15). Provide a static replacement, such as text plus a percentage or a step count.
   - Consider forcing reduced motion while the game is running; propose it to the owner (D8).
8. **Chamfers (only if the new design keeps them, §3.2).**
   - **The `-fx-shape` route keeps insets** (J11). The 27-ga renderer honours background-fill and border-stroke insets and refits the shape into each inset box. It scales the shape non-uniformly, so a chamfer's leg length changes with the aspect ratio and slightly with each inset ring.
   - For resizable panels, a custom Region that builds its shape from width, height and a styleable chamfer-size property keeps the angles constant.
   - SP-6 compares both routes.
   - **Never chamfer table cells or inputs inside tables.**
9. **Glow.**
   - Use glow only on single focal elements, never per row or per cell (J13).
   - Glow never counts toward any contrast or focus requirement. A solid indicator must pass on its own.
10. **Typography gaps (J4, J7, J8).**
    - The font choice must do the work that letter-spacing and `tnum` would do in web CSS.
    - If you use uppercase labels, transform them in the view layer with an explicit Locale, starting from sentence-case bundle text.
    - Keep sentence-case text in `accessibleText`, because screen readers may spell out all-caps words letter by letter. Check this with Narrator or NVDA.
11. **Table focus (J24).**
    - Table cells get `:focused` from the table's focus model but **never `:focus-visible`**.
    - Style cell focus like Modena: `.table-view:focused:cell-selection > .virtual-flow > .clipped-container > .sheet > .table-row-cell > .table-cell:focused`. This needs cell selection to be enabled.
    - Draw the indicator **inside** the cell, because neighbouring cells in the clipped container paint over anything outside its bounds.
    - Outside tables, use `:focus-visible`.
12. **Accessibility scope (J22).**
    - Screen-reader support is Windows-only; document this.
    - There are no live regions, so results must stay available as visible, focusable text (notification panel, history). Toasts are never the only channel.
13. **HiDPI (J21).**
    - Test at 100, 125, 150, 175, 200 and 250 %.
    - Avoid sub-pixel hairlines.
    - Prefer vector icons (`-fx-shape`) to raster images.
14. **Window chrome (J23).** HeaderBar with `StageStyle.EXTENDED` is optional and needs the owner's decision (D9). Keep DECORATED as the fallback.

### Spikes

- Write each spike as a small, runnable, self-contained program under `tools/design-system/spikes/<SP-n>/`. It states its question and the evidence it expects.
- Record each result in the ADR. If you cannot run it, commit it with instructions and mark it **"open – not run"** with the reason. Never invent results.

- **SP-1** Do size, radius and duration lookups (including em units) resolve correctly?
- **SP-2** Theme switching with options A, B and C, including the title bar.
  - Check that theme, high contrast and reduced motion apply on: the main Scene, a Dialog/Alert, a Tooltip, a ContextMenu, a ComboBox/ChoiceBox popup and a Popup-based panel. Document the mechanism that applies them to every new window.
  - Windows high-contrast detection (`Windows.SPI.HighContrast`), including changes at runtime.
  - Is there any GTK key or theme name that signals high contrast on Linux?
- **SP-3** Do custom lookup names with our prefix (e.g. `-xx-surface-1`) resolve, both in Modena-mapped rules and in our own rules?
- **SP-4** Font loading:
  - which family and face names are visible after loading static TTFs on Windows and Linux (`Font.getFamilies()`, `Font.getFontNames()`);
  - how regular and bold are mapped;
  - the default instance of a variable TTF;
  - glyph fallback;
  - the temp-file copy (J6), and loading via a `file:` URL inside a jlink image as an alternative;
  - whether the Windows "Text size" slider changes `Font.getDefault()`.
- **SP-5** Icons:
  - Do all icons get a consistent optical size and aspect ratio when their path bounds differ from their viewBox? With `-fx-scale-shape: true` the path bounds, not the viewBox, are scaled (inference). Remedies: include the viewBox corner points in the path, or use SVGPath with explicit scaling.
  - Stroke-based icons: test outline conversion, an SVGPath with `-fx-stroke`, and a Region with a transparent background and a border stroke. Check whether the stroke width scales with the shape.
- **SP-6** (only after §3.2 is settled) Chamfer: `-fx-shape` with insets vs a custom Region with a styleable chamfer size. Check the border stroke, the focus ring and fractional HiDPI scales.
- **SP-7** Effects and transitions in a TableView with 200 rows, on the hardware pipeline and on `-Dprism.order=sw` (with `-Djavafx.pulseLogger=true`). Which transition property names apply to Region backgrounds and borders?
- **SP-8** JPMS: can the `ui` module load its CSS and fonts via `getClass().getResource(...).toExternalForm()`, and do relative `@font-face` URLs work inside the module?
- **SP-9** Table cell states. Test a TableCell with all of these at once: a confidence border, a focus indicator, a deviation background, row selection, row hover and editing, at 100/125/150/250 %.
  - Does an inner focus ring with a gap inside the confidence border work?
  - Or should confidence move to an inner Region or icon slot, or a different edge?
  - Is a 2 px solid focus ring visible on every deviation background?
  - Does F8 with `scrollTo` keep the focused cell fully visible below the sticky column header and the summary header?

## 5. Token architecture

### Layers

1. **Primitive tokens.** Raw values; component CSS never uses them directly.
   - A neutral ramp (blue-tinted greys).
   - An accent ramp.
   - One ramp per tone-family hue.
2. **Semantic tokens:**
   - surface levels `surface-0` … `surface-n`, plus sunken, raised, popover and scrim;
   - borders: subtle, default, strong, and a frame accent;
   - text: primary, secondary, muted, disabled, inverse, on-accent, link;
   - accent: default, hover, pressed, subtle background;
   - focus ring (solid) and an optional focus glow (decorative only);
   - selection: row and text;
   - **tone families**: a small, fixed set, each with `fg` (text and icon), `bg`, `border` and, where needed, `bg-strong`:
     - neutral, accent, info, success, caution, critical;
     - the dedicated **confidence** family (ok / confirm / select / correct);
     - the dedicated **deviation** family (minor / major / no reference / reference outdated);
     - **environment/mode** (test vs production).
3. **Component tokens.** Use them only for documented, deliberate exceptions.

**Every state in §6 maps to one tone family, plus its own icon and its own label.** States are not separated by giving each one its own hue.

### Categories (all required)

- **Colour**, as above.
- **Typography:**
  - families per role (§9);
  - a size scale in em, relative to the root size;
  - minimum sizes (§9);
  - line spacing;
  - weights, addressed by family or face name (J7);
  - rules for when to use uppercase.
- **Spacing:** a scale, with density variants "compact" (the default for the review table) and "comfortable".
- **Shape:**
  - a radius scale and, if used, chamfer sizes;
  - border widths: at least 1 px; **state-carrying borders at least 2 px at 100 %**;
  - no sub-pixel hairlines.
- **Elevation:** surface levels by lightness, optional shadows, and glow tokens for focus and active states only.
- **Motion:** durations, easings and reduced-motion behaviour.
- **Iconography:** sizes, and minimum stroke or feature width (§9).
- **Layout:**
  - sidebar and rail widths;
  - breakpoints via viewport media queries (JavaFX 26+), used only for density and layout;
  - table row heights per density: **at least 24 px at 100 % in every density**, including every interactive in-cell control, expressed in em and checked by the token lint (WCAG 2.5.8);
  - crop-thumbnail height (R-UI-3, readable size). The trade-off against compact row height is part of D8.

### Themes

- **Dark:** the primary theme and the default.
- **Light:** a full variant, designed on its own (not an inversion), with its own contrast results.
- **High contrast: mandatory.**
  - Target 7:1 for text (WCAG 1.4.6 AAA).
  - Solid borders.
  - No translucency, no glow, no background-only state signals.
  - **Applied automatically while `Windows.SPI.HighContrast` is true**, unless the user explicitly chose another theme; listen for changes at runtime. This replaces the Windows high-contrast support that our author sheets override (J18).
  - Where feasible, evaluate taking colours from the Windows contrast-theme keys (J16). If you do, run the contrast check at runtime.
  - On Linux there is no reliable signal (SP-2); offer manual selection.
  - Automatic HC is an exception to 02 §7 "independent of the OS". Ask the owner (A1) and update 02 §7 in the same commit.
- **Dark-theme luminance band.** Propose, document and lint these rules; the owner confirms the values at A2 (D3):
  - primary text contrast between 7:1 and a documented maximum;
  - primary text is never #FFFFFF;
  - no large surface above a documented relative luminance;
  - no bright light-only areas (dialogs, banners) in the dark theme.

Every theme defines **every** semantic token; a script checks this.

### Machine-readable token file

- It is the single source for:
  - token names and descriptions;
  - values per theme;
  - the **declared fg/bg pairings** that the contrast script tests;
  - the **co-occurrence list** of state colours that can appear together on one screen, which the CVD script tests (§7).
- **Format: DTCG 2025.10.** The W3C Design Tokens Community Group repository (github.com/design-tokens/community-group, HEAD 2026-09-08) contains versioned schemas under `schemas/src/2025.10/`. A stable 2025.10 release was announced on 2025-10-28 (seen in a search result; w3.org and designtokens.org were unreachable). Confirm the status on designtokens.org.
  - In that format a colour value is an object: `{colorSpace, components[, alpha][, hex]}`.
  - JavaFX CSS has only sRGB (`rgb`/`hsb`), so restrict colour tokens to `colorSpace: "srgb"` with `hex`.
  - Document dimension and duration tokens as em or ms values for the generator and the lint.
  - If DTCG does not fit, use a documented JSON schema of your own and justify it in the ADR.

### User colour overrides (R-UI-11, 02 §7)

R-UI-11 requires state colours to be "configurable in the theme", and 02 §7 says they are "replaceable in the theme". The ADR decides how. Options:

- a settings page with a token picker;
- a user CSS file restricted to looked-up colour declarations on `.root`.

If a user stylesheet is allowed, the app validates it before loading it:

- only `<token>: <color>;` declarations;
- no `url()`, `@import`, `@font-face` or `image-pattern`. These would reopen runtime loading (S-23).

The app runs the same contrast and CVD checks at runtime and shows any failing pairs; otherwise the theme is shown as "unchecked". Document this in `states.md` and the ADR.

### Generated artefacts (D10)

CLAUDE.md says "never commit generated artefacts". List every generated file in the ADR, for example:

- the token CSS sheets, if generated from the token file;
- `icons.css`, if regenerated from an upstream archive;
- the generated results section of `accessibility-report.md`;
- preview PNGs.

Each one either is committed with a `GENERATED` header and a drift check, as an owner-approved exception, or stays out of the repository.

Previews live in the session scratchpad, or in a gitignored `design-previews/` folder that you add to `.gitignore` in the same commit.

## 6. Semantic state language

**The table below is a snapshot from 2026-10-08.** At execution time, rebuild the state inventory from the current 01, 02, 07 and 11:

- sealed `permits` lists and enums;
- R-* rows that mention icon, text, status or messages;
- quoted UI messages.

Report every difference from this table in `states.md`.

Every state needs:

- a tone family (§5);
- an icon. Icons are **distinct glyphs within a group, and across the deviation family vs generic caution/critical**;
- an English label (and, where useful, a description) as ResourceBundle keys. Labels are distinct within a group;
- a CSS pseudo-class or style class;
- an example in the gallery and in `states.md`.

**Colour is never the only signal.**

| Group | States (domain terms of 11 §A1) | Source | Visual channel |
|---|---|---|---|
| Recognition confidence (per field) | **ok** (at or above the send threshold); **confirm** (below the threshold, one candidate, e.g. 0.75); **select** (ambiguous, several candidates, 0.60); **correct** (unreadable or implausible, 0.30); also **confirmed by the user**, and optionally **double confirmed** (OCR and VLM agree, 0.97) as a subtle marker | R-UI-4, 07 §2.6 | **Icon plus border on the field** plus a text reason – never the background |
| Deviation from UEX (per field) | **equal** (no marking); **minor** ("please cross-check"); **major** (confirmation required, blocks submission); **no reference** (new at this terminal); **reference outdated** (lowers the level by one step and shows the label) | R-UI-10, R-UI-11, 07 §2.6 | **Background plus Δ badge** (absolute and %), plus the UEX value and its age, plus the image crop |
| Row and report level | unexpected commodity; missing commodity (R-VAL-5); "possibly incomplete" (stitching gap); counters for deviating, strongly deviating and unreviewed fields | R-UI-12, 07 §2.5b | Row marker, summary header, chip |
| Report state | **Draft** (sub-states: ready / blocked with reason / with findings / to be checked (I5)). Blocked reasons include: terminal unresolved, fields below threshold, major deviation unconfirmed, environment or version not accepted, observation too old (I6, R-VAL-6). **Released; Queued; WaitingForCooldown** (with remaining time); **Submitted; PartiallyAccepted; OutcomeUnknown; Rejected; Withdrawn** | 02 §3 `ReportState`, 11 §A3, R-UI-1, R-SUB-4, R-SUB-9 | State chip plus icon plus plain-word label |
| Submission job and error class | queued; waiting-for-cooldown; sending; succeeded; partially-accepted; outcome-unknown; held (acceptance closed); paused (account); failed. Error classes: transient, account, acceptance closed, report-fixable, permanent | 02 §6, R-SUB-11 | Chip, notification, history row |
| Capture | Imported; Scanned; Failed (with reason, R-CAP-1c); environment pending; not a terminal (R-OCR-16); set aside "HDR or unsupported colour encoding – use an SDR copy"; set aside "HDR screenshot format (JPEG XR/AVIF/EXR) – not supported"; "HDR original – SDR copy used" (needs no action); AI pending | 11 §A3, R-CAP-1, R-CAP-1c, R-CAP-8, R-OCR-16 | Queue row, import result |
| Capture check and capture quality | Capture-check result lines, each with icon and text: format and colour encoding, image size, panel located, price-digit cap height against the limits, tone state, plus advice. Per-source notices from R-OCR-18 ("Recent captures from <folder> look washed out – see capture tips"), with "Don't show again for this source" | R-CAP-9, R-OCR-17, R-OCR-18 | Diagnostics/onboarding result list; non-modal queue notice |
| Watched folder | manual / automatic; watched / not watched (with reason); polling active; missing | R-CAP-1e | Settings and status |
| Environment and mode | LIVE, PTU, EPTU, HOTFIX, TECH-PREVIEW. **Test mode (`is_production=0`) vs production**: always visible, fixed on each report at release, and shown on report and history rows | R-SUB-8, 11 §A1 | A persistent, unmistakable indicator in every view that does not use the critical family; production is labelled too |
| Game state | running; closed; unknown (treated as running) | R-VLM-2 | Status bar or top bar |
| AI (VLM) | Mode: Off / Automatic / Always (Always carries a warning). Ollama: not installed / not running / model missing / ready. Job: pending / running / done / cancelled because the game started. Also "running on CPU" and pull progress | R-VLM-1 … R-VLM-8 | Status, settings, queue row |
| Connection and data | online; offline (with cached data and its age); refreshing; **no reference data yet**; reference data outdated; UEX not accepting reports; key valid / invalid / user not allowed / disabled (queue paused); cooldown remaining | R-UI-8, R-UI-13, R-SUB-5, R-API-7, R-SUB-11 | Status bar, banner, empty states |
| Findings severity | Proposed: info / warning / blocking. Finding types: copy the current `Finding` permits list from 02 §3. On 2026-10-08 it was: Ambiguous, OutOfTolerance, NoReference, Repaired, Inconsistent, Unreadable, PartialCard, Superseded, UnvalidatedGameVersion, LowContrastCapture, ClippedHighlights, SmallText, TextTooSmall, Downscaled. **Downscaled is already "info only" per 02 §3.** Mapping the others is a decision for the Reporting domain: propose a mapping and ask; do not encode it silently | 02 §3 | Icon plus text in the findings list |
| Generic feedback | success, info, warning, danger/destructive, neutral, update available | R-NF-7 | Toasts, banners, buttons |

### Rules

- **Separate channels for separate dimensions** (R-UI-4):
  - confidence uses the border and an icon;
  - deviation uses the background and a Δ badge;
  - focus, selection and hover each get a channel of their own.
- **Table focus.**
  - The focus indicator is drawn inside the cell (J24): a solid ring at least 2 px wide at 100 %, never glow only.
  - It meets 3:1 against every deviation background and selection tint.
  - The confidence border sits further inside, or moves to a different edge or an icon slot. SP-9 decides.
- **Selection** is not a full background fill that hides deviation backgrounds. Consider a left accent bar plus a light tint, and check its contrast.
- **Focus not obscured (WCAG 2.4.11).**
  - F8, Shift+F8 and `scrollTo` keep the focused cell fully visible below the sticky column header and the summary header.
  - No toast, slide-in panel or rail overlay covers it.
- **Essential information is never tooltip-only.**
  - JavaFX tooltips appear on mouse hover and hide after 5 s by default (J25).
  - The confidence reason, findings, UEX value and age, and the full commodity name are shown in a focus-driven **detail strip** that follows the focused cell, and also in `accessibleText`/`accessibleHelp`.
  - Tooltips are supplementary only: indefinite show duration, dismissed with Esc, never covering the focused cell.
- **Combination matrix.** `states.md` and the gallery show the combinations that can occur. Example: a cell that is "confirm" and "major deviation" and "reference outdated" at once, in a selected and hovered row, while focused and being edited. Every combination must stay legible and pass the contrast check.
- **Pseudo-classes.**
  - Keep the four existing ones from 02 §7.
  - Propose a complete naming scheme, e.g. `:confidence-confirm`, `:confidence-select`, `:confidence-correct`, `:confirmed`, `:reference-outdated`.
  - Update 02 §7 in the same commit.
  - The ViewModel maps `FieldAssessment` to pseudo-classes only; there are no colours in Java code (09 §7).
- **Conflict to resolve with the owner (D7).**
  - R-UI-10 names the hues yellow (minor), orange (major) and blue (no reference).
  - Blue clashes with the cyan/blue accent.
  - Yellow vs orange, and orange vs a red danger colour, may collapse under protanopia and deuteranopia; show the simulations.
  - Option 1: keep the hues and separate them by lightness, icon and Δ-badge weight.
  - Option 2: change the hue mapping, and update R-UI-10 and R-UI-11 in the same commit.
  - Ask before deciding. "No pure red/green" (R-UI-11) applies throughout.
- **Screenshot overlays** (source-region highlight, crop and corner handles, redaction preview):
  - They must be visible on orange/red **and** blue terminal imagery, and on washed-out HDR captures.
  - Use a double stroke (dark plus light) and optionally dim the rest of the image. Never rely on hue alone.
  - Text and controls drawn over imagery (zoom controls, labels) sit on an opaque scrim token, which is contrast-checked.
- **Data display conventions.** Agree these with the single formatter (09 §7):
  - numerals right-aligned, in a font with tabular digits;
  - thousands separators;
  - units (aUEC, SCU) as muted suffixes (still ≥ 4.5:1, §7);
  - Δ with an explicit sign, using the minus glyph U+2212. Java's English `NumberFormat` emits a hyphen-minus, so the formatter must substitute it explicitly;
  - percent format: "+12%" (English convention) vs "+12 %" (as written in R-UI-11) is an owner decision (D8). If it differs from R-UI-11, update R-UI-11 in the same commit;
  - age as "3 days ago";
  - status names and the number of status levels come from UEX reference data, never from constants (F14);
  - long, nearly identical commodity names must never lose their distinguishing token (e.g. "Size 1" vs "Size 7"). Prefer wrapping, or a middle ellipsis, with the full name in the detail strip.

## 7. Accessibility requirements, scripted checks and manual protocol

### Requirements (WCAG 2.2, adapted to a desktop app)

Check the criteria and thresholds at https://www.w3.org/TR/WCAG22/.

- **1.4.3** At least 4.5:1 for text.
  - Use 4.5:1 for all table and UI text; 3:1 only for genuinely large text.
  - **Every text token except "disabled"** meets 4.5:1 on every surface it can appear on. This includes muted suffixes, ages, placeholders and the disclaimer.
- **1.4.11** At least 3:1 for anything that carries meaning without text, against adjacent colours: component boundaries needed to identify a control, state indicators, meaningful icons and focus indicators.
- **1.4.1** Colour is never the only means of conveying information.
- **1.4.13** Content on hover or focus is dismissible, hoverable and persistent. See the tooltip rule in §6.
- **Focus:**
  - 2.4.7 (focus visible) and 2.4.11 (focus not obscured).
  - For the focus ring, aim for 2.4.13 (AAA).
  - Glow never counts toward any of these.
- **2.5.8** Targets of at least 24×24 px at 100 %.
  - Keyboard operability is **not** one of the criterion's exceptions.
  - Rows and interactive in-cell controls are at least 24 px high in every density. Non-interactive badges are exempt.
  - If the owner wants denser rows, record that as a documented, owner-approved non-conformance.
- **Text size:** a setting of up to 200 % without loss of content or function, at a defined minimum window size (propose one, e.g. 1280×720).
- **Disabled controls** are exempt from the contrast criteria but must stay discernible.
- **Keyboard:** every action can be reached by keyboard, the focus order is logical, and shortcuts are shown.
- **Screen readers (Windows):** icon-only controls, state chips and state cells get `accessibleText` and `accessibleRole` (and `accessibleHelp` where useful) from the ResourceBundle.

### Scripted checks

- **Form:** JDK-only, single-file Java programs (see §0 for JDK compatibility), with no third-party libraries.
- **Location:** under `tools/design-system/`.
- **Later integration:** write them so they can become Gradle tasks in `build-logic` and part of `./gradlew check`.
- **Self-test:** each script has a self-test mode.

1. **Contrast check.**
   - Uses WCAG relative luminance and the contrast ratio.
   - Covers every declared fg/bg pairing in every theme, including:
     - translucent colours, composited over each surface they can appear on;
     - state backgrounds combined with selection and hover;
     - Δ badges;
     - the focus ring against adjacent surfaces and every deviation background;
     - the overlay scrim.
   - Includes any Modena-derived colours (§4 item 2), unless they are overridden.
   - Checks the dark-theme luminance band (§5).
   - Self-tests: #000000 on #FFFFFF = 21.0:1; #777777 on #FFFFFF ≈ 4.48:1, which fails 4.5:1.
2. **CVD check.**
   - **Method:**
     - Simulate protanopia, deuteranopia and tritanopia at severity 1.0, using the matrices of Machado, Oliveira and Fernandes (2009, IEEE TVCG 15(6)) applied in linear RGB.
     - If feasible, cross-check tritanopia with Brettel, Viénot and Mollon (1997, JOSA A 14(10)).
   - **Gated sets.** The pairwise CIEDE2000 test, for normal vision and each simulation, runs only where colour does real work:
     - confidence levels;
     - deviation levels;
     - findings severity;
     - generic feedback;
     - test vs production;
     - every pair in the token file's **co-occurrence list** (e.g. a major-deviation cell next to a danger button or a "blocked" chip).
   - **Other groups.** The check only confirms that each tone family differs from its surfaces. `states.md` shows that icon plus label separate the states.
   - **Threshold.** Fail below a minimum ΔE00 that you choose, justify with a cited source and record.
   - **Reports.** A greyscale (achromatopsia) report and simulated swatch images are produced for information.
   - **Self-test.** CIEDE2000 against the published test data of Sharma, Wu and Dalal (2005, Color Research & Application 30(1)).
   - Check all formulas and matrices at their sources.
3. **Lint.**
   - No literal colours (hex, `rgb()`, `hsb()`, named colours) outside the token sheets.
   - No `-fx-` prefix on our own tokens.
   - Forbidden name segments (§2).
   - Every semantic token is defined in every theme, and every referenced token exists.
   - Every state in §6 has its tokens, a distinct icon and a distinct label key within its group, and across deviation vs generic caution/critical.
   - Every transition is inside a reduced-motion guard.
   - Row heights and in-cell targets are at least 24 px at 100 %.
   - Warn on properties that are not in the JavaFX 27 CSS reference (build a whitelist from cssref at 27-ga).
4. **Font check.**
   - The SHA-256 of each vendored file matches `NOTICE`.
   - In the numerals font:
     - 0–9 have identical advance widths;
     - `+` and `−` (U+2212) have identical widths to each other, as do `±` if used;
     - `,` and `.` have identical widths to each other;
     - whether these equal the digit width is reported for information only. Right-aligned columns use a fixed format per column, so consistent separators keep alignment.
   - Glyph coverage includes:
     - every character used in UI texts;
     - Δ (U+0394), − (U+2212), ±, ×, %, …, →, ←, ↑, ↓, ●, ·, ≥, ≤, – and —, and curly quotes;
     - Latin-1 Supplement and Latin Extended-A, because commodity names can be localized via `global.ini` (R-L10N-2).

### Manual accessibility protocol

Run it against the gallery and record each item in `accessibility-report.md` as pass, fail, or "not run – reason":

- a keyboard-only walkthrough of every component, including the rail and the slide-in panels;
- focus is visible and not obscured in every component state, including table cells during F8 navigation;
- text size 200 % at the minimum window size, with no meaningful content truncated;
- each scale (100–250 %) via `-Dglass.win.uiScale` and `-Dglass.gtk.uiScale`;
- reduced motion on and off;
- the Windows contrast theme switched on, so the HC theme applies automatically;
- Narrator (or NVDA) reading a state cell, a state chip and an icon-only button;
- a **glyph-confusability review** of the numerals font (§9);
- a greyscale gallery render.

### Results

- Results, both passes and failures, go into `accessibility-report.md`. The scripted part is a generated section with the date, the tool's commit and the inputs.
- Fix failures; do not waive them. Any waiver needs the owner's approval and a written rationale.

## 8. Component specifications

For each component, `components.md` specifies:

- anatomy;
- the tokens it uses;
- sizes in em;
- density variants;
- states: default, hover, **focus** (`:focus-visible`, or the table-cell rule), pressed, selected, disabled, error/invalid, read-only, loading;
- keyboard behaviour and focus order;
- `accessibleRole`, `accessibleText` and bundle keys;
- the JavaFX implementation: control, style classes, pseudo-classes, and a custom control if needed (as spike code before M0);
- an example in the gallery.

### Components

1. **App shell:**
   - sidebar and icon rail (§3.4: hover **and** focus expansion, shortcut, Esc);
   - the top bar with global status: environment and test mode, game, AI, connection and data age, plus a notification bell that opens a slide-in panel;
   - a status bar with the background-loading status line (R-UI-8), queue counts and cooldowns;
   - the app's own name always visible (§2);
   - optionally a HeaderBar variant (D9).
2. **App, window and tray icon** (R-UI-9):
   - original geometry, with no RSI emblem, Star Citizen or Squadron 42 logo, manufacturer logo, ship silhouette or UEX logo, and no resemblance to any of them;
   - legible at 16×16 in the tray, on light and on dark taskbars;
   - ICO/PNG sizes for 100–250 % and a Linux `.desktop` icon;
   - the licence of any icon glyph used is recorded in `NOTICE`;
   - the owner approves it at Checkpoint B.
3. **Tabs:** for report sections and settings sections.
4. **Buttons:**
   - variants: primary, secondary, tertiary/ghost, danger/destructive, icon-only, toggle;
   - shortcut hints;
   - "Accept all confident" and "Release all ready" as examples.
5. **Inputs:**
   - text field and numeric field (right-aligned, with unit suffix);
   - **a searchable terminal combobox**: fuzzy search over name, nickname, location and system; a filter by star system; "recently used" (R-MAN-1). When the terminal is ambiguous, the choice is mandatory and **nothing is preselected silently**;
   - a status selector with number-key shortcuts for the status levels provided by UEX. Currently 1–7 per R-MAN-3, but the count and names come from the reference data and are never hard-coded;
   - container-size chips (multi-select) and an environment selector;
   - checkbox, radio button, toggle switch;
   - threshold fields that show their documented defaults;
   - a folder path picker;
   - **the UEX secret-key field**: masked, with a reveal option, never logged, labelled as the UEX key, with the "never your RSI password" hint and a link to where UEX issues it (§2).
6. **Dense editable TableView** (report rows):
   - cell selection enabled; focus styling per J24 and §6;
   - rows in screen order, each with its crop thumbnail;
   - per-cell confidence and deviation states and the Δ badge;
   - the UEX reference value and its age. In manual entry it is shown as a reference and never pre-filled (R-MAN-2);
   - a confirmation control. A confirmation applies to exactly that value and must be repeated after any change (R-UI-12);
   - F8 / Shift+F8 navigation;
   - the focus-driven detail strip (§6);
   - the summary header, sticky column headers, sort order saved per user, and empty rows.
   - Also cover the queue, history and session-overview lists (R-UI-14).
7. **Screenshot pane:**
   - zoom and pan;
   - a source-region highlight for the focused field (R-UI-3);
   - crop and corner handles (R-UI-6), adjustable by keyboard;
   - a redaction overlay and the full-size preview confirmation "contains no balance or personal data" (R-SUB-7);
   - gallery examples use synthetic imagery only (§2).
8. **Cards and panels:**
   - frame style per the D5 decision: a plain line frame, or chamfer or corner brackets only if the new design keeps them (§3.2);
   - a header strip with a label;
   - elevation levels.
9. **Badges and chips:** state chips, the environment chip, source chips (OCR, VLM, manual, UEX reference), the Δ badge, count badges, and status dots (always with text).
10. **Dialogs:**
    - deviation confirmation;
    - destructive actions: withdraw, "delete all local data" (R-NF-9);
    - consent for cloud models or non-local hosts, with a privacy notice (R-VLM-6);
    - the dialog shown when the key changes to a different user (R-SUB-12).

    No dialog opens by itself while the game may be running.
11. **Toasts and notification panel:**
    - non-modal, and they never steal focus or cover the focused cell;
    - "Clear all";
    - results also persist in the history, because there is no live region.
12. **Progress:**
    - determinate (a model pull with its size);
    - indeterminate (refresh);
    - per-row processing;
    - an AI-running indicator;
    - all with static reduced-motion variants (§4 item 7).
13. **Onboarding wizard:**
    - steps: key and `/user` check, suggested folders that the user confirms, an optional capture check (R-CAP-9), a test submission, the game localization file, and the optional AI;
    - a step indicator;
    - the disclaimer.
14. **Settings forms:**
    - grouped sections, including: theme (incl. high contrast), density, reduce motion (tri-state), text size, test mode, thresholds, folders, AI and hosts, and colour overrides (§5);
    - validation messages.
15. **Empty, error and offline states:**
    - no reference data yet (R-UI-13);
    - offline with cached data;
    - folder missing;
    - Ollama not installed, shown in settings and never as a startup error;
    - not a terminal;
    - nothing to review;
    - capture-check results (R-CAP-9);
    - the connection-test results (R-UI-15).
16. **Supporting elements:**
    - tooltips (supplementary only, §6), menus and context menus;
    - scrollbars and split panes;
    - a diagnostics/log viewer in a monospace font;
    - the About dialog with the disclaimer and the licences of the bundled fonts, icons and OCR models (from `NOTICE`).

## 9. Typography and icons

Check every licence, version and hash at the source, with exact versions.

- Fonts and icons are bundled: no runtime download, and no `@font-face` with http(s) URLs.
- Add the licence files, and record in `NOTICE` the name, version, source (commit or release tag), licence and the SHA-256 of each vendored file.
- **Licence policy:** CLAUDE.md currently allows Apache, MIT, BSD, EPL and LGPL. Fonts will need SIL OFL 1.1, and some icon sets would need ISC.
  - Ask the owner to approve adding them (D10), then update CLAUDE.md in the same commit.
  - GPL is not allowed, because the project licence is undecided.

### Fonts

**Roles:**

- display (sparingly);
- headings and labels;
- UI and body text;
- dense table text;
- **numerals** (price, SCU, Δ). This font must have **tabular digits by default**, because JavaFX cannot switch on `tnum` (J8);
- monospace (diagnostics, raw API bodies).

Aim for at most three families. The candidates are in **Appendix C**.

**Eligibility rules:**

- **Static files only.**
  - A family is eligible only if the upstream project or google/fonts ships static TTF/OTF files at a pinned tag or commit.
  - The one exception: the variable file's default instance (verified by SP-4) is the only weight needed.
  - JavaFX handles no variable axes and no WOFF2 (J5).
- **No modifications.**
  - Never generate instances or subsets yourself. Under the OFL, modification triggers the Reserved Font Name rules.
  - Fonts with a Reserved Font Name (Orbitron, Electrolize, Saira, Share, Plex) would have to be renamed.
- **Weights.** Weights other than regular and bold are addressed by their face or family name (J7). Check the internal names on Windows and Linux (SP-4).

**Legibility rules (hard):**

1. **Glyph confusability.** The numerals font must pass a review of these pairs:
   - 0/O/D, 1/l/I/7, 5/S, 8/B/3, 6/G, 2/Z;
   - rendered as a specimen at the smallest table size, in every theme;
   - the specimen goes into `accessibility-report.md`.

   Prefer a slashed or dotted zero and a distinct 1 and 7.
2. **Minimum sizes.** Propose minimums and have the owner confirm them at A2. Starting point:
   - no text below 0.85 em of the root, and none below 11 px at 100 %;
   - numerals and the commodity name at 1.0 em or larger.
3. **Weight.** Body, table and numeral text is Regular (400) or heavier, never Light or ExtraLight.
4. **Condensed and display faces** only for headings of 1.25 em or larger and for short labels.
5. **Row titles in dense lists** use a medium weight (if the family has one) and slightly muted primary text (O4).

**Forbidden fonts:**

- Univia Pro (commercial: MyFonts, Adobe Fonts);
- Agency FB (Font Bureau/Microsoft, not redistributable);
- any Fankit font, including Banu and Xi'an.

**Loading:** decide in the ADR between `@font-face` and `Font.loadFont`, and between jar resources and plain files in the app image. Bear the temp-file copy in mind (J6, S-24).

### Icons

**Requirements:**

- a free licence;
- a consistent grid;
- coverage of every semantic state, with distinct glyphs (§6);
- **no thin or light weights**: stroke or feature width of at least about 1.5 px at 100 %, checked at 16 px;
- a style that matches the new design (D6 picks the set and licence at A1; the style variant is confirmed at A2).

Fill-based paths work directly with Modena's pattern: a Region whose `-fx-shape` is coloured through `-fx-background-color` with a looked-up colour and sized in em. Stroke-based sets need one of three routes:

- outline conversion;
- an SVGPath with `-fx-stroke`;
- a Region with a transparent background and a border stroke. The stroke width may not scale with the shape (evaluate in SP-5).

The candidates are in **Appendix D**.

**Vendoring:**

- Vendor only the path data of the icons you need into a resource (e.g. `icons.css` with `.icon-xyz { -fx-shape: "…"; }`).
- Include the upstream LICENSE, and record the version, source URL, tarball or commit and SHA-256 in `NOTICE`.
- **Do not add npm to the build.** A small script that regenerates the subset from a pinned upstream archive is fine. Its output is a generated artefact (§5).

**Icon spec:**

- `states.md` maps each state to its icon.
- Sizes in em; colour through tokens.
- Icon-only buttons get `accessibleText` and a tooltip.

## 10. Deliverables and file locations

| Deliverable | Location |
|---|---|
| Principles, rationale, guardrails, how to use and extend the system, naming conventions, open points and assumptions | `docs/design-system/README.md` |
| Token tables: name, value per theme, purpose, declared pairings, contrast, provenance tag | `docs/design-system/tokens.md` |
| Component specs (§8) | `docs/design-system/components.md` |
| State language (§6): tone family, tokens, icon, label key, pseudo-class, combination matrix, differences from the snapshot | `docs/design-system/states.md` |
| Scripted results, manual protocol, glyph specimen, method, thresholds, date | `docs/design-system/accessibility-report.md` |
| Every source, with URL, access date, what was taken from it, confidence and verification status; blocked hosts; the original FAQ wording | `docs/design-system/references.md` |
| ADR "Design system", status "Proposed" until the owner accepts it. Covers decisions, options, consequences, spike results, generated artefacts and open points | `docs/adr/0003-design-system.md` |
| Machine-readable token file | proposed: `ui/design-tokens/tokens.json` (decide in the ADR) |
| JavaFX CSS: token sheets (dark, light, high contrast), component sheet, icons sheet; only JavaFX-supported CSS | `ui/src/main/resources/space/uexdatarunner/ui/theme/` (placeholder package, §0) |
| Fonts with OFL.txt per family; icon licence | below the theme folder, e.g. `…/theme/fonts/<family>/` and `…/theme/icons/LICENSE` (or as the ADR decides, see J6) |
| Check scripts (§7), the optional generator, the synthetic-imagery generator, and a README listing the commands | `tools/design-system/` |
| Spikes, including custom controls before M0 | `tools/design-system/spikes/<SP-n>/` |
| Preview gallery: every component in every state and every theme, plus the combination matrix | **If the build exists:** a dev-only launcher that is not packaged, e.g. a `tools/design-gallery` module that depends only on `ui` (not on adapters), with its own ResourceBundle. **If not:** a single-file `DesignGallery.java` runnable with a local JavaFX 27 SDK on JDK 25+ (`java --module-path <sdk>/lib --add-modules javafx.controls DesignGallery.java`). In both cases, document how screenshots are taken |
| Notices | `NOTICE` at the repository root (create it if missing) |

**Plan updates, in the same commit as the change they describe:**

- **02:**
  - §7: theming, theme files, high-contrast exception, pseudo-class set, font loading, and the views list with the Session overview;
  - §2: module tree, if you add tools or a gallery.
- **01:**
  - R-UI-7: high contrast, text size;
  - R-UI-10 and R-UI-11: if the hue words or the percent format change;
  - a note that screen-reader support is Windows-only;
  - new assumptions from A17 on (trademark line, FAQ wording, unverified RSI values).
- **09:**
  - §7: the "no literal colours" lint;
  - §11: the design-system checks in the definition of done.
- **10:** a measure for vendored fonts and icons.
- **04:** the spikes, placed in M0 or M1.
- **CLAUDE.md:**
  - add `docs/design-system/` to the required-reading table;
  - add fonts and icons, with versions, to the stack table;
  - extend the licence list once the owner approves.
- **README:** add the plan-table links. Change the disclaimer only after the owner approves the wording.
- **`.gitignore`:** entries for local reference material and `design-previews/`, if used.

## 11. Process and owner checkpoints

1. **Stage 1 – Read and verify.**
   - Work through §0 and §3.3.
   - Rebuild the state inventory (§6).
   - Run the spikes you can run.
   - Do not freeze any tokens.
2. **Checkpoint A1: one message, then STOP.** It contains:
   - a short summary of findings: what was verified, what was not, which hosts were blocked;
   - **the evidence request (Appendix E)**;
   - the decisions below, which need no reference evidence. For each: 2–3 concrete options, the trade-offs and your recommendation.

   | ID | Decision |
   |---|---|
   | D1 | Design-system name and token prefix, with conflict searches |
   | D6 | Icon set and licence (the style variant is confirmed at A2) |
   | D7 | Deviation hues: the R-UI-10 conflict (§6). Options with CVD simulations of provisional colours |
   | D8 | Defaults: dark theme independent of the OS; automatic HC as an exception (§5); compact density, with the crop-thumbnail height vs row-height trade-off; reduce-motion tri-state, and reduced motion while the game is running; a "follow OS" theme option; the percent format "+12%" vs "+12 %" |
   | D9 | Window chrome: native DECORATED, or HeaderBar/EXTENDED |
   | D10 | Policy: add OFL-1.1 (and ISC if needed) to the CLAUDE.md licence list; the list of generated artefacts and whether each may be committed; fetching a JDK 25+ and JavaFX jars into a scratch directory for spikes |
   | D11 | Disclaimer wording and placements; how the trademark line is handled until it is verified |
   | – | The Findings severity mapping (§6) |

3. **Stage 2 – Foundations** (see the Execution plan). Palette values stay PROVISIONAL.
4. **Checkpoint A2: once the evidence has arrived, then STOP.** For each decision: options, a rendered preview, the trade-offs and your recommendation.

   | ID | Decision |
   |---|---|
   | D2 | Primary accent hue (cyan, light blue or teal) and how it stays distinct from the tone families |
   | D3 | Surface palette and the luminance band (§5) |
   | D4 | Font pairing (heading/label, UI/body, numerals), shown on a realistic dense table with long, similar commodity names, prices, Δ badges and states, using invented sample data; minimum sizes |
   | D5 | Corner language and frame details, following the new design (§3.2) |
   | D6 | Icon style variant |

   **Previews** can be PNGs rendered by a JDK-only Java2D script with the candidate fonts and colours, a local HTML page, or gallery screenshots if JavaFX runs.
   - Say clearly when a preview only approximates JavaFX rendering.
   - Keep previews out of the repository unless the owner wants them committed (§5).
5. **If the owner is not available,** continue only with clearly marked, provisional choices that are cheap to change. Never make D2–D5 or other irreversible choices.
6. **Stage 3 – Build.** Then **Checkpoint B: STOP.**
   - Show the gallery or previews in every theme, the accessibility report so far, the **confusion review** (§2) and the app icon.
   - Collect feedback and iterate.
7. **Stage 4 – Finish.**
   - Run the CVD check and the manual protocol, update the plan docs and run the self-check (§13).
   - Commit only with the owner's go-ahead, in small, focused commits with English messages in the imperative mood. Stage only your own changes (§0).
   - If a build exists, `./gradlew check` must be green.

## 12. Acceptance criteria

- **States are complete.** Every state in the rebuilt inventory (§6) has:
  - a tone family and semantic tokens in every theme;
  - an icon that is distinct within its group;
  - an English ResourceBundle label key that is distinct within its group;
  - a pseudo-class or style class;
  - an example in `states.md` and in the gallery.

  Differences from the snapshot are listed.
- **The combination matrix is covered.** It is documented, shown in the gallery and covered by the contrast check.
- **The checks pass, and the results are documented.**
  - The contrast check passes for every declared pairing in dark, light and high contrast, including the luminance band.
  - The CVD check passes for the gated sets and the co-occurrence list listed in `states.md`.
  - The manual protocol has a recorded result (pass, fail, or "not run – reason") for every item.
  - Everything is in `accessibility-report.md`, with dates.
- **Table focus works.** Cell focus is visible, at least 2 px, solid, meets 3:1 on every deviation background and selection tint, and is never obscured during F8 navigation (SP-9 result or a "not run" reason).
- **Targets meet 2.5.8.** Rows and interactive in-cell controls are at least 24 px at 100 %, or there is an owner-approved non-conformance.
- **Essential information is never tooltip-only.**
- **The CSS follows the rules.**
  - Component CSS contains no literal colours, and the lint passes.
  - The CSS uses only properties from the JavaFX 27 CSS reference.
  - Every transition is guarded by reduced motion.
  - Theme, HC and reduced motion apply to all windows and popups.
- **Components are fully specified.** Every component in §8 has a spec covering all interaction states, keyboard and focus behaviour and accessibility properties, plus a gallery example.
- **Fonts and icons are clean.**
  - Licences and exact versions were checked at the source.
  - Only static files are used, with no self-made instances or subsets.
  - Licence files are present, and the SHA-256 is in `NOTICE`.
  - The legibility rules (§9) are met.
  - There are no runtime downloads.
  - No new Maven dependency was added without the owner's approval and the full supply-chain steps.
- **Brand and legal.**
  - No proprietary assets.
  - No forbidden name segments, and the lint enforces this.
  - The confusion checklist (§2) is met and signed off.
  - The disclaimer exists as bundle keys and meets 4.5:1.
  - The trademark line is neutral until it is verified.
  - The key field cannot be mistaken for an RSI login.
- **The documents are complete.** ADR-0003 and `references.md` meet §10, all unverified items are clearly marked, and every generated artefact is listed with its handling.
- **The plan stays consistent.** The plan documents are updated in the same commit as each change, no foreign uncommitted changes are included, and everything in the repository is in English.

## 13. Self-check before you finish

- [ ] Contrast, CVD, lint and font scripts were run, all self-tests pass, and the results are in the report.
- [ ] The manual protocol was run, or each item is marked "not run" with its reason.
- [ ] Every state has a tone family, tokens, a distinct icon, a distinct label key, a pseudo-class and a gallery example.
- [ ] Focus, selection, hover, confidence and deviation each use a distinct channel, the table-cell focus is drawn inside the cell, and the combination matrix is legible.
- [ ] Test mode vs production is visible in every view.
- [ ] Overlays are visible on orange/red and blue imagery and on washed-out captures, and the sample imagery is synthetic.
- [ ] No literal colours outside the token sheets, no `-fx-` names for our own tokens, no forbidden name segments.
- [ ] Dark, light and high-contrast themes define every token. HC applies automatically on Windows.
- [ ] Fonts and icons: licence, version, source and SHA-256 are in `NOTICE`; licence files are present; static files only; nothing is downloaded at runtime.
- [ ] There are no proprietary or Fankit assets, nothing was extracted from the game, launcher or website, and no blocked host was routed around.
- [ ] The disclaimer exists as bundle keys; the trademark line is neutral or verified; any README change waits for the owner.
- [ ] Every RSI/Spectrum observation is cited with its URL, access date, confidence and era (NEW/LEGACY), and unverified values are tagged.
- [ ] Spike results are recorded, or marked "open – not run" with the reason.
- [ ] The plan docs (02 §7, 01, 09, 10, 04, CLAUDE.md, README, `.gitignore`) are updated wherever the design system changes them.
- [ ] Everything in the repository is in English; UI texts come only from ResourceBundles.
- [ ] Reference screenshots, DevTools dumps and previews are not committed unless the owner wants them.

## 14. Honesty rules

- Never present a guessed value as RSI's or Spectrum's. Tag where every value comes from, and whether it is NEW or LEGACY.
- Never claim a spike, contrast, CVD, HiDPI or screen-reader result that you did not actually compute or run. Write "not run" and give the reason.
- Record unverified items as open points in the ADR and in `docs/design-system/README.md`. Record requirement-level assumptions as A17 onward in `01-requirements.md`.
- If the design conflicts with a requirement or an architecture decision, ask the owner and update the documents in the same commit. Examples: the R-UI-10 hue words, the R-UI-11 percent format, 02 §7 "independent of the OS".
- End with a short report:
  - what was delivered;
  - what was verified, and how;
  - what is still an assumption;
  - what the owner still needs to decide or provide.

Start with the Execution plan, Stage 1.

---

## Appendix A: RSI and Spectrum observations (leads – re-verify)

"Era" says which design an observation describes: **NEW** is the 2025/2026 redesign, **LEGACY** is earlier.

| ID | Era | Observation | Source | Confidence | Suggests for us |
|---|---|---|---|---|---|
| O1 | NEW | RSI overhauled its website in stages in 2025. The navigation bar and account dashboard came first. Then came a Pledge Store with "a new, cleaner interface and simplified navigation", and ship pages restructured "to better highlight its role, focus, and what's included in the pledge". The same approach was reportedly applied to Comm-Link, Issue Council, Community Hub and the RSI Launcher. The announcement is dated about 7 May 2025, but sources disagree on the date. | https://robertsspaceindustries.com/spectrum/community/SC/forum/1/thread/pledge-store-refresh-rolling-out (search snippets only); https://x.com/TheRubenSaurus/status/1920153122560766179 | medium | Cleaner, simpler navigation; detail pages that lead with role and focus |
| O2 | NEW | Spectrum Redesign Phase 1, reportedly live around 22 July 2026 (date from search metadata). Described as "a cleaner, more modern experience designed to make Spectrum easier to navigate, easier to read". Changes: an updated layout; a redesigned sidebar (communities, bookmarks, navigation); Direct Messages in their own panel, opened from the top navigation bar; a mobile navigation bar. | https://robertsspaceindustries.com/spectrum/community/SC/forum/1/thread/spectrum-redesign-phase-1-now-live (search snippets) | medium. One researcher found no official confirmation | Left sidebar plus a top bar that opens slide-in panels |
| O3 | NEW | Phase 2, reportedly around 8–12 Sept 2026. The Thread List page got "a new, more linear design, making it easier to scan, browse, and find the topics you care about". Next on the roadmap: the mini profile, then threads and messages, then Markdown rebuilt from scratch. | https://robertsspaceindustries.com/spectrum/community/SC/forum/1/thread/spectrum-redesign-phase-2-now-live ; https://robertsspaceindustries.com/en/community/devtracker (search snippets) | medium | Linear, scannable lists |
| O4 | NEW | After Phase 2, CIG reportedly acknowledged three problems. (a) Thread titles were too bright and bold; a fix was planned. (b) The sort order reset when the page was reopened (a bug). (c) The new-reply counter had been removed and was "coming back shortly". | DevTracker (search summaries) | medium-low | Medium weight and slightly muted titles in dense lists; keep counters; save sort order per user |
| O5 | NEW | User feedback on the new sidebar: collapsing it hides it completely. Suggested fixes: move rarely used areas into a menu, and open the sidebar as a floating overlay on hover so the page does not reflow. | https://robertsspaceindustries.com/spectrum/community/SC/forum/5/thread/the-new-spectrum-is-cool-and-all-but (search summary) | medium | Collapsible rail with an overlay (hover and focus); no reflow |
| O6 | mixed | Thread-list rows show inline meta such as "Replies: 405 ● Views: 12122 ● Votes: 689"; pinned threads sit on top; there is a "Sort by" control (cached snapshot). LEGACY (Spectrum 0.3.5, 2017): flags for threads with staff posts, a yellow unread marker, per-user sort and view settings; 0.3.6 added buttons that jump to staff posts. Unknown whether these survive the redesign. | https://robertsspaceindustries.com/spectrum/community/SC/forum/3?page=1&sort=newest (cached snapshot); https://robertsspaceindustries.com/en/comm-link/transmission/15957-Spectrum-Alpha-035-Live ; https://starcitizen.tools/Spectrum_Alpha_0.3.6 | medium | Row markers, an "unreviewed" marker, jump navigation |
| O7 | LEGACY / unknown | Spectrum has offered light and dark themes since Alpha 0.3 (Feb 2017). Third-party extensions suggest both still existed in 2025. Unknown after the 2026 redesign. | https://robertsspaceindustries.com/en/comm-link/transmission/15741-Spectrum-Alpha-Is-Live ; https://greasyfork.org/scripts/559897-auto-dark-mode ; Chrome Web Store "Star Citizen - Better dark RSI webstyle" (search snippets) | medium | Dark first, plus a real light theme |
| O8 | LEGACY (Dec 2024) | Pre-redesign Spectrum CSS, seen through a third-party userstyle. The dark theme is a class on the app root (`#app.theme-dark`). Tokens are CSS custom properties: a primary ramp `--color-primary-400…900`, layered surfaces `--theme-bg-color-N`, `--theme-text-color`, `--color-alert-unread`. No hex values are visible. | https://github.com/33kk/uso-archive , file `data/usercss/177505.user.css` | medium | Two-layer tokens; the Appendix E snippet can dump such properties |
| O9 | unknown | Notifications appear as temporary toasts and are collected in a bell panel with "Clear All" and per-item actions. Settings are grouped as Theme, Notifications, Reactions and emojis, Media, Date and Time, Blocked Members, and Forum settings. | https://support.robertsspaceindustries.com/hc/en-us/articles/115013325208-Spectrum-Forums-and-Chat-Lobbies (search snippets) | medium | Toasts plus a persistent panel; grouped settings |
| O10 | LEGACY | Badges and titles are labels with icons, shown next to a name. Presence statuses: Online, Away, Do Not Disturb, Invisible. No colours are documented. | https://starcitizen.tools/Titles ; https://robertsspaceindustries.com/en/comm-link/transmission/15741-Spectrum-Alpha-Is-Live | medium | Chips made of icon plus label; status dots only with text |
| O11 | unknown date | Brandfetch's automatically extracted, unclaimed profile of robertsspaceindustries.com lists #0A1D29 (very dark navy), #54ADF7 (light blue) and #A9B3BD (blue-grey). | https://brandfetch.com/robertsspaceindustries.com (search snippet) | low | Palette direction only |
| O12 | LEGACY (2015–2018) | The RSI hangar look, from the CSS of the HangarXPLOR extension:<br>• Electrolize for numbers (cyan #25e4ff) and small uppercase labels (#6c84a2).<br>• Navy rules #0b2031 and #162a3f; translucent panel rgba(12,19,25,0.9).<br>• Focused input: border #00B7CA, text #42C6E7, inset glow `inset 0 0 20px rgba(0,112,200,0.3)`.<br>• "On" indicator #00e7ff with a cyan glow.<br>Note: #6c84a2 on #0b2031 computes to about 4.3:1, which fails 4.5:1. | https://github.com/dolkensp/HangarXPLOR/blob/release/src/web_resources/HangarXPLOR.css (commit 2f49b1d) | medium | Only if the new design keeps it (§3.2) |
| O13 | LEGACY | Fonts:<br>• The 2013 RSI Fan Site Kit included Orbitron.<br>• Spectrum headlines used Orbitron in 2017 (per a userstyle comment).<br>• A 2019 fan-forum post claims Univia Pro (commercial) for body text, and Electrolize, Orbitron and Share Tech Mono for headings and UI.<br>• Wikipedia credits Agency FB for the Star Citizen logo.<br>The fonts of the current site are unknown. | https://robertsspaceindustries.com/en/comm-link/transmission/12995-Fan-Focus-Citizen-Card-Update-Fan-Site-Kit-amp-More ; uso-archive `data/usercss/141210.user.css` ; https://www.starcitizenfrance.fr/forum/viewtopic.php?t=2332 ; https://en.wikipedia.org/wiki/Agency_FB | low | Never Univia Pro or Agency FB; Orbitron/Electrolize only as accents |
| O14 | recent | RSI Launcher 2.x was described as a "fresh, modernized UI". Since 2.16 it shows a video background when a card is hovered. Version 2.0.3 added a reduce-motion option that stops the video backgrounds. | Launcher release-note threads on Spectrum (search snippets) | medium | Reduce motion as a first-class setting; no video |
| O15 | fan | Fan-made RSI-like palettes (e.g. the MIT-licensed Spectrum CIG Tracker) use accents #64c8ff and #1d9aeb, navy surfaces #1e3444 and #163d59, and the font Exo. These are fan choices. | https://github.com/f4sh/Spectrum-CIG-Tracker | low | Confirms that "RSI-like" means cyan/light blue on deep navy |

**Unknowns** about the current design:

- surface, text, accent and border values;
- fonts, sizes, weights, letter-spacing and use of uppercase;
- radii, and whether chamfers or brackets are still used;
- spacing and elevation;
- the icon set;
- motion timing;
- whether a light theme still exists;
- badge and status colours.

## Appendix B: JavaFX 27 facts (re-verify)

**Sources,** unless noted otherwise:

- the CSS reference at tag `27-ga`: https://github.com/openjdk/jfx/blob/27-ga/modules/javafx.graphics/src/main/docs/javafx/scene/doc-files/cssref.html ;
- the release notes `release-notes-22.md` … `release-notes-27.md` in https://github.com/openjdk/jfx/tree/master/doc-files ;
- the named source files at tag `27-ga`.

openjfx.io, bugs.openjdk.org and mail.openjdk.org were unreachable, so their copies were not read directly.

| # | Fact (as of 27-ga) | Confidence |
|---|---|---|
| J1 | JavaFX 27 is GA on Maven Central (`org.openjfx:javafx-controls:27`) and requires JDK 25 or newer. | high |
| J2 | There is no `var()`, `calc()`, `color-mix()` or `alpha()`. Only **looked-up colours** exist: define a name on `.root`, reference it elsewhere, override it per subtree.<br>• **Colour functions:** rgb/rgba, hsb/hsba, `derive()`, `ladder()`, linear-/radial-gradient, image-pattern, repeating-image-pattern.<br>• **Effect functions:** `dropshadow`, `innershadow`.<br>• **Easing functions:** a separate category.<br>JavaFX uses HSB, not HSL.<br>**`derive()` is non-linear** (`com.sun.javafx.util.Utils.deriveColor`):<br>• the requested shift is scaled by 0.6–1.6 depending on the base colour's perceived brightness;<br>• lightening also desaturates (`hsb[1] *= 1 − b`);<br>• darkening of colours with brightness < 0.2 is dampened by 0.6 (exactly our navy surfaces);<br>• hue is truncated to an int. | high (verified in Utils.java 27-ga) |
| J3 | Sizes, radii and fonts as lookups are **not documented**. `CssParser.parseSize` accepts an identifier, but a code comment says lookups are meant for paints. A 2024 openjfx-dev post reported that size lookups do not work. RFE JDK-8231646 appeared unresolved; its status is unverified. | medium |
| J4 | `@font-face` honours only `src`. The family name comes from the font file. `-fx-font-family` takes no fallback list. The font shorthand has no line-height. | high |
| J5 | The font loader handles TTF, OTF-CFF, TTC and WOFF 1.0. There is **no WOFF2** and no handling of variable-font axes (no `fvar` code), so bundle only static instances. | high (variable support absent by inference) |
| J6 | Fonts loaded from `jar:` URLs or streams are copied to a temp file `+JXF*.tmp` in `java.io.tmpdir` and deleted by a shutdown hook. Only `file:` URLs are used in place. Compare with 02 §8 and S-24. | high |
| J7 | `-fx-font-weight` can only pick the regular or the bold face (`PrismFontLoader`: `bold = weight >= BOLD`; JDK-8087799). Medium and semibold must be addressed by their own family or face name. | high |
| J8 | There is no letter-spacing, no text-transform, and no font-feature-settings or font-variant. This means **no `tnum`** and no CSS uppercase. | high |
| J9 | The default font size is a fixed 13 px on Linux; Windows reads it from the system font. Modena sizes most things in em. | high |
| J10 | Custom pseudo-classes: `PseudoClass.getPseudoClass(...)` plus `Node.pseudoClassStateChanged(...)`. Custom styleable properties: `CssMetaData`, `StyleableProperty`, `StyleablePropertyFactory`. | high |
| J11 | `-fx-shape` takes an SVG path; `-fx-scale-shape` (default true) and `-fx-position-shape` (default true) control it.<br>• The Region javadoc says background insets and radii are ignored when a shape is set.<br>• **The 27-ga implementation differs:** `NGRegion.renderBackgroundShape` ignores radii but honours background-fill insets and refits the shape into each inset box via `resizeShape()`. Border-stroke insets are honoured the same way. Modena relies on this: `.check-box:selected > .box > .mark { -fx-background-insets: 1 0 -1 0, 0; }` on a shaped node.<br>• Only the top border's width, colour and style are used for a shaped border.<br>• Border images are ignored. | high (verified in NGRegion.java and modena.css 27-ga) |
| J12 | Radii are round or elliptical only; there is no bevel or corner-shape. Routes to chamfers:<br>(a) `-fx-shape` with a polygon. Insets work (J11), but `resizeShape` scales non-uniformly: "Proportions are not maintained when resizing" (verified). The chamfer angle and leg length therefore change with the aspect ratio, and inset rings are rescaled copies.<br>(b) A custom Region with a styleable chamfer-size property that builds its own shape.<br>(c) A clip. It clips the content but cannot be stroked.<br>(d) A 9-slice border image. It cannot use colour tokens and is ignored when a shape is set. | high |
| J13 | `-fx-effect` in CSS supports only `dropshadow` and `innershadow`; Glow and Bloom are available only from code. Effects are costly on the software pipeline (`-Dprism.order=sw`, the fallback when no usable GPU is found). There is no current benchmark. | medium |
| J14 | CSS transitions exist since 23. Background and Border can be interpolated since 24. `linear()` easing exists since 26. Transitions do not run when a value is set from code or bound. | high |
| J15 | Transitions are **not** disabled automatically for reduced motion. In 27 only some built-in skins check `isReducedMotion()`: TitledPane, TabPane, Pagination and TableRow. ProgressBar and ProgressIndicator are not among them. | high |
| J16 | `Platform.getPreferences()` (22+) offers colorScheme and accent colours; `reducedMotion`, `reducedTransparency` and `reducedData` since 24.<br>• Windows keys include `Windows.SPI.HighContrast`, `Windows.SPI.HighContrastColorScheme` and `Windows.UIColor.*`. There is **no typed high-contrast property**.<br>• `Scene.getPreferences()` (25+) overrides per Scene and drives media queries. Each property falls back to the **platform**, not to the owner window. Whether dialog and popup scenes inherit an owner's override is **unverified** (SP-2).<br>• Since 26 the title bar follows the scene's colour scheme. | high (popup behaviour: unverified) |
| J17 | Media queries:<br>• 25: `prefers-color-scheme`, `prefers-reduced-motion`, `prefers-reduced-transparency`, `prefers-reduced-data`, `-fx-prefers-persistent-scrollbars`.<br>• 26: `width`, `height`, `aspect-ratio`, `orientation`, `display-mode`, with range syntax.<br>• 27: `-fx-supports-conditional-feature`, `-fx-platform`, and conditional `@import`. The cssref "Limitations" section still says the `@import` media list is not parsed, but the 27-ga code parses it.<br>There is no `prefers-contrast` or `forced-colors`. | high |
| J18 | Modena 27 has no dark variant and no `@media` rules. Windows high-contrast sheets are added only when the user-agent stylesheet is Modena, and author stylesheets override them. JDK-8336097 (fixed in 24) concerns user-agent lookups that are redefined in author sheets. | high |
| J19 | Pseudo-classes: `:focus-visible` and `:focus-within` (19+); `:first-child`, `:last-child`, `:only-child` and `:nth-child(even|odd)` (24+). There is no `:active` and no `:focus`. | high |
| J20 | Table hooks:<br>• Style classes: `.table-view`, `.table-row-cell`, `.table-cell`, the column-header substructure.<br>• Pseudo-classes: `:selected`, `:empty`, `:filled`, `:even`/`:odd`, `:last-visible`.<br>• Other: `-fx-fixed-cell-size`. There is no `:editing`.<br>The style classes of a TableColumn propagate to its cells. | high |
| J21 | HiDPI:<br>• Windows: per monitor; override with `-Dglass.win.uiScale`.<br>• Linux: one fractional scale per screen (Xft DPI / 96); override with `-Dglass.gtk.uiScale`.<br>• Scale fixes landed in 25–27.<br>• Raster images: `@2x`/`@3x` names, with a `@1x` fallback since 24. | high |
| J22 | Screen readers: Windows uses UI Automation. **Linux has no screen-reader bridge (no AT-SPI) in 27-ga.** There is no live-region, alert or status role and no announce API. | high |
| J23 | HeaderBar plus `StageStyle.EXTENDED` is final in 27. The app supplies the title and icon. On unsupported platforms it falls back to DECORATED. | high |
| J24 | **Table cell focus:**<br>• `Cell` sets its focus through `Node.setFocused(value)`, which calls `setFocusQuietly(value, false)`. Cells therefore get `:focused` but never `:focus-visible`.<br>• Modena draws the focused cell **inside** the cell: `-fx-background-insets: 0, 1, 2` under `.table-view:focused:cell-selection > … > .table-row-cell > .table-cell:focused`.<br>• In row-selection mode the row gets `:focused` instead. | high (verified in Node.java, Cell.java and modena.css 27-ga) |
| J25 | `Tooltip` has a default `showDuration` of 5000 ms and is triggered by mouse hover. No keyboard-focus trigger was found. | medium (keyboard behaviour: unverified) |

## Appendix C: Font candidates (re-verify every entry)

**Data sources:**

- google/fonts `main`, METADATA.pb and OFL.txt, read 2026-10-08, plus TTF name/head tables;
- upstream repositories as noted;
- digit widths measured from cmap and hmtx.

All fonts are SIL OFL 1.1.

| Family | Version | Files available | Reserved Font Name | Default digits | Possible role |
|---|---|---|---|---|---|
| Orbitron | 2.001 | **variable only on Google Fonts** (`Orbitron[wght].ttf`, 400–900); check upstream for static files | "Orbitron" | proportional | Display accent only, the strongest RSI association; eligible only with static files or the default instance alone |
| Electrolize | 1.002 | single static weight | "Electrolize" | proportional | Labels and accents (LEGACY look) |
| Rajdhani | 1.201 | static 300–700 | – | proportional | Headings and labels (squared, condensed) |
| Saira Condensed / Semi Condensed | 0.072 (upstream variable v1.001) | static 100–900 | "Saira" | proportional | Headings and labels |
| Chakra Petch | 1.000 | static 300–700 plus italics | – | proportional | Headings and labels |
| Oxanium | 2.000 | variable on Google Fonts (default instance ExtraLight); **static TTFs upstream** in `sevmeyer/oxanium` branch `master`, `fonts/ttf/Oxanium-{ExtraLight…ExtraBold}.ttf`; pin a tag or commit | – | **tabular** | Numerals (Regular or heavier) |
| Barlow / Barlow Semi Condensed | 1.408 | static 100–900 | – | proportional | UI and body |
| Inter | 4.001 on Google Fonts (upstream v4.1) | variable (opsz, wght) on Google Fonts; check whether the upstream release ships static TTFs | – | proportional | UI and body, if static files exist |
| Titillium Web | 1.002 | static 200/300/400/600/700/900 plus italics; **no 500 (Medium)** | – | **tabular** | UI and numerals; the medium-weight rule can only be met via SemiBold |
| IBM Plex Mono | 2.3 on Google Fonts (npm `@ibm/plex-mono` 2.5.0) | static | **"Plex"** (google/fonts `ofl/ibmplexmono/OFL.txt`) | tabular (mono) | Numerals and diagnostics |
| JetBrains Mono | 2.211 on Google Fonts (upstream v2.304) | **variable on Google Fonts** (`JetBrainsMono[wght].ttf`, 100–800); static files only from the upstream release, verify the asset | – | tabular (mono) | Diagnostics and numerals |
| Share Tech Mono | 1.003 | single weight | "Share" | tabular (mono) | HUD accent only |
| Exo 2 | 2.010 | **variable on Google Fonts**; static instances needed | check | proportional | Alternative |
| Michroma (1.100, Eurostile-like), Tomorrow (2.002), Bai Jamjuree (1.000) | – | check | check | proportional | Alternatives or accents |

**Sources:**

- https://github.com/google/fonts/tree/main/ofl/<family>/
- https://github.com/sevmeyer/oxanium
- https://github.com/rsms/inter/releases
- https://github.com/JetBrains/JetBrainsMono/releases
- https://github.com/Omnibus-Type/Saira
- https://github.com/jpt/barlow
- https://registry.npmjs.org/@ibm/plex-mono

## Appendix D: Icon candidates (re-verify every entry)

npm data was queried on 2026-10-08, with licences read from the tarballs.

- **Material Symbols** (`@material-symbols/svg-400` 0.47.6, 2026-10-02, Apache-2.0).
  - A repackaging of google/material-design-icons, in outlined, rounded and sharp styles, 7,854 icons each.
  - viewBox `0 -960 960 960`; each icon is a single fill path.
  - **Sharp** suits an angular look, **rounded** a softer one. D6 decides after the evidence arrives.
- **Phosphor** (`@phosphor-icons/core` 2.1.1, 2024-03-29, MIT).
  - Fill-based, viewBox 256, 1,512 icons.
  - Use the **regular or bold** weights only; thin and light fail the minimum-stroke rule (§9).
- **IBM Carbon** (`@carbon/icons` 11.90.0, 2026-10-07, Apache-2.0).
  - Fill-based, often with several paths per icon, which must be concatenated.
  - The package ships a `telemetry.yml`, so vendor only the path data.
- **Stroke-based sets:** Tabler 3.49.0 (MIT) and Lucide (`lucide-static` 1.53.0, ISC). See §9 for the three routes; evaluate in SP-5.
- **Avoid:**
  - Remix Icon 4.9.1: a custom "Remix Icon License v1.0" since Jan 2026, although its package.json still says Apache-2.0.
  - `@mdi/svg` 7.4.47: mixed licences, last release 2023-12.
- **Ikonli** 12.4.0 (`org.kordamp.ikonli`, Maven Central, Apache-2.0, JPMS):
  - an icon-font alternative whose packs lag behind upstream;
  - it would be a new dependency, needing the lockfile, verification metadata and a justification;
  - consider it only if vendoring does not work out.

## Appendix E: Evidence request for the owner (paste into Checkpoint A1)

**Before sharing anything, crop or redact:**

- handles and avatars;
- e-mail addresses;
- pledges and store credit;
- balances.

Nothing you send will be committed to the repository.

**1. Screenshots** at 100 % browser zoom, in the dark theme and also the light theme if one is offered:

- the RSI home page, the pledge store, a ship page, and the account dashboard (logged in);
- the Spectrum community page with its sidebar, a thread list, a thread, the DM panel, the notification panel and Spectrum's settings.

**2. DevTools values.**

- In the Elements panel, select an element (it becomes `$0`), then paste the snippet below into the Console. It copies a JSON summary to the clipboard; send it along.
- Check the `page` field for personal data before sending.
- Please do this for these elements:
  - page, sidebar and panel/card backgrounds;
  - a list row: default, hover and selected;
  - a divider;
  - primary, secondary and muted text;
  - a link;
  - the unread/notification badge;
  - a tag chip;
  - an input field: default and focused;
  - the primary button: default, hover and focus;
  - the page heading, a section heading, a navigation item, a thread title, meta text and a small label.
- One run also dumps all CSS custom properties (`--*`) defined in readable stylesheets. That may give the whole palette at once.

The snippet below is untested in a browser; it has only been syntax-checked. If it fails, send the values from the "Computed" tab instead, and the font file names from the Network tab (filter: Font).

```js
// Select an element in the Elements panel first ($0), then paste this into the Console.
// It reads styles only (no page text) and copies a JSON summary to the clipboard.
(() => {
  const el = (typeof $0 !== 'undefined' && $0) || document.body;
  const cs = getComputedStyle(el);
  const props = ['color', 'background-color', 'background-image', 'opacity',
    'border-top-color', 'border-top-width', 'border-top-left-radius', 'box-shadow',
    'outline-color', 'outline-width', 'outline-offset', 'font-family', 'font-size',
    'font-weight', 'line-height', 'letter-spacing', 'text-transform',
    'font-variant-numeric', 'transition-duration', 'transition-timing-function'];
  const computed = Object.fromEntries(props.map(p => [p, cs.getPropertyValue(p)]));
  const names = new Set();
  const collect = style => {
    for (let i = 0; i < style.length; i++) if (style[i].startsWith('--')) names.add(style[i]);
  };
  collect(getComputedStyle(document.documentElement));
  const walk = list => {
    for (const r of list) { if (r.style) collect(r.style); if (r.cssRules) walk(r.cssRules); }
  };
  for (const sheet of document.styleSheets) {
    try { walk(sheet.cssRules); } catch (e) { /* cross-origin sheet: not readable */ }
  }
  const customProperties = {};
  for (const node of [document.documentElement, document.body, el.closest('[class*="theme"]')]) {
    if (!node) continue;
    const label = node.tagName.toLowerCase() + (node.id ? '#' + node.id : '');
    const ncs = getComputedStyle(node);
    for (const n of names) {
      const v = ncs.getPropertyValue(n).trim();
      if (v) customProperties[label + ' ' + n] = v;
    }
  }
  const out = {
    page: location.hostname + location.pathname,
    element: el.tagName.toLowerCase(),
    classes: String(el.getAttribute('class') || '').slice(0, 200),
    computed,
    customProperties,
  };
  copy(JSON.stringify(out, null, 2));
  return out;
})();
```

**3. Optional, if you have time:**

- the transition durations you notice on hover and focus;
- whether the redesigned pages still use any angled/chamfered corners, corner brackets or glows;
- whether Spectrum still offers a light theme.
