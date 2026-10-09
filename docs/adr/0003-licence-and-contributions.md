# ADR-0003: Licence GPL-3.0-or-later, contributions under DCO and CLA, Star Citizen Fan Kit unit

> **Doc type:** Decision record — Accepted. Last reviewed: 2026-10-08.

- **Status:** Accepted (2026-10-08)
- **Decider:** @greluc
- **Answers:** [ADR-0001](../plan/03-language-decision.md) §5 (the open licence decision) and the licence item of [08-review.md](../plan/08-review.md) §D
- **Related:** [`CLAUDE.md`](../../CLAUDE.md) ("Stack rules", "Licence", "Star Citizen Fan Kit unit", "Git and pull requests"), [`CONTRIBUTING.md`](../../CONTRIBUTING.md), [`CLA.md`](../../CLA.md), [`docs/cla-signatures.md`](../cla-signatures.md), [`NOTICE`](../../NOTICE), [`REUSE.toml`](../../REUSE.toml), [open-points register](0000-open-points.md) (O-1 to O-43, O-79 to O-82)

## Context

- The repository is public but had no licence: the governance review on 2026-10-08 read the repository through the GitHub API and found no detected licence and no branch protection on `main`. Without a licence nobody may reuse the work, and contributions cannot be accepted on clear terms.
- ADR-0001 §5 left the licence open. It recommended GPL-3.0-or-later if code from basetool is ported and, until then, allowed re-implementation by concept only.
- The owner's other projects hold material worth reusing:
  - **basetool-sc-extractor:** OCR pipeline code. GPL-3.0-or-later (its README); its source files carry no per-file header, and its README names the holder `Basetool` (2026).
  - **basetool:** the CLA, the Code of Conduct, the security policy and the Star Citizen Fan Kit unit with byte-exact tests. GPL-3.0-only; its Spotless header names Lucas Greuloch.
- Licences of what the plan ships, read from the Maven Central POMs and the shipped artefacts on 2026-10-08:

| Component | Licence | Note |
|---|---|---|
| OpenJDK runtime (jlink image), jpackage launcher, `msica.dll` | `GPL-2.0-only WITH Classpath-exception-2.0` | Third-party code under `legal/` (zlib, ICU, IJG, FreeType and others); GPLv2 §3 source duty |
| OpenJFX 27 (Maven Central jars) | `GPL-2.0-only WITH Classpath-exception-2.0` | The jars carry no legal files |
| ONNX Runtime 1.30.0 | MIT | Statically linked third-party code (MPL-2.0 Eigen, BSL-1.0, curl, public-domain code and others); telemetry on by default |
| Jackson 3.2.3 (with the Jackson 2.x annotations) | Apache-2.0 | `jackson-core` bundles MIT code (FastDoubleParser) |
| sqlite-jdbc 3.53.4.0 | Apache-2.0 AND BSD-2-Clause | SQLite itself: public-domain dedication (not read at sqlite.org) |
| SLF4J 2.0.20 | MIT | |
| Logback 1.6.5 | EPL-2.0 OR LGPL-2.1-only | EPL-2.0 is incompatible with the GPL without a Secondary License designation |
| PP-OCRv6 models and dictionary | Apache-2.0 (PaddleOCR repository) | Hugging Face model-card licence not read (O-20) |
| Microsoft C/C++ runtime (DLLs from JDK and JavaFX; statically linked into the launcher and `msica.dll`) | Microsoft redistribution terms | O-12 |
| WiX parts embedded in the MSI | MS-RL | Binaries from WiX 6 on also carry the Open Source Maintenance Fee EULA (O-11) |

- Build, test and CI tools are never distributed. Several are not, or not clearly, GPL-compatible (JUnit, jqwik and JaCoCo under EPL-2.0 without a Secondary License; TestFX under EUPL-1.1). `gradle/actions` v6.4.0 runs a proprietary caching component unless `cache-provider: basic` is set.
- The corpus holds Star Citizen screenshots, and the app works with Cloud Imperium game content and names. basetool and basetool-android show the Fan Kit unit and pin it with tests; the extractor uses a diverging variant, and basetool's README is not compliant.
- Outside contributions are expected: corpus captures, misread reports, documents and later code.

## Options

**Licence**

| | A: GPL-3.0-only | B: GPL-3.0-or-later (chosen) | C: permissive (Apache-2.0 or MIT) | D: AGPL-3.0-or-later |
|---|---|---|---|---|
| Port extractor code | yes; the result becomes `-only` | yes | no | yes |
| Port basetool code | yes | yes; such a release is effectively `-only` until the code is relicensed | no | yes |
| Our code can flow into the owner's other projects | not into the extractor | into all of them | yes | not without the network clause |
| Later GPL versions | need a CLA grant or a §14 proxy | automatic | – | automatic |
| Shipped stack compatible | yes, with Logback under LGPL-2.1-only | yes, with Logback under LGPL-2.1-only | yes | yes |

**Contributions:** DCO only; DCO plus CLA (chosen); nothing until the first external contribution.

**Star Citizen content:** stay asset-free with a plain non-affiliation disclaimer; adopt the Fan Kit unit as in basetool (chosen).

## Evaluation

- **B** keeps the extractor's licence, lets code move into every project of the owner, follows the recommendation of ADR-0001 §5 and leaves later GPL versions open. Nothing shipped forces `-only`, as long as Logback is used under its LGPL-2.1-only option. A would block the flow into the extractor. C would forbid porting the GPL code that motivated the decision and give up copyleft for a community tool. D brings duties for network use that a desktop client does not need.
- **DCO plus CLA:** the DCO records the origin of every commit; the CLA is a one-time licence and patent grant that lets the project handle a licence-variant question or a GPL-3.0 §7 permission (O-12) without collecting every contributor's consent again. Its costs are a public roster (kept to name, handle, commit e-mail, date and CLA version) and a text derived from the Apache ICLA whose own licence is not yet confirmed (O-42). @greluc chose basetool's model, so all his projects work the same way.
- **Fan Kit unit:** it is the form @greluc already uses and tests in basetool and basetool-android. As recorded there, the Fankit Agreement asks for its clause 2(g) notice wherever Cloud Imperium material appears, and the Guidelines tie the logo to the §2b trademark line. The CIG documents themselves could not be re-read during the review (see "Unverified at the source").

## Decision

1. **Licence:** GPL-3.0-or-later. `LICENSE` is the FSF's plain-text GPL-3.0, unmodified (35,149 bytes, SHA-256 `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`, byte-identical to the extractor's `LICENSE` and to Debian's `common-licenses/GPL-3`); `LICENSES/GPL-3.0-or-later.txt` is a byte-identical copy, and CI compares the two.
2. **Holder and headers:** the copyright line names `Lucas Greuloch` (O-3); an external contributor adds one line of the same form for themself in each file they change. From M0 every source file starts with an SPDX copyright-text line and an SPDX licence-identifier line; Spotless enforces it on Java files, and `reuse lint` covers `package-info.java`, `module-info.java` and every other file. Configuration files carry no header and are annotated in `REUSE.toml` (placement in FXML and `.properties`: O-40).
3. **Contributions:**
   - Every commit carries a DCO 1.1 sign-off made with `git commit -s`, never typed by hand; a CI check enforces it from M0. GitHub noreply addresses are accepted when they match the commit author (O-33).
   - Every contributor signs the CLA (`CLA.md`, version 1.0, adapted from basetool's CLA, which is modelled on the Apache ICLA) once, through a pull request titled `cla: sign — <handle>` that adds a row to `docs/cla-signatures.md` and quotes the acceptance sentence verbatim. @greluc is the first roster entry. The PR template carries a CLA checkbox; a contribution is merged only when its author is on the roster.
   - In Claude sessions, commits are authored under @greluc's identity `Lucas Greuloch (greluc) <lucas.greuloch@gmail.com>` with `git commit -s`, and the model is credited in a `Co-Authored-By:` trailer (authorised by @greluc). The planning-phase commits made before this decision were re-authored under @greluc's identity with a sign-off on 2026-10-08; their `Co-Authored-By:` trailers name the model.
   - Commit messages follow Conventional Commits (O-5); pull requests are never squash-merged (O-29).
4. **Licence policy for shipped components** (canonical list: `CLAUDE.md`, "Stack rules"):
   - Allowed: Apache-2.0, MIT, ISC, BSD-2-Clause, BSD-3-Clause, LGPL-2.1-only, LGPL-2.1-or-later, LGPL-3.0-only, LGPL-3.0-or-later, MPL-2.0 without the "Incompatible With Secondary Licenses" notice, GPL-3.0-or-later, and `GPL-2.0-only WITH Classpath-exception-2.0` for the OpenJDK runtime and OpenJFX. EPL only where a GPL Secondary License is designated.
   - Logback is used under LGPL-2.1-only, never under EPL-2.0; `NOTICE` and the About dialog record the election.
   - GPL-3.0-only, every other licence and proprietary terms need @greluc's prior approval; AGPL is excluded by policy.
   - Fonts only under SIL OFL-1.1 (or another free font licence @greluc approves), unmodified, as separate files with their licence file.
   - Build, test and CI tools that are never distributed may use any OSI-approved licence, but nothing proprietary and nothing that runs a proprietary component by default: every `gradle/actions` step sets `cache-provider: basic` or `cache-disabled: true`.
   - The licence gate is `app.cash.licensee` 1.14.1 from M0 (O-16), with a recorded S-1 exception (published only on Maven Central; PGP key fingerprint `1D21 7F84 75EE E9F1 9AB8 DD6B 793F D575 1A0F 0780` in its pull request), an `allowUrl` alias for JavaFX under the deprecated ID `GPL-2.0-with-classpath-exception`, and a hand-kept list for what it cannot see.
5. **Notices and source duties:**
   1. The full GPL-3.0 text is in the repository and in every installer and app image.
   2. `NOTICE` lists every shipped component, generated from the shipped artefacts, not from upstream repositories: the JDK `legal/` files; the OpenJFX legal files, added by hand because the Maven jars carry none; ONNX Runtime's `ThirdPartyNotices.txt` (entries that are not in the shipped CPU natives are marked as not applicable); the IJG and FreeType credits; Logback's copyright line and LGPL-2.1 notice. jlink hides jar notices inside `lib/modules`, so a build step exports them into a visible notices directory, and a test checks that every entry has its files.
   3. The About dialog (Help menu) shows the Appropriate Legal Notices of GPL-3.0 §5(d): the copyright line, the no-warranty statement, that the program may be conveyed under GPL-3.0-or-later, and the licence text, bundled and readable offline. It also shows an "Open-source licences" view generated from the licence-gate report, Logback's notice, the source link and the Fan Kit unit. R-UI-18 in [01](../plan/01-requirements.md) records this.
   4. Corresponding Source: releases are built only from public, signed tags. Each GitHub release links the tagged source, which includes the build scripts, `build-logic`, lockfiles, verification metadata and the jpackage and WiX resources. It attaches the source archives of every GPL-2.0, LGPL, MPL-2.0 and MS-RL component it ships: the exact OpenJDK vendor build, OpenJFX `27-ga`, Logback, ONNX Runtime's MPL-2.0 parts such as Eigen, and the WiX parts. Permissive components are linked in `NOTICE` by pinned tag and Maven `-sources` jar (sqlite-jdbc also with its SQLite version). The JDK comes from one pinned vendor that publishes exact source archives (O-25). `.gitattributes` uses no `export-ignore`, so GitHub's source archives stay complete.
   5. Packaging: the jpackage `.deb` `copyright` file is replaced with a DEP-5 file via `--resource-dir`; a Windows `--license-file` adds no terms, because the GPL needs no acceptance; a plain app-image ZIP is published next to the MSI, so that the GPL work is also available outside the container that carries MS-RL parts.
   6. The Microsoft C/C++ runtime relies on the GPL-3.0 System Library definition for now. A legal review comes before the first release; a narrow GPL-3.0 §7 permission, if wanted, is added before the first external contribution (O-12).
   7. No credential is compiled into a released binary; every user enters their own UEX key and, if UEX requires one, app token.
6. **Content that is not under the GPL** is named in `REUSE.toml`, `NOTICE`, `README.md` and `corpus/README.md`:
   - corpus screenshots: Star Citizen game content of Cloud Imperium, `LicenseRef-Game-Screenshots`, no rights granted by this project (O-18, O-28); their `expected.json` transcriptions are under the project licence (O-32);
   - Fan Kit logo files: `LicenseRef-Fankit-Agreement`;
   - `CODE_OF_CONDUCT.md`: CC-BY-SA-4.0;
   - the Gradle Wrapper and the OCR models: Apache-2.0;
   - recorded UEX responses: UEX data, not committed until UEX allows it ([06](../plan/06-uex-api.md) open point 7, O-19).
7. **Star Citizen Fan Kit unit**, adopted as in basetool:
   - One coupled unit: the "Made By The Community" logo, the trademark line of the Fan Kit Guidelines §2b and the notice of the Fankit Agreement clause 2(g). The three parts are always rendered together and never folded away.
   - The notices are verbatim, byte-exact and English in every locale, never translated or tidied. They are pinned by their SHA-256 (§2b line `154deedbb183aeec2ba7f05c3d16416e55ceee634a36ec1f9ee4785b26467fa1`, clause 2(g) notice `703b53f047769fe1a5299b53454cc79a36e2b3a09129291baf2c6948ce740933`) and quoted in `CLAUDE.md` and `NOTICE` §4, taken from basetool's `FanKitComplianceMvcTest`.
   - Placements: the About dialog and the onboarding start screen in the app, and `README.md`; whether these satisfy the placement rule of Guidelines §2b and the "reasonably prominent location" of clause 2(g) is O-79. Tests pin the unit byte for byte; a change needs @greluc's approval.
   - The logo files are used unmodified and named by file name and SHA-256 in `NOTICE` §4; the app uses the band asset `made-by-the-community.png` (SHA-256 `b6869016da2220a61dc6306f8365dcce006ddb6d8b405545c6d4540166469b6b`). Which variant goes on which surface is O-27.
   - @greluc has accepted the Fankit Agreement (confirmed in chat on 2026-10-08); the checked kit version is `Fankit_2025_11_19`. The acceptance date is O-26.
8. **Porting:** code from basetool-sc-extractor and basetool may be copied or translated under the conditions in `CLAUDE.md` ("Working rules"): upstream notices preserved, origin and modification recorded in `NOTICE` (GPL-3.0 §5(a)), basetool code kept `GPL-3.0-only` until relicensed by its copyright holders, TDD and the project's conventions applied.

## Consequences

- **Documents changed with this ADR:**
  - `CLAUDE.md`, `CONTRIBUTING.md`, `CLA.md`, `docs/cla-signatures.md`, `CODE_OF_CONDUCT.md`, `.github/SECURITY.md`, `.github/CODEOWNERS`, the PR template and issue forms, `LICENSE`, `LICENSES/`, `NOTICE`, `REUSE.toml`, `docs/dependency-pins.md`, `docs/adr/0000-open-points.md` and the Fan Kit logo files under `docs/images/fankit/`.
  - 03 §5 gets the status line `§5 answered by ADR-0003 (2026-10-08)`.
  - 04 M0: the licence is decided; new M0 items are the licence gate, `reuse lint`, gitleaks, the DCO check, the English check, the header check for `package-info.java` and `module-info.java`, the `LICENSE` comparison and the ONNX Runtime telemetry spike.
  - 07 §3: porting from the extractor is now allowed under the porting rules.
  - 08 is a historical record: a new dated section records the decision.
  - 09 §3, §9, §10 and §11: the no-comments rule, the suppression register, the licence check and the Definition of Done items for `NOTICE` and `REUSE.toml`.
  - 10: S-1 (Licensee and the CI tools as recorded exceptions), S-5 (governance paths need the maintainer's review), S-6 (`reason` attribute), S-11 (WiX major chosen in M1, OSMF EULA), S-13 (no version comment, `docs/dependency-pins.md`), S-14 (permissions of the gate jobs), S-20 (`cache-provider: basic`), S-24 (no Gluon jmods; OpenJFX legal files), S-28 (owner bypass), S-29 (licence policy), S-31 (`.github/SECURITY.md`), new S-32 to S-37 and threats T11–T13.
  - 01: new R-UI-18 (About dialog), R-UI-19 (Fan Kit unit), R-NF-12 (no telemetry), R-DOC-3, R-DOC-4, R-SEC-10 and R-SEC-11; updated R-DOC-2, R-L10N-1, R-NF-2, R-NF-4 and R-SEC-3. 02 §8: JavaFX from Maven Central jars and `javafx.cachedir` in the app's cache directory; 02 §9: the data-flow inventory, including ONNX Runtime's telemetry until it is mitigated.
  - `README.md`: a licence section and the Fan Kit unit; `corpus/README.md`: the licence paragraph; `docs/release-process.md`: source archives, notices check and the WiX EULA step.
  - `docs/prompts/design-system.md` planned its own ADR-0003; that ADR takes the next free number when the prompt is rewritten (pending; until then its status line marks the superseded parts).
- **Positive:** the published work has clear terms; the owner's GPL code can be reused here and this project's code in his other projects; contributors, agents and reviewers follow the same governance as in basetool.
- **Negative and risks:**
  - Every release carries compliance work: notices, source archives, the licence gate and a legal review before the first release.
  - Ported basetool code turns a release into effectively `GPL-3.0-only` until it is relicensed.
  - The CLA roster publishes contributors' names and commit addresses.
  - The Microsoft C/C++ runtime question stays open until the legal review (O-12).
  - The Fan Kit unit and the corpus screenshots depend on Cloud Imperium's terms, which were not re-read at the source (O-26 to O-28, O-79).
  - Whether AI-generated text and code are protected by copyright is unsettled; it matters for the copyright line and for relicensing statements (O-24).
- **Revisit** before the first release (legal review, notices, source archives), before the first external contribution (GPL-3.0 §7 permission, O-12), and when Cloud Imperium publishes a new Fan Kit version.

## Unverified at the source

The review environment could not reach gnu.org, spdx.org, developercertificate.org, contributor-covenant.org, robertsspaceindustries.com, huggingface.co, sqlite.org, wixtoolset.org and Microsoft's licensing pages. Therefore:

- the FSF compatibility verdicts and the GPL FAQ positions (EPL, MS-RL, OFL, CC-BY-SA-4.0) were taken from the licence texts themselves, not from the FSF;
- `LICENSE` matches the extractor's and Debian's copies, but was not compared with gnu.org's `gpl-3.0.txt`;
- the Fan Kit strings, the kit version and the placement and legibility rules come from basetool, basetool-android and their tests; the Fankit Agreement and Guidelines were not re-read;
- the DCO 1.1 text, the PP-OCRv6 model-card licences, SQLite's public-domain statement, the WiX licence terms and Microsoft's redistribution terms were not read at their source.

The register tracks these points: O-82 (the licence facts above, the GCC and mingw-w64 start-up code in the sqlite-jdbc Windows DLL and the Code of Conduct's copyright holder), O-20 (model-card licences) and O-26, O-27 and O-79 (Fan Kit documents).
