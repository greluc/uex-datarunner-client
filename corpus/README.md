# OCR Test Corpus

This directory contains the **public** part of the golden corpus (see `docs/plan/07-ocr-concept.md` §4). The private part lives outside the repo and is included via `UEXDR_CORPUS_DIR`.

## Rules

- The balance ("CURRENT BALANCE") is **always redacted** before an image ends up here. No player names, no chat windows.
- Each entry has a directory `<location>-<nr>/` with the images and `expected.json`.
- `expected.json` stays at `"verified": false` until a human has independently checked the transcription. Unverified entries count in the eval only as "smoke", not as a golden metric.
- **Original screenshots** (lossless, original resolution) are preferred. Images uploaded via chat are probably scaled and recompressed; this is noted in `source` (schema version 2: `sourceFidelity`).
- An entry with `sourceFidelity: reencoded-unknown` (assumption A6, e.g. `pyro-gateway-stanton-01`) never counts for a resolution or capture-method class. It counts only for its theme and text-height band and as a source for synthetic variants.
- **Real class captures go into the public corpus by default**, after redaction, so that CI can gate their classes. Captures from other contributors are added only with their consent to publication. The private corpus holds only captures that cannot be redacted without destroying the evaluated region. A class that exists only privately has the status `insufficient` in CI and is evaluated locally before every release (`docs/release-process.md`).
- **Redaction** uses filled rectangles over the balance and personal data (player names, chat) only. It must not touch the shop panel, the location field or the anchors, and it must not re-encode the rest of the image. PNG originals stay PNG. JPEG originals are redacted with a block-level lossless wipe (e.g. `jpegtran -wipe`); if that is not possible, they are decoded once and stored as PNG, and `sourceFidelity` records it.
- **Schema version 2** of `expected.json` (07 §4) adds per-capture `conditions` (capture method, HDR, renderer, resolutions, …) and a `measured` object. Unknown values are `null`, never guessed. `pyro-gateway-stanton-01` is migrated in M2 with unknown values set to `null`.
- Corpus data outside the Git repository (Git LFS, a separate corpus repository, `UEXDR_CORPUS_DIR`) has a `SHA256SUMS` file per entry; the eval verifies it before running.
- If the public corpus grows beyond 200 MB in total (start value), the images move to Git LFS or to a separate corpus repository pinned by commit and SHA-256 manifest (decision by the project owner). CI never fetches unpinned corpus data.

## Entries

| Entry | Content | Status |
|---|---|---|
| `pyro-gateway-stanton-01` | Pyro Gateway (Stanton), blue theme: 4× Buy (scrolled, without overlap), 1× Sell | transcribed, not verified |
