# Design system

> **Doc type:** Living reference — draft (JavaFX handoff stage 2; JavaFX mechanism superseded by ADR-0005). Last reviewed: 2026-10-12.

> **Note 2026-10-12:** [ADR-0005](../adr/0005-rust-and-egui.md) replaces JavaFX with egui. Tallyline continues as the design system; its egui rendition (the crate `tallyline-egui`) replaces the JavaFX CSS handoff described below. The token values, state names and contrast results in this directory stay; the JavaFX-specific parts are marked where they occur. What of the crate enters the repository is open point O-112 (see "egui rendition (ADR-0005)" below).

The app's design system is **Tallyline** (working name, D1; trademark search open). It was designed in Claude Design from the brief [`docs/prompts/design-system.md`](../prompts/design-system.md) its update [`design-system-update-2026-10.md`](../prompts/design-system-update-2026-10.md) and the correction brief [`design-system-update-2-2026-10.md`](../prompts/design-system-update-2-2026-10.md); the repository follows Design System version `1791542897-a5d3` and screens canvas `1791541917-46e6`. It is translated into the repository by the JavaFX handoff [`design-system-javafx-handoff.md`](../prompts/design-system-javafx-handoff.md). The decisions behind it are [ADR-0004](../adr/0004-design-system.md) (partly superseded by [ADR-0005](../adr/0005-rust-and-egui.md) on 2026-10-12: §3, §4, §6, §7 and the JavaFX spikes) and register point O-83.

| File | What belongs there |
|---|---|
| [`tokens.md`](tokens.md) | The token files, their schema, the conversion and validation rules and the JavaFX mapping |
| [`states.md`](states.md) | Every user-visible state, its state id and its pseudo-class or style class; open design defects |
| [`accessibility-report.md`](accessibility-report.md) | Contrast results on the token values; later the colour-vision, rendered-sheet and manual checks |
| `components.md`, `references.md` | From stages 3 and 4 of the handoff: component specifications, and the sources and references |

The Claude Design artifacts (system and screens canvas) are private references; the repository keeps no copy of them. A local export may sit in the gitignored `design-previews/`.

## egui rendition (ADR-0005)

Added 2026-10-12. [ADR-0005](../adr/0005-rust-and-egui.md) D6 keeps Tallyline and replaces its JavaFX CSS handoff by an egui rendition. The owner delivered it as the Rust crate `tallyline-egui` v0.1.0 (egui 0.36.2). **The crate is not in the repository yet**; adopting it is open point O-112 in the [register](../adr/0000-open-points.md).

**What the crate provides:**
- colour tokens for the dark, light and high-contrast themes (99 tokens, aliases resolved), the type ramp, spacing, radius, width and size constants, motion times and the shadows;
- an egui `Style` mapping with theme, density, text-size and reduced-motion settings;
- the 156 Material Symbols Rounded icons used by the system, as codepoint constants of a subset icon font;
- egui ports of `Icon`, `Button`, `IconButton`, `Chip`, `DeltaBadge`, `Field`, `TableCell`, `Panel` and `Toast`, an interactive gallery and a GPU-free PNG snapshot example;
- contrast tests against the system's floors (its `cargo test` passes: one unit test, five contrast tests, one doctest).

**What it does not port yet:** the in-cell editing input, the toggle chip, navigation items, the window shell (top bar with window controls, sidebar and rail), the DetailStrip with the readable crop, and the screenshot overlays. The tokens for all of them are available.

**egui limits** (listed in the crate's README): one set of widget visuals, so built-in widgets show keyboard focus as a 2 pt stroke on the frame edge and pressed buttons look focused; one selection colour; no line height per `TextStyle`; no font weights (one font family per face); an approximated shadow blur; missing glyphs fall back to egui's fonts. The state names in [states.md](states.md) and the contrast results in [accessibility-report.md](accessibility-report.md) are unaffected.

**Findings and what @greluc decided on 2026-10-12 (O-112):**
1. Its generators `tools/gen_tokens.py` and `tools/build_icons.py` are Python. *Decided:* Python stays outside the repository with the design system; only the generated `tokens.rs` and `icon.rs` are committed and are updated only from there.
2. The icon font `MaterialSymbolsRounded-Tallyline.ttf` is a modified, subset Apache-2.0 font; ADR-0004 §5 chose path data, and `CLAUDE.md` allowed only unmodified SIL OFL fonts. *Decided:* the font is approved as a named exception; `NOTICE` records the modification (Apache-2.0 section 4(b)), the source, the SHA-256 and the licence text.
3. About 30 plain `//` comments and many undocumented public items conflict with the code-comment rule and `#![deny(missing_docs)]`. *Decided:* cleaned up before the import (D11).
4. The licence in `Cargo.toml` is a placeholder (MIT OR Apache-2.0); the project licence is GPL-3.0-or-later and every file needs the licence header. *Decided:* fixed before the import.
5. Its `tokens.json` carries only `"version": 1`, so its equality with design-system version `1791542897-a5d3` is not proven. *Decided:* reconciled in M0; until then `ui/design-tokens/` (DTCG) and the crate's `design/tokens.json` exist side by side, and an M0 test checks that they contain the same values.
6. The Java tools under `tools/design-system/` were removed on 2026-10-12 (Git history keeps them); an `xtask` task replaces them in M0.
7. D1 to D11 of O-83 stand unchanged for the egui rendition.
8. The crate is imported in M0 as its own pull request, after the cleanup of points 3 and 4; until then it stays outside the repository.

Until the M0 reconciliation, `ui/design-tokens/` remains the current source of the token values.
