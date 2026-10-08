# OCR Test Corpus

This directory contains the **public** part of the golden corpus (see `docs/plan/07-ocr-concept.md` §4). The private part lives outside the repo and is included via `UEXDR_CORPUS_DIR`.

## Rules

- The balance ("CURRENT BALANCE") is **always redacted** before an image ends up here. No player names, no chat windows.
- Each entry has a directory `<location>-<nr>/` with the images and `expected.json`.
- `expected.json` stays at `"verified": false` until a human has independently checked the transcription. Unverified entries count in the eval only as "smoke", not as a golden metric.
- **Original screenshots** (lossless, original resolution) are preferred. Images uploaded via chat are probably scaled and recompressed; this is noted in `source`.

## Entries

| Entry | Content | Status |
|---|---|---|
| `pyro-gateway-stanton-01` | Pyro Gateway (Stanton), blue theme: 4× Buy (scrolled, without overlap), 1× Sell | transcribed, not verified |
