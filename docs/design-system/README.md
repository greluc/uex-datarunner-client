# Design system

> **Doc type:** Living reference — draft (JavaFX handoff stage 2). Last reviewed: 2026-10-09.

The app's design system is **Tallyline** (working name, D1; trademark search open). It was designed in Claude Design from the brief [`docs/prompts/design-system.md`](../prompts/design-system.md) its update [`design-system-update-2026-10.md`](../prompts/design-system-update-2026-10.md) and the correction brief [`design-system-update-2-2026-10.md`](../prompts/design-system-update-2-2026-10.md); the repository follows Design System version `1791542897-a5d3` and screens canvas `1791541917-46e6`. It is translated into the repository by the JavaFX handoff [`design-system-javafx-handoff.md`](../prompts/design-system-javafx-handoff.md). The decisions behind it are [ADR-0004](../adr/0004-design-system.md) (Proposed) and register point O-83.

| File | What belongs there |
|---|---|
| [`tokens.md`](tokens.md) | The token files, their schema, the conversion and validation rules and the JavaFX mapping |
| [`states.md`](states.md) | Every user-visible state, its state id and its pseudo-class or style class; open design defects |
| [`accessibility-report.md`](accessibility-report.md) | Contrast results on the token values; later the colour-vision, rendered-sheet and manual checks |
| `components.md`, `references.md` | From stages 3 and 4 of the handoff: component specifications, and the sources and references |

The Claude Design artifacts (system and screens canvas) are private references; the repository keeps no copy of them. A local export may sit in the gitignored `design-previews/`.
