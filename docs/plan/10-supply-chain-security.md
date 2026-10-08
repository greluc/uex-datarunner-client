# Supply Chain Security

Goal: Neither a compromised dependency, a tampered build tool or CI action, nor a subverted release artifact should bring code onto users' machines unnoticed. The client handles a UEX secret key and runs alongside the game – a compromised build would be a direct attack on the users.

**Honest limit:** Supply chain security is risk reduction, not a guarantee. The following measures primarily prevent *unnoticed* changes (tampered artifacts, re-pointed tags, silently bumped versions). Against a maintainer who publishes a malicious, correctly signed version, only a cooldown period, review and a small number of dependencies help.

## 1. Threat model

| # | Attack vector | Example |
|---|---|---|
| T1 | Tampered or replaced artifact in the repository or in transit | Artifact is replaced after publication; a mirror delivers different bytes |
| T2 | Malicious **new version** of a genuine dependency (maintainer account takeover) | Patch release with malicious code is adopted automatically |
| T3 | Typosquatting or dependency confusion via additional repositories | Package with the same name in JitPack or `mavenLocal` |
| T4 | Tampered **Gradle wrapper** (`gradle-wrapper.jar`) or tampered Gradle distribution | Modified JAR in a PR |
| T5 | Compromised **GitHub Action** (tag is re-pointed to a malicious commit) | Incident involving `tj-actions/changed-files` (March 2025) |
| T6 | Tampered **binary assets**: ONNX models, native libraries, the build's JDK | Replaced model from Hugging Face; JDK download |
| T7 | Tampering **after** the build (release files replaced) | Installer in the release is swapped |
| T8 | Native libraries that are extracted at runtime into a **shared temp directory** (DLL/SO planting) | sqlite-jdbc and ONNX Runtime extract natives to `java.io.tmpdir` by default |
| T9 | Runtime loading of code or models from unversioned sources | Model download "latest"; auto-update without signature verification |
| T10 | Optional AI models via Ollama (downloaded by the user) | Tampered model in a third-party registry |

## 2. Measures

### 2.1 Resolving dependencies (T1–T3)

| ID | Measure | Implementation |
|---|---|---|
| S-1 | **Only two repositories:** Maven Central for libraries, Gradle Plugin Portal only for plugins. No `mavenLocal()`, no JitPack, no snapshot repos. | `settings.gradle.kts`: `dependencyResolutionManagement { repositoriesMode = RepositoriesMode.FAIL_ON_PROJECT_REPOS }`; repository content filter (`exclusiveContent`/`content { includeGroup… }`) for the Plugin Portal |
| S-2 | **Only fixed versions** from the version catalog; no dynamic versions (`+`, `latest.release`, ranges), no `SNAPSHOT`s | `resolutionStrategy { failOnDynamicVersions(); failOnChangingVersions() }` in `build-logic` for all configurations |
| S-3 | **Dependency locking** for all configurations including the buildscript classpath; transitive versions are frozen | `dependencyLocking { lockAllConfigurations() }`; `gradle.lockfile` checked in; changes only via `--write-locks` in a separate commit |
| S-4 | **Dependency verification:** SHA-256 checksum **and** PGP signature of every artifact (libraries, plugins, transitive) | `gradle/verification-metadata.xml` with `verify-metadata` and `verify-signatures` = `true`; trusted keys in `gradle/verification-keyring.keys` (checked in). A build with an unknown artifact or key **fails**. |
| S-5 | **New keys or checksums only with review:** update via `./gradlew --write-verification-metadata pgp,sha256 --export-keys help` in a separate PR. The diff shows every new key or hash; new key fingerprints are checked against the publisher's project page or release announcement and documented in the PR. | `CODEOWNERS`: `gradle/verification-metadata.xml`, `gradle/verification-keyring.keys`, `gradle/libs.versions.toml`, `*.lockfile`, `.github/workflows/**` → review by the maintainer mandatory |
| S-6 | Artifacts without a signature (if unavoidable) only with a checksum and an **explicit justification** in `verification-metadata.xml` (comment "why no PGP") | Review |

### 2.2 Updates (T2)

| ID | Measure | Implementation |
|---|---|---|
| S-7 | **Cooldown period for new versions:** Dependabot proposes version updates no earlier than **7 days** after publication. Experience shows that compromised releases are often discovered within days. **Security updates** are exempt from this. | `.github/dependabot.yml`: `cooldown: default-days: 7` (if not specified, the GitHub docs state a default of 3 days); ecosystems `gradle` and `github-actions` |
| S-8 | Updates individually or in small groups, never auto-merged; every update updates the lockfile **and** the verification metadata in the same PR (Dependabot cannot do this itself → manual follow-up commit) | Review checklist in 09 §11 extended |
| S-9 | For every update, briefly check: changelog, new transitive dependencies (diff in the lockfile), new signing key? | PR template |

### 2.3 Build tools (T4, T6)

| ID | Measure | Implementation |
|---|---|---|
| S-10 | **Secure the Gradle wrapper:** `distributionSha256Sum` in `gradle/wrapper/gradle-wrapper.properties`; CI checks `gradle-wrapper.jar` against the official checksums | `gradle/actions/wrapper-validation` (gradle/actions v6.4.0, pinned by commit SHA) in every workflow before the first Gradle invocation |
| S-11 | Do not obtain the **JDK of the release build** via the toolchain auto-download, but explicitly via `actions/setup-java` (distribution Temurin, fixed version) | `org.gradle.java.installations.auto-download=false` in CI; locally Foojay may be used, releases are built **only** in CI |
| S-12 | No build scripts from the network (`apply(from = "https://…")`), no plugins outside the catalog | ArchUnit cannot check this → review plus CI grep for `apply(from` with a URL |

### 2.4 CI/CD (T5, T7)

| ID | Measure | Implementation |
|---|---|---|
| S-13 | **Pin actions by full commit SHA** (version as a comment), never by tag or branch | `uses: actions/checkout@<40-hex-sha> # v7.0.1`; Dependabot (`github-actions`) updates SHA and comment |
| S-14 | **Minimal permissions:** `permissions: contents: read` as the workflow default; write permissions only in the release job (`contents: write`, `id-token: write`, `attestations: write`) | Workflow review; no `pull_request_target` with checkout of PR code |
| S-15 | Release builds only from **protected tags** on protected `main`; release job in a GitHub environment with approval | Branch/tag protection, environment "release" |
| S-16 | **Build provenance (SLSA):** every release artifact gets a signed attestation | `actions/attest-build-provenance` (v4.2.2, SHA-pinned); users can verify with `gh attestation verify` |
| S-17 | **Checksums** (`SHA256SUMS`) for all artifacts; verification instructions in the README | Release job |
| S-18 | **SBOM** (CycloneDX) for every release, as a release asset | Gradle plugin `org.cyclonedx.bom` 3.5.0 |
| S-19 | **Vulnerability scan** in every PR and daily: lockfiles against OSV | `google/osv-scanner` v2.6.0 on `gradle.lockfile`/`verification-metadata.xml`; known critical/high vulnerabilities in runtime dependencies **block** the merge (exceptions only documented with an expiry date) |
| S-20 | Report the dependency graph to GitHub so that Dependabot alerts also see transitive Gradle dependencies | `gradle/actions/dependency-submission` |
| S-21 | Optional: code signing of the Windows installers (Authenticode) against SmartScreen warnings and tampering | Costs a certificate → decision of the project owner; until then attestation plus checksums |

### 2.5 Binary assets and runtime (T6, T8, T9, T10)

| ID | Measure | Implementation |
|---|---|---|
| S-22 | **ONNX models** are obtained at a fixed Hugging Face revision (commit hash, not `main`). The SHA-256 is recorded in `NOTICE` and in a checksum file; a Gradle task verifies before packaging, the app verifies when loading. Mismatch = abort with a clear message. | `adapter-ocr` (loading), `build-logic` (build check) |
| S-23 | **No runtime downloads of code or OCR models.** Optionally downloadable models (e.g. additional writing systems, R-L10N-3) only from fixed, configured URLs with a fixed, stored SHA-256 | Review; `adapter-ocr` |
| S-24 | **Do not extract natives into the shared temp:** sqlite-jdbc and ONNX Runtime load their natives from an app-specific directory (shipped in the jlink image or extracted into a user-owned, non-world-writable directory) | System properties for the native paths (e.g. `org.sqlite.tmpdir`); **check the exact property names for ONNX Runtime in M0** |
| S-25 | **Update notice without auto-update:** the app only shows "new version available" with a link to the GitHub release page; it downloads and launches nothing by itself | `adapter-platform` |
| S-26 | **TLS everywhere**, no disabling of certificate verification (not even "for debugging") | ArchUnit rule: implementations of `X509TrustManager`/`HostnameVerifier` only in the one reviewed composite class for `Windows-ROOT` |
| S-27 | **Ollama models (optional):** the app shows the model digest from `/api/tags`. The user can **pin** a model to a digest; a differing digest produces a warning. Recommend models only from the official Ollama registry. The integrity of the download itself is Ollama's responsibility, not ours. | `adapter-vlm`, R-VLM-7 |

### 2.6 Repository and process

| ID | Measure |
|---|---|
| S-28 | Branch protection for `main`: PR required, green CI, review by CODEOWNERS, no force push; 2FA for all maintainers |
| S-29 | **Few dependencies** (09 §9): every new dependency needs a justification, a license check and a look at its maintenance status. Is there a PGP key on Central? How many maintainers? |
| S-30 | **Reproducibility:** archives with `isPreserveFileTimestamps = false` and `isReproducibleFileOrder = true`. Honestly: jpackage installers (MSI/deb) are not bit-for-bit reproducible; the attestation (S-16) partially compensates for this. |
| S-31 | Security reports: `SECURITY.md` with a reporting channel (privately via GitHub Security Advisories) |

## 3. What this achieves in practice

| Threat | Covered by | Residual risk |
|---|---|---|
| T1 tampered artifact | S-3, S-4 (hash + signature) | low |
| T2 malicious new version | S-7 cooldown period, S-8/S-9 review, S-19 scan | **medium** – a correctly signed malicious version from a genuine maintainer is only detected through time and review |
| T3 typosquatting/confusion | S-1, S-2, S-5 | low |
| T4 wrapper/Gradle | S-10 | low |
| T5 actions | S-13, S-14 | low |
| T6 models/natives/JDK | S-4, S-11, S-22 | low |
| T7 tampering after build | S-16, S-17 | low, provided users verify; without code signing (S-21) very few do |
| T8 native planting | S-24 | low after implementation |
| T9 runtime loading | S-23, S-25 | low |
| T10 Ollama models | S-27 | **medium** – outside our control; the feature is optional and local |
