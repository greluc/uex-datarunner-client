# Release and hotfix process

Implements R-NF-11 and the release-related parts of [10-supply-chain-security.md](plan/10-supply-chain-security.md). This is a plan; it is followed from release 0.1 on and refined with experience.

## Versioning

- SemVer: `MAJOR.MINOR.PATCH`.
  - PATCH: fixes and recognition-profile updates.
  - MINOR: features.
  - MAJOR: a change to persisted data after which a downgrade loses data or needs the user to act. Every format change still ships a tested forward migration (09 §6, §10).
- The app version is shown in the UI, written to the logs and sent in the User-Agent (R-API-7).
- `CHANGELOG.md` follows "Keep a Changelog".

## Release checklist

1. `main` is green on Windows and Linux: `./gradlew check` (format check, Error Prone/NullAway, all tests, ArchUnit, JaCoCo gate); the full PIT run on the release commit meets the mutation-score gate (11 §B3); the OSV scan (S-19) on the release commit shows no blocking finding (runtime dependencies, CVSS ≥ 7.0 or unscored), apart from documented exceptions that have not expired.
2. Corpus evaluation (R-QA-3): no regression in any `gated` class; no new silently wrong field in any class; every required R-QA-2 class that is not `gated` is named "not verified" in the release notes; classes that exist only in the private corpus are evaluated locally with `UEXDR_CORPUS_DIR`. If the recommended Ollama model or the evaluated-model list changes, the bake-off report is linked in the release notes, and the resource (tag, digest(s), approximate download size, metrics; R-VLM-11, S-27) is updated in the same release.
3. Live smoke test against UEX with `is_production=0` and a test key (manual, opt-in): onboarding check, one manual report, one OCR report, one withdrawal.
4. Database migrations tested: an empty DB and the DB of the previous release (with test data) migrate to the new version; the previous release refuses the new schema (R-NF-11); the pre-migration backup of a DB with an un-checkpointed WAL passes `PRAGMA integrity_check`, and restoring it in the previous release sends Released and Queued reports back to Draft and marks OutcomeUnknown ones (see 'After a restore').
5. Update the changelog and the user guide (`docs/user/`).
6. Create a signed tag on protected `main`. The release workflow then runs read-only build jobs (installers, archives, SBOM, packaged-app smoke test) and a separate publish job without Gradle that computes SHA256SUMS, attests and publishes (S-14 to S-18). The release notes record the bundled JDK build and the installer tool versions (S-11).
7. Verify the published artifacts:
   - `gh attestation verify` on one artifact per OS;
   - install or unpack **every** published artifact (MSI, ZIP, `.deb`, `tar.gz`) on a clean Windows and a clean Linux machine (or VM) and start the app; on Linux start it once in an X11 session and once in a Wayland session via XWayland (R-NF-1);
   - from 1.0 on, the accessibility walkthrough of R-UI-17: keyboard-only use of the main flows, Narrator or NVDA on Windows, 200 % text size at the minimum window size, and scale factors 100–250 % including a monitor pair with different scale factors.

## Security release

Users get fixes for the bundled runtime and libraries only through our releases (10 S-11, 03 §4).

1. **Triggers:**
   - each quarterly JDK security update (Oracle/OpenJDK Critical Patch Update in January, April, July and October; dates are published in advance): rebuild with the new Temurin update of the bundled JDK line (S-11; the scheduled pin check flags it);
   - the end of support of the bundled JDK line: JDK 27 support ends when JDK 28 ships (March 2027), so the move to JDK 28 is released before the April 2027 update;
   - a blocking OSV finding (S-19) against the latest release tag, or an advisory for a bundled native component (ONNX Runtime, sqlite-jdbc, JavaFX);
   - a confirmed report via `SECURITY.md` (S-31).
2. **Procedure:** a PATCH release from `main` with the full release checklist, including the corpus evaluation (a JDK update can change image decoding). Target: published within 7 days of the JDK update or of the confirmed report (start value). The CHANGELOG and the release notes get a 'Security' entry; a confirmed vulnerability in our own code also gets a GitHub Security Advisory.

## Upgrade and downgrade

- **Backup:** before every migration the app, holding the single-instance lock (R-NF-10), opens the DB (this recovers a hot journal or WAL) and writes the backup through SQLite (`VACUUM INTO 'backup/<timestamp>-v<old>.db'` or the online backup API), never as a file copy. It checks the backup with `PRAGMA integrity_check` and aborts the migration if that fails, then writes a marker row (`backup_meta`: taken at, from version) into the backup. Every build from 0.1 on writes this marker. The config file is saved next to it under the same timestamp. Working copies are not backed up; a missing one is shown as 'image no longer available'. The last 3 backups are kept. An `adapter-storage` test takes a backup from a DB with an un-checkpointed WAL.
- **Downgrade:** an older build that finds a newer schema version refuses to start with a clear message and never writes to the newer database. It offers **Restore backup** for the most recent compatible backup. The dialog shows the backup date and what is lost since then (history and withdrawal IDs, cooldowns, queue, processed-file register). The newer DB file is kept under a new name, never overwritten. Every build from 0.1 on has this action.
- **After a restore:** a build that opens a DB carrying a `backup_meta` marker treats it as restored, also after a manual file copy. Released reports go back to Draft by `cancel`, and queued jobs (including waiting, held or paused ones) are returned as after any restart (`SubmissionReturned`, R-SUB-2). Both get the finding `PossiblyAlreadyReceived` ('may have been sent by a newer version after <taken at>') and need explicit confirmation before release. `OutcomeUnknown` reports keep their state (there is no transition to Draft, 11 §A3), get the same finding and are resolved via `data_info` or by the user (R-SUB-9). The import and catch-up scans import files modified after <taken at> only with the finding 'possibly already submitted'. Then the marker is cleared. The hard observation-age limit (R-VAL-6) bounds the remaining risk.
- **Config file:** carries a schema version and has its own migration, with the same rules; it is backed up together with the DB.

## Star Citizen patch day (hotfix process)

1. **Detection.** The `UnvalidatedGameVersion` cap (R-OCR-19) protects UEX once the app has observed the new UEX game version (R-CAP-3b): from then on, recognized values of the new version are not sendable without confirmation. Before UEX publishes the new version, or if a patch keeps the version string, only the deviation checks (R-UI-10) apply (A17). Note when UEX published the new version, to verify that assumption.
2. **Capture.** On the new version, capture fresh screenshots for at least one blue and one orange terminal, buy and sell, and add them to the corpus (public after redaction, `corpus/README.md`). If the patch notes mention graphics options, HDR, the renderer, upscaling or screenshots, also capture one paired HDR on/off set with the SC screenshot key (if an HDR display is available; otherwise note the unchecked HDR behaviour in the changelog), and re-check every in-game setting name quoted in the user guide and the in-app hints (R-DOC-1, R-OCR-17, R-OCR-18). Outdated tips are corrected in the same PATCH release. Transcribe each new capture into `expected.json` with `conditions.gameVersion` set and have the transcription independently verified (`verified: true`) before step 3; unverified captures never validate a version (R-OCR-19).
3. **Measure.** Run the corpus evaluation (the eval disables the `UnvalidatedGameVersion` cap, 07 §4):
   - If no existing class regresses, add the new game version to each layout profile whose verified new-version captures meet the R-OCR-19 criteria (no silently wrong field, accuracy not below the profile's frozen baseline), and make a PATCH release. Profiles whose theme was not captured and verified on the new version keep the cap.
   - If something regresses, fix the layout profile or recognition rule test-first (11 §B1), then make a PATCH release.
4. **Target.** A validated profile within a few days of a patch. Until then the cap stays in place; manual confirmation still works.

## UEX API change

1. Contract tests with recorded responses fail, or users report errors (issue template with diagnostics bundle).
2. Verify the change against the live API with `is_production=0`; update `docs/plan/06-uex-api.md`.
3. Fix test-first and make a PATCH release.
4. If the faulty version sent bad data, coordinate with UEX: User-Agent identification (R-API-7), withdrawal via `data_remove` if needed.
