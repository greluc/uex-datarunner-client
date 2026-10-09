# Design-system tools

Scripts for the design tokens in [`ui/design-tokens/`](../../ui/design-tokens/) and the spikes of the JavaFX design-system handoff ([`docs/prompts/design-system-javafx-handoff.md`](../../docs/prompts/design-system-javafx-handoff.md), [ADR-0004](../../docs/adr/0004-design-system.md)). The documentation they serve is in [`docs/design-system/`](../../docs/design-system/).

| File | Purpose |
|---|---|
| `ConvertTokens.java` | Converts a Claude Design system export into the DTCG 2025.10 token files, `pairings.json` and `states.json` |
| `ValidateTokens.java` | Validates the token files against the conversion rules; fails on any violation and fixes nothing |
| `Contrast.java` | WCAG contrast of every declared pairing and the derived checks, with a self-test; writes the contrast section of `docs/design-system/accessibility-report.md` |
| `Json.java`, `Tokens.java` | Shared JSON reader and writer, and the token loader with alias resolution |
| `spikes/SP-<n>/` | One spike per directory; each states its question and evidence, and its result is recorded in ADR-0004 |

Run each entry file with the JDK's source launcher (JDK 22 or later, multi-file source programs, JEP 458; tested on Temurin 27+35):

```bash
java tools/design-system/ValidateTokens.java ui/design-tokens
```

```bash
java tools/design-system/Contrast.java --self-test
```

The spikes need the JavaFX 27 jars on the module path (`--module-path <dir> --add-modules javafx.controls`), from a scratch directory outside the repository; screenshots and logs go to a scratch directory or the gitignored `design-previews/`, never into the repository.
