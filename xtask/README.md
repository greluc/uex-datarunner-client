# xtask

Repository checks and generators, run as `cargo xtask <task>` (ADR-0005 D12). Nothing here ships in the application.

| Task | What it does |
|---|---|
| `check` | `cargo fmt --all --check`, Clippy with `-D warnings`, rustdoc with `RUSTDOCFLAGS=-D warnings`, all tests, and the requirement-tag check, in that order; every Cargo call that resolves dependencies passes `--locked` |

The requirement-tag check lists the tests with `cargo test --workspace --locked -- --list` and validates the tags in their names against the requirement table of `docs/plan/01-requirements.md` and the qualifiers of `docs/plan/04-roadmap.md`. The grammar is in `docs/plan/11-ddd-and-tdd.md` section B4.

The crate has no dependency beyond the standard library. It is a build tool, not a context crate: its modules are public so that the tests and `main.rs` can reach them, and the `api` / `api::model` export rule of the context crates does not apply. Its code is tested at three seams: the tag parser (`tags`), the requirement registry (`registry`, `listing`) and the check plan (`check`).
