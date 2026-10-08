# Requirements

Priorities: **M** = Must (1.0), **S** = Should (1.x), **C** = Could (later).

## Goal

Players (UEX DataRunners) use the client to capture commodity terminal data from Star Citizen and provide it to UEX. There are two ways to do this:

- **manually:** fast, keyboard-driven input, with current UEX values shown as reference (not prefilled, R-MAN-2)
- **via screenshot:** OCR, validation, review, submission

Current UEX data (terminals, commodities, latest prices, status levels, game version, tolerances) serve as **default values and constraints** for recognition and selection.

## Terms

The complete, binding domain language (Ubiquitous Language) is in [11-ddd-and-tdd.md](11-ddd-and-tdd.md) §A1. Only the short version here:

| Term | Meaning |
|---|---|
| **Capture** | A screenshot |
| **Scan** | A capture after OCR analysis |
| **Report** | The aggregation of all scans of one terminal and one side (buy = "Buy", sell = "Local Market Value") within a time window (grouping rule R-OCR-13). The report is the unit sent to `POST /data_submit`. |
| **Send threshold** | Minimum confidence above which a field may be sent without confirmation (initial value 0.80, see [07](07-ocr-concept.md) §2.6). Fields below it must be confirmed or corrected. |
| **Prior** | The UEX reference value (latest or average price, SCU, status) for the combination of terminal, commodity and side |

## Scope

| ID | Requirement | Prio |
|---|---|---|
| R-SCOPE-1 | Type `commodity` (buy and sell side) | M |
| R-SCOPE-2 | Data model and API layer prepared for `item`, `vehicle_buy`, `vehicle_rent` | M (model) / C (UI) |
| R-SCOPE-3 | **Not** in scope for 1.0: trade route planner (feature of the original), fleet/trades features | – |

## Capture (CAP)

| ID | Requirement | Prio |
|---|---|---|
| R-CAP-1 | **User-defined image folders:** add, remove and enable/disable any number of folders. Configurable per folder: environment (**automatic from the path** – default – or fixed LIVE/PTU/EPTU/HOTFIX/TECH-PREVIEW or **ask for every image**), include subfolders (yes/no), file types (default `png`, `jpg`, `jpeg`), optionally "only files newer than …". The SC screenshot folders are only **suggested** during onboarding; the user confirms them. | M |
| R-CAP-1a | **Manual import (default):** A click on **"Import"** (globally for all active folders or per folder) imports all files that have not been processed yet. Result: "n new images taken over, m skipped". Optionally there is a preview list for deselecting individual files. | M |
| R-CAP-1b | **Automatic import (opt-in per folder):** When enabled, newly created files are taken over immediately. Technique: `WatchService` (`ENTRY_CREATE`/`ENTRY_MODIFY`) plus polling fallback (default 2 s) if the WatchService is unreliable (Wine prefixes, network drives) or cannot be registered. On startup and after enabling, a catch-up scan runs for files created while the app was closed or the mode was off (can be disabled). | M |
| R-CAP-1c | **Read only when fully written:** A file counts as complete when its size and `lastModified` are stable for a quiet period (default 750 ms) and it can be decoded as an image. Partial files are retried (with backoff, at most 30 s) and then shown as an error with a reason. | M |
| R-CAP-1d | **Processed-file register** (SQLite): path, size, `lastModified` and content hash of every imported file. This way neither "Import" nor the watcher reads the same file twice, even after renaming or copying (hash). Per file there is a "Re-import" action. | M |
| R-CAP-1e | Status visible per folder: mode (manual/automatic), watched yes/no (and why not, e.g. folder missing or no permissions), polling active, last file, number of pending files. If a folder is missing, it is flagged and not silently removed. | M |
| R-CAP-2 | Further input paths: drag & drop, Ctrl+V, file dialog (multiple selection), optional clipboard monitoring with fingerprint deduplication | M |
| R-CAP-3 | The environment is derived from the folder path (or set fixed per folder), the game version from UEX (`game_versions`). If the environment is unknown, the app asks; there is no hard-coded fallback. | M |
| R-CAP-3a | **Environment mapping to UEX:** UEX only knows `live` and `ptu`. Default mapping: LIVE and HOTFIX → `live`, PTU/EPTU/TECH-PREVIEW → `ptu`. Configurable; the mapping must be verified against UEX (assumption A9). If no reports are accepted for the target (`is_accepting_*`), submission is blocked. | M |
| R-CAP-3b | **Game version at capture time:** On import, the version valid at capture time is recorded (cached state of `game_versions`). If the version changed between capture and submission (patch day), the app warns and requires a confirmation or discards the report. The app keeps a local history of observed version changes (timestamp of first observation). If a capture is older than the last observed change and the app was not running in between (e.g. catch-up import after a patch), the version is marked **uncertain** and the user must confirm it. | M |
| R-CAP-4 | Automatic detection of the SC installation: RSI Launcher log `Launching Star Citizen … from (…)`, running process `Bin64/StarCitizen.exe`, default paths. On Linux additionally Wine/Proton prefixes (see assumption A3). | S |
| R-CAP-5 | Capture time from the file name (regex, interpreted in the system time zone), fallback to `lastModified`. The app detects duplicates via content hash. | M |
| R-CAP-6 | Optional cleanup function: delete or archive originals after successful submission. Default is **off**. | S |
| R-CAP-7 | **Working copies:** Per capture, the app stores the perspective-corrected panel crops (not the full-screen screenshot) in its data directory. Upload, AI re-check and review therefore also work if the original was moved or deleted. Retention: up to 7 days after successful submission (configurable). | M |

## Manual capture (MAN)

| ID | Requirement | Prio |
|---|---|---|
| R-MAN-1 | Terminal selection via search field (fuzzy over name, nickname, location, system), filter by star system, "recently used" | M |
| R-MAN-2 | After choosing the terminal, the list of the terminal's known commodities appears (from `commodities_prices`). The current UEX values are shown **as a reference next to the input field** (including the age of the value), **not as a prefilled value**. Only rows that were actively entered or explicitly taken over by keystroke are sent. This way no outdated values end up unchecked at UEX as a "fresh" report. | M |
| R-MAN-3 | Full keyboard operation: Tab/Enter, number shortcuts for status levels 1–7 | M |
| R-MAN-4 | Manual reports go through the same validation (VAL) as OCR reports | M |
| R-MAN-5 | Manual reports can attach a screenshot. If a screenshot is mandatory for the user (evaluation phase of new DataRunners, `screenshot_required`), this is shown in advance and submission without an image is blocked (assumption A11). | M |
| R-MAN-6 | Manual reports have an environment (default LIVE, selectable) and use the current UEX game version of that environment | M |

## Recognition (OCR)

| ID | Requirement | Prio |
|---|---|---|
| R-OCR-1 | Local OCR (no cloud) with PaddleOCR models (detection + recognition) via ONNX Runtime; models bundled, license and hash evidence in `NOTICE` | M |
| R-OCR-2 | Localization of the terminal panel via text and color anchors, perspective correction. If it fails, the user can set the corners or the crop manually; the app never aborts hard. | M |
| R-OCR-3 | Theme-agnostic: orange/red (e.g. Patch City/Pyro), blue and other terminal themes. Layout profiles are data-driven. | M |
| R-OCR-4 | Recognition of side (Buy / Local Market Value, active tab) and section (IN STOCK / OUT OF STOCK / IN DEMAND / NO DEMAND / SELLABLE CARGO) | M |
| R-OCR-5 | Robust number parser: strip currency symbol `¤`, determine thousands and decimal separators structurally, optional K/M suffix, suffix `/SCU` | M |
| R-OCR-6 | Repair of confusable digits ({0,6,8,9}, possibly others from the corpus), only with an unambiguous candidate and with a plausibility witness | M |
| R-OCR-7 | Commodity names are resolved against the UEX vocabulary and localized `global.ini` names. Robust against line breaks and overlap with the SCU value. | M |
| R-OCR-8 | Terminal and location recognition primarily from the location field under "YOUR INVENTORIES", matched against the assortment | M |
| R-OCR-9 | "AVAILABLE CARGO SIZE (SCU)" → `container_sizes` | S |
| R-OCR-10 | Second reader for the status: color and fill level of the vertical stock bar or the color of the status text, matched against the OCR status text | S |
| R-OCR-11 | Stitching of multiple scrolled screenshots of one terminal or side – **even without overlap** (order via the scrollbar, merging via the commodity ID). Cut-off cards at the edge are only taken over if the card is fully visible (bottom edge detected). On conflicts, the value read further away from the viewport edge is preferred. Possible gaps (scrollbar, UEX assortment) are shown. | M |
| R-OCR-12 | Processing in the background (bounded parallelism), the UI never blocks. Goal: < 3 s per screenshot on a mid-range CPU (**target value, will be measured**). | M |
| R-OCR-13 | **Report grouping:** Scans belong to one report if terminal, side and environment are the same and the time gap to the previous scan of the group is ≤ 10 min (configurable). The user can split and merge reports in the UI. A scan with an unclear terminal forms its own group until the terminal is resolved. | M |
| R-OCR-14 | **Neighbor names** (distinguished only by a short token such as "SIZE 1"/"SIZE 7") are only assigned automatically if the distinguishing token was read exactly, otherwise `Ambiguous` (07 §2.5). | M |
| R-OCR-15 | Dimmed cards (hypothesis A14: not buyable) are read with their own contrast normalization and not discarded as a partial card. | S |
| R-OCR-16 | **Not a terminal screenshot:** images that do not show a commodity terminal (other UI, wrong tab, "YOUR INVENTORIES" selection elsewhere) are recognized as such and set aside with a notice and a "process anyway / crop manually" action – never a generic "failed to process" error (fix for F7). | M |

## Optional AI recognition (VLM)

The classic OCR (above) is the **default path** and always runs, even while playing. In addition, a **local vision language model (VLM)** can be enabled via [Ollama](https://ollama.com). It needs a lot of VRAM or CPU and therefore by default runs **only when Star Citizen is closed**. This way you can work in the game without AI and have the AI re-check afterwards.

| ID | Requirement | Prio |
|---|---|---|
| R-VLM-1 | Setting "AI recognition" with three values: **Off** (default), **Automatic – only when the game is closed**, **Always** (expert, with a warning about VRAM and FPS losses) | S |
| R-VLM-2 | Game detection: the process `StarCitizen.exe` is checked periodically (default every 5 s). Windows: path ends in `Bin64\StarCitizen.exe`. Linux: command line or arguments of the Wine/Proton process contain `StarCitizen.exe` (assumption A7). The RSI Launcher alone does not count as "game running". "Closed" only applies after a hysteresis (default 30 s), so that a restart of the game does not start the AI. | S |
| R-VLM-3 | Game start during an AI run: the running job is aborted immediately and the model unloaded (`keep_alive: 0`). Affected scans return to the state "AI pending"; no data is lost. | S |
| R-VLM-4 | Re-check after the game ends: the app offers (or runs automatically, configurable) to re-read all **not yet sent** reports with the VLM. Selectable are "only reports with warnings" (default) or "all". Only reports in state **Draft** are re-read; released, queued or submitted reports are never changed by the AI. Progress and cancellation are possible at any time. | S |
| R-VLM-5 | Fusion: VLM and classic OCR are **independent readers**. Agreement increases the confidence ("double confirmed"); contradiction creates a finding `Ambiguous` with both candidates. VLM values go through the **same** vocabulary resolution and validation (prior, tolerances, consistency). No reader alone enforces a value that contradicts the validation. | S |
| R-VLM-6 | Strictly local: default host `http://localhost:11434`, configurable. Cloud models (Ollama Cloud, suffix `:cloud`) and non-localhost hosts only after explicit confirmation with a privacy notice. Otherwise images never leave the computer. | S |
| R-VLM-7 | Model management: check Ollama reachability and version (`/api/version`), show installed models (`/api/tags`), pull the recommended model on request (`/api/pull` with progress). Ollama itself is **not** shipped; the app shows installation instructions. | S |
| R-VLM-8 | Hardware notice: after loading the model, `size_vram` is compared with `size` (`/api/ps`). If the model does not run fully on the GPU, the app warns ("slow, running on CPU"). The speed per image is shown in the UI. | S |
| R-VLM-9 | Input to the VLM is the **perspective-corrected panel crop** (shop panel or location field), not the whole screenshot. This is smaller and faster, and the balance is not passed along. | S |
| R-VLM-10 | Output format: prompt as a versioned resource; temperature 0; answer as `KEY: value` lines plus a Markdown table, processed by a deterministic parser (see [07](07-ocr-concept.md) §2.7). If the answer is truncated (`done_reason == "length"`), exactly one retry with a larger token limit follows. | S |
| R-VLM-11 | The recommended default model is determined by a **bake-off on our corpus** (not by model card). Candidates include `qwen3-vl:8b-instruct`/`qwen3-vl:4b-instruct`, tried and tested at basetool; their suitability for commodity terminals is **unproven**. | S |
| R-VLM-12 | The feature is fully optional: without Ollama the app is fully usable, without error messages at startup. | M |

## Validation (VAL)

| ID | Requirement | Prio |
|---|---|---|
| R-VAL-1 | Rule-based confidence per field; the model's confidence values are not used directly. Every warning has a reason code that is shown in the UI. | M |
| R-VAL-2 | Plausibility against the UEX prior: a deviation > `price_variation` % or > `scu_variation` (from `data_parameters`; units per assumption A15) is never rejected automatically – it is marked as a **major deviation** and must be confirmed by the user before submission (R-UI-10). Genuine price changes are therefore possible, but never unchecked. | M |
| R-VAL-2a | **No prior available** (new commodity at the terminal, no history): as a substitute, the commodity-wide average (`commodities.price_buy`/`price_sell`) is used with double tolerance. If that is missing too, no prior-based repair takes place; the field carries the finding "no reference value" and lies below the send threshold if it contains a confusable digit. | M |
| R-VAL-2b | **Repairs only automatic with an independent witness:** A repair that relies only on the prior lies below the send threshold (confirmation by keystroke). It is only taken over automatically if an independent witness agrees (glyph topology, second reader/VLM). Rationale: the prior is a possibly outdated value; otherwise genuine price changes would be silently "repaired back" to old values. | M |
| R-VAL-3 | Consistency rules: status ↔ SCU (e.g. "Out of Stock" ⇒ 0 SCU), side ↔ section, commodity is buyable or sellable at this terminal (`is_buyable`/`is_sellable`) | M |
| R-VAL-4 | Terminal ambiguity: if there are multiple candidates, the selection is mandatory; the app silently preselects nothing | M |
| R-VAL-5 | Missing commodities: commodities that are listed at the terminal according to UEX but do not appear on any scan are shown. Optionally the user can mark them as `is_missing`. | S |

## Review UI (UI)

| ID | Requirement | Prio |
|---|---|---|
| R-UI-1 | Input list (queue) with status: waiting / processed / review / ready / sent / error | M |
| R-UI-2 | Report editor: terminal (mandatory dropdown on ambiguity), side, game version, table of commodity rows with name, status, SCU, price/SCU and container sizes | M |
| R-UI-3 | Rows appear in screen order. For each row there is the image crop in readable size; when a field has focus, the source region in the screenshot is highlighted. | M |
| R-UI-4 | Indication of the **recognition confidence** (ok / confirm / select / correct) plus a text reason (not only color, for accessibility). Visually **separate** from the deviation marking (R-UI-10): recognition confidence as an icon and border on the field, deviation as background color plus Δ badge. This keeps it recognizable whether "poorly read" or "different from UEX" is the problem. | M |
| R-UI-5 | "Accept all confident"; navigation from problem to problem (F8/Shift+F8) | S |
| R-UI-6 | Manual cropping and corner correction when localization fails | M |
| R-UI-7 | Own theme (light/dark), HiDPI, freely resizable windows | M |
| R-UI-8 | Starts immediately; loading in the background with a status line; usable offline with the cached data state (submission only once connected) | M |
| R-UI-9 | Tray mode with notification "n new screenshots" | C |
| R-UI-10 | **Deviation marking against UEX:** Every field (price, SCU, status, container sizes) is compared with the previous UEX value for the same combination of terminal, commodity and side and **marked by color**. Levels:<br/>• **equal** – no marking<br/>• **deviating within tolerance** – subtle marking (yellow), notice "please cross-check"<br/>• **strongly deviating** (> `price_variation` % or > `scu_variation`, status ≥ 2 levels apart) – strong marking (orange) **and confirmation required** (submission block until confirmation)<br/>• **no reference value** (new at this terminal) – notice marking (blue)<br/>Applies equally to OCR and manual reports. Thresholds come from `data_parameters` or the settings, not from constants. | M |
| R-UI-11 | **Make cross-checking easy:** For marked fields the following are visible: UEX value, age of the UEX value ("3 days ago"), difference absolute and in % (e.g. "+12 %"), plus the image crop of the source. The marking is **not only colored** but additionally encoded as an icon and text (color vision deficiency). The colors are chosen to be suitable for color vision deficiency (no pure red/green) and configurable in the theme. An **outdated** UEX value (older than configurable, default 7 days) weakens the marking by one level and is shown as "reference outdated". | M |
| R-UI-12 | **Summary before submission:** The report shows at the top the number of deviating and strongly deviating fields; via keyboard you jump from deviation to deviation (same navigation as R-UI-5). A confirmation applies to exactly the confirmed value; if the value is changed afterwards, it must be confirmed again. In addition, **unexpected commodities** (unknown at UEX for this terminal) and **missing commodities** (listed at UEX, not found in the scan, R-VAL-5) are marked. | M |

## Submission (SUB)

| ID | Requirement | Prio |
|---|---|---|
| R-SUB-1 | `POST /data_submit` with `id_terminal`, `type`, `is_production`, `game_version`, `prices[]` (separate buy and sell rows), optionally `container_sizes`, `screenshot`, `details` | M |
| R-SUB-2 | Submission queue: rate-limit compliant (120 req/min global, max. 500 rows per report, at most 1000 report rows per 30 min – UEX counts each price row as a report), bounded parallelism, retry with backoff only on 429/5xx/network errors. The queue is **persistent** (SQLite): reports queued offline or before a crash are not lost and, after a restart, are only sent after renewed release. | M |
| R-SUB-3 | Submission block on unresolved fields or an unclear terminal | M |
| R-SUB-4 | Persistent history (SQLite): report IDs (`ids_reports`), time, payload hash, 5-minute cooldown per terminal/commodity/environment (conservatively **without** side until the UEX rule is clarified – assumption A12); withdrawal via `data_remove`. Optionally the history shows the processing status at UEX (`data_info`). | M |
| R-SUB-5 | Before submission: check `data_parameters` (`is_accepting_reports` or `is_accepting_ptu_reports`, `commodity.is_accepted`). For this, a state at most 15 min old is used (not the daily cache), because these flags change on patch day. | M |
| R-SUB-6 | Mapping of all known UEX error codes to clear user messages (ResourceBundle keys) with a recommended action | M |
| R-SUB-7 | Upload screenshot: perspective-corrected crop "SHOP INVENTORY" plus location field, **balance always redacted**, JPEG or PNG < 10 MB (target ~1–2 MP). If a report consists of multiple scrolled captures but the API has only **one** `screenshot` field, the shop crops are composed vertically into one image (assumption A11: accepted by UEX). | M |
| R-SUB-8 | Test mode (`is_production=0`), switchable in the settings. Default: **on** in development builds and CI; **off** in release builds after onboarding (the onboarding offers a test submission first). The active mode is always visible in the UI. | M |

## UEX integration (API)

| ID | Requirement | Prio |
|---|---|---|
| R-API-1 | Local cache of the reference data (SQLite), TTL per endpoint (see [06](06-uex-api.md)), refresh in the background, display of the data state | M |
| R-API-2 | Price prior: `commodities_prices?id_terminal=` (up to 10 IDs per request) when opening a terminal; if needed `commodities_prices_all` (TTL 30 min) | M |
| R-API-3 | Configurable base host with fallback, OS truststore, system proxy | M |
| R-API-4 | Secret key validation and DataRunner status during onboarding and at startup | M |
| R-API-5 | Uniform envelope evaluation (`status`, `http_code`, `message`, `data`); defensive parsing (numbers as strings, 0/1 flags) | M |

## Localization (L10N)

| ID | Requirement | Prio |
|---|---|---|
| R-L10N-1 | UI language is English. All UI texts are nevertheless kept in ResourceBundles (`messages.properties`) so further languages can be added later without code changes. | M |
| R-L10N-2 | Game language: commodity and status names as well as tab labels from the SC `global.ini` (remove BOM, formats `key=value` and `key,P=value`) | S |
| R-L10N-3 | Further OCR writing systems (Cyrillic, Korean) via optionally downloadable models | C |

## Quality assurance (QA)

| ID | Requirement | Prio |
|---|---|---|
| R-QA-1 | Golden corpus: real screenshots plus hand-transcribed expected values (JSON). **Storage location outside the public repo** if the images contain private data (balance!). If necessary, check in only redacted versions. | M |
| R-QA-2 | The corpus covers at least: Buy and Sell tab, scrolling with and without overlap, red/orange themes (Patch City), blue theme with uppercase font (Pyro Gateway Stanton – first entry available), Pyro, Nyx, Gateway, long and nearly identical names, dimmed cards, 4K/1440p/1080p/ultrawide | M |
| R-QA-3 | Eval CLI: field accuracy per field type, share of "silently wrong" (wrong and marked as confident) – **target ≈ 0**, share of "flagged", runtime. The value runs in CI as a regression test (thresholds that may only rise). | M |
| R-QA-4 | OCR digest test: hash of all raw OCR outputs over the corpus, so that model or runtime updates happen deliberately | S |

## Non-functional (NF)

| ID | Requirement | Prio |
|---|---|---|
| R-NF-1 | Windows 10/11 x64 and Linux x64 (glibc distros, X11; under Wayland via XWayland – native Wayland support of JavaFX is not proven, XWayland operation must be tested in M4) | M |
| R-NF-2 | Distribution: Windows MSI and portable ZIP; Linux `.deb` and portable `tar.gz` (app image from jpackage); each with bundled runtime | M |
| R-NF-3 | Resources: heap limited (initial value `-Xmx512m`, to be determined by measurement), ONNX session is released after inactivity | S |
| R-NF-4 | Secret key (and, if required, a user-provided UEX app token, A2) only in the OS keystore (Windows Credential Manager, Linux Secret Service). Fallback is a file with permissions 0600 after an explicit warning. Never in logs. | M |
| R-NF-5 | Configuration: atomic writes, schema versioning, visible errors. Storage locations: Windows `%APPDATA%`, Linux `$XDG_CONFIG_HOME`/`$XDG_DATA_HOME`/`$XDG_CACHE_HOME`. | M |
| R-NF-6 | Logging: rotating log file, secrets masked, diagnostics export (logs plus system info, without secrets) | M |
| R-NF-7 | Update notice via GitHub Releases, at most once a day, can be disabled | S |
| R-NF-9 | "Delete all local data" action (cache, history, working copies, logs, stored secrets); the uninstaller documentation names the data directories. | S |
| R-NF-8 | No interference with the game: no access to process memory, no input injection, no overlays. Only files, the clipboard and the **process list** (for path detection and game detection, R-VLM-2) are read. | M |

## Supply chain (SEC)

Details, threat model and implementation: [10-supply-chain-security.md](10-supply-chain-security.md).

| ID | Requirement | Prio |
|---|---|---|
| R-SEC-1 | Only Maven Central and (for plugins) the Gradle Plugin Portal; fixed versions; dependency locking for all configurations | M |
| R-SEC-2 | Gradle dependency verification with SHA-256 **and** PGP signatures for all artifacts incl. plugins; changes only via reviewed PR (CODEOWNERS) | M |
| R-SEC-3 | Gradle wrapper validation in CI and `distributionSha256Sum`; release JDK not via toolchain auto-download | M |
| R-SEC-4 | GitHub Actions pinned by commit SHA, minimal `permissions`, release only from protected tags | M |
| R-SEC-5 | Dependabot with a 7-day cooldown period for version updates (security updates excluded); OSV scan blocks known critical/high vulnerabilities | M |
| R-SEC-6 | Release with SHA256SUMS, CycloneDX SBOM and signed build provenance (GitHub Artifact Attestations) | M |
| R-SEC-7 | ONNX models with a fixed revision and SHA-256 check at build time and at load time; no runtime downloads of code; no auto-update | M |
| R-SEC-8 | Do not extract native libraries (sqlite-jdbc, ONNX Runtime) into a shared temp directory | M |
| R-SEC-9 | Code signing of the Windows installers | C (decision by project owner, cost) |

## Assumptions (to be verified)

| ID | Assumption |
|---|---|
| A1 | The header for the secret key is called `secret_key`; the documentation summaries are contradictory (`secret-key`). Must be checked against the live API with `is_production=0`. |
| A2 | It is unclear whether `data_submit` additionally requires an **app token** (`Authorization: Bearer`) and how an open-source client distributes it. A token embedded in the distribution can be extracted. Options: consult UEX or a token entered by the user. |
| A3 | SC on Linux runs via Wine/Proton (e.g. LUG Helper, Lutris). Screenshots then end up in the prefix under `drive_c/Program Files/Roberts Space Industries/StarCitizen/<CHANNEL>/screenshots`. Path and structure must be checked on real Linux installations. |
| A4 | The Windows default path for screenshots is `…\StarCitizen\<CHANNEL>\screenshots`. It is shown during onboarding and can be confirmed, not set blindly. |
| A5 | The status levels 1–7 and their names come from `commodities_status`. The mapping of the in-game texts (e.g. "Max Inventory", "Out of Stock" on the sell side) to the codes must be verified on the corpus. |
| A6 | The Patch City screenshots (2000×1125) may be scaled. Target resolutions must be tested with original screenshots. |
| A7 | On Linux the running game can be detected via `ProcessHandle.info().command()`/`arguments()` of the Wine/Proton process by `StarCitizen.exe`. To be checked on real installations (Wine, Proton, Lutris); fallback is a scan of `/proc/*/cmdline`. |
| A8 | A local VLM reads commodity terminals more accurately than the classic OCR or complements it usefully. So far this is only proven for refinery panels (basetool); for our case it is only checked by the bake-off in M5. If the benefit is small, the feature stays small or is dropped. |
| A9 | UEX assigns HOTFIX reports to `live` and EPTU/TECH-PREVIEW reports to `ptu` (or accepts them at all). |
| A10 | On the **sell side**, the displayed "… SCU" number corresponds to the UEX field `scu_sell`. UEX additionally knows `scu_sell_stock`; which field the number in the terminal means is unclear. On the buy side, "SHOP QUANTITY" presumably corresponds to `scu_buy`. |
| A11 | A screenshot composed of multiple scroll crops is accepted by UEX; manual reports without a screenshot are accepted for established DataRunners. |
| A12 | The UEX 5-minute duplicate lock applies per (terminal, commodity) – whether the side (Buy/Sell) is distinguished is open. |
| A13 | The terminal theme (blue vs. orange) correlates with the star system (e.g. Gateway station on the Stanton vs. Pyro side). Used at most as a weak hint. |
| A14 | A card is dimmed if the available stock is smaller than the smallest offered container size (IRON 4 SCU < 8; Fluorine 2 < 8; DynaFlex 13 < 16). To be confirmed on the corpus. |
| A15 | `data_parameters.commodity.price_variation` is a percentage and `scu_variation` an absolute SCU amount. Units are unconfirmed (06 open point 6); until verified, the thresholds are configurable and default to conservative values. |
