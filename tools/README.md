# Tools

Developer tools that are never shipped with the app. Each subdirectory says what it holds and how to run it.

| Path | What belongs there |
|---|---|
| [`design-system/`](design-system/) | Scripts that convert, validate and check the design tokens, and the JavaFX spikes of the design-system handoff (ADR-0004) |
| `ocr-eval/` | The OCR evaluation harness, from M2 ([04](../docs/plan/04-roadmap.md)) |

Before the Gradle build exists (M0), tools are single-entry Java programs run with the JDK's source launcher; they use only the JDK and, for the spikes, JavaFX jars from a scratch directory outside the repository. Nothing they generate is committed unless a document says so.
