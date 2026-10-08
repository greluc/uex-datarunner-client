# UEX Datarunner Client

Cross-platform desktop client (Windows + Linux) for Star Citizen DataRunners. It captures commodity terminal data – **manually or via screenshot OCR** – and submits it to [UEX Corp](https://uexcorp.space). Current UEX data serves as default values and plausibility bounds for recognition.

While the game is running, a lightweight local OCR does the work. Optionally, a local AI model (via Ollama) re-checks the results once the game is closed.

> Status: **planning**. There is no runnable code yet.

## Plan

| Document | Contents |
|---|---|
| [01 – Requirements](docs/plan/01-requirements.md) | Functional and non-functional requirements, assumptions |
| [02 – Architecture](docs/plan/02-architecture.md) | Modules, data flow, domain model, platform integration |
| [03 – Language decision (ADR)](docs/plan/03-language-decision.md) | Java vs. Rust, evaluation, decision: Java 27 + JavaFX 27 |
| [04 – Roadmap](docs/plan/04-roadmap.md) | Milestones M0–M5 with acceptance criteria, risks |
| [05 – SC-Datarunner-UEX bug analysis](docs/plan/05-datarunner-bug-analysis.md) | Known bugs of the predecessor and our fixes |
| [06 – UEX API](docs/plan/06-uex-api.md) | Endpoints, payload, open verification points |
| [07 – OCR concept](docs/plan/07-ocr-concept.md) | Pipeline, screenshot observations, lessons from basetool-sc-extractor |
| [08 – Review](docs/plan/08-review.md) | Plan review: corrected errors, closed gaps, open points |
| [09 – Engineering principles](docs/plan/09-engineering-principles.md) | Modularisation, clean code, tests, definition of done – with enforcement |
| [10 – Supply-chain security](docs/plan/10-supply-chain-security.md) | Threat model, securing dependencies, build, CI and releases |
| [11 – DDD and TDD](docs/plan/11-ddd-and-tdd.md) | Ubiquitous language, bounded contexts, aggregates and invariants; test-driven approach |
| [Release process](docs/release-process.md) | Versioning, release checklist, upgrade/downgrade, patch-day and API-change procedures |
| [ADR-002 – Modules by bounded context](docs/adr/0002-modules-by-bounded-context.md) | Why the core is cut by bounded context and adapters by technology |

## Disclaimer

Unofficial community project, not affiliated with UEX Corp or Cloud Imperium Games / Roberts Space Industries.
