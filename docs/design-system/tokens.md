# Design tokens

> **Doc type:** Living spec — draft (JavaFX handoff stage 2, ADR-0004 Proposed). Last reviewed: 2026-10-09.

The design tokens of the app's design system, Tallyline, live in [`ui/design-tokens/`](../../ui/design-tokens/). They were converted from the Claude Design system Tallyline, version `1791537441-68f0` (2026-10-09), by [`tools/design-system/ConvertTokens.java`](../../tools/design-system/ConvertTokens.java). The token files are the repository's source of truth; the Claude Design artifact is a reference and is never read at build or run time. Why this schema and these rules: [ADR-0004](../adr/0004-design-system.md).

## Files

| File | Contents |
|---|---|
| `tokens.resolver.json` | The [DTCG Resolver Module 2025.10](https://www.designtokens.org/TR/2025.10/resolver/) document: one set `base` and one modifier `theme` with the contexts `dark` (default), `light` and `high-contrast` |
| `base.tokens.json` | Theme-independent tokens in the [DTCG Format Module 2025.10](https://www.designtokens.org/TR/2025.10/format/): `font-face`, `font-family`, `typography` (20 styles), `spacing` (10), `radius` (3), `width` (6), `size` (18), `motion` (2 durations, 1 easing) |
| `theme-<id>.tokens.json` | Per theme: `color` (99 tokens, [DTCG Color Module 2025.10](https://www.designtokens.org/TR/2025.10/color/), sRGB with `hex`) and `shadow` (3) |
| `pairings.json` | The declared foreground/ground pairs with their required ratio per theme (575) and the co-occurrence list, from the system's "Accessibility pairings" section |
| `states.json` | The state ids with label, icon, tokens and accessible text (171), from the system's "State language" section; their mapping is [states.md](states.md) |

The three DTCG modules are Final Community Group Reports of 28 October 2025 (checked on designtokens.org on 2026-10-09). Our own data sits under the `$extensions` key `space.uexdatarunner` (placeholder until the package name is decided, O-8).

## Conversion rules

- **Colours:** every value is either an alias `{color.<name>}` or an sRGB object with `components` (0–1), `alpha` and the six-digit `hex`. Only `scrim` is translucent, and only in dark and light; high contrast has no translucent colour.
- **Shadows:** single layer, zero spread. A shadow that is `none` in a theme is kept as a fully transparent shadow marked `"none": true` in `$extensions`, so every theme defines every token.
- **Typography:** DTCG `typography` with `letterSpacing` 0 and a unitless `lineHeight` (line height ÷ font size); the line height in px is kept in `$extensions` as `lineHeightPx`. Weights are 400, 600 and 700; 600 exists only as its own face (see "JavaFX mapping").
- **Font families:** one family per style, without the fallback chain of the web previews (J4).
- **Lengths:** px at the 14 px base, as in the system.
- **Motion:** the system's README defines 120 ms (hover, pressed), 160 ms (selection, panel open) and `ease-out`, but its `tokens.json` has no motion family; the converter adds them as `motion.duration-hover`, `motion.duration-select` and `motion.easing-standard` (`[0, 0, 0.58, 1]`).
- **Validation:** [`tools/design-system/ValidateTokens.java`](../../tools/design-system/ValidateTokens.java) fails loudly and fixes nothing. It checks that every colour and shadow token has a value in every theme, that aliases resolve without cycles, that every name matches `[a-z0-9-]`, that colours are sRGB objects or aliases whose components agree with their `hex`, that high contrast has no translucent colour and no shadow, that shadows have one layer and no spread, that type styles have no letter spacing and use a listed face for their weight, that rows and in-cell targets are at least 24 px, and that every pairing and state names known tokens and unique ids. Last run: passed (2026-10-09).

## JavaFX mapping

| Token | JavaFX 27 CSS |
|---|---|
| colour `<name>` | looked-up colour `-tl-<name>` on `.root`, written as `#rrggbb` or `rgba(r, g, b, a)`; never `-fx-`, never `hsl()` or a named colour (J2). The prefix is required: an unprefixed lookup such as `red` overrides the named colour (SP-3) |
| length `<n>px` | written as a **literal** `<n ÷ 14>em` into the generated sheets where the node's font is the root font; an `em` resolves against the font of the node that uses it (SP-1), so exceptions are listed per component in `components.md`. Size lookups fail for padding, insets, radii and border widths (SP-1), so lengths are never runtime lookups |
| `typography` | `-fx-font-family` with one family (a list is no fallback), a literal `-fx-font-size` in em (a lookup is a parse error, SP-1), `-fx-font-weight` 400 or 700 only; weight 600 is `-fx-font-family: "IBM Plex Sans SmBld"` at normal weight, because weights up to 600 render regular (SP-4). The family names are "IBM Plex Sans", "IBM Plex Sans SmBld" and "JetBrains Mono NL"; IBM Plex Sans lacks ● ▲ ▼ ⚠, which are drawn as icons |
| `lineHeight` | `-fx-line-spacing` = line height − the face's natural line height at that size; the natural line height is measured per face in SP-4, and `lineHeightPx − fontSize` is only an upper bound |
| `shadow` | `dropshadow(gaussian, <colour>, <radius>, 0, <x>, <y>)` with `radius = 1.5 × blur` (a CSS blur radius `b` is a Gaussian with σ = b ÷ 2; the JavaFX kernel radius is 3σ), capped at 127; `none` sets no effect; one effect per node (J13) |
| `motion` | `transition` with literal durations inside `@media not (prefers-reduced-motion)`; a duration lookup fails in transitions (SP-1) but works for tooltip delays |

Root font size = max(`Font.getDefault().getSize()`, 14 px) × the user's text-size factor (handoff §5).

## Values

**Type styles** (px at the 14 px base; family `sans` = IBM Plex Sans, `mono` = JetBrains Mono NL):

| Style | Family | Size / line height | Weight |
|---|---|---|---|
| `display-name` | sans | 40 / 44 | 700 |
| `heading-1` | sans | 22 / 28 | 700 |
| `heading-2` | sans | 18 / 24 | 600 |
| `heading-3` | sans | 15 / 20 | 600 |
| `label-caps` | sans | 12 / 16 | 600 |
| `body` | sans | 14 / 20 | 400 |
| `body-strong` | sans | 14 / 20 | 600 |
| `ui-small` | sans | 13 / 18 | 400 |
| `caption` | sans | 12 / 16 | 400 |
| `button` | sans | 14 / 20 | 600 |
| `chip` | sans | 13 / 18 | 600 |
| `table-header` | sans | 12 / 16 | 600 |
| `table-name` | sans | 14 / 20 | 600 |
| `table-text` | sans | 14 / 20 | 400 |
| `table-meta` | sans | 13 / 18 | 400 |
| `numerals` | mono | 14 / 20 | 400 |
| `numerals-strong` | mono | 14 / 20 | 700 |
| `numerals-large` | mono | 18 / 24 | 700 |
| `mono-log` | mono | 13 / 18 | 400 |
| `keycap` | mono | 12 / 16 | 400 |

**Spacing:** 2, 4, 6, 8, 12, 16, 24, 32 px; cell padding 8 px (compact) and 12 px (comfortable). **Radius:** 0, 2, 4 px. **Width:** hairline 1, state 2, focus ring 2, focus gap 1, frame accent 2, selection mark 4 px.

**Size:** rows 28 px (compact) and 36 px (comfortable); controls 32 px, small controls and in-cell controls 24 px; icons 16 and 20 px; rail 48, sidebar 232, top bar 40, status bar 28 px; crop thumbnails 20 and 28 px (outside the table only); detail-strip crop 32–96 px high; minimum window 1280 × 720 px; Fan Kit logo frame 96 px.

The colour values per theme are in the theme files; their contrast results are in [accessibility-report.md](accessibility-report.md).

## Regenerating

```bash
java tools/design-system/ConvertTokens.java <export>/design-system ui/design-tokens "Claude Design system Tallyline, version <version>"
```

```bash
java tools/design-system/ValidateTokens.java ui/design-tokens
```

```bash
java tools/design-system/Contrast.java ui/design-tokens <report section file> "<inputs label>"
```

The scripts need JDK 22 or later (multi-file source launcher, JEP 458; tested on Temurin 27+35) and nothing else. A new system version is converted, validated and checked in one commit, together with the documents it changes.
