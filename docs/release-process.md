# Release and hotfix process

> **Doc type:** Living spec — binding; followed from release 0.1 on. Last reviewed: 2026-10-09.

Implements R-NF-11 and the release-related parts of [10-supply-chain-security.md](plan/10-supply-chain-security.md). This is a plan; it is followed from release 0.1 on and refined with experience.

## Versioning

- SemVer: `MAJOR.MINOR.PATCH`.
  - PATCH: fixes and recognition-profile updates.
  - MINOR: features.
  - MAJOR: a change to persisted data after which a downgrade loses data or needs the user to act. Every format change still ships a tested forward migration (09 §6, §10).
- The app version is shown in the UI, written to the logs and sent in the User-Agent and the `X-Client-Version` header (R-API-7, O-86).
- `CHANGELOG.md` follows "Keep a Changelog" with an `[Unreleased]` section. Every user-visible change adds its entry in the same commit as the change: one to three sentences on what changed and why it matters to the user, plus the requirement ID ([CLAUDE.md](../CLAUDE.md)).
- The version comes only from the signed `v<MAJOR>.<MINOR>.<PATCH>` tag: CI passes it to Gradle, which writes it into the build information read by the UI, the logs and the User-Agent; nobody edits a version by hand, and local builds carry a fixed development version.

## Release checklist

1. `main` is green on Windows and Linux: `./gradlew check` (format check, Error Prone/NullAway, all tests, ArchUnit, JaCoCo gate); the full PIT run on the release commit meets the mutation-score gate (11 §B3); the OSV scan (S-19) on the release commit shows no blocking finding (runtime dependencies, CVSS ≥ 7.0 or unscored), apart from documented exceptions that have not expired.
2. Corpus evaluation (R-QA-3): no regression in any `gated` class; no new silently wrong field in any class; every required R-QA-2 class that is not `gated` is named "not verified" in the release notes; classes that exist only in the private corpus are evaluated locally with `UEXDR_CORPUS_DIR`. If the recommended Ollama model or the evaluated-model list changes, the bake-off report is linked in the release notes, and the resource (tag, digest(s), approximate download size, metrics; R-VLM-11, S-27) is updated in the same release.
3. Live smoke test against UEX with `is_production=0` and a test key and the tester's own app token (manual, opt-in): onboarding check, one manual report, one OCR report. Decided 2026-10-09 (O-89): the earlier "one withdrawal" step is removed until UEX confirms that `data_remove` works on `is_production=0` submissions ([06](plan/06-uex-api.md) open point 29), because a withdrawal must never touch production data.
4. Database migrations tested: an empty DB and the DB of the previous release (with test data) migrate to the new version; the previous release refuses the new schema (R-NF-11); the pre-migration backup of a DB with an un-checkpointed WAL passes `PRAGMA integrity_check`, and restoring it in the previous release sends Released and Queued reports back to Draft and marks OutcomeUnknown ones (see 'After a restore').
5. Check that `CHANGELOG.md`, `README.md` and the user guide (`docs/user/`) are complete. They are updated with every user-visible change in the same commit; this step only checks them.
6. Create a signed tag on protected `main`. The release workflow then runs read-only build jobs (installers, archives, SBOM, packaged-app smoke test) and a separate publish job without Gradle that computes SHA256SUMS, attests and publishes (S-14 to S-18). The release notes record the bundled JDK build and the installer tool versions (S-11). The release also publishes the plain Windows app image as a ZIP next to the MSI and attaches the source archives of R-DOC-4 (see 'Licence compliance'); the release notes link the tag, which is the Corresponding Source.
7. Verify the published artifacts:
   - `gh attestation verify` on one artifact per OS;
   - install or unpack **every** published artifact (MSI, ZIP, `.deb`, `tar.gz`) on a clean Windows and a clean Linux machine (or VM) and start the app; on Linux start it once in an X11 session and once in a Wayland session via XWayland (R-NF-1);
   - from 1.0 on, the accessibility walkthrough of R-UI-17: keyboard-only use of the main flows, Narrator or NVDA on Windows, 200 % text size at the minimum window size, and scale factors 100–250 % including a monitor pair with different scale factors.
   - licence duties (R-DOC-3, R-DOC-4, R-UI-18, R-UI-19): every artefact contains `LICENSE`, `NOTICE` and the notices directory; the `.deb` installs the DEP-5 `copyright` file; the MSI adds no licence terms; the About dialog shows the legal notices, the open-source licences and the source link of this version; the Fan Kit unit appears in the About dialog and on the onboarding start screen; the source archives are attached and match their pinned SHA-256 values.

## Licence compliance

The release duties that follow from the project licence (GPL-3.0-or-later, [ADR-0003](adr/0003-licence-and-contributions.md)) and from the licences of the bundled components. `NOTICE` lists every component.

- **Notices.** Every installer and archive ships `LICENSE`, `NOTICE` and the notices directory (R-DOC-3), assembled from the shipped artefacts, not from upstream repositories (the two can differ). The About dialog shows the Appropriate Legal Notices, Logback's copyright line with its LGPL-2.1 reference and the credits of `NOTICE` §6 (R-UI-18).
- **Source archives.** Every GitHub release links the tagged source and attaches the complete source of every shipped component under GPL-2.0, LGPL, MPL-2.0 or MS-RL (R-DOC-4): the exact source archive of the bundled OpenJDK vendor build (O-25), OpenJFX `27-ga`, Logback, Eigen at the commit ONNX Runtime pins, and the source of the WiX files in the MSI. A read-only build job fetches them from pinned URLs and checks them against committed SHA-256 values; the publish job attaches them. Permissive components are linked from `NOTICE` by pinned tag and Maven Central `-sources` jar. Whether ONNX Runtime's other native dependencies are archived as well is open.
- **Windows packages.** Next to the MSI the release publishes the plain app image as a ZIP, so the GPL work is also available outside the MSI, which embeds MS-RL files of WiX; the MS-RL text and the WiX source go with every MSI. The MSI adds no licence terms (default: no `--license-file`; GPL-3.0 §9 requires no acceptance).
- **WiX version and EULA.** The WiX major version is chosen in M1 after jpackage 27 has been tested with each candidate (O-11). From v6 on, the official WiX binaries come with the Open Source Maintenance Fee EULA; such a version is used only after the project owner has accepted the EULA explicitly, recorded here:

  | WiX version | OSMF EULA | Accepted by | Date |
  |---|---|---|---|
  | not chosen yet (M1) | – | – | – |

- **`.deb`.** jpackage's default `copyright` file is replaced via `--resource-dir` with a DEP-5 file that lists the bundled components and their licences.
- **Before the first release only:** a legal review of the Microsoft C/C++ runtime in the Windows packages (the DLLs from the JDK and JavaFX, and the runtime linked statically into the jpackage launcher and `msica.dll`). Until then the project relies on the GPL-3.0 System Library definition. Whether a narrow GPL-3.0 §7 additional permission is added is decided before the first external contribution (O-12).
- **Star Citizen Fan Kit.** `NOTICE` §4 records the checked kit version (`Fankit_2025_11_19`). The Fankit Agreement lets Cloud Imperium change its documents (clause 11, as recorded in basetool); when a newer kit is published, the project owner re-checks the unit (R-UI-19) against it before the next release.

## Security release

Users get fixes for the bundled runtime and libraries only through our releases (10 S-11, 03 §4).

1. **Triggers:**
   - each quarterly JDK security update (Oracle/OpenJDK Critical Patch Update in January, April, July and October; dates are published in advance): rebuild with the new Temurin update of the bundled JDK line (S-11; the scheduled pin check flags it);
   - the end of support of the bundled JDK line: JDK 27 support ends when JDK 28 ships (March 2027), so the move to JDK 28 is released before the April 2027 update;
   - a blocking OSV finding (S-19) against the latest release tag, or an advisory for a bundled native component (ONNX Runtime, sqlite-jdbc, JavaFX);
   - a confirmed report via `.github/SECURITY.md` (S-31).
2. **Procedure:** a PATCH release from `main` with the full release checklist, including the corpus evaluation (a JDK update can change image decoding). Target: published within 7 days of the JDK update or of the confirmed report (start value). The CHANGELOG and the release notes get a 'Security' entry; a confirmed vulnerability in our own code also gets a GitHub Security Advisory.

## Upgrade and downgrade

- **Backup:** before every migration the app, holding the single-instance lock (R-NF-10), opens the DB (this recovers a hot journal or WAL) and writes the backup through SQLite (`VACUUM INTO 'backup/<timestamp>-v<old>.db'` or the online backup API), never as a file copy. It checks the backup with `PRAGMA integrity_check` and aborts the migration if that fails, then writes a marker row (`backup_meta`: taken at, from version) into the backup. Every build from 0.1 on writes this marker. The config file is saved next to it under the same timestamp. Working copies are not backed up; a missing one is shown as 'image no longer available'. The last 3 backups are kept. An `adapter-storage` test takes a backup from a DB with an un-checkpointed WAL.
- **Downgrade:** an older build that finds a newer schema version refuses to start with a clear message and never writes to the newer database. It offers **Restore backup** for the most recent compatible backup. The dialog shows the backup date and what is lost since then (history and withdrawal IDs, cooldowns, queue, processed-file register). The newer DB file is kept under a new name, never overwritten. Every build from 0.1 on has this action.
- **After a restore:** a build that opens a DB carrying a `backup_meta` marker treats it as restored, also after a manual file copy. Released reports go back to Draft by `cancel`, and queued jobs (including waiting, held or paused ones) are returned as after any restart (`SubmissionReturned`, R-SUB-2). Both get the finding `PossiblyAlreadyReceived` ('may have been sent by a newer version after <taken at>') and need explicit confirmation before release. `OutcomeUnknown` reports keep their state (there is no transition to Draft, 11 §A3), get the same finding and are resolved via `data_info` or by the user (R-SUB-9). The import and catch-up scans import files modified after <taken at> only with the finding 'possibly already submitted'. Then the marker is cleared. The hard observation-age limit (R-VAL-6) bounds the remaining risk.
- **Config file:** carries a schema version and has its own migration, with the same rules; it is backed up together with the DB.

## Star Citizen patch day (hotfix process)

1. **Detection.** The `UnvalidatedGameVersion` cap (R-OCR-19) protects UEX once the app has observed the new UEX game version (R-CAP-3b): from then on, recognised values of the new version are not sendable without confirmation. Before UEX publishes the new version, or if a patch keeps the version string, only the deviation checks (R-UI-10) apply (A17). Note when UEX published the new version, to verify that assumption.
2. **Capture.** On the new version, capture fresh screenshots for at least one blue and one orange terminal, buy and sell, and add them to the corpus (public after redaction, `corpus/README.md`). If the patch notes mention graphics options, HDR, the renderer, upscaling or screenshots, also capture one paired HDR on/off set with the SC screenshot key (if an HDR display is available; otherwise note the unchecked HDR behaviour in the changelog), and re-check every in-game setting name quoted in the user guide and the in-app hints (R-DOC-1, R-OCR-17, R-OCR-18). Outdated tips are corrected in the same PATCH release. Transcribe each new capture into `expected.json` with `conditions.gameVersion` set and have the transcription independently verified (`verified: true`) before step 3; unverified captures never validate a version (R-OCR-19).
3. **Measure.** Run the corpus evaluation (the eval disables the `UnvalidatedGameVersion` cap, 07 §4):
   - If no existing class regresses, add the new game version to each layout profile whose verified new-version captures meet the R-OCR-19 criteria (no silently wrong field, accuracy not below the profile's frozen baseline), and make a PATCH release. Profiles whose theme was not captured and verified on the new version keep the cap.
   - If something regresses, fix the layout profile or recognition rule test-first (11 §B1), then make a PATCH release.
4. **Target.** A validated profile within a few days of a patch. Until then the cap stays in place; manual confirmation still works.

## UEX API change

1. Contract tests with recorded responses fail, or users report errors (issue template with diagnostics bundle).
2. Verify the change against the live API with `is_production=0`; update `docs/plan/06-uex-api.md`.
3. Fix test-first and make a PATCH release.
4. If the faulty version sent bad data, coordinate with UEX: User-Agent and `X-Client-Version` identification (R-API-7), withdrawal via `data_remove` per row if needed (only rows not yet consolidated, R-SUB-4).
5. An HTML (non-JSON) HTTP 404 on a write holds the whole queue as "API changed" (R-SUB-11) until a PATCH release adapts the client or the user retries.
