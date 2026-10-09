# OCR Test Corpus

This directory contains the **public** part of the golden corpus (see `docs/plan/07-ocr-concept.md` §4). The private part lives outside the repo and is included via `UEXDR_CORPUS_DIR`.

## Rules

- The balance ("CURRENT BALANCE") is **always redacted** before an image ends up here. No player names, no chat windows.
- Each entry has a directory `<location>-<nr>/` with the images and `expected.json`.
- `expected.json` stays at `"verified": false` until a human has independently checked the transcription. Unverified entries count in the eval only as "smoke", not as a golden metric. Unverified entries and entries with `gameVersion: null` never validate a game version for a layout profile (R-OCR-19).
- **Original screenshots** (lossless, original resolution) are preferred. Images uploaded via chat are probably scaled and recompressed; this is noted in `source` (schema version 2: `sourceFidelity`).
- An entry with `sourceFidelity: reencoded-unknown` (assumption A6, e.g. `pyro-gateway-stanton-01`) never counts for a resolution or capture-method class. It counts only for its theme and text-height band and as a source for synthetic variants.
- **Real class captures go into the public corpus by default**, after redaction, so that CI can gate their classes. A capture is added only if the contributor took it themselves or the person who captured it consented, and the contributor agrees to its publication in `corpus/public/` ([CLA](../CLA.md) §6, [CONTRIBUTING.md](../CONTRIBUTING.md)). The private corpus holds only captures that cannot be redacted without destroying the evaluated region. A class that exists only privately has the status `insufficient` in CI and is evaluated locally before every release (`docs/release-process.md`).
- **Redaction** uses filled rectangles over the balance and personal data (player names, chat) only. It must not touch the shop panel, the location field or the anchors, and it must not re-encode the rest of the image. PNG originals stay PNG. JPEG originals are redacted with a block-level lossless wipe (e.g. `jpegtran -wipe`); if that is not possible, they are decoded once and stored as PNG, and `sourceFidelity` records it.
- **Schema version 2** of `expected.json` (07 §4) adds per-capture `conditions` (capture method, HDR, renderer, resolutions, …) and a `measured` object, and moves the section to the card (`sectionHeaders` per capture; `section` and, for partial cards, `partialEdge` per card). Unknown values are `null`, never guessed. `pyro-gateway-stanton-01` is migrated in M2 with unknown values set to `null`.
- Schema version 2 entries also carry the expected UEX IDs (`expectedTerminalId`, `expectedCommodityId`), capture times (`capturedAt`) and a frozen `reference/` directory of recorded, anonymised UEX responses (07 §4). Without them an entry counts only as smoke for prior-dependent metrics. Whether UEX data may be published here is open (06 open point 7); until it is cleared, `reference/` directories live in the private corpus under the same entry id.
- **"Report a misread" exports** (R-QA-5) become a new entry only if the contributor allows it in the misread form, under the same consent rule. They contain redacted panel crops, not full frames, so, like `reencoded-unknown` entries, they count only for their theme and text-height band, never for a resolution or capture-method class. The exported values stay `"verified": false` until a maintainer has checked them. The exported reference snapshot goes into the entry's `reference/` directory with `priorProvenance`, in the private corpus until 06 open point 7 is cleared.
- Corpus data outside the Git repository (Git LFS, a separate corpus repository, `UEXDR_CORPUS_DIR`) has a `SHA256SUMS` file per entry; the eval verifies it before running.
- If the public corpus grows beyond 200 MB in total (start value), the images move to Git LFS or to a separate corpus repository pinned by commit and SHA-256 manifest (decision by the project owner, O-43). CI never fetches unpinned corpus data.

## Licence

The images in `corpus/public/` are screenshots of Star Citizen. The game content they show is copyright Cloud Imperium Rights LLC and Cloud Imperium Rights Ltd. They are **not** covered by the project licence (GPL-3.0-or-later): [`REUSE.toml`](../REUSE.toml) marks them `LicenseRef-Game-Screenshots` ([`LICENSES/LicenseRef-Game-Screenshots.txt`](../LICENSES/LicenseRef-Game-Screenshots.txt)), and [`NOTICE`](../NOTICE) §3 lists them. They are here only as test data for the text recognition; this project grants no rights in them, and any use of the game content is subject to Cloud Imperium's terms. Whether those terms allow the screenshots in a public repository is not verified yet: the project owner checks the RSI Terms of Service and the Fankit and Fandom FAQ (O-28 in [`docs/adr/0000-open-points.md`](../docs/adr/0000-open-points.md)). If they do not, the images are removed.

- The transcriptions (`expected.json`) and every other text file here are under the project licence, GPL-3.0-or-later.
- The images are never shipped in the installers or app archives. They are part of the repository and of GitHub's source archive of every tag (the repository uses no `export-ignore`).
- Recorded UEX data (`reference/` directories, the reference snapshot of a misread export) gets its own licence marking only once UEX allows publishing it ([06](../docs/plan/06-uex-api.md) open point 7); until then it stays in the private corpus.
- The `REUSE.toml` patterns cover `.jpg`, `.jpeg` and `.png` files under `corpus/public/`; an image in any other format needs a new pattern in the same pull request.

## Entries

| Entry | Content | Status |
|---|---|---|
| `pyro-gateway-stanton-01` | Pyro Gateway (Stanton), blue theme: 4× Buy (scrolled, without overlap), 1× Sell | transcribed, not verified |
