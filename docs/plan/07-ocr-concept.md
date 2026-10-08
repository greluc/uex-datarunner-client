# OCR concept

## 1. Observations on real screenshots (Patch City, Pyro)

The five screenshots supplied by the project owner (2000×1125, Buy and Local Market Value tab, scrolled) show the current terminal layout. What we infer from them:

| Element | Observation | Consequence |
|---|---|---|
| Header | "COMMODITIES" at the top left; "CURRENT BALANCE: ¤9,484,456 aUEC" at the top right | The header serves as an anchor. **Never transmit the balance**, redact it in the upload. |
| Left panel "YOUR INVENTORIES" | Dropdown with the current location ("PATCH CITY"), below it IN DEMAND / NO DEMAND / CANNOT SELL | **Location field = best source for the terminal assignment** (text in capital letters, letter-spaced) |
| Right panel "SHOP INVENTORY" | Tabs "Buy" and "Local Market Value"; the active tab is filled with a strong orange-red, the inactive one is dark | Determine the side (BUY/SELL) via the tab background (color or luminance), additionally via the OCR text |
| Sections | Buy: "IN STOCK" (expanded), below "OUT OF STOCK" (collapsed, "+"). Sell: "SELLABLE CARGO", "IN DEMAND" | Remember the section per card; consistency rules |
| Card | Icon · vertical status bar (color/fill level) · name · status text (colored) · on the right "SHOP QUANTITY" (Buy only) · "333 SCU" · "¤3,237/SCU" · "AVAILABLE CARGO SIZE (SCU)" with boxes [1][2][4][8][16][24][32] | Field positions relative to the card; second reader for the status from bar and color |
| Currency symbol | `¤` sits directly in front of the number, without a space | Cause of the "9" prefix in the original (F1): separate it geometrically |
| Numbers | Thousands separator comma ("6,000", "¤36,000/SCU"), whole aUEC | Structure parser (F3/F4) |
| Long names | "Recycled Material Composite" overlaps with "0 SCU" → glued together "Compos0iSCU" | Name resolution via fuzzy prefix, SCU regex inside the token (F6) |
| Scrolling | Cards cut off at the top and bottom edges (e.g. only "AVAILABLE CARGO SIZE" without a name; card without the cargo row) | Accept edge cards only with complete fields; merge via commodity ID |
| Dimmed cards | Fluorine (2 SCU, containers 8–32) and DynaFlex (13 SCU, containers 16–32) are dimmed. First interpreted as "fade-out at the viewport edge"; **corrected** after the Pyro Gateway screenshots (§1b): there, IRON is dimmed in the middle of the list (4 SCU < smallest container 8). | Hypothesis A14: dimmed = not buyable in any container size. Contrast normalization per card; the dimming is **not** an edge effect. For duplicates the following still applies: prefer the reading further from the edge (cut-off edge cards). |
| Hover highlight | One card has a red background (mouse over it) | Contrast normalization per card, not global |
| Font | Digits in HUD style (e.g. "16" resembles "lb", "4" resembles "Ч") | Classify cargo sizes via a closed set {1,2,4,8,16,24,32} and the order, do not read them freely |
| Interference | HUD elements at the edge ("94%"), scene in the middle | Crop to the panel before reading |
| Perspective | Slightly skewed or in perspective | Homography to a normalized size |

## 1b. Observations: Pyro Gateway (Stanton), blue theme

Five more screenshots (4× Buy scrolled, 1× Sell) are stored as the first public corpus entry in `corpus/public/pyro-gateway-stanton-01/` (balance redacted, transcription **not yet** verified by a human).

| Element | Observation | Consequence |
|---|---|---|
| Theme and font | Blue, **capital letters, monospace-like font**, unlike Patch City (orange, mixed case, rounded font) | At least two font and layout profiles; matching case-insensitive; the corpus must cover both |
| Location field | "PYRO GATEWAY" – **without** a system suffix, although the terminal is "Pyro Gateway (Stanton)" | Confirms F12: the location text alone is not unambiguous. Disambiguation via the assortment, the session context and weakly via the theme (hypothesis A13: blue = Stanton side); when in doubt, mandatory selection. |
| Scrolling **without overlap** | The 4 Buy screenshots join exactly page by page; no card appears twice | Stitching must **not** rely on overlap. Order and gaps come from the **scrollbar** (position and length of the thumb on the right) and the UEX assortment. Possible gaps between two images are shown as a warning. |
| Partial cards | At the bottom edge only the card header is visible (buy-1, buy-2); on the Sell side the price of HUMAN FOOD BARS is half cut off ("¤490/SCU") | A card only counts if its bottom edge is visible. A half-readable price must **not** be accepted. |
| Nearly identical names | "SHIP AMMUNITION – SIZE 1" … "SIZE 7" differ only by one digit; names wrap onto two lines | Fuzzy matching alone is dangerous here (see §2.5, rule for near-duplicate names) |
| Status colors | Buy: "VERY LOW INVENTORY" **red**; Sell: "VERY LOW INVENTORY" **green** | The color is inverted between the sides (consistent with `commodities_status`). The second reader for the status via color must know the side. |
| Sell side | Quantity without "SHOP QUANTITY", but with stock status (1,482 SCU, VERY LOW INVENTORY) | Indication that the number means the terminal's stock level (A10: rather `scu_sell_stock` than `scu_sell` – clarify with UEX) |
| Maximum values | Many goods 12,000 SCU, Argon 24,000 SCU at "MAX INVENTORY" | Plausibility witness: "MAX INVENTORY" ⇒ SCU equals the known maximum of this terminal or this commodity (from the UEX history) |

## 2. Pipeline in detail

### 2.1 Preprocessing and locate

1. **Downscale via box filter** to ~1/4 for the anchor search. basetool measured that bicubic loses hatched UI elements: 25 % vs. 83 % hit rate.
2. **Coarse OCR** (detection plus recognition only, on the downscaled image or on tiles) finds the text anchors "SHOP INVENTORY", "YOUR INVENTORIES", "COMMODITIES" and "AVAILABLE CARGO SIZE". Their positions define the panel geometry.
3. **Panel frame:** The orange frame lines of the panel are searched for via edge detection and line fitting to determine the four corners for the homography.
   - Fallback: corners from the text anchors plus layout profile.
   - Last fallback: manual.
4. **Homography** to a normalized width (e.g. 1000 px for the shop panel), then bilinear resampling.
5. **Theme-agnostic:** For recognition the max channel or luminance is used; color only serves for tab, status and hover.

### 2.2 OCR

- **Models:** PaddleOCR **PP-OCRv6 small** detection (DBNet) and recognition (CTC), ONNX export from Hugging Face (`PaddlePaddle/PP-OCRv6_small_{det,rec}_onnx`), Apache-2.0. SHA-256 is documented in `NOTICE`.
  - According to basetool these are ~10 MB + ~21 MB, the dictionary has 18,708 entries.
  - The exact files are downloaded and hashed ourselves when integrating them.
- **Start parameters** (from basetool, to be retuned on our own corpus):
  - shorter side ≥ 736, rounded to a multiple of 32
  - binarization 0.2, box threshold 0.45, unclip factor 1.4
  - recognition height 48
- **Checks:**
  - Number of dictionary classes against the model's output shape (protection against a wrong dictionary)
  - Model hash on load
- **Runtime:** ORT session lazy, close on inactivity; limit intra-op threads (the game runs in parallel).
- **Later option** (not 1.0): our own recognition model fine-tuned to the SC HUD font, if the corpus shows a need.

### 2.3 Layout

- Delimit cards via the recurring structure: name at the top left, "… SCU" at the top right, price "/SCU" on the right below it, label "AVAILABLE CARGO SIZE" at the bottom.
- Row clustering via vertical overlap, columns via x centers. Tolerances are given **relative** to the normalized width, not as fixed pixels.
- Screen order (`screenOrder`) via the y position.

### 2.4 Field parsers (pure, property-tested)

- **Price** (procedural, not a single regex; an earlier regex version rejected e.g. `96705/SCU` and numbers without separators):
  1. Strip the suffix `/SCU` (tolerant: `/5CU`, `SCU` without slash). If it is missing, the token is not a price.
  2. Remember an optional suffix `K`/`M`.
  3. If the first character is a non-digit character (`¤`, `@`, `¢`, …), it is discarded. If it is a digit from the confusion set of the currency symbol (`9`, `8`, `0`, to be determined on the corpus), **two candidates** are created: with and without this digit. If the geometric separation (§2.1) yields a separate glyph box in front of the number, only "without" is used.
  4. Evaluate separators structurally:
     - Groups of exactly 3 digits after `,`/`.`/space count as thousands separators.
     - A last separator followed by 1–2 digits is decimal.
     - Numbers without any separator are valid.
     - Contradictory patterns yield the finding `Unreadable`.
  5. All candidates go as `BigDecimal` to the scoring (§2.5); there the prior decides with the witness rule.
- **SCU:** `(\d[\d,.\s]*)\s*SCU`, also inside glued tokens.
- **Status:** against the status names from `commodities_status` and the localized names; the second reader is the status bar (fill level ≈ percentage band) or the text color.
- **Cargo sizes:** number of boxes plus OCR, matched against the ascending subsets of {1,2,4,8,16,24,32}.

### 2.5 Resolution and validation with UEX data

1. **Terminal:**
   - The location field is matched against all location names and yields the candidate terminals (`type=commodity`, `is_available_live`, not player-owned).
   - If there are several terminals at one location, they are checked against the assortment (Jaccard of the recognized commodity IDs against the assortment from `commodities_prices`).
   - In addition there is the session context.
2. **Commodity:**
   - First matched against the **terminal's assortment**, only then globally.
   - The score combines normalized Levenshtein with a prefix bonus.
   - Minimum distance to the second best; otherwise `Ambiguous`.
   - **Rule for near-duplicate names:** If the best candidates differ only in a short token (e.g. "SIZE 1" vs. "SIZE 7", digits, Roman numerals), this token must have been read **exactly**. Confusable digits (1/7, 6/8 …) lead to `Ambiguous`. Additional witness: the price order according to the UEX prior (for Ship Ammunition the price rises with the size).
3. **Price prior:**
   - Candidates (raw, without prefix character, confusable variants, K/M scaling) are scored against the last value or `price_*_avg_week` of this terminal and this commodity, tolerance `price_variation` %.
   - If **exactly one** candidate lies within the tolerance, it is proposed.
     - If it is the raw value, everything is clean.
     - If it is a repaired value, it is only accepted **automatically** (0.85) if an independent witness agrees (glyph topology positive or second reader/VLM equal). Without a witness it gets 0.75, i.e. **below the send threshold**; the user confirms by key press (R-VAL-2b).
   - If none or several lie within the tolerance, the raw value is proposed and flagged as `OutOfTolerance`/`Ambiguous`.
   - **Real price changes** are thus neither blocked nor silently "repaired back" to the old value.
   - **Without a prior** (R-VAL-2a): commodity average with double tolerance; otherwise no repair.
4. **Consistency:** "Out of Stock" ⇒ SCU 0; side ↔ section; `is_buyable`/`is_sellable`.
5. **Glyph topology** (hole counting for 0/6/8/9, concept from basetool, reimplemented) works only as a **veto** against repairs.

### 2.5b Stitching without overlap

1. Captures of the same terminal, the same side and the same time window are sorted by **scrollbar position**. The fallback is the capture time.
2. Cards are merged via the resolved `CommodityId`. If images overlap, that is an additional witness (values read twice must match), but not a prerequisite.
3. **Gap check:**
   - If the scrollbar sections do not cover the list without gaps, or if commodities are missing that UEX lists for the terminal, the report shows "possibly incomplete" together with the missing names.
   - Sending is still allowed; only what was read is then sent.

### 2.6 Confidence (rule-based)

**Send threshold: 0.80** (start value). Fields below it must be confirmed or corrected; a confirmation sets the field to "confirmed by the user".

| State | Confidence (start values, to be calibrated on the corpus) |
|---|---|
| Read identically by OCR and VLM, validated | 0.97 |
| Clean, within the prior band | 0.95 |
| Clean, but without prior (no reference value), without confusable digits | 0.85 |
| Repaired, unambiguous, **with independent witness** | 0.85 |
| Single-reader value (other reader unreadable), validated | 0.85 |
| Repaired only via the prior (without witness) | 0.75 → **confirm** |
| Outside the tolerance, otherwise plausible | 0.70 → **confirm** |
| Ambiguous / conflict between scans or readers | 0.60 → **select** |
| Unreadable / implausible | 0.30 → **correct** |

**Deviation vs. confidence – two separate dimensions:**

- *Confidence* says how reliably the value was **read**.
- *Deviation* says how strongly the value deviates from the **previous UEX state** (R-UI-10).

Both are computed separately (`DeviationLevel`: `EQUAL`, `MINOR`, `MAJOR`, `NO_REFERENCE`, each with the flag `referenceStale`) and displayed separately. For the submission gate the **stricter** of the two applies: `MAJOR` always requires a confirmation, even with high reading confidence. A transposed number that was read cleanly but deviates strongly from the UEX value is therefore never sent unchecked.

The report confidence equals the worst mandatory field, not the mean, so that individual errors are not "averaged away".

### 2.7 Optional AI second reader (VLM via Ollama, only while the game is closed)

**Why two readers?**

- basetool measured that **different** readers make different errors. Their two VLMs contradicted each other in 8 of 430 cells, but never agreed on the same wrong value.
- Running the same model twice, on the other hand, achieves nothing (0 of 5 errors found, 2 new ones created).
- Classic OCR and VLM are such a decorrelated pair.

The numbers come from the refinery domain; whether they transfer is checked by the bake-off (assumption A8).

**Flow:**

1. While the game is running, the classic OCR delivers results immediately. Reports with warnings are queued for the AI; the user can still correct and send them manually at any time.
2. When the game is closed (hysteresis), the AI queue processes the queued or all unsent reports. The input is the perspective-corrected panel crops (shop panel, location field), limited to an edge of approx. 1000–1500 px.
3. The result runs through the parser and vocabulary resolution (identical to OCR). Then OCR and VLM are **fused per field**, and only the fused result is validated once (prior, consistency, confidence). Afterwards the report is re-stitched.
4. If the game starts, the request is aborted and the model is unloaded; the jobs are retained.

**Prompt** (`vlm/src/main/resources/prompts/shop_panel_v1.txt`, versioned):

- describes the card layout (name, status text, quantity "… SCU", price "¤…/SCU", cargo boxes) and the active tab
- requires exact transcription ("digit by digit, correct nothing, `?` for unreadable")
- explicitly ignore the currency symbol and the balance
- Response format:

  ```
  TAB: Buy
  LOCATION: PATCH CITY
  | name | status | scu | price_per_scu | cargo_sizes |
  |---|---|---|---|---|
  | Omnapoxy | Medium Inventory | 333 | 3,237 | 1,2,4,8,16 |
  ```

**Why Markdown instead of a JSON schema?** basetool measured free Markdown output plus a deterministic parser against schema-enforced JSON: 0.9872 vs. 0.9821, with fewer semantic errors. This has to be confirmed on our own corpus. Ollama supports structured output (`format`); that is a bake-off variant.

**Fusion rules per field:**

| OCR | VLM | Result |
|---|---|---|
| Value a | same value a | a, confidence 0.97 "double-confirmed" |
| a | b ≠ a, differing in only one confusable position | Glyph topology and prior decide unambiguously, otherwise `Ambiguous` (both candidates in the UI) |
| a | b, strongly different | The candidate within the prior band wins only if exactly one lies within it; otherwise `Ambiguous` |
| unreadable | b | b as candidate; after validation 0.85 (single-reader value) if plausible, otherwise below the send threshold |
| a | unreadable | unchanged (OCR confidence) |
| Commodity/terminal resolution differs | | always `Ambiguous` → mandatory selection |

The AI **never** overwrites fields already confirmed or corrected by the user; deviations are only shown as a hint.

**Limits (honestly):**

- basetool states ~4 s/image for the 8B model on an RTX 5090 and ~53 s/image on the CPU.
- The hardware tiers in basetool are ≥ 12 GB VRAM (8B) and ≥ 8 GB (4B) respectively.
- Values for our panels must be measured.
- On weak hardware the AI is an "overnight" feature. The classic OCR therefore remains the primary path.

## 3. Insights adopted from basetool-sc-extractor (GPL-3.0 – concepts only)

| Adopted | Not adopted |
|---|---|
| PP-OCRv6 small via ORT, without OpenCV | Local VLM as the **primary** reader – for us only an **optional second reader while the game is closed** (§2.7), because it needs 8–12 GB VRAM or ~50 s/image on the CPU |
| Box filter downscale for the anchor search | Refinery-specific rules and color constants |
| Read numbers as text first, then parse deterministically | Fixed 4K geometry fallback (we use text anchors) |
| Confusable set and unambiguous repair with witnesses | |
| Glyph topology as veto | |
| Rule-based confidence instead of model self-assessment | |
| Stitching: downweight edge readings, flag conflicts | |
| Decorrelated second readers (different methods, not the same model twice) | |
| Eval harness: golden corpus outside the repo, digest test, candidate comparison via the pipeline result | |
| Crop dumps for visual inspection (a reskin broke the localization there unnoticed) | |
| Hint to the user: chromatic aberration to 0 | |

## 4. Measurement and test concept

- **Corpus structure:** `corpus/<id>/image.png` + `expected.json` (terminal, side, rows with all fields, `screenOrder`) + `meta.json` (resolution, theme, location, game version).
- **Measure readers separately:** The eval runs for "OCR only", "VLM only (model X)" and "fusion". The model recommendation follows from the fusion result on the corpus, not from the model card. VLM runs are opt-in (`UEXDR_VLM_HOST`), because CI has no GPU.
- **Metrics per field type:**
  - exactly right
  - correctly flagged (wrong, but flagged)
  - **silently wrong** (wrong and classified as confident) – the most important metric, target ≈ 0
  - runtime
- **CI** checks the public, redacted partial corpus. The private corpus is included locally via an environment variable (`UEXDR_CORPUS_DIR`).
- The **Patch City screenshots** are the first corpus entry. They must be checked in as files (redact the balance first) or be stored in the private corpus.
