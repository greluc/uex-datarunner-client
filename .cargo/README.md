# .cargo

Cargo configuration of the workspace. `config.toml` defines the `cargo xtask` alias, which runs the `xtask` package with `--locked`, and sets `AWS_LC_SYS_USE_SYSTEM=0` for every build, so the build script of `aws-lc-sys` compiles the vendored AWS-LC and never links a system-installed one (ADR-0006). Because Cargo passes `[env]` to build scripts, this holds for local builds, CI and packaging alike. Nothing else belongs here: toolchain pinning is `rust-toolchain.toml`, lints are in the root `Cargo.toml` and `clippy.toml`.
