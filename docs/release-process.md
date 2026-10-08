# Release and hotfix process

Implements R-NF-11 and the release-related parts of [10-supply-chain-security.md](plan/10-supply-chain-security.md). This is a plan; it is followed from release 0.1 on and refined with experience.

## Versioning

- SemVer: `MAJOR.MINOR.PATCH`.
  - PATCH: fixes and recognition-profile updates.
  - MINOR: features.
  - MAJOR: breaking changes to persisted data that cannot be migrated.
- The app version is shown in the UI, written to the logs and sent in the User-Agent (R-API-7).
- `CHANGELOG.md` follows "Keep a Changelog".

## Release checklist

1. `main` is green on Windows and Linux (`./gradlew check`: tests, ArchUnit, JaCoCo, PIT gate, OSV scan).
2. Corpus evaluation: no regression in any corpus class (resolution, HDR, theme); "silently wrong" ≈ 0 (R-QA-3).
3. Live smoke test against UEX with `is_production=0` and a test key (manual, opt-in): onboarding check, one manual report, one OCR report, one withdrawal.
4. Database migrations tested: an empty DB and the DB of the previous release (with test data) migrate to the new version; the previous release refuses the new schema (R-NF-11).
5. Update the changelog and the user guide (`docs/user/`).
6. Create a signed tag on protected `main`. The release workflow then builds the installers and archives and publishes SHA256SUMS, the SBOM and attestations (S-15 to S-18).
7. Verify the published artifacts:
   - `gh attestation verify` on one artifact per OS;
   - install on a clean Windows and a clean Linux machine (or VM) and start the app.

## Upgrade and downgrade

- **Backup:** before every migration, the SQLite file is copied to `backup/<timestamp>-v<old>.db`; the last 3 backups are kept.
- **Downgrade:** an older build that finds a newer schema version refuses to start with a clear message and the path of the most recent compatible backup. It never writes to the newer database.
- **Config file:** carries a schema version and has its own migration, with the same rules.

## Star Citizen patch day (hotfix process)

1. **Detection.** The `UnvalidatedGameVersion` cap (R-OCR-19) protects UEX automatically: after a patch, OCR values are not sendable without confirmation.
2. **Capture.** On the new version, capture fresh screenshots for at least one blue and one orange terminal, buy and sell, and add them to the private corpus.
3. **Measure.** Run the corpus evaluation:
   - If nothing regresses, add the new game version to the validated versions of the layout profiles and make a PATCH release.
   - If something regresses, fix the layout profile or recognition rule test-first (11 §B1), then make a PATCH release.
4. **Target.** A validated profile within a few days of a patch. Until then the cap stays in place; manual confirmation still works.

## UEX API change

1. Contract tests with recorded responses fail, or users report errors (issue template with diagnostics bundle).
2. Verify the change against the live API with `is_production=0`; update `docs/plan/06-uex-api.md`.
3. Fix test-first and make a PATCH release.
4. If the faulty version sent bad data, coordinate with UEX: User-Agent identification (R-API-7), withdrawal via `data_remove` if needed.
