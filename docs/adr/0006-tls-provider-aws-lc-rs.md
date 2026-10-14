# ADR-0006: `aws-lc-rs` as the `rustls` crypto provider

> **Doc type:** Decision record — Accepted. Last reviewed: 2026-10-14.

- **Status:** Accepted (2026-10-14)
- **Decider:** @greluc, in chat on 2026-10-14, after the M0 TLS provider check that [ADR-0005](0005-rust-and-egui.md) D2 and D4 and O-110 call for. The check first recommended `ring`; @greluc objected to its release gap on a security-critical component, and the recommendation was changed to `aws-lc-rs`.
- **Related:** O-110 and O-126 in [the register](0000-open-points.md), [09](../plan/09-engineering-principles.md) §3b, [10](../plan/10-supply-chain-security.md) S-6, S-26, S-34, [02](../plan/02-architecture.md) §8 and §9, [01](../plan/01-requirements.md) R-API-3

## Context

ADR-0005 D4 fixes `reqwest` with `rustls`. `rustls` needs a crypto provider, and the pure-Rust candidate (`rustls-rustcrypto`) is marked "do not use in production" by its own authors, so the provider is a registered native-code exception (D2): `ring` (C and assembler) or `aws-lc-rs` (AWS-LC, C and assembler). The system certificate store comes from `rustls-platform-verifier`. O-110 asked for the licence of the native parts against the licence policy, the `cargo-vet` and `cargo-audit` state, the build prerequisites on both CI operating systems and the platform verifier.

## Method

Read at source on 2026-10-14: the published crates `ring` 0.17.14, `aws-lc-rs` 1.18.2, `aws-lc-sys` 0.46.0, `rustls` 0.23.45, `rustls-webpki` 0.103.15, `rustls-platform-verifier` 0.7.1, `rustls-native-certs` 0.8.5, `openssl-probe` 0.2.1 and `reqwest` 0.13.5 (each the newest stable release on crates.io that day), unpacked from `static.crates.io` and read file by file (licence files, per-file headers, `Cargo.toml`, build scripts). Three throwaway crates outside the repository (`reqwest` with `rustls-no-provider` and `rustls/ring`; `reqwest` with its `rustls` feature, which selects `aws-lc-rs`; `reqwest` with `rustls-no-provider` and `rustls/aws-lc-rs`) were built in release mode on Windows 11 (MSVC, no NASM, no CMake) and each performed a real `GET` against the UEX API host, which returned `200 OK` through the platform verifier. `cargo-deny` ran with the repository's `deny.toml`, `cargo-vet` with the four imports of `supply-chain/config.toml`, `cargo-audit` against the RustSec advisory database at commit `6aed846` (2026-10-13). The Linux build was not run (the WSL distribution on the review machine has no toolchain); its prerequisites come from the build scripts and the runner image documentation, and the first pull request that adds the dependency confirms them in CI.

## Findings

| | `ring` 0.17.14 | `aws-lc-rs` 1.18.2 with `aws-lc-sys` 0.46.0 |
|---|---|---|
| Licence, as declared | `Apache-2.0 AND ISC` | `ISC AND (Apache-2.0 OR ISC)` and a seven-term expression for `aws-lc-sys` |
| Licence, as read in the files | New code ISC; code from BoringSSL and the OpenSSL perlasm files Apache-2.0 (per-file headers); `fiat` Apache-2.0; `once_cell` polyfill MIT OR Apache-2.0. The Go and Chromium BSD-3-Clause texts in `LICENSE-BoringSSL` cover test and build support that is not compiled in | Per-file SPDX markers: Apache-2.0 OR ISC OR MIT-0, Apache-2.0, ISC, Apache-2.0 OR ISC, Apache-2.0 OR ISC OR MIT, BSD-3-Clause (jitterentropy), MIT, MIT OR CC0-1.0. Jitterentropy is offered as BSD-style or GPL-2.0 and the `LICENSE` file elects BSD-3-Clause; Keccak and AES reference code are CC0. Test-only third parties (googletest, Go, Wycheproof) are not compiled in |
| Against the allowlist | every compiled component is on it | every compiled component is on it; the BSD-style jitterentropy text, s2n-bignum, mlkem-native, mldsa-native and fiat each need their own `NOTICE` entry |
| Native code | C and assembler: 391 files, 1.5 MB packed, 8.3 MB unpacked | C, C++ and assembler: 2044 files, 9.7 MB packed, 68 MB unpacked |
| Binary blobs in the crate | 17 prebuilt COFF objects for Windows (`pregenerated/*.o`), next to their `.asm` sources | 26 prebuilt NASM objects for Windows x86-64 (`builder/prebuilt-nasm`), used when NASM is absent |
| Windows build | a C compiler (MSVC); no Perl, no NASM: the packaged crate uses `pregenerated/` | a C compiler; `rustls`' `aws_lc_rs` feature enables `prebuilt-nasm`, so neither NASM nor CMake is needed (verified: "Building with: CC"). Installing NASM changes the build path |
| Linux build | a C compiler (not run) | a C compiler (not run); the CC builder needs neither CMake nor Perl nor Go. A system-installed AWS-LC is auto-detected and linked instead of the vendored source unless `AWS_LC_SYS_USE_SYSTEM=0` |
| Spike, cold release build of a `reqwest` client | 12 s, 3.37 MB binary | 31 to 34 s, 4.24 to 4.31 MB binary |
| `Cargo.lock` packages | 133 | 155 with `reqwest`'s `rustls` feature, 142 with `rustls-no-provider`; in both, `ring` is still locked (an optional dependency of `rustls-webpki`) and so is vendored and seen by every lockfile tool; with `ring` chosen no `aws-lc-*` crate is locked |
| `cargo-deny` | licences and sources pass; the only error is the unlicensed spike crate | same |
| `cargo-vet` (four imports, all targets) | 118 crates without an audit | 138 crates without an audit |
| `cargo-vet` for the provider itself | no import audits 0.17.x (Google has only a 0.16.12 to 0.16.20 delta) | no import audits `aws-lc-rs` or `aws-lc-sys` |
| `cargo-audit` | no finding | no finding |
| Advisory history | RUSTSEC-2025-0009 (patched at 0.17.12); the unmaintained notice RUSTSEC-2025-0007 was withdrawn on 2025-02-22 | five advisories in March 2026 (RUSTSEC-2026-0044 to -0048: X.509 name constraints, AES-CCM timing, PKCS7 and CRL logic), all patched at 0.39.0 or earlier; none is reachable through `rustls`, which verifies certificates with `rustls-webpki` or the operating system |
| Maintenance | latest release 0.17.14 on 2025-03-11, none since; `main` receives commits (latest 2026-07-22); 0.17.15 is "TBD" in `RELEASES.md`; in issue 2840 ("Request for new release", opened 2026-05-12) the maintainer announced a release "in the next couple of weeks" on 2026-05-19 and was still asking for help debugging issue 2868 on 2026-07-16; the `rustls` team has had access since February 2025 | releases weekly (latest 2026-10-12), Amazon-maintained |
| Key exchange | X25519, secp256r1, secp384r1 | adds the hybrid X25519MLKEM768 groups, which `rustls` prefers when its `prefer-post-quantum` feature is on |

`rustls-platform-verifier` 0.7.1 (MIT OR Apache-2.0) has no native code of its own and takes the provider from `reqwest`, which passes it in (`Verifier::new(provider)`). On Windows it calls the operating system's certificate chain engine through `windows-sys` (an OS API, not a bundled library), checks revocation of the end certificate with a 10 second URL retrieval timeout, and so lets Windows contact the issuing CA's OCSP and CRL endpoints (errors about an unknown revocation status are ignored, so an unreachable responder delays a handshake by up to the timeout but does not fail it); that destination is not in the data-flow inventory (O-126). On Linux it loads the roots once at start through `rustls-native-certs` and `openssl-probe` (`SSL_CERT_FILE`, `SSL_CERT_DIR`, then the usual distribution bundles such as `/etc/ssl/certs/ca-certificates.crt` and `/etc/pki/tls/certs/ca-bundle.crt`) and verifies with `rustls-webpki`, without revocation checking. `reqwest` 0.13.5 offers `rustls` (provider `aws-lc-rs`) and `rustls-no-provider`; with the latter the application must install the provider before the first client is built, or `reqwest` panics at construction.

## Alternatives considered

Read on crates.io and the projects' repositories on 2026-10-14:

- **`ring`:** see the table. Not rejected on licence, build or audit state, but on the release gap: a fix for a vulnerability would depend on one maintainer's next release, which has been announced for months.
- **`graviola` with `rustls-graviola`** (0.4.x, June 2026, by a `rustls` maintainer; Apache-2.0 OR ISC OR MIT-0): Rust plus formally verified s2n-bignum assembly, no C compiler. x86-64 and aarch64 only, with CPU feature requirements (AVX2 and others on x86-64), described by its own README as "very new, exercise due caution"; no audit found. It is still assembler, so D1 would apply. Too young now; a candidate for the revisit.
- **Operating-system cryptography** (`reqwest` with `native-tls`: SChannel on Windows, OpenSSL on Linux): updates arrive through Windows Update and the distribution, but it replaces `rustls` (ADR-0005 D4) and the platform verifier, needs a system `libssl` on Linux and so affects the ZIP, `tar.gz` and `.deb` packages.
- **`rustls-cng`, `rustls-symcrypt`** (Windows-only providers): two providers, one per operating system, for no clear gain.
- **`rustls-rustcrypto`:** "do not use in production" by its own authors.

## Decision

Use `aws-lc-rs` as the `rustls` provider: `reqwest` with `default-features = false` and `rustls-no-provider`, and `rustls` with the `aws-lc-rs` feature (without the default `prefer-post-quantum`, unless a later decision wants it), both pinned in the M0 dependency pull request. The explicit variant locks 142 crates against 155 for `reqwest`'s own `rustls` feature, which would add `quinn` and its dependencies that are never compiled. The `adapter-uex` composition installs the provider exactly once at start-up, a test builds a client and fails if none is installed, and `rustls-platform-verifier` provides the system store.

Reasons:

- Both candidates are clean on the licence allowlist, `cargo-audit` and the build, so those do not decide; the maintenance cadence does. `aws-lc-rs` released on 2026-10-12 and ships fixes within the advisory cycle (all five 2026 advisories are patched), while `ring` has had no release for 19 months.
- The size argument against `aws-lc-sys` (68 MB unpacked) is weak: only the per-package hash list is committed, not the vendor tree (O-120). `ring` is locked and vendored when `aws-lc-rs` is chosen anyway.
- On Windows `rustls` enables `prebuilt-nasm`, so a C compiler is the only prerequisite, as for `ring`.

Accepted costs: a larger native surface that cannot be audited by reading; five advisories in March 2026 show the exposure, although none is reachable through `rustls`; the build depends on a system AWS-LC and on NASM when present, so CI and packaging set `AWS_LC_SYS_USE_SYSTEM=0`; more `NOTICE` entries.

## Consequences

- 09 §3b registers `aws-lc-rs` with `aws-lc-sys` (native part, reason, licence, revisit 2027-04-13). `NOTICE` gets the entries when the dependency enters the build: the `aws-lc-sys` `LICENSE` with its Apache-2.0, ISC, MIT, BSD-3-Clause and MIT-0 texts, the jitterentropy notice (BSD-3-Clause elected over the GPL-2.0 alternative), and the notices of s2n-bignum, mlkem-native, mldsa-native, fiat and the CC0 reference code.
- No `cargo-vet` import covers `aws-lc-rs` or `aws-lc-sys`, and 10 S-6 forbids adding a new dependency to the bootstrap exemptions. When the dependency is added, both get either our own audit in `supply-chain/audits.toml` or a justified exemption with a row in the suppression register (09 §3a) and a revisit date; the same holds for every other crate of the `reqwest` stack (138 in the spike), which is O-123's verification, not this decision's. `ring` stays in the lock as an optional dependency of `rustls-webpki` and needs the same treatment, and the native-code register test sees its `links` key: 09 §3b lists it as a locked, never compiled package.
- The vendor review (10 S-9) lists the 26 prebuilt NASM objects of `aws-lc-sys`; they are rebuilt from source with NASM and compared before the first release, or the review records why not.
- CI and packaging set `AWS_LC_SYS_USE_SYSTEM=0`, and a check fails the build if the build script reports an auto-detected system AWS-LC.
- The Linux prerequisites (a C compiler on `ubuntu-latest`) are confirmed by the first CI run of the dependency pull request.
- The Linux certificate locations in 02 §8 are the ones `rustls-native-certs` reads, including the `SSL_CERT_FILE` and `SSL_CERT_DIR` overrides.
- Revisit on 2027-04-13, or earlier when an `aws-lc-sys` advisory that affects `rustls` use is unpatched after 30 days, or when `graviola` has an independent audit and a stable release.
