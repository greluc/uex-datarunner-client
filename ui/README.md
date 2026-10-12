# ui

> **Amended 2026-10-12 ([ADR-0005](../docs/adr/0005-rust-and-egui.md)):** the UI is built with egui, not JavaFX. The future `ui` crate holds the views; the CSS-oriented parts described in the design-system documents (user-agent sheets, pseudo-classes, JavaFX font and icon loading) no longer apply. `design-tokens/` remains the current source of the token values until open point O-112 reconciles it with the `tokens.json` of the Tallyline egui crate in M0.

Until the Cargo workspace exists (M0), this directory holds only [`design-tokens/`](design-tokens/): the design tokens of the design system in the DTCG 2025.10 format, described in [`docs/design-system/tokens.md`](../docs/design-system/tokens.md). The Java scripts that converted and validated them were removed on 2026-10-12. Once the crate exists, its purpose lives in its crate-level rustdoc and this file goes.
