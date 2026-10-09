# ADR-0004: Design system Tallyline: DTCG tokens, own JavaFX theme replacing Modena

> **Doc type:** Decision record — Proposed. Last reviewed: 2026-10-09.

- **Status:** Proposed (2026-10-09); accepted at the end of the JavaFX handoff (stage 4) by @greluc
- **Related:** [`docs/prompts/design-system.md`](../prompts/design-system.md) (brief), [`design-system-update-2026-10.md`](../prompts/design-system-update-2026-10.md) (update brief), [`design-system-update-2-2026-10.md`](../prompts/design-system-update-2-2026-10.md) (correction brief), [`design-system-javafx-handoff.md`](../prompts/design-system-javafx-handoff.md) (handoff), [`docs/design-system/`](../design-system/), register O-83, O-84, O-105, O-106; 01 R-UI-1…19; 02 §7, §8

## Context

The app needs one visual language for a dense, keyboard-first review tool that must never submit anything wrong silently: every value quick to compare, every state readable by colour, icon and text (R-UI-4, R-UI-10…12, R-UI-17). The owner had the design system built in Claude Design ("Tallyline", Design System version `1791542897-a5d3`, screens canvas `1791541917-46e6`, 2026-10-09). Claude Design produces web previews and a token file in its own list grammar; it does not write JavaFX CSS and does not touch the repository. JavaFX 27 CSS is not web CSS: no `var()`, `calc()` or colour mixing, looked-up colours only, no letter spacing or text transform, one effect per node, font weights regular and bold only (handoff §4, re-verified 2026-10-09).

## Decisions

### 1. The owner's design decisions (O-83)

D1 name Tallyline (provisional; trademark search open) with the token prefix `-tl-`; D2 accent sky `#7cc3ef`; D3 lifted slate-blue ground (option C) – **deviates from brief §4.1, by the owner's choice**; D4 IBM Plex Sans with JetBrains Mono NL – **deviates from brief §6.3, by the owner's choice**; D5 small radius plus one focal-frame accent, no chamfers; D6 Material Symbols Rounded, FILL 1, wght 400, GRAD 0, opsz 48; D7 deviation hues option 2 (minor yellow, major orange, no reference neutral with a dashed border); D8 comfortable 36 px rows by default, middle ellipsis for long names, reduced motion forced while the game runs, theme follows the OS, percent as "+12%"; D9 integrated header bar with the native title bar as fallback; D11 layout B for the About dialog and the onboarding start screen; the findings severity mapping; row selection as tint plus leading bar; app-icon concept (c) "Line"; the state-language proposals, including "Accept all confident" for minor deviations (R-UI-5); the review table without a crop column (R-UI-3, amended 2026-10-09).

### 2. Token schema: DTCG 2025.10 with the Resolver Module

The repository's token files are in the Design Tokens Community Group format, version 2025.10 (Format, Color and Resolver Modules; Final Community Group Reports of 28 October 2025): a resolver document with a base set and a `theme` modifier for `dark`, `light` and `high-contrast`, colours as sRGB objects with `hex`, our own data under `$extensions` `space.uexdatarunner`. The declared pairings and the state ids, which DTCG has no type for, are in `pairings.json` and `states.json` beside them. Details: [tokens.md](../design-system/tokens.md).

- **Why not Claude Design's grammar:** it is a tool-specific list format; DTCG is a published, tool-neutral format with a theme mechanism (the resolver), so other tools can read the tokens and the format is not ours to maintain.
- **Why not a single DTCG file with theme values in `$extensions`:** tools would see only one theme.
- **Cost:** the resolver splits the tokens into four files, and our scripts implement only the subset we use (one base set, one modifier, one file per context).

### 3. Modena is replaced

The app does not keep JavaFX's Modena user-agent sheet and override its lookups; it ships its own complete user-agent sheet built from the tokens (decided by @greluc at checkpoint A, 2026-10-09). Every stock control the app uses is styled by our sheet; nothing relies on Modena's derived colours, so the contrast check needs no reproduction of `derive()` or `ladder()`. The minimum scope of the replacement is the result of SP-3 (see "Spike results").

### 4. Themes and high contrast

Three themes: dark (default), light, high contrast. On Windows the high-contrast theme follows the system setting (`Platform.getPreferences()` `Windows.SPI.HighContrast`, watched for changes) while the theme setting is "Follow OS" or has never been changed; an explicitly chosen Dark or Light theme wins (decided 2026-10-09, R-UI-17, 02 §7). On Linux the user chooses it. Every window and popup (dialogs, alerts, tooltips, context menus, combo-box popups, popups) gets the same theme. The theme is applied by mechanism A: one token sheet per theme, swapped on every window through a `Window.getWindows()` listener, plus the scene's colour scheme and reduced-motion preference set on every window (SP-2; decided 2026-10-09 by @greluc at checkpoint B).

### 5. Fonts and icons (O-84)

The fonts are the static TTF files of IBM Plex Sans (Regular, SemiBold, Bold) and JetBrains Mono NL (Regular, Bold) under SIL OFL-1.1, vendored unmodified in stage 3 from pinned upstream sources and compared with the files the system holds (SHA-256 in the system's final reply). JavaFX picks only regular or bold by weight (J7), so the SemiBold face is reached through its own family name. Icons are the path data of the needed Material Symbols glyphs (Apache-2.0), generated from a pinned `@material-symbols/svg-400` archive into `icons.css`; no icon font. Fonts are loaded by `file:` URL from plain files in the app image, so nothing is copied to a temp file (SP-4, SP-8; the jlink image is checked in M0).

### 6. Scripts, not a build step, until M0

Until the Gradle build exists, the conversion, validation and contrast checks are JDK-only programs in `tools/design-system/`, run with the source launcher. Each entry file is self-contained apart from the shared `Json.java` and `Tokens.java` in the same directory (multi-file source programs, JEP 458); this deviates from the handoff's "single-file" wording only in sharing those two helpers. Once the build exists, the token sheets are generated by the build from the token files.

### 7. Generated artefacts

None is committed: the token sheets and `icons.css` will be generated by the build once it exists; previews and screenshots go to a scratch directory or the gitignored `design-previews/`. The only generated content in the repository is the contrast section of `accessibility-report.md`, written by `Contrast.java`.

## Alternatives considered

On 2026-10-09 @greluc asked whether another Java desktop toolkit would fit better, even without Linux. The check used current primary sources: Maven Central metadata, the toolkits' source code and official documentation, and accessibility conformance reports.

**Result:** JavaFX 27 is the only option that meets every hard requirement. Dropping Linux does not make any alternative eligible.

| Option (version, date) | Why not |
|---|---|
| Swing + FlatLaf (`com.formdev:flatlaf` 3.7.2, 2026-07-09, Apache-2.0) | Best theming fidelity and a table that natively combines a focused cell with a selected row. But on Windows screen readers work only through the Java Access Bridge: NVDA and JAWS read it, **Narrator does not** (JDK-8051722; Oracle's JDK accessibility report names the Access Bridge as the only path) |
| Compose Multiplatform for Desktop (1.12.1, 2026-09-22, Apache-2.0) | Screen readers only through the Java Access Bridge, so **no Narrator** (JetBrains' desktop accessibility documentation); its jars have no module names, so no jlink image; no first-party data grid; adds Kotlin to the stack, and Kotlin's support for JVM target 27 is unverified |
| SWT (3.135.0, 2026-09-07) | EPL-2.0 without a designated Secondary License, which the FSF lists as incompatible with the GPL; automatic module only; native widgets cannot carry the design; MSAA/IAccessible2, no UIA provider |
| Embedded web UI (JCEF `me.friwi:jcefmaven` 152.0.6; WebView2 through SWT; JavaFX WebView) | It would reproduce the design 1:1, but each route breaks a rule. JCEF downloads its natives at first run by default, or adds 171 MB, and lags Chrome by four major versions. WebView2 from Java is maintained only through SWT (licence). JavaFX WebView has no screen-reader access to web content (JDK-8177382). Every route also needs a second JavaScript code base and a JSON bridge |
| Qt Jambi (6.12.0, 2026-10-01, LGPL-2.1/GPL-3 bindings on Qt LGPL-3.0) | Native UIA, high contrast and expressive QML theming, but Qt's own binaries are not on Maven Central (they come from the Qt installer), and the bindings depend on essentially one maintainer |
| Native Windows UI (WinUI4K; Win32 through FFM) | WinUI4K is a research prototype that downloads the Windows App SDK at start-up; Win32 owner-draw would mean writing a toolkit |
| Dear ImGui (`imgui-java` 1.92.7.1) | No screen-reader support |

**What stays open for JavaFX** (M0 spike in 04):
- Narrator is covered by Oracle's JavaFX 26 accessibility report, which tested Narrator and JAWS. **NVDA through JavaFX's UIA provider is unverified.** Both are checked on the review table.
- JavaFX reads no Windows text-size setting (none in the jfx27 preference map), so the app's own text-size setting (R-UI-7) stays necessary.

## Spike results

Run on 2026-10-09 with Temurin 27+35 and the JavaFX 27 `win` jars on Windows 11 (3840 × 2160, scale 1.0, platform colour scheme dark). The spike sources are in `tools/design-system/spikes/`; their logs and screenshots stayed in a scratch directory.

| Spike | Result | Consequence |
|---|---|---|
| SP-1 lookups | **Partly.** Single-value size lookups work (`-fx-spacing`, `-fx-pref-width`, `-fx-min-height`, `-fx-graphic-text-gap`, `-fx-fixed-cell-size`), and so do tooltip durations, including `indefinite`. They fail with a `ClassCastException` for every insets-, radius- and border-width-typed property (`-fx-padding`, `-fx-label-padding`, `-fx-background-radius`, `-fx-background-insets`, `-fx-border-width`, `-fx-border-radius`), and for `transition-duration`; the property keeps its default. They are a parse error in `-fx-font-size` and the `transition` shorthand. An `em` length resolves against the font of the node that uses it | Colours are looked-up colours; sizes, radii, border widths, font sizes and durations are written as **literal values** into the generated sheets; density is a style class with literal values, not a lookup swap |
| SP-2 theme switching | **A works for every window; B and C do not.** Swapping one sheet per theme on every window (`Window.getWindows()` listener) restyled every open window and popup. With `prefers-color-scheme`, popup controls evaluate media queries against their owner scene, but a plain `Popup` and dialogs use their own scene, which falls back to the platform, and open popups did not restyle reliably. Only the scene's colour scheme changes the title bar and the HeaderBar buttons. `Windows.SPI.HighContrast` is readable and a change listener registers; switching the OS setting itself was not tried | **Mechanism A**, plus `Scene.getPreferences().setColorScheme(…)` and `setReducedMotion(…)` on every window for the title bar, the HeaderBar and the motion guard. Hide open popups before `Platform.exit()` (one native crash with popups open) |
| SP-3 lookups and Modena replacement | **Confirmed with findings.** `-tl-*` lookups work in Modena-mapped and own rules. An unprefixed lookup such as `red` overrides the named colour, so the prefix is required; lookup names are case-insensitive; a missing lookup only logs a warning. An empty user-agent sheet leaves controls unusable (0 px scroll bars, invisible check marks, transparent popups); a minimal own sheet of about 220 lines made the tested controls usable (Modena 27: 3,556 lines) | Our user-agent sheet must cover at least `.root`, labels, buttons, text fields, combo boxes and their popups, scroll bars, table view (headers, rows, cells), tooltips, context menus and check boxes; the rest of the inventory is specified in `components.md`. A theme test fails on any "while converting value" CSS warning |
| SP-4 fonts | **Confirmed.** Family names: "IBM Plex Sans" (Regular and Bold), "IBM Plex Sans SmBld" (SemiBold, its own family), "JetBrains Mono NL" (Regular and Bold). Weights 100–600 render regular, 700 and above bold; weight 600 needs the SmBld family. A family list is no fallback. IBM Plex Sans lacks ● ▲ ▼ ⚠; JetBrains Mono NL lacks the thin space. Fonts loaded by `file:` URL leave no temp copy; streams, `jar:` URLs and `@font-face` with a `jar:` source each copy once (deleted on a normal exit). `Font.getDefault()` is 12 px here; the Windows text-size slider was not tried; the jlink part was not run | Load fonts by `file:` URL from plain files in the app image (02 §8, S-24); map weight 600 to "IBM Plex Sans SmBld"; draw ● ▲ ▼ ⚠ as icons |
| SP-5 icons | Open – not run: the `@material-symbols/svg-400` package is not fetched yet (stage 3, download needs the owner's approval). Checked without the package on 2026-10-09: all 156 names in the "Icons used" table of version `1791542897-a5d3`, and every icon name in its previews, boards and state language, exist as `rounded/<name>-fill.svg` in the file listing of `@material-symbols/svg-400@0.47.6` (jsDelivr package API) | — |
| SP-6 chamfers | Not needed: D5 has no chamfers | — |
| SP-7 200-row table | **Confirmed.** Hardware pipeline: about 142 frames per second with or without effects. Software pipeline: 60 frames per second with one effect on the focused cell, 29.5 with an effect on every cell carrying a state. Recycled cells do not animate their state on scroll; with reduced motion set on the scene, no transition ran | One effect at most (the focused cell); confidence and deviation stay borders and backgrounds |
| SP-8 JPMS | **Confirmed.** `getResource(…).toExternalForm()` loads a module's stylesheet, closed or open, exploded or in a jar; a relative `@font-face` source resolves; a bare path string fails in a closed module. Fonts inside a module jar are always temp-copied. The jlink image (`jrt:` URLs) was not run | Load sheets through `Class.getResource`; keep font files outside the module jar |
| SP-9 cell state stacking | **Confirmed, with one structural finding.** The no-reference dash, the confidence border and the focus ring stack as three border strokes inside the cell, with the 1 px gap showing the deviation background, at 100, 125, 150 and 250 %. Selection (tint plus 4 px bar) and hover never hide a deviation background. A middle ellipsis works at a fixed 36 px; a CSS escape such as `6` is not decoded. `scrollTo` never puts a row under the sticky header. In cell-selection mode the focused cell gets `:focused` (never `:focus-visible`) but rows never get `:selected`; in row-selection mode the row gets `:selected` but no cell gets `:focused` | Cell-selection mode plus the row pseudo-class `:row-selected` driven by the selection model (02 §7); write "…" literally in the sheet |

## Consequences

- The plan's requirements R-UI-3, R-UI-5, R-UI-11 and R-UI-17 and 02 §7 were amended with the owner's decisions of 2026-10-09; "unreviewed" entered 11 §A1.
- The pseudo-class set of 02 §7 gained `:double-confirmed`, `:confirmation-suspended`, `:deviation-worsened`, `:ai-hint`, `:not-sent` and the row classes `:unreviewed`, `:newer-observation-sent` and `:row-selected`; `:needs-confirmation` now shows every case that blocks release (decided 2026-10-09 by @greluc, checkpoint B; [states.md](../design-system/states.md)).
- The sixteen design defects of version `1791537441-68f0` were fixed by the correction brief in version `1791541897-0513`, and the two found in that version were fixed in `1791542897-a5d3` ([states.md](../design-system/states.md), "Design defects"). The crop thumbnails are retired: the DetailStrip and the deviation-confirmation dialog show the readable crop (R-UI-3, [tokens.md](../design-system/tokens.md)).

## Unverified

- The minimum feature width of the icons at 16 px (not measured yet).
- Claude Design's contrast and colour-vision figures are indicative; the authoritative contrast run on the token values passed (2026-10-09); the colour-vision run and the checks on the rendered JavaFX sheets are open.
