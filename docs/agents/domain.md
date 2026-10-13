# Domain docs

> **Doc type:** Living reference — current. Last reviewed: 2026-10-13.

How the engineering skills consume this repository's domain documentation. The layout is **single-context**.

## Before exploring, read these

- **The ubiquitous language**: `docs/plan/11-ddd-and-tdd.md` §A1 (terms, bounded contexts, aggregates, events). A root `CONTEXT.md` does not exist; if `/domain-modeling` creates one, it must not duplicate §A1 and 11 stays the source of truth.
- **Requirements and architecture**: `docs/plan/01-requirements.md` and `docs/plan/02-architecture.md` for the area you are about to work in.
- **`docs/adr/`**: read the ADRs that touch the area. Check `docs/adr/0000-open-points.md` before proposing anything that looks undecided; a decided point is reopened only by @greluc.

If one of these does not cover your topic, proceed silently; do not flag its absence.

## File structure

```
/
├── CLAUDE.md
├── docs/
│   ├── plan/        requirements, architecture, roadmap, ubiquitous language (11)
│   └── adr/         decision records and the open-points register
└── crates/          from M0
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in 11 §A1. Do not drift to synonyms. If the concept is missing there, either you are inventing language the project does not use (reconsider) or there is a real gap (fix it in the same commit, per `CLAUDE.md`).

## Flag ADR conflicts

Requirements (`R-…`) and accepted ADRs are normative. If your output contradicts one, surface it explicitly and stop for @greluc's decision rather than silently overriding:

> _Contradicts ADR-0002 (modules by bounded context), but worth reopening because…_
