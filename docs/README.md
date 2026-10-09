# Documentation

> **Doc type:** Living reference — current. Last reviewed: 2026-10-08.

`docs/` is the project's single source of truth ([`CLAUDE.md`](../CLAUDE.md), "The documentation is the project"). This file says what belongs where. A new directory or top-level file under `docs/` gets its row here in the same commit.

| Path | What belongs there |
|---|---|
| [`plan/`](plan/) | The plan, numbered 01–11: requirements (01), architecture (02), language decision ADR-0001 (03), roadmap (04), bug analysis of the predecessor (05), UEX API notes (06), OCR concept (07), plan reviews (08), engineering principles (09), supply-chain security (10), DDD and TDD (11) |
| [`adr/`](adr/) | `0000-open-points.md`, the single register of open owner decisions and open verifications, and the decision records `NNNN-kebab-title.md` (ADR-0001 lives in `plan/03-language-decision.md`) |
| [`prompts/`](prompts/) | Prompts for AI tools, each with its run status |
| [`images/fankit/`](images/fankit/) | The Star Citizen Fan Kit logo files only; they are not under the project licence (`NOTICE` §4, `REUSE.toml`) |
| `user/` | The user guide (R-DOC-1), from M1; its images are redacted screenshots only |
| [`release-process.md`](release-process.md) | Versioning, release checklist, upgrade and downgrade, Star Citizen patch-day and UEX API-change procedures |
| [`dependency-pins.md`](dependency-pins.md) | The tags of SHA-pinned GitHub Actions and the versions and checksums of CI tools |
| [`cla-signatures.md`](cla-signatures.md) | The public roster of CLA signatures |

Diagrams are Mermaid or plain-text trees, never binary files. Images in `docs/` are only redacted user-guide screenshots and the Fan Kit logo files.
