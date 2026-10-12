# Tools

Developer tools that are never shipped with the app. Each subdirectory says what it holds and how to run it.

| Path | What belongs there |
|---|---|
| [`design-system/`](design-system/) | Scripts that convert, validate and check the design tokens, and the JavaFX spikes of the design-system handoff (ADR-0004). Superseded by [ADR-0005](../docs/adr/0005-rust-and-egui.md) (2026-10-12): replaced by an `xtask` task once O-112 is settled |
| `ocr-eval/` | The OCR evaluation harness, from M2 ([04](../docs/plan/04-roadmap.md)) |

Before the build exists (M0), the existing tools are single-entry Java programs run with the JDK's source launcher; they use only the JDK and, for the spikes, JavaFX jars from a scratch directory outside the repository. Nothing they generate is committed unless a document says so.

Since 2026-10-12 ([ADR-0005](../docs/adr/0005-rust-and-egui.md) D12) new repository tooling is Rust: checks and generators become tasks of an `xtask` crate. The Java programs stay only until their `xtask` replacement exists; no new Java tool is added.
