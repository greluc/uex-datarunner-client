# Dependency pins

> **Doc type:** Living reference — current. Last reviewed: 2026-10-12.

This file records what the pins in `.github/` cannot say themselves: the tag each pinned GitHub Action resolves to, and the version and checksum of every CI tool that a workflow downloads or installs outside the crate dependency graph. Rust crates are pinned elsewhere: in `Cargo.toml`, `Cargo.lock`, the `cargo vendor` directory and the `cargo-vet` records (S-3 to S-6). Amended 2026-10-12: the Gradle-based pins were replaced by the Cargo-based ones when the implementation language changed to Rust ([ADR-0005](adr/0005-rust-and-egui.md)).

## Rule

- Every `uses:` line in `.github/workflows/` pins an action by its full 40-character commit SHA, never by tag or branch (S-13). The line carries no version comment, because configuration files carry no comments; the table below records the tag instead.
- A pin and its row change in the same commit. A `uses:` line without a row, or a row without a `uses:` line, is drift and is fixed in the commit that notices it.
- Before a SHA is pinned or bumped, check that the tag resolves to it, for example with `git ls-remote https://github.com/<owner>/<repo> 'refs/tags/<tag>*'`. For an annotated tag, pin the commit of the peeled line (`refs/tags/<tag>^{}`), not the tag object. Record the date of the check.
- Dependabot proposes action updates by SHA, after the cooldown of S-7. The pull request updates the row; the reviewer repeats the check above.
- An action step whose default behaviour includes a proprietary component is configured to switch that component off (S-37). The "Notes" column records how.
- A CI tool that a workflow downloads is pinned by version and by the SHA-256 of the downloaded file, verified before it runs (S-13, S-19, S-36); a Python tool is installed from a hash-locked requirements file. The SHA-256 is taken from the release's own checksum file and checked against the downloaded file.
- A tool installed with Cargo (`cargo-deny`, `cargo-audit`, `cargo-vet`, `cargo-mutants`, `cargo-llvm-cov`, `cargo-packager`) is installed at an exact version with `--locked`, or downloaded as a release binary with a fixed SHA-256, and has its row below. Which of the two a tool uses is chosen and verified in M0.
- A value marked "read from source" was taken from the named tool's source code and has not yet been compared with the released asset. It is re-checked against the release asset, and the row updated, before the pin is relied on.

## GitHub Actions

| Action | Tag | Commit SHA | Checked on | Used in | Notes |
|---|---|---|---|---|---|

No action is pinned yet.

## CI tools

| Tool | Version | Download | SHA-256 | Checked on | Used in |
|---|---|---|---|---|---|
| WiX Toolset (pre-installed for the packager; the packager must not download it) | 3.11.2 | the WiX 3.11.2 binaries archive that `cargo-packager` would otherwise fetch itself; the exact URL is recorded when the hash is re-checked against the release asset | `2c1888d5d1dba377fc7fa14444cf556963747ff9a0a289a3599cf09da03b9e2e` (read from cargo-packager's source; not yet compared with the release asset) | 2026-10-12 (read from source) | Windows installer job (M1 spike, ADR-0005 D7, O-116) |
| `cargo-packager` | 0.11.8 | crates.io, installed with `--locked` at the exact version (or a checksummed release binary; chosen in M0) | to be recorded in M0 (the crates.io checksum of the `.crate` file, or the SHA-256 of the release binary) | 2026-10-12 (version only) | installer jobs (M1, ADR-0005 D7); its NSIS target is not used, because the packager checks the NSIS download with SHA-1 only and a plugin not at all |

No other CI tool is pinned yet.
