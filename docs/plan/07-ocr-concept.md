# OCR concept

## 1. Observations on real screenshots (Patch City, Pyro)

The five screenshots supplied by the project owner (2000×1125, Buy and Local Market Value tab, scrolled) show the current terminal layout. What we infer from them:

| Element | Observation | Consequence |
|---|---|---|
| Header | "COMMODITIES" at the top left; "CURRENT BALANCE: ¤9,484,456 aUEC" at the top right | The header serves as an anchor. **Never transmit the balance**, redact it in the upload. |
| Left panel "YOUR INVENTORIES" | Dropdown with the current location ("PATCH CITY"), below it IN DEMAND / NO DEMAND / CANNOT SELL | **Location field = best source for the terminal assignment** (text in capital letters, letter-spaced) |
| Right panel "SHOP INVENTORY" | Tabs "Buy" and "Local Market Value"; the active tab is filled with a strong orange-red, the inactive one is dark | Determine the side (BUY/SELL) by comparing the two tab fills (relative luminance and saturation, §2.1 item 7), with the section and label text as witness |
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

1. **Anchor-search scale:** the coarse pass runs on a box-filtered copy at factor f = min(1, target height / image height), with a target height of 1080 px (start value). It never uses a fixed fraction and never upscales. basetool measured that bicubic loses hatched UI elements (25 % vs. 83 % hit rate with the box filter). A fixed factor such as 1/4 would shrink the anchors of a 1080p capture to ≈ 3 px, below the reading limit.
   - **Native-resolution retry:** the pass is repeated at native resolution if no anchor is found, or if the smallest anchor found ("SHOP INVENTORY"/"YOUR INVENTORIES") is below 8 px cap height at factor f. Only then does the scan become `NotLocated` or get `TextTooSmall`. Without this retry, a 4K capture from twice the distance (anchors ≈ 11 px native, ≈ 5.5 px after f = ½) would fail although it passes R-OCR-17.
   - **No detector size limit:** the frame is never downscaled further to satisfy a detector limit; PaddleOCR's pipeline default `max_side_limit` of 4000 would shrink 5120×1440 by 0.78 and 5760×1080 by 0.69. If a side exceeds the detector tile size (setting), the frame is processed in overlapping tiles. The overlap is at least the width of the widest anchor line (start value 0.3 × frame height), so that every anchor lies completely inside one tile.
   - **Ultrawide and multi-monitor:** the panel can sit far off-centre; the search covers the full width, there is no centre crop.
2. **Coarse OCR** (detection plus recognition only, on the downscaled image or its tiles) finds the text anchors "SHOP INVENTORY", "YOUR INVENTORIES", "COMMODITIES" and "AVAILABLE CARGO SIZE". Their positions define the panel geometry.
3. **Panel frame:** the frame lines of the panel are searched for via luma-gradient edges and line fitting, with thresholds relative to the panel's own gradient percentiles; no theme colour is used. The result gives the four corners for the homography.
   - Fallback: corners from the text anchors plus layout profile.
   - Last fallback: manual.
4. **Homography and resampling:**
   - The working size is chosen so that the reference glyph (R-OCR-17) reaches a fixed target height (start value 22 px, via the layout-profile ratio; ≈ 1000 px shop-panel width for the Stanton theme). Layout tolerances are fractions of this size.
   - Resampling is bilinear where the local scale is ≥ 1 and uses an area (box) prefilter where it is < 1.
   - The normalized panel is used for detection and layout only. Recognition crops follow PaddleOCR's own two-step path on the **original raster**: first a perspective crop at about 1:1 scale, then a resize to height 48. They are not cut from the already resampled normalized panel.
   - The resize to height 48 reproduces the training preprocessing: OpenCV `INTER_LINEAR` semantics (pixel-centre convention), BGR channel order, (x/255 − 0.5)/0.5, and right padding with 0 in the normalized space (mid-grey). A golden test against reference tensors pins this.
   - No bicubic or Lanczos pre-upscaling: in the preliminary measurement (one theme, third-party export) it brought no gain and more confident errors.
5. **Channels:** the recognizer gets RGB in the trained (BGR) order. Classic edge and geometry steps use luma Y by default, because 4:2:0 JPEG halves the chroma resolution and blurs coloured strokes in the max channel. The max channel remains an evaluated alternative per layout profile, because saturated blue or red strokes have low luma contrast (F8). Colour is used only for tab, status, stock bar and hover; it is averaged over stroke or fill areas and never sampled per pixel.
6. **Text-size gate (R-OCR-17):** the reference glyph height (cap height of the price digits) is estimated from the anchors in the original image via the layout-profile ratios, then measured on the price digits of every fully visible card; the smallest value counts, because perspective makes cards differ by 5–10 %. Limits and findings (`SmallText`, `TextTooSmall`) are defined in R-OCR-17. Upscaling cannot invent detail. If the decoder reduced the image to the pixel budget (`Downscaled`), the gate uses the raster that is actually read.
7. **HDR and tone handling (R-OCR-18), in this order:**
   1. *Decode* (adapter): 8-bit sRGB, or the file is set aside as HDR-encoded (R-CAP-8).
   2. *Locate* uses only tone-invariant cues: text anchors, and luma-gradient edges with thresholds relative to the frame's own gradient percentiles. It never uses absolute luminance or colour.
   3. *Per-panel tone analysis* runs on the located shop panel and the location field, and per card for dimmed or hover cards. It measures the black point (p1), the white point (p99.5), the share of stroke pixels with at least one channel at the maximum code value **compared with the layout profile's SDR reference share**, and the median saturation relative to the layout profile's SDR reference. Tone classes (start thresholds, calibrated on paired captures per capture method):
      - `NORMAL`;
      - `LOW_CONTRAST`: raised black and/or compressed range; recoverable → `LowContrastCapture`;
      - `PQ_SUSPECTED`: raised black, white point well below full scale, saturation strongly reduced. This is the expected signature of an untagged PQ PNG (*assumption*, to be verified with NVIDIA-overlay captures) → `LowContrastCapture` with this subtype;
      - `CLIPPED`: the saturated stroke share is clearly above the SDR reference, i.e. more than intact glyph cores explain; detail is lost → `ClippedHighlights`.
   4. *Normalization* (linear stretch p1 → 0, p99.5 → 1, optional gamma if the mid-tones are compressed) feeds the colour decisions and the classic geometry steps (card borders, scrollbar, glyph topology). By default the **recognizer gets the un-stretched crop**. A stretched recognizer input is an eval switch; M2 decides it from the eval results, possibly per tone class.
   5. *Confidence cap:* the scan still runs. Start rule: under `LowContrastCapture` or `ClippedHighlights` every numeric field gets at most "confirm". Once confidence exists (M3), the cap is kept or lifted **per finding and class**: for `LOW_CONTRAST` it is lifted only if "silently wrong" is 0 for that class; `PQ_SUSPECTED` and `ClippedHighlights` keep the cap. A preliminary experiment during planning (third-party PP-OCRv6 small export, unverified corpus entry) found no recognition loss from an affine washout at 6–10 px price height; clipping and bloom were not simulated. The finding text carries the capture advice for the detected class (R-DOC-1).
   6. *Colour decisions are relative within the panel:*
      - Active tab: the tab whose fill has both the higher normalized luminance and the higher saturation, by a margin (setting). This is a comparison, not a reference colour. If the two measures disagree or the margin is not reached, the tab reader abstains, and the side comes from the section and label text ("IN STOCK"/"SHOP QUANTITY" vs. "SELLABLE CARGO"/"IN DEMAND").
      - Status colour: the nearest hue range for this side from the layout profile, with saturation measured relative to the panel's own saturation distribution. Hue tolerances are calibrated on paired captures; a simulated PQ-as-sRGB capture shifted hue by up to ≈ 10°.
      - UEX `commodities_status` colours are only a weak hint (*assumption*: they are web colours, not in-game colours).
      - The status colour reader stays a **second reader** (R-OCR-10) and abstains on `CLIPPED` or `PQ_SUSPECTED` panels. The tab comparison stays active there because it is relative (the order held in the simulation), but its result then needs the text witness.
   7. Bright terminals and glare (predecessor fix in v0.6.8.4, F8/F15 context) are handled by the same per-panel analysis; the crop excludes areas outside the panel.

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
- **Currency glyph separation:** inside the price token, the character boxes are split at gaps; a leading box whose width/height ratio and position match the `¤` glyph profile (calibrated on the corpus) is marked as the currency glyph.

### 2.4 Field parsers (pure, property-tested)

- **Price** (procedural, not a single regex; an earlier regex version rejected e.g. `96705/SCU` and numbers without separators):
  1. Strip the suffix `/SCU` (tolerant: `/5CU`, `SCU` without slash). If it is missing, the token is not a price.
  2. Remember an optional suffix `K`/`M`.
  3. If the first character is a non-digit character (`¤`, `@`, `¢`, …), it is discarded. If it is a digit from the confusion set of the currency symbol (`9`, `8`, `0`, to be determined on the corpus), **two candidates** are created: with and without this digit. If the geometric separation (§2.3) yields a separate glyph box in front of the number, only "without" is used.
  4. **Letter–digit confusables** in the number part (B→8, O/D/Q→0, S→5, I/l/|→1, Z→2, G→6; the set is calibrated on the corpus) become digit candidates, and their positions are marked as confusable. Like the candidates of step 3, they go to the scoring with the witness rule (§2.5). A letter is read as a magnitude suffix only if it is exactly `K` or `M` at the end (step 2); `B` is never a suffix.
  5. Evaluate separators structurally:
     - Groups of exactly 3 digits after `,`/`.`/space count as thousands separators.
     - A last separator followed by 1–2 digits is decimal.
     - Numbers without any separator are valid.
     - Contradictory patterns yield the finding `Unreadable`.
  6. All candidates go as `BigDecimal` to the scoring (§2.5); there the prior decides with the witness rule.
- **SCU:** `(\d[\d,.\s]*)\s*SCU`, also inside glued tokens. The letter–digit confusable step of the price parser applies here too.
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
5. **Glyph topology** (hole counting for 0/6/8/9, concept from basetool, reimplemented) classifies a digit or **abstains**. Its role is defined once here and applies everywhere: a classification that **agrees** with a repair candidate counts as an independent witness (R-VAL-2b); a classification that **contradicts** it is a veto; abstention counts as neither.

### 2.5b Stitching without overlap

1. Captures of one group are sorted by **scrollbar position** (grouping by terminal, side, environment and time window is done by `ReportGrouper` in `reporting`, R-OCR-13). The fallback is the capture time.
2. Cards are merged via the resolved `CommodityId`. If images overlap, that is an additional witness (values read twice must match), but not a prerequisite.
3. **Gap check:**
   - If the scrollbar sections do not cover the list without gaps, or if commodities are missing that UEX lists for the terminal, the report shows "possibly incomplete" together with the missing names.
   - Sending is still allowed; only what was read is then sent.

### 2.6 Confidence (rule-based)

**Send threshold: 0.80** (start value). Fields below it must be confirmed or corrected; a confirmation sets the field to "confirmed by the user".

| State | Confidence (start values, to be calibrated on the corpus) |
|---|---|
| Read identically by OCR and VLM, validated | 0.97 |
| Clean read (no unresolved confusable digit) | 0.95 |
| Repaired, unambiguous, **with independent witness** | 0.85 |
| VLM-only value (OCR unreadable), validated | 0.85 |
| Repaired only via the prior (without witness), or confusable digit without any prior to check against (R-VAL-2a) | 0.75 → **confirm** |
| Ambiguous / conflict between scans or readers | 0.60 → **select** |
| Unreadable / implausible | 0.30 → **correct** |

**Per-digit certainty:** a reading counts as "clean" only if two conditions hold. First, the **minimum** per-digit probability over the digits of the number part reaches the digit threshold (start value 0.80, setting); separators and the `/SCU` suffix do not count, and the line mean is not used. The per-digit probability is the highest softmax probability among the CTC frames that emitted that digit. Second, the reference glyph height is ≥ 8 px (R-OCR-17). Otherwise the field is rated like "confusable digit without witness" (0.75 → confirm). Reason: small text produces confidently wrong readings that a mean hides. The model probability can only lower the rule-based confidence, never raise it (R-VAL-1).

**Caps from capture findings** (they only lower confidence): `UnvalidatedGameVersion` below the send threshold (R-OCR-19); `SmallText`, `LowContrastCapture` and `ClippedHighlights` at most 0.75 → confirm (R-OCR-17, R-OCR-18); `TextTooSmall` at most 0.30 → correct.

Confidence deliberately does **not** depend on whether a value lies within the prior tolerance – that is the job of the deviation dimension below. (An earlier version also lowered confidence for out-of-tolerance values, counting the same fact twice.)

**Deviation vs. confidence – two separate dimensions:**

- *Confidence* says how reliably the value was **read**.
- *Deviation* says how strongly the value deviates from the **previous UEX state** (R-UI-10).

Both are computed separately (`Deviation`: `EQUAL`, `MINOR`, `MAJOR`, `NO_REFERENCE`) and displayed separately. **Stale reference rule** (defined once here): if the prior is older than the staleness limit (default 7 days, R-UI-11), the level is lowered by one step (`MAJOR` → `MINOR`, `MINOR` → `EQUAL`) and the field shows "reference outdated"; `NO_REFERENCE` stays. The lowered level is the one that counts for I2. For the submission gate the **stricter** of the two applies: `MAJOR` always requires a confirmation, even with high reading confidence. A transposed number that was read cleanly but deviates strongly from the UEX value is therefore never sent unchecked.

The report confidence equals the worst mandatory field, not the mean, so that individual errors are not "averaged away".

### 2.7 Optional AI second reader (VLM via Ollama, only while the game is closed)

**Why two readers?**

- basetool measured that **different** readers make different errors. Their two VLMs contradicted each other in 8 of 430 cells, but never agreed on the same wrong value.
- Running the same model twice, on the other hand, achieves nothing (0 of 5 errors found, 2 new ones created).
- Classic OCR and VLM are such a decorrelated pair.

The numbers come from the refinery domain; whether they transfer is checked by the bake-off (assumption A8).

**Flow:**

1. While the game is running, the classic OCR delivers results immediately. Reports with warnings are queued for the AI; the user can still correct and send them manually at any time.
2. When the game is closed (hysteresis), the AI queue processes the flagged or all **Draft** reports (never released, queued or submitted ones). The input is the perspective-corrected panel crops (shop panel, location field), limited to an edge of approx. 1000–1500 px.
3. The result runs through the parser and vocabulary resolution (identical to OCR). Then OCR and VLM are **fused per field**, and only the fused result is validated once (prior, consistency, confidence). Afterwards the report is re-stitched.
4. If the game starts (mode Automatic), the request is aborted and the model is unloaded; the jobs are retained.

**Prompt** (`adapter-vlm/src/main/resources/prompts/shop_panel_v1.txt`, versioned):

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
| a | b ≠ a, differing in only one confusable position | If glyph topology classifies the digit (witness), that candidate wins with 0.85; if only the prior favours one candidate, it is proposed with **0.75 (confirm)** – never sendable on the prior alone (R-VAL-2b); otherwise `Ambiguous` (both candidates in the UI) |
| a | b, strongly different | `Ambiguous` (both candidates in the UI); the candidate within the prior band is pre-selected as a suggestion, but the field stays below the send threshold |
| unreadable | b | b as candidate; after validation 0.85 if plausible (this 0.85 applies only to a value read by the **VLM alone**), otherwise below the send threshold |
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

- **Corpus structure:** `corpus/public/<id>/` (or `$UEXDR_CORPUS_DIR/<id>/` for the private part) with the images and one `expected.json` (source and redaction info, location, theme, game version, per capture: side, section, cards with all fields and `screenOrder`, partial/dimmed flags, `verified` flag). Schema version 1: see `corpus/public/pyro-gateway-stanton-01/expected.json`.
- **Schema version 2** (introduced in M2) adds a `conditions` object **per capture** (not per entry, because a pair or ladder entry mixes conditions); entry-level values such as `gameVersion` are defaults that a capture may override. Fields:
  - `fileResolution`, `gameResolution`, `displayResolution`, `windowMode` (fullscreen/borderless/windowed), `fovSetting`;
  - `renderer` (DX11/Vulkan), `upscaler` (with percentage), `gammaBrightnessContrast`;
  - `windowsHdr` (on/off), `gameHdr` (on/off/unavailable), `refWhiteNits`, `peakNits`, `windowsSdrContentBrightness`;
  - `captureMethod` (`sc-key`, `gamebar-png`, `win-prtscn`, `snipping-tool`, `nvidia-overlay`, `steam`, `sharex`, `obs`, `spectacle`, `gamescope`, `other`) with `captureToolVersion` and `captureToolOptions` (e.g. HDR color corrector on);
  - `os`, `channel`, `gameVersion`;
  - `container` (jpeg/png8/png16/bmp), `transfer` (sRGB/PQ/scRGB), `sourceFidelity` (`original`, `original-redacted-lossless`, `reencoded-unknown`);
  - `ladder` (`{"id": …, "factor": …, "step": …}`).

  A `measured` object holds `priceCapHeightPx` (smallest fully visible card, source pixels, R-OCR-17), `displayHeightFraction` and `textHeightBand`. `tools/ocr-eval annotate` proposes these values from hand-placed anchors; a human confirms them. Unknown values are `null`, never guessed. The eval refuses captures without `conditions`; schema-1 entries are migrated with unknown fields set to `null`.
- **Measure readers separately:** The eval runs for "OCR only", "VLM only (model X)" and "fusion". The model recommendation follows from the fusion result on the corpus, not from the model card. VLM runs are opt-in (`UEXDR_VLM_HOST`), because CI has no GPU.
- **Corpus classes:** every `conditions` dimension, the text-height band (R-QA-2) and the theme form classes. The eval reports metrics and a status per class (R-QA-3), so that e.g. "4K + HDR + Game Bar PNG" cannot regress unnoticed behind a good average.
- **Synthetic class variants** (`tools/ocr-eval`, deterministic, parameters in a versioned `transforms.json`). Classes without enough real captures are covered by transforms of real corpus images. Transforms are pure functions on `ImageRaster`; JPEG re-encoding goes through the encoder/decoder adapters. Each output is tagged `synthetic:<id>(<parameters>)`, and its price cap height is recomputed from the source's measured value and the scale factor.

  | ID | Simulates | Parameters (start values) |
  |---|---|---|
  | `scale-box` | lower resolution, larger distance or wider FOV | area (box) downscale of the whole frame to a price cap height of 12, 10, 9, 8, 7, 6, 5 and 4.5 px; only steps below the source's own cap height |
  | `scale-nearest` | rendering without anti-aliasing (worst case) | as `scale-box`, nearest neighbour |
  | `upscale-bilinear` | 1440p/4K frame geometry, runtime, memory | ×1.5 and ×2 bilinear; never used to judge legibility (adds no detail) |
  | `canvas-wide` | 21:9, 32:9, windowed, multi-monitor | source frame placed left, centre and right in 3440×1440, 5120×1440 and 3840×1080 canvases; a source taller than the canvas is first box-downscaled to the canvas height (e.g. 2000×1125 → 1920×1080); the filler is a mirrored, blurred copy of the scene (no flat border that locate could exploit) |
  | `tone-lift` | washed-out SDR capture of an HDR desktop | sRGB code values mapped affinely to [0.30, 0.85] and [0.45, 0.80], then gamma 0.7 and 0.8, requantised to 8 bit |
  | `tone-clip` | naive 8-bit capture of the scRGB desktop (**assumption**, inferred from Microsoft's scRGB documentation, not a documented Windows behaviour) | linearise, multiply by SDR white / 80 nits (×1.5, ×2.5, ×3.75 for 120/200/300 nits), clip to 1.0, sRGB-encode |
  | `tone-pq-as-srgb` | untagged 8-bit PQ PNG read as sRGB (**assumption**, per the hdrfix README on the NVIDIA overlay) | linearise, map 1.0 to 203 nits (BT.2408 reference white), PQ inverse EOTF (SMPTE ST 2084), 8 bit, interpreted as sRGB |
  | `jpeg` | other encoders, re-encoded uploads | quality 75, 85, 95, 4:2:0 |
  | `combo` | realistic worst case inside the envelope | `scale-box` 8 px → `tone-lift` [0.30, 0.85] gamma 0.8 → `jpeg` 85 |

  - Synthetic variants are reported as their own classes and **never** count as coverage of an R-QA-2 class. They show robustness against the modelled effect only; in-engine low-resolution rendering, highlight bloom, hue shifts and tool-specific tone mapping still need real captures.
  - **Validity check:** for every real resolution or distance ladder (R-QA-2), the eval compares the `scale-box` variant of the highest-resolution capture with the real lower-resolution capture of the same view at equal price cap height (field accuracy and minimum per-digit probability). It reports the **reading limit** – the smallest price cap height at which price accuracy is still at the base step's level – once for synthetic and once for real. If the real limit is higher, the R-OCR-17 limits derived from synthetic variants are raised by the difference (rounded up to 0.5 px). Possible causes include the kiosk's render-to-texture resolution and TAA.
  - For each sweep step the eval also reports locate success and the anchor cap height after the locate downscale.
- **Metamorphic gate** (CI on Windows and Linux, all public entries; needs no verified transcription). The reference is the verified expected value if the entry is verified, and otherwise the pipeline output on the unmodified capture `img`. The envelope is the variants with price cap height ≥ the R-OCR-17 confirm limit, plus all `tone-*`, `jpeg`, `canvas-wide` and `upscale-bilinear` variants. For every variant `T(img)` **inside the envelope**:
  1. every field at or above the send threshold on `T(img)` equals the reference; a confident field with a different value is a *metamorphic silently-wrong* and fails the build. If the reference field is unreadable or below the send threshold (unverified entry), a confident value on `T(img)` is reported for manual review and does not fail the build;
  2. fields that drop below the send threshold count as "flagged" and are reported, but do not fail the build;
  3. the located panel quad of `T(img)`, mapped back into source coordinates, deviates by at most 1 % of the panel diagonal from the quad on `img` (start value);
  4. side, section, the number of complete cards and `screenOrder` are identical.

  For variants **below the lower limit**, every scan carries `TextTooSmall` or a locate/not-a-terminal finding (R-OCR-16), and no numeric field reaches the send threshold. Variants between the lower and the confirm limit may only produce numeric fields at "confirm" or lower.

  **Staging:** from M2, rules 3 and 4 gate, and the raw field strings of `T(img)` are compared with those of `img` and reported. The M2 slice has no confidence yet, so the send-threshold rules (1, 2 and below the envelope) become gates in M3 together with confidence. The gate shows invariance, not correctness; correctness comes from verified golden entries. It lets CI test the R-OCR-17/R-OCR-18 claims with the single public entry that exists today.
- **Capture protocol for pairs and ladders** (written in M2 as `docs/user/corpus-capture.md`, so contributors can follow it):
  1. Stand still at the terminal in interaction mode, with the cursor off the cards (no hover) and the same scroll position for all captures of a ladder. Hints and global chat off, `r_DisplayInfo` text off the screen. Keep the FOV setting fixed.
  2. Per step, change exactly one factor (R-QA-2 ladders). After changing HDR or a graphics setting, wait until the image is stable (start value ≥ 5 s; eye adaptation on in-world screens is unverified), then capture with every method of the ladder in a fixed order. Capture a whole ladder within a short window (start value 5 min), because stock and prices can change on a live server.
  3. Record the `conditions` (schema version 2) immediately, including tool version, Ref-White, peak brightness, the Windows "SDR content brightness" value and the renderer.
  4. Keep the originals byte-identical (no re-save, no crop). The public copy is redacted without touching the evaluated region (rules in `corpus/README.md`).
  5. Take SC-key HDR pairs with both renderers (DX11 and Vulkan) while both are selectable.
- **Pair consistency metric:** for each pair or ladder step (same view, one factor changed), the eval reports the fields whose value or send-threshold status differs from the base step. A field that is confident in both captures but has different values is a silently-wrong candidate in at least one of them; it is checked by hand, because the in-game value may have changed between the captures. This is detectable before the transcription is verified.
- **Resolution assumption check:** the resolution ladder reports the measured price cap height per step. It confirms or refutes two assumptions that the `scale-*` variants rely on: that text height scales with vertical resolution at a constant FOV setting, and that 3440×1440/5120×1440 match 2560×1440 (Hor+, community sources).
- **A16 decision from the pairs (M2):** for each capture method and renderer, the eval reports panel black point, white point, mid-tone median and the share of panel pixels at the maximum code value, for each HDR step against the Windows-HDR-off base. Each method is classified as `identical`, `tone-mapped`, `lifted` (washed out), `clipped` or `HDR container`; the result goes into A16 and the HDR advice of R-DOC-1. Until a method is classified, R-DOC-1 makes no recommendation for it.
- **Decode fixtures** (`adapter-files`; generated inside the test, no committed binary blobs):
  1. an 8-bit and a 16-bit PNG generated from the same public corpus image yield `ImageRaster`s that differ by at most 1 code value;
  2. a 16-bit PNG with a linear-RGB `iCCP` profile decodes to the same sRGB values as the plain PNG within 2 code values (this fails with `ImageIO.read` and with the JDK PNG reader alone, which only store `iCCP` as metadata);
  3. a 16-bit PNG with `cICP` 9/16/0/1 (PQ) is never read as sRGB: it is set aside with the R-CAP-8 reason;
  4. a JPEG with an APP2 ICC profile, a progressive JPEG and baseline JPEGs with 4:4:4 and 4:2:0 subsampling, all generated from the same source, decode to the source's sRGB values within a stated tolerance (start value: 3 code values mean absolute error);
  5. header-only stubs of JPEG XR, AVIF and OpenEXR are detected by magic bytes regardless of the file extension and get the R-CAP-8 "HDR format" message; JPEG data in a file named `.png` is decoded as JPEG;
  6. an input above the pixel budget is reduced at decode time with the `Downscaled` finding; a 7680×4320 input stays within the heap budget (R-OCR-17, R-NF-3).
- **Tone fixtures** (generated in tests from SDR corpus images, no HDR hardware needed): 8-bit untagged PQ-as-sRGB, linear ×1.5 and ×2.5 with clipping (SDR-brightness simulation), affine washout, PNG with `iCCP` (Display P3), and `gAMA` only. Expected: the R-CAP-8 outcome, the tone class of §2.1 item 7, and never a silently wrong field (golden tests).
- **Washed-out detector metric:** the eval reports a confusion matrix of `LowContrastCapture` per tone class: real captures with Windows HDR off, real captures per HDR step and capture method, `tone-lift`, `tone-clip` and `tone-pq-as-srgb`. The expected label of each synthetic variant is computed from its transform parameters applied to the source's measured panel range, not by the detector itself. Gates (start values): no finding on any real normal-contrast capture with Windows HDR off (0 false positives, because users ignore a finding that fires on good captures; glare captures are labelled and reported separately); the finding is present on every synthetic variant whose expected label is "washed out". `tone-clip` results are reported separately, because a contrast stretch cannot restore clipped detail.
- **Metrics per field type:**
  - exactly right
  - correctly flagged (wrong, but flagged)
  - **silently wrong** (wrong and classified as confident) – the most important metric, target ≈ 0
  - runtime and peak heap (per resolution class)
- **CI** checks the public, redacted partial corpus. The private corpus is included locally via an environment variable (`UEXDR_CORPUS_DIR`).
- The first public entry is `pyro-gateway-stanton-01` (unverified, `sourceFidelity: reencoded-unknown`). The **Patch City screenshots** still have to be supplied as files (redact the balance first) or stored in the private corpus.
