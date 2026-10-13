# .cargo

Cargo configuration of the workspace. `config.toml` defines the `cargo xtask` alias, which runs the `xtask` package with `--locked`. Nothing else belongs here: toolchain pinning is `rust-toolchain.toml`, lints are in the root `Cargo.toml` and `clippy.toml`.
