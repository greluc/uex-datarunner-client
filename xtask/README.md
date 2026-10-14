# xtask

Repository checks and generators, run as `cargo xtask <task>` (ADR-0005 D12). Nothing here ships in the application.

| Task | What it does |
|---|---|
| `check` | The supply-chain guard, `cargo fmt --all --check`, Clippy with `-D warnings`, rustdoc with `RUSTDOCFLAGS=-D warnings`, all tests, and the requirement-tag check, in that order; every Cargo call that resolves dependencies passes `--locked` |
| `guard` | The CI guard of 10 S-4: reads `Cargo.lock`, every `Cargo.toml`, every `.cargo/config.toml`, every `rust-toolchain.toml` and everything under `.github/`, and fails on a package without checksum or from another source, on `[patch]`, `[replace]`, `[source]`, `[registries]`, the listed environment overrides, a `git` or `registry` dependency, a Cargo command without `--locked`, and a toolchain pin that is not an exact version. It needs no network |
| `vendor-hashes write` | Vendors the dependencies with `cargo vendor --locked --versioned-dirs` into `target/xtask-vendor` under an empty `CARGO_HOME`, hashes every package directory, writes `supply-chain/vendor-hashes.txt` and deletes the tree. Needs the network |
| `vendor-hashes verify` | The same vendor run, then compares the tree with the committed list and fails on a missing, unexpected or changed package. Needs the network; CI runs it on both systems |

The requirement-tag check lists the tests with `cargo test --workspace --locked -- --list` and validates the tags in their names against the requirement table of `docs/plan/01-requirements.md` and the qualifiers of `docs/plan/04-roadmap.md`. The grammar is in `docs/plan/11-ddd-and-tdd.md` section B4.

The supply-chain tasks are specified in `docs/plan/10-supply-chain-security.md` (S-4 and its implementation note); the list format and the update procedure are in `supply-chain/README.md`.

The crate is a build tool, not a context crate: its modules are public so that the tests and `main.rs` can reach them, and the `api` / `api::model` export rule of the context crates does not apply. Its code is tested at these seams: the tag parser (`tags`), the requirement registry (`registry`, `listing`), the check plan (`check`), the mapping of an error to the exit code and its report (`exit`), the lockfile reader (`lockfile`), the hash list and its comparison (`hashlist`), the guard rules over file texts (`guard`), the file collector (`sources`), the vendor tree hashing and verification against fake trees on disk (`vendor`), and the negative tests that run a real Cargo against a stale `Cargo.lock` (`negative`).

## Dependencies

`xtask` has two direct dependencies, declared once in `[workspace.dependencies]` of the root `Cargo.toml` (10 S-2). Both are build tooling and never reach the shipped binary. All crates below are pure Rust with no `links` key and no C, C++ or assembler source; the versions are the latest ones older than the 7-day cooldown of 10 S-7 when they were added (2026-10-14).

| Crate | Version | Purpose | Licence (SPDX) | Source | Build script | Procedural macro |
|---|---|---|---|---|---|---|
| `sha2` | 0.11.0 | SHA-256 of the vendored files | `MIT OR Apache-2.0` | crates.io, `github.com/RustCrypto/hashes` | no | no |
| `toml` | 1.1.6+spec-1.1.0 | Reads `Cargo.lock`, manifests, Cargo configuration and the toolchain pin | `MIT OR Apache-2.0` | crates.io, `github.com/toml-rs/toml` | no | no |

Compiled transitive crates: `cfg-if`, `cpufeatures` (and `libc` on Linux, with a build script that only probes `rustc`), `digest`, `block-buffer`, `crypto-common`, `hybrid-array`, `typenum` for `sha2`; `serde_core` (build script that probes `rustc` and writes a generated module), `serde_spanned`, `toml_datetime`, `toml_parser`, `winnow` for `toml`. All are `MIT OR Apache-2.0` except `winnow` (`MIT`). `serde_derive`, `proc-macro2`, `quote`, `syn`, `unicode-ident`, `indexmap`, `hashbrown`, `equivalent` and `toml_writer` are in `Cargo.lock` and vendored but not compiled.
