# Bug analysis of SC-Datarunner-UEX and fixes in our client

## Sources – please note

- SC-Datarunner-UEX is **closed source**. The repo contains only the README, images and Pages configuration; the binaries are under "Releases". There is no source code we could inspect. The binaries were deliberately **not** decompiled.
- The basis is the public issues (#2–#38), the release notes (v0.5.1–v0.12.0) and the README.
- Comments in the issues were not visible without logging in. A mapping "issue → fix version" is therefore often only **[inferred]**, by comparing dates.
- In addition there are **our own observations** on the Patch City screenshots supplied by the project owner and on the official app screenshot in the README. They are marked as **[observed]**.

## Overview: problem → cause → fix in our client

The "Req" column refers to requirements in [01-requirements.md](01-requirements.md).

### OCR and parsing

| # | Problem (source) | Cause (known/suspected) | Our fix | Req |
|---|---|---|---|---|
| F1 | **Currency symbol `¤` is read as `9`/`@`** → "¤6,705" becomes "96705" ([#32](https://github.com/Shebuka/SC-Datarunner-UEX/issues/32)) | The symbol sits directly in front of the digits and is recognized as part of the number | 1. Price parser following the pattern `<symbol><number>/SCU`: the symbol is separated geometrically via the first glyph in the price token and never counted as a digit. 2. Candidate generation with and without the leading glyph, scored against the UEX price prior (last price at this terminal, `price_variation` from `data_parameters`). 3. If that is not unambiguous, the field is flagged, never silently accepted. | R-OCR-5, R-VAL-2 |
| F2 | **6↔8 confusion** (known limitation since v0.5.1, also after PaddleOCR) | HUD font, blur, chromatic aberration | Confusable set {0,6,8,9} (concept from basetool, reimplemented). Repair only with an unambiguous candidate; **automatic only with an independent witness** (glyph topology, second reader), otherwise confirmation by key press (R-VAL-2b). The topological glyph classifier (counting holes) additionally serves as a veto. Hint to the user: set chromatic aberration to 0. | R-OCR-6, R-VAL-2b |
| F3 | **Decimal/thousands separators** – locale confusion `123.456,67` vs. `123,456.78` ([#21](https://github.com/Shebuka/SC-Datarunner-UEX/issues/21)), decimal places ([#5](https://github.com/Shebuka/SC-Datarunner-UEX/issues/5)) | Parser knows only one format | Structure-based number parser per 07 §2.4: groups of 3 count as thousands separators; a last separator followed by 1–2 digits is decimal. Decimal prices are supported (UEX `price_*` is float). The locale of the OS setting is ignored. | R-OCR-5 |
| F4 | **Prices off by a factor of 1000** after SC 4.7 (full aUEC instead of K/M) ([#19](https://github.com/Shebuka/SC-Datarunner-UEX/issues/19)) | Parser assumed a K/M suffix | Parse both forms: K/M suffix optional, scaling explicit. The plausibility check against the UEX prior catches factor-1000 errors. | R-OCR-5, R-VAL-2 |
| F5 | **Multi-line names** ("Recycled Material" + "Composite", "Ship Ammunition" + "Size 1") (v0.8.1/v0.12.0) | Line-based OCR without vocabulary | Names are resolved **against the UEX vocabulary** (incl. localized names from `global.ini`), not line by line. Multiple text boxes of one card are matched together. | R-OCR-7 |
| F6 | **[observed] Name and SCU quantity overlap**: "Recycled Material Compos**0 SCU**ite" (Patch City screenshots); in the README screenshot the SCU number is missing for exactly this entry | In the game, long names run into the right-aligned quantity | SCU parser per 07 §2.4 (`(\d[\d,.\s]*)\s*SCU`) also *inside* glued-together tokens; the rest of the name is resolved via prefix/fuzzy match. If the quantity is unreadable, it is flagged (not sent empty). | R-OCR-7, R-VAL-1 |
| F7 | **"Failed to process image"** ([#3](https://github.com/Shebuka/SC-Datarunner-UEX/issues/3), [#8](https://github.com/Shebuka/SC-Datarunner-UEX/issues/8), [#18](https://github.com/Shebuka/SC-Datarunner-UEX/issues/18)); cause according to v0.6.8.1 an overly strict template matching on bright terminals | A fixed reference image for the perspective correction | Localization via **text anchors** ("SHOP INVENTORY", "YOUR INVENTORIES", "COMMODITIES") plus color anchors – no pixel template. **Never abort hard:** if localization fails, the user sets the four corners manually or selects the region with the mouse; afterwards the normal pipeline runs. | R-OCR-2, R-UI-6 |
| F8 | **Red terminals / Nyx layout / Pyro font** ([#23](https://github.com/Shebuka/SC-Datarunner-UEX/issues/23), [#30](https://github.com/Shebuka/SC-Datarunner-UEX/issues/30)) | Color thresholds tuned to one theme | Theme-agnostic preprocessing (luminance/max channel instead of a fixed color). Multiple layout profiles are loaded data-driven as a JSON resource. The Patch City screenshots (red/orange) belong in the test corpus from the start. | R-OCR-3, R-QA-2 |
| F9 | **Order in the app ≠ order in the game** ([#37](https://github.com/Shebuka/SC-Datarunner-UEX/issues/37)) | Sorting by an internal criterion | The review list keeps the **screen order**, also after stitching. Each row shows its image crop at a readable size. | R-UI-3 |
| F10 | **Overall accuracy, "half the data manual"** ([#32](https://github.com/Shebuka/SC-Datarunner-UEX/issues/32), [#37](https://github.com/Shebuka/SC-Datarunner-UEX/issues/37)) | Combination of F1–F9 | Multi-stage: vocabulary constraint, UEX prior, confusable repair, second reader (status band color/fill level, cargo size boxes), measurable via the golden corpus with regression tests | R-QA-1..4 |

### Terminal and location

| # | Problem (source) | Cause (known/suspected) | Our fix | Req |
|---|---|---|---|---|
| F11 | **Wrong location/terminal assignment** (Lorville CBD↔Rustville [#9](https://github.com/Shebuka/SC-Datarunner-UEX/issues/9), Admin↔CBD [#11](https://github.com/Shebuka/SC-Datarunner-UEX/issues/11), MIC-L5↔Ruin Station [#38](https://github.com/Shebuka/SC-Datarunner-UEX/issues/38), TDD↔MTP, Gateways) | Header OCR plus color heuristic; ambiguity is too easily overlooked | 1. The primary source is the location field under "YOUR INVENTORIES" (in the Patch City screenshots: "PATCH CITY"), matched against all location names and nicknames from UEX. 2. Candidates are checked against the assortment: the recognized commodities must match the terminal's known commodities (`commodities_prices?id_terminal=`). 3. Context from previous screenshots of the same session (same star system). 4. If there is more than one candidate above the threshold, the selection is **mandatory** (blocking, prominent at the top of the card); nothing is silently preselected. | R-OCR-8, R-VAL-4, R-UI-2 |
| F12 | **Gateway terminals with the same names in the neighboring system** (open since v0.6.8); **[observed]** confirmed: for "Pyro Gateway (Stanton)" the location field only shows "PYRO GATEWAY" | Name alone is not unambiguous | Disambiguation via star system context (last location/session); otherwise mandatory selection with the system shown in the dropdown | R-VAL-4 |
| F13 | **Low-confidence values are sent anyway** (43 %/30 % → HTTP 200, [#38](https://github.com/Shebuka/SC-Datarunner-UEX/issues/38)) | No submission gate | **Hard submission block**: fields below the threshold must be confirmed or corrected. "Send All" skips such reports and states the reason. In addition, **deviations from the previous UEX value** are highlighted in color; strong deviations must be confirmed even if they were read confidently. | R-SUB-3, R-UI-10 |

### Environment, configuration and capture

| # | Problem (source) | Cause (known/suspected) | Our fix | Req |
|---|---|---|---|---|
| F14 | **HOTFIX environment "Unknown" → fallback to game version "4.1" → rejection** ([#27](https://github.com/Shebuka/SC-Datarunner-UEX/issues/27)) | Static fallback | Folder name mapping LIVE/PTU/EPTU/HOTFIX/TECH-PREVIEW configurable. The game version **always** comes from `GET /game_versions` (live/ptu). If the environment is unknown, the user must choose; there is **no hardcoded fallback**. Before sending, it is checked whether `is_accepting_reports` or `is_accepting_ptu_reports` is set. | R-CAP-3, R-SUB-5 |
| F15 | **Environment not detected in VM/Docker** ([#22](https://github.com/Shebuka/SC-Datarunner-UEX/issues/22)), separate PTU path ([#12](https://github.com/Shebuka/SC-Datarunner-UEX/issues/12)) | Autodetection mandatory | Any number of watched folders, environment manually overridable per folder | R-CAP-1, R-CAP-3 |
| F16 | **Dependence on the Print key** – remapped key ([#33](https://github.com/Shebuka/SC-Datarunner-UEX/issues/33)), no numpad ([#16](https://github.com/Shebuka/SC-Datarunner-UEX/issues/16)), no images imported ([#35](https://github.com/Shebuka/SC-Datarunner-UEX/issues/35)), Ctrl+V requested ([#26](https://github.com/Shebuka/SC-Datarunner-UEX/issues/26)) | Folder watching only | User-defined folders, either via a click on "Import" or automatically when new files are created (WatchService plus polling fallback, e.g. for Wine/network drives; reading only after writing has completed); in addition drag & drop, Ctrl+V, file import, clipboard monitoring (optional). Diagnostic display "Folder is being watched, last image: …". | R-CAP-1, R-CAP-2 |
| F17 | **config.ini is not created / settings get lost / app exits after setup** ([#24](https://github.com/Shebuka/SC-Datarunner-UEX/issues/24), [#28](https://github.com/Shebuka/SC-Datarunner-UEX/issues/28), v0.12.0) | Faulty persistence | Atomic writes (temp + `ATOMIC_MOVE`), schema version and migration, errors are shown in the UI (never silently), startup self-test of the directories | R-NF-5 |
| F18 | **Localization path outdated after reinstall** (v0.10.0/v0.12.0) | Fixed path | Find `global.ini` via SC installation detection (RSI Launcher log, default paths, Wine prefix), reload on change (mtime/hash) | R-L10N-2 |

### Submission and connection

| # | Problem (source) | Cause (known/suspected) | Our fix | Req |
|---|---|---|---|---|
| F19 | **Cooldown only in memory**; deleting a row circumvents the 5-minute block (open) | No persistent state | Submission history in **SQLite**, cooldown per (terminal, commodity, environment) persistent; conservatively without side until assumption A12 is resolved. Withdrawal via `data_remove` is logged as well. | R-SUB-4 |
| F20 | **No connection to the API** in some networks ([#15](https://github.com/Shebuka/SC-Datarunner-UEX/issues/15), [#29](https://github.com/Shebuka/SC-Datarunner-UEX/issues/29), [#31](https://github.com/Shebuka/SC-Datarunner-UEX/issues/31)); suspected certificate/proxy | Proxy/TLS | 1. On Windows the **OS truststore** is used (`Windows-ROOT`, SunMSCAPI), so corporate/AV proxies with their own CA work. 2. System proxy via `ProxySelector`. 3. Configurable hosts with fallback (`api.uexcorp.space` → `api.uexcorp.uk`). 4. Diagnostic dialog "Test connection" with status, HTTP code and body; secrets are masked in it. | R-API-3, R-NF-6 |
| F21 | **Sending hangs / one dialog per report** (v0.12.0) | UI thread, unbounded parallelism | Send queue on virtual threads, semaphore (default 2 in parallel), token bucket for 120 req/min, non-modal progress in the list | R-SUB-2 |
| F22 | **Errors without details** ("Failed to send data:" [#4](https://github.com/Shebuka/SC-Datarunner-UEX/issues/4), "error 28" [#17](https://github.com/Shebuka/SC-Datarunner-UEX/issues/17)) | Error texts not mapped | All known UEX `status` codes are mapped to understandable, localized messages with a recommended action. Unknown codes appear with the raw code. | R-SUB-6 |
| F23 | **"User is not a datarunner"** ([#2](https://github.com/Shebuka/SC-Datarunner-UEX/issues/2)), secret key errors ([#10](https://github.com/Shebuka/SC-Datarunner-UEX/issues/10)) | No upfront check | During onboarding, `GET /user` checks the key and reads `is_datarunner`/`is_datarunner_banned`. If needed there is a link to the DataRunner sign-up, and sending is blocked in advance. | R-API-4 |
| F24 | **Update check via the unauthenticated GitHub API (60 req/h)** | Too frequent polling | Update check at most once per day, `ETag`/`If-None-Match`, can be disabled | R-NF-7 |

### Platform and UI

| # | Problem (source) | Cause (known/suspected) | Our fix | Req |
|---|---|---|---|---|
| F25 | **Windows only** | Design decision | Windows and Linux, including Wine/Proton prefix detection | R-NF-1 |
| F26 | **White background/readability** ([#6](https://github.com/Shebuka/SC-Datarunner-UEX/issues/6)), tutorial windows too large ([#25](https://github.com/Shebuka/SC-Datarunner-UEX/issues/25)) | Theme/DPI handling | Own CSS theme (light/dark) independent of the OS theme, all windows scalable, HiDPI test at 100/150/250 % | R-UI-7 |
| F27 | **SCU container sizes missing** in reports ([#13](https://github.com/Shebuka/SC-Datarunner-UEX/issues/13), open) | Not read out | Recognize "AVAILABLE CARGO SIZE (SCU)" boxes (set ⊆ {1,2,4,8,16,24,32}, ascending) → field `container_sizes` | R-OCR-9 |
| F28 | **No installer / tray** ([#36](https://github.com/Shebuka/SC-Datarunner-UEX/issues/36), open) | – | MSI (Windows) and `.deb` plus portable archive (Linux); tray mode with notification "n new screenshots" as a later feature | R-NF-2, R-UI-9 |
| F29 | **Commodities only** ([#20](https://github.com/Shebuka/SC-Datarunner-UEX/issues/20), open) | – | Data model from the start for `type` = `commodity`/`item`/`vehicle_buy`/`vehicle_rent`; items and vehicles follow after 1.0 | R-SCOPE-2 |

### Privacy

| # | Problem (source) | Cause (known/suspected) | Our fix | Req |
|---|---|---|---|---|
| F30 | **[observed] Balance in the upload screenshot.** "CURRENT BALANCE" is shown at the top right of the terminal; an uncropped screenshot would transmit it to UEX. Whether the original does this is unknown. | – | The upload screenshot contains only the region **"SHOP INVENTORY" plus location field**. The balance is **always** redacted; both are covered by tests. | R-SUB-7 |
