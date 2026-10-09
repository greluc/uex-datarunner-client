# UEX API 2.0 – working notes

> **Doc type:** Living spec — binding. Last reviewed: 2026-10-09.

> **Sources.** Corrected 2026-10-09: the earlier notes were written without access to the official documentation and rested on search snippets, third-party copies and open-source clients ([3P]). On 2026-10-09 the official documentation and the live API were checked. Tags:
>
> - **[DOC]:** the official documentation, `https://uexcorp.space/api/documentation/` and its endpoint pages `…/documentation/id/<method>_<resource>/`, read 2026-10-09
> - **[LIVE]:** read-only GET calls against `api.uexcorp.space/2.0` and `api.uexcorp.uk/2.0` on 2026-10-09, without any user key or app token (auth behaviour probed with a dummy key)
> - **[RN]:** UEX dev notes, `GET /release_notes` (entries from 2026-04-01 to 2026-09-29)
> - **[TERMS]:** `https://uexcorp.space/about/terms` (updated 2026-07-02)
> - **[3P-OBS]:** production traffic of other clients, seen in the predecessor's own logs and in responses pasted into its public issue tracker (section "Observed by third parties")
> - **[3P]:** third-party clients and copies (`dolejska-daniel/uexcorp-openapi`, `Zamotic/UexCorpDataRunnerClient`, `Hybris95/UEX-Trader`, `ByteCollectiveIO/sc-nav`, PyPI `uex-lib`); a [3P] statement not confirmed by [DOC] or [LIVE] stays unverified
>
> Nothing here was verified with a write call: no `data_submit`, `data_edit` or `data_remove` has been sent (`CLAUDE.md`, "External writes"). The review of the plan against these facts is [08](08-review.md) §N.

## Basics

| Topic | Value | Source |
|---|---|---|
| Base URL | The documentation names `https://api.uexcorp.uk/2.0/` on every page. `https://api.uexcorp.space/2.0/` answers as well with identical data; both are behind Cloudflare. Which host UEX considers primary, and whether both accept writes, is open (point 4) | DOC, LIVE |
| URL forms | `/{resource}/`, `/{resource}/{param}/{value}/…` and `/{resource}/?{param}={value}&…` | DOC, LIVE |
| App auth | `Authorization: Bearer <app token>`; the token belongs to an app created under "My Apps" (`https://uexcorp.space/api/apps/`). Required for `data_submit`, `data_edit`, `data_remove`, `data_info`, `data_monitor`, `commodities_averages`, `user_notifications`; not required for the public reference endpoints below. Without a token those endpoints answer HTTP 403 `not_allowed` ("Sorry. Authentication error."), with an invalid one HTTP 403 `access_denied` ("Invalid bearer token.") | DOC; LIVE (on `data_info`, `data_monitor`, `commodities_averages`) |
| User auth | Header `secret-key: <key>` (the DataRunner's secret key from the UEX profile) on `data_submit`, `data_edit`, `data_remove`, `data_info`, `user`, `user_notifications`. The server also accepts `secret_key`; the client sends the documented `secret-key` | DOC; LIVE (both spellings reach the key check of `GET /user`) |
| Client version lock | The owner of an app token can lock it to a client version; the API then accepts only requests whose `X-Client-Version` header matches. The client sends `X-Client-Version` with its build version, the same version as in the User-Agent, on every request (R-API-7; decided 2026-10-09, O-86) | DOC |
| Envelope | `{"status":"ok","http_code":200,"data":…,"message":""}`. Errors carry a real HTTP status (400 for missing input, 403 for auth) and a string code in `status`; `http_code` repeats the HTTP status; `message` may hold text. Internal error: `{"status":"error","http_code":500,…}`. An unknown resource answers HTTP 404 with an **HTML page**, not JSON | DOC, LIVE |
| Rate limit | 172,800 requests per day (or 120 per minute); when exceeded `status: requests_limit_reached` (HTTP status not documented). No rate-limit or `Retry-After` header was observed | DOC, LIVE |
| Data types | Numbers are JSON numbers (prices are whole numbers in practice, `weight_scu` int or float), flags are int 0/1, timestamps are Unix seconds. Corrected 2026-10-09: no numeric strings were observed in any reference endpoint; parsing stays tolerant of them. `commodities_status.code` is an int although [DOC] says string | LIVE |
| Caching | Reference responses carry `Cache-Control: public, max-age=…` (e.g. 86400 on `game_versions`); Cloudflare served them uncached (`cf-cache-status: DYNAMIC`). The client uses no HTTP cache, so freshness rules (R-SUB-5) are not undermined | LIVE |

## Submitting: `POST /data_submit`

Documentation slug: `post_data_submit`. Bearer token and `secret-key` header.

```json
{
  "id_terminal": 89,
  "type": "commodity",
  "is_production": 0,
  "game_version": "4.10.1",
  "container_sizes": "1,2,4,8,16,24,32",
  "faction_affinity": 15,
  "date_added": 1791500000,
  "screenshot": "<base64 png/jpg>",
  "details": "optional",
  "prices": [
    {"id_commodity": 1, "price_sell": 120, "scu_sell": 593, "status_sell": 2},
    {"id_commodity": 24, "price_buy": 136, "scu_buy": 529, "status_buy": 1}
  ]
}
```

Report level [DOC]:

- Required: `id_terminal`, `type` (`commodity | item | vehicle_buy | vehicle_rent`), `is_production` (1 production, 0 testing), `prices[]`.
- `game_version` string|null; **defaults to the LIVE version** when omitted. Only the current LIVE or PTU version is accepted (`invalid_game_version`: "game version does not exist. LIVE or PTU accepted only").
- `container_sizes` string|null: CSV subset of {1,2,4,8,16,24,32}, **per report** – there is no per-row field (point 9). The client sends one report per distinct set: rows of one terminal and side are grouped by their identical sent set, and rows without a sent set go into a report without the field (R-OCR-9; decided 2026-10-09, O-92).
- `faction_affinity` int, −100..100, only for `commodity` and `item`: the reporter's affinity with the faction that runs the terminal (point 27). The client sends it only when the user entered it (R-SUB-13; decided 2026-10-09, O-93).
- `date_added` int|null: "Report date (optional; must be a past date within the last 30 days)"; error `invalid_date` (point 18). The client sends the report's `observedAt` in Unix seconds, and the R-VAL-6 hard limit stays at or below 30 days (decided 2026-10-09, O-91).
- `details` string|null.
- `screenshot` string: png or jpg in base64, "up to 10.00 MB"; "required for new datarunners (90-day evaluation period)"; `screenshot_required` is "primarily applicable to users in the evaluation period or under restrictions". Whether the 10 MB limit applies to the base64 string or to the decoded image is open (point 10); the error is called `screenshot_length_exceeds_limit`. The client keeps the base64 string at most 10,000,000 bytes long (R-SUB-7; decided 2026-10-09, O-98).

Row level for `type = commodity` [DOC]:

- `id_commodity` (required).
- `price_buy` **or** `price_sell` (UEC per SCU; "only one input is allowed"; left empty when `is_missing`).
- `is_missing` int: 1 when the commodity is no longer sold at the terminal.
- `scu_buy` / `scu_sell` int: "inventory amount displayed at terminal" – the number the terminal shows, on either side (A10).
- `status_buy` / `status_sell` int: 1 (out of stock) to 7 (maximum), see `commodities_status`.
- `quality` int|null, 0–1000. Every live price row reports `quality` 0; commodity terminals do not show it.
- **A row carries one side only.** Errors: `has_both_price_buy_and_price_sell`, `has_both_scu_buy_and_scu_sell`, `cannot_have_both_status_buy_and_status_sell`, `cannot_have_both_price_buy_and_status_sell`, `cannot_have_both_price_buy_and_scu_sell`, `cannot_have_both_price_sell_and_status_buy`, `cannot_have_both_price_sell_and_scu_buy`. One payload may mix buy rows and sell rows (the official example does); the plan still sends one report per side (point 14).
- A row needs a price or `is_missing` (`has_no_prices_and_no_is_missing_set`).

Limits and results:

- At most 500 rows per submission (`max_rows_exceeded`) [DOC].
- `too_many_reports`: more than 1000 reports in the last 30 minutes [DOC]. One UEX report is one row (see `data_info`), so the budget counts rows (point 16).
- `duplicated_report`: "a report for the same item and location has been submitted within the last 5 minutes" [DOC]; observed as HTTP 400 with an empty `message` [3P-OBS]. A removed report no longer blocks a new one (`data_remove`). Side, user, test-mode and LIVE/PTU scope: point 11.
- Success output [DOC]: `ids_reports` **string|null**, `date_added` int, `username` string|null. Corrected 2026-10-09: the earlier [3P] notes showed `ids_reports` as an array; the documentation types it as a string, while real production responses carry an array of numeric strings and `date_added` as a numeric string (see "Observed by third parties"). The parser accepts both shapes. The string's format, its order against `prices[]` and the encoding of partial acceptance are open (point 24).
- [RN 2026-08-26]: "Data submissions that fail now tell you which entry was rejected and why, instead of a generic error"; "Data submissions that only partially go through now say so, instead of reporting success." The response shape of both is not documented.

Documented response codes of `data_submit` [DOC] (the documentation's comment lines are shifted by one from `too_many_reports` on; the meanings below follow the obvious reading):

| Group | Codes |
|---|---|
| Server | `service_unavailable`, `database_error`, `image_upload_error`, `image_storage_error` |
| Auth and account | `no_api_found` (authentication error), `missing_secret_key`, `invalid_secret_key`, `user_not_found`, `user_not_allowed` (temporary ban, not a datarunner, …), `user_disabled` (banned or blocked); undocumented but observed on other endpoints: `not_allowed`, `access_denied` (HTTP 403, app token missing or invalid) |
| Acceptance | `ptu_reports_not_allowed`, `type_not_available` |
| Report | `invalid_input` (invalid JSON), `missing_type`, `invalid_type`, `missing_id_terminal`, `terminal_not_found`, `not_allowed_player_terminal`, `missing_prices_array`, `invalid_prices_array`, `invalid_prices_array_format`, `reference_key_not_supplied`, `invalid_game_version`, `invalid_date`, `max_rows_exceeded`, `faction_affinity_under_minimum_range`, `faction_affinity_under_maximum_range` (sic, for > 100), `faction_affinity_not_allowed_for_current_type`, `screenshot_required`, `screenshot_length_exceeds_limit` |
| Rate | `too_many_reports`, `duplicated_report`; the global quota answers `requests_limit_reached` |
| Row (commodity) | `no_commodities_found`, `missing_id_commodity`, `invalid_id_commodity`, `has_no_prices_and_no_is_missing_set`, the seven one-side codes above, `invalid_status_buy`, `invalid_status_sell`, `invalid_quality` |
| Row (other types) | `no_items_found`, `no_categories_found`, `no_vehicles_found`, `invalid_id_item`, `missing_id_category`, `invalid_id_category`, `id_item_or_name_not_provided`, `missing_category_or_subcategory`, `invalid_category`, `invalid_subcategory`, `missing_id_vehicle`, `invalid_id_vehicle`, `has_prices_and_is_missing_set` |
| Success | `ok` |

Error classes (R-SUB-11; every known code has exactly one class, whatever the HTTP status; codes not listed count as permanent). This table mirrors the requirement and covers every documented `data_submit` code (decided 2026-10-09, O-87).

| Code / answer | Class | Handling |
|---|---|---|
| connect-phase failure, HTTP 429, `requests_limit_reached` (HTTP status undocumented), `too_many_reports` | transient (not processed, R-SUB-9) | retry with backoff; for the last three, wait for the row budget or `Retry-After` (none observed) |
| `duplicated_report` (HTTP 400, empty `message`) | cooldown | cooldown for every row key of the report, the job waits and is sent again; after an unknown outcome see R-SUB-9 |
| `missing_secret_key`, `invalid_secret_key`, `user_not_found`, `user_not_allowed`, `user_disabled`, `user_not_verified` (undocumented, observed); HTTP 401/403 without a known code | account | pause the whole queue; re-check the key read-only with `GET /user` first (spurious key errors under UEX overload); a failed re-check shows "re-authentication required", a passing one "UEX reported a key error, the key checked out – retry now" (R-UI-16) |
| `no_api_found`, HTTP 403 `not_allowed` (token missing), HTTP 403 `access_denied` (token invalid) | account (app token) | pause the whole queue; persistent "app token missing or invalid" state with an action |
| `is_accepting_reports`/`is_accepting_ptu_reports` = 0, `commodity.is_accepted` = 0, `global.is_datacenter_enabled` = 0 (pre-send check, R-SUB-5); `ptu_reports_not_allowed`, `type_not_available` | acceptance closed | hold, re-check every 15 min (R-VAL-6 applies) |
| HTML (non-JSON) HTTP 404 on a write | API changed | hold the whole queue with the reason "API changed" until an app update or the user's "retry now" |
| `screenshot_required`, `max_rows_exceeded`, `terminal_not_found`, `invalid_date`, `screenshot_length_exceeds_limit` | report-fixable | back to Draft with a finding (`terminal_not_found`: select the terminal again, refresh the vocabulary; `invalid_date`: check the observation time and the system clock; `screenshot_length_exceeds_limit`: re-encode smaller) |
| `invalid_game_version` | permanent (not logged as a client bug) | Rejected (final); "duplicate as new draft" is not offered, because a duplicate keeps the capture-time version (R-CAP-3b, open point 21, A18) |
| `not_allowed_player_terminal` | permanent (stale vocabulary: player-owned terminals are not in it) | Rejected, with "duplicate as new draft" |
| `invalid_input`, `missing_type`, `invalid_type`, `missing_id_terminal`, `missing_prices_array`, `invalid_prices_array`, `invalid_prices_array_format`, `reference_key_not_supplied`, `no_commodities_found`, `missing_id_commodity`, `invalid_id_commodity`, `has_no_prices_and_no_is_missing_set`, the seven one-side codes, `invalid_status_buy`, `invalid_status_sell`, `invalid_quality`, `faction_affinity_under_minimum_range`, `faction_affinity_under_maximum_range`, `faction_affinity_not_allowed_for_current_type` | permanent (suspected client bug, logged with the app version) | Rejected, with "duplicate as new draft" |
| `service_unavailable`, `database_error`, `image_upload_error`, `image_storage_error` | – (unknown outcome, R-SUB-9) until UEX states that they come before processing (point 25) | never retried automatically |
| any HTTP 5xx without a known code; request timeout, connection reset or other I/O failure after connecting; empty or unparseable answer other than an HTML 404 | – (unknown outcome, R-SUB-9) | never retried automatically |

The codes of the other report types (`item`, `vehicle_*`) are classified when those types are built (R-SCOPE-2); until then they count as permanent.

## Reading, correcting and withdrawing reports

Added 2026-08-10 [RN]. All three need the Bearer token and the `secret-key` header [DOC].

- **One UEX report is one row** (one terminal, one commodity, one side). A plan `Report` ([11](11-ddd-and-tdd.md) §A1) is one UEX *submission*; its rows become as many UEX reports, each with its own ID ("UEX report ID", 11 §A1) and lifecycle status.
- **Decided 2026-10-09 (O-88, O-89, O-90, O-103):** the history shows each row's lifecycle status, polled from `data_info` at most every 5 minutes (manual refresh at most once a minute) with `limit=100` and the `username` filter, until a final status (`consolidated`, `declined`, `expired`) or 14 days after sending; "accepted" for the local prior update (R-VAL-7) is the `ok` of `data_submit`, and a later `declined` or `expired` reverts it. Rows are mapped to report IDs only after a `data_info` lookup of terminal, commodity and `date_added`. Withdrawal is one `data_remove` per row, offered only while `is_editable` = 1, with a per-row outcome. `data_edit` is not used (R-SUB-10).
- **Lifecycle status** (`data_info.status`): `pending` (waiting for the approval bot), `under_review` (a moderator decides), `queued` (PTU report parked until that version goes live), `approved` (cleared, waiting for consolidation), `consolidated` (folded into the live prices), `declined` (rejected as inconsistent or invalid), `expired` (sat unapproved too long). UEX aims to process reports within 24 hours [TERMS §6.2.5].
- **Report IDs are not unique**: "Report IDs repeat across the table's partitions, so the ID alone does not identify a single row — compare `date_added`."

`GET /data_info` [DOC]:

- Filters: `id`, `type`, `id_terminal`, `status`, `username`, `limit` (1–100, default 50). No time window, no own-only filter (filter by `username`), no paging; order not documented.
- Every datarunner can read every report; removed reports are not listed; only API report types are listed.
- Output per report: `id`, `type`, `status`, `id_user`, `username`, `id_terminal`, `id_commodity`, `id_item`, `id_category`, `id_vehicle`, `name`, `price_buy`, `price_sell`, `price_rent`, `scu_buy`, `scu_sell` ("as reported, not the projected demand"), `status_buy`, `status_sell`, `quality`, `container_sizes`, `faction_affinity`, `details`, `game_version`, `is_missing`, `is_new_item`, `is_new_at_location`, `is_ptu_report`, `is_contested`, `is_owner`, `is_editable` (1 on own reports until they consolidate), `has_attachments`, `has_comments`, and the timestamps `date_added`, `date_modified`, `date_checked`, `date_approved`, `date_declined`, `date_consolidated`, `date_expired`, `date_queued`.
- Codes: `service_unavailable`, `access_denied`, `missing_secret_key`, `invalid_secret_key`, `user_not_found`, `user_disabled`, `user_not_allowed`, `invalid_type`, `invalid_status`, `invalid_limit`, `ok`.

`POST /data_edit` [DOC]:

- Input: `id`, `is_production`, and the **complete** row (`price_buy`/`price_sell`, `is_missing`, `scu_*`, `status_*`, `quality`, `container_sizes`, `faction_affinity`, `details`). **Omitted fields are cleared**, not kept.
- Terminal, type, commodity and game version cannot change. An edit sends the report back to the start of the approval pipeline.
- Own reports only, and only until they consolidate (`report_consolidated`); another user's report answers `report_not_found`.
- Output: `id`, `type`, `is_new_at_location`, `date_modified` (0 when `is_production` is 0).
- Additional codes: `missing_id`, `report_not_found`, `report_consolidated`, `invalid_container_size`.

`DELETE /data_remove/id/{id}/` [DOC]:

- **One report (row) per call.** No `is_production` parameter is documented.
- Own reports only, until they consolidate. Soft delete: the report stops counting, leaves the approval queue and disappears from `data_info`; a removed report no longer blocks a new one for the same terminal and commodity.
- Output: `id`, `type`, `date_removed`. Codes: `missing_id`, `report_not_found`, `report_consolidated`, `type_not_available` and the auth codes.

## Reference data

All public (no token, no key) unless marked. Field lists [DOC], values and types [LIVE] 2026-10-09.

| Endpoint | Use | TTL (proposal) |
|---|---|---|
| `GET /commodities` | Vocabulary (205 entries): `id`, `name` (unique), `code` (**not unique**: 21 codes are shared, e.g. by an ore and its refined form), `kind` (free text with typos, never a key), `is_buyable`, `is_sellable`, `is_available`, `is_available_live` (1 for all but one), `is_visible`, `is_raw`, `is_temporary`, `is_illegal`, `is_buggy`, `price_buy`/`price_sell` (commodity-wide "average / SCU"; 0 for 24 of 99 buyable and 48 of 157 sellable commodities = no value), … `slug` is documented but absent, `uuid` is null everywhere. `is_buyable`/`is_sellable` are global flags that disagree with real terminal rows (56 buy rows at terminals belong to commodities with `is_buyable` 0, e.g. Hydrogen Fuel and Quantum Fuel; 14 sell rows to `is_sellable` 0), so they are no assortment check; the consistency check uses the terminal's side from `commodities_prices` instead (R-VAL-3; decided 2026-10-09, O-94) | 1 h |
| `GET /terminals?type=commodity` | Terminals (161: Stanton 117, Pyro 35, Nyx 9): `id`, `name` ("TDD - Trade and Development Division - Area 18"), `nickname` ("TDD Area 18"), `displayname` ("Area18", "the name displayed on the terminal screen"), `fullname` ("Commodity Shop - …"), location IDs and names, `id_faction`, `is_available`, `is_available_live` (0 on 47), `is_visible` (0 on 40), `is_player_owned` (1 on 1), `is_affinity_influenceable` (1 on 15), `max_container_size` (one int; 0 = unknown on 33; values 1, 16, 24, 32), `game_version` (last update). **`displayname` is not unique**: 19 values repeat, e.g. Area18 ×3, Lorville ×2, New Babbage ×2, Orison ×2, Pyro Gateway ×4, Nyx Gateway ×4, Stanton Gateway ×4, several Lagrange stations ×2–3. `mcs` is deprecated and differs from `max_container_size` on 128 terminals; never read it. Type `commodity_raw` (23 "Refinery Ore Sales" terminals, own `commodities_raw_prices`) has no `data_submit` type and is not part of the vocabulary. No PTU availability flag exists. Decided 2026-10-09 (O-96, O-87): `displayname` joins the location matching and the manual search; a repeated `displayname` is `Ambiguous` unless the assortment decides; refinery screens are recognised as unsupported; player-owned terminals leave the vocabulary | 12 h |
| `GET /commodities_prices?id_terminal=a,b,…` | **Prior** per terminal. `id_terminal` takes up to 10 IDs (11 answer `exceeded_id_terminal_query_limit`); without input HTTP 400 `missing_required_input`. **One row per (terminal, commodity), one side only**: the traded side is the side with a price > 0; the other side's price, SCU and status are 0 (status 0, not null; detect the side by price, since 2 sell rows carry status 0). Fields: `price_*` (last), `_min/_max/_avg` with `_week`/`_month`, `price_*_users`; `scu_buy` (last) and its statistics; **`scu_sell_stock`** = last inventory reported at the location (the sell-side counterpart of what a datarunner submits as `scu_sell`), while **`scu_sell` here is the forecast demand**; `status_*` and statistics; `volatility_*`; `faction_affinity` (average affinity of reporters); `container_sizes` (CSV **per row**, i.e. per terminal and commodity; empty on 20 rows); `quality` (0); `game_version`; `date_added`; `date_modified` (age of the prior; median 5.2 days, 90th percentile 10.1 days on 2026-10-09). `scu_*_max` are maxima of reported values, not a capacity; no field gives a terminal's capacity. All rows carry LIVE versions (none had the PTU version), so there is **no PTU prior** | 30 min |
| `GET /commodities_prices_all` | Compact snapshot of all rows (2,603): `price_*`, `price_*_avg`, `scu_buy`, `scu_sell_stock`, `scu_sell`, their averages, `status_*`, `container_sizes`, `quality`, `date_added`, `date_modified`, names (fallback/offline) | 30 min |
| `GET /commodities_status` | `{"buy":[…7],"sell":[…7]}`, each level `code` (int 1–7), `name`, `name_short`, `name_abbr`, `percentage` ("0-14%"), `percentage_start`, `percentage_end`, `colors` (a word: red, orange, blue, green). Names and colours **differ per side**: buy 7 "Maximum Inventory (Full)" (MA), sell 7 "Maximum Inventory (No Demand)" (ND); buy 1–2 red, 3 orange, 4–5 blue, 6–7 green; sell 1–2 green, 3–4 blue, 5 orange, 6–7 red. Level 1 "Out of Stock (Empty)" covers 0–14 %. The in-game texts ("MAX INVENTORY", "OUT OF STOCK") equal neither `name` nor `name_short` | 1 day |
| `GET /game_versions` | `{"live":"4.10.1","ptu":"4.10.2"}`; [DOC] `ptu` is "empty if there is no PTU set" (treat `""` and `null` as no version). Version strings are **not always three-part** ("4.9", "3.24", "3.24.2a") | 1 day; additionally fetched at startup, before an import batch is versioned, on game start and ≤ 15 min before every send attempt (R-CAP-3b, R-SUB-5); every successful fetch updates the observed version history |
| `GET /game_versions_all` | All versions known to UEX in chronological order (166 on 2026-10-09), each `{id, game_version, date_added}` (release date); early ones carry wiki names ("Hangar Module") | used only to measure A17; never to seed or order the observed version history (decided 2026-10-09, O-99) |
| `GET /data_parameters` | Confirmed nesting: `global` {`is_accepting_reports`, `is_accepting_ptu_reports`, `is_datacenter_enabled`, `game_version`, `game_version_ptu`, `evaluation_period_days` (90)}; one object per report type (`commodity`, `item`, `vehicle_rent`, `vehicle_buy`) with `is_accepted` (acceptance of that report type), `price_variation`, `ttl`, `notification` (staff alert text or null); `commodity` adds `is_temporary_enabled` and `scu_variation`. Units: `price_variation` is a **percentage** ([DOC] "Variation Tolerance (+/-)" table: commodities 25 %, equal to the live value 25); `ttl` is "days until a price is considered outdated" (commodities 15); `scu_variation` (5000) has no documented unit. `global.game_version`/`game_version_ptu` equalled `/game_versions` on 2026-10-09. Decided 2026-10-09: `commodity.ttl` is the default staleness limit (R-UI-11, O-95); the `notification` of the sent types is shown in the status area as untrusted plain text, and `is_datacenter_enabled` = 0 counts as acceptance closed (R-SUB-5, O-100) | 1 day for tolerances and `ttl`; **≤ 15 min before every send attempt** for `global.is_accepting_*` and `commodity.is_accepted` (R-SUB-5) |
| `GET /star_systems`, `/planets`, `/moons`, `/orbits`, `/space_stations`, `/cities`, `/outposts`, `/poi`, `/factions` | Location names for the "YOUR INVENTORIES" field and disambiguation | 1 day |
| `GET /user` | With `secret-key`: `username`, `is_datarunner`, `is_datarunner_banned`, `is_staff`, `date_added`, `date_disabled`, …, and **personal data** (`email`, `discord_username`) that the client discards at the anti-corruption layer and never stores, logs or exports (decided 2026-10-09, O-97; 02 §9 N1). No evaluation-period flag and no DataRunner start date. Without key and username: HTTP 400 `missing_secret_key_or_username`; invalid key: HTTP 403 `invalid_secret_key`; `user_not_allowed` = banned or disabled | – |
| `GET /data_monitor` (Bearer) | Per terminal: TTL status of its prices, `ids_reports` (int array of pending, unconsolidated LIVE reports), `has_ptu_reports`, `last_update`. Not used by the plan | – |
| `GET /release_notes` | UEX dev notes (`id`, `date_updated`, `content` as HTML); the read-only source for noticing API changes | – |

## Observed by third parties (2026-10-09)

Production behaviour seen in the predecessor's own log of one session on 2026-10-08 (83 `data_submit` calls, read without decompiling), in production responses that users pasted into its public issue tracker ([Shebuka/SC-Datarunner-UEX#38](https://github.com/Shebuka/SC-Datarunner-UEX/issues/38), 2026-08-18/19) and in open-source clients ([08](08-review.md) §O). Tag **[3P-OBS]**. These are observations of other clients' traffic; our own `is_production=0` write check still has to confirm them before an adapter relies on them.

- **`ids_reports` is a JSON array of numeric strings**, one per row (`"ids_reports":["964307","964311"]`), in all 63 successful responses of the session and in #38; the count equalled the number of rows. IDs ascend within a response but **interleave across concurrent submissions**. This contradicts the documented `string|null`. Whether the order equals `prices[]` is not visible.
- **`date_added` comes back as a numeric string** and equals the time the client sent, in Unix seconds (all 63 responses; in #38 a few seconds after the screenshot's file time, so a client-side processing time rather than the capture time itself), so the input is accepted and echoed. The documentation types the output as an int.
- **`duplicated_report` is HTTP 400 with an empty `message`** (10 of 10); the answer names no row. The [RN 2026-08-26] promise to name the rejected entry was not visible in `status` or `message`.
- **A read timeout does not mean "not processed"**: ten submissions timed out at the client's 15 s limit while UEX was slow; all ten resends 77–97 s later were answered `duplicated_report`.
- Rows with SCU 0 and status 1 were answered `ok` (#38).
- Host: a UEX e-mail of December 2024 named `api.uexcorp.space` the dedicated API host ([Hybris95/UEX-Trader#206](https://github.com/Hybris95/UEX-Trader/issues/206)); the predecessor's switch to `api.uexcorp.uk` failed for users behind TLS-inspecting proxies (`SSL: WRONG_VERSION_NUMBER`, [#15](https://github.com/Shebuka/SC-Datarunner-UEX/issues/15)), and it went back to `.space` with `.uk` as fallback. Writes to `.space` are proven by #38.
- Some text fields are **HTML-escaped** (`Grey&apos;s Market`), which matters for name matching; the anti-corruption layer unescapes them once (R-API-5, O-104).
- An undocumented code `user_not_verified` (account not verified with RSI) is handled by one client.
- During UEX overload, users saw spurious secret-key errors that later went away ([#10](https://github.com/Shebuka/SC-Datarunner-UEX/issues/10), [#29](https://github.com/Shebuka/SC-Datarunner-UEX/issues/29)).
- A commenter writing as a UEX reviewer said reports **without container sizes often need manual review** ([#13](https://github.com/Shebuka/SC-Datarunner-UEX/issues/13)).
- One client reports that UEX's bot protection blocks calls from an Electron main process, and another saw a 403 from a GitHub Actions runner; clients that send a browser User-Agent exist. A plain Java `HttpClient` worked for every GET on 2026-10-09; a write from it is unverified.
- `commodity.price_variation` was 60 in a mid-2025 recording and is 25 today; `scu_variation` stayed 5000.
- The documented Bearer requirement is recent or was ambiguous before: one client released in June 2026 sends only `secret-key`, and a summary of the documentation from July 2026 read "Bearer Token (via `secret-key` header)". The live 403 `not_allowed` is real for `data_info` today.

## Terms of use

[TERMS] §1.4 allows personal, non-commercial use. §5 (API terms): endpoints may change or be removed, users must follow the documentation, access can be terminated for misuse. §3.7: whoever uploads an image confirms the right to share it and grants UEX a licence; public content may be reused by third parties. §3.8: accounts only from age 18. §6 (Datarunner agreement): reports must be accurate and "reported from the location specified"; unusual reports may be declined; repeated improper reports can lead to locks or bans. The terms say nothing about third-party clients, a User-Agent convention or republishing API responses (point 7). Corrected 2026-10-09: the terms live at `/about/terms`, not `/about/legal`.

## Open points

Status as of 2026-10-09. **Answered** points keep their number and their answer; a point marked **write** can only be settled with an `is_production=0` write call that @greluc approves per call; **UEX** means asking UEX.

1. **Answered:** the header is `secret-key` [DOC]; `secret_key` reaches the same check [LIVE].
2. **Answered:** the app token is required for `data_submit`, `data_edit`, `data_remove` and `data_info` [DOC, LIVE]. Decided 2026-10-09 (O-85): each user creates an app under "My Apps" and enters its token next to the secret key; both are validated before they are saved. Whether one token may be shared is moot, because none is shared.
3. **Partly answered:** `status_*` is optional (only `id_terminal`, `type`, `is_production` and `id_commodity` are required); `container_sizes` is per report (point 9).
4. **Partly answered:** both `api.uexcorp.uk` (named by the documentation) and `api.uexcorp.space` serve the API. Open (UEX): which is primary and whether both accept writes; [3P-OBS] points to `.space` as primary with `.uk` as fallback, and `.uk` failed behind some TLS-inspecting proxies. Per endpoint: the reference endpoints need neither credential; `data_*`, `data_monitor`, `commodities_averages`, `user_notifications` need the token; `data_*`, `user`, `user_notifications` need `secret-key`.
5. **Answered:** `GET /user` works with the secret key alone (no token) and returns the account fields above; it also returns `email` and `discord_username`.
6. **Partly answered:** the full code list is in the table above; `price_variation` is a percentage, `ttl` is in days; auth failures answer HTTP 403 **with** a code (`not_allowed`, `access_denied`, `invalid_secret_key`); `commodity.is_accepted` is per report type. Open (UEX): the unit of `scu_variation` and the meaning of `global.is_datacenter_enabled`.
7. **Partly answered:** the terms are read (above); they neither allow nor forbid publishing recorded responses (UEX, O-50).
8. **Answered:** the displayed number is submitted as `scu_sell`; the matching prior field is `scu_sell_stock` (A10). Corpus evidence: Agricultural Supplies at Pyro Gateway (Stanton) shows 1,482 SCU, UEX `scu_sell_stock` 1482, `scu_sell` 5187. Open (write): whether UEX stores a reported 0 – rows whose screen shows 0 SCU keep a non-zero `scu_sell_stock`; [3P-OBS] rows with SCU 0 are accepted (`ok`).
9. **Answered (shape):** `container_sizes` exists only per report in `data_submit`, while UEX keeps it per row; 104 of 135 terminals with rows have different sets per commodity. [3P-OBS]: reports without sizes often go to manual review. Decided 2026-10-09 (O-92): one report per distinct set (R-OCR-9). Open (write): whether UEX copies the report value to every row.
10. **Partly answered:** `GET /user` exposes no evaluation flag or start date; `screenshot_required` also applies to restricted users. Open (write, UEX): stitched images, whether `is_production=0` enforces the screenshot, whether the 10 MB limit is on the base64 length, and whether a `data:` prefix is tolerated. Decided 2026-10-09 (O-98): the client applies the limit to the base64 length (≤ 10,000,000 bytes), which is safe either way.
11. **Partly answered:** the block is keyed by "the same item and location" for 5 minutes and is lifted by `data_remove`. Open (write): side, user, `is_production=0` and LIVE/PTU scope.
12. **Partly answered:** `is_missing` = 1 when the commodity is no longer sold there; the price is left empty. Open (write): whether `scu_*` and `status_*` are allowed with it.
13. **Partly answered:** UEX knows only LIVE and PTU (`game_versions`, `is_accepting_ptu_reports`, `ptu_reports_not_allowed`, `is_ptu_report`, `queued`). Open (UEX): where HOTFIX, EPTU and TECH-PREVIEW belong (A9).
14. Mixing in one payload is allowed; one report per side stays the plan's choice (each side has its own screenshot).
15. **Answered:** see "Reading, correcting and withdrawing reports". `data_info` can find a user's reports without IDs by `username` and `id_terminal` (≤ 100, no time filter, order undocumented).
16. **Partly answered:** the 1000-per-30-min limit counts reports, i.e. rows. Open (UEX): per user or per app token, and whether test submissions count.
17. **Answered:** `data_edit` replaces the whole row, re-queues it for approval and works only until consolidation. Decided 2026-10-09 (O-90): it is not offered; R-SUB-10 stays withdraw plus duplicate.
18. **Answered (shape):** `date_added` is an optional report-level timestamp no older than 30 days (`invalid_date`). [3P-OBS]: production clients send a client-side time close to the capture in Unix seconds and UEX echoes it. Decided 2026-10-09 (O-91): the client sends `observedAt` there. Open (write): whether UEX orders "latest" values and the duplicate block by it; our own approved `is_production=0` write check confirms the effect before R-VAL-8 is relaxed.
19. **Answered:** there is no PTU price data (every prior row carries a LIVE version) and no PTU availability flag on terminals or commodities; PTU reports are parked as `queued`.
20. **Answered:** there is no User-Agent convention; UEX offers the `X-Client-Version` lock. Decided 2026-10-09 (O-86): every request carries `X-Client-Version` with the build version.
21. **Partly answered:** only the current LIVE or PTU version is accepted. Open (write, on the next patch): whether the previous version is accepted for a while.
22. **Answered:** the `data_parameters` nesting is confirmed and `is_accepted` is per report type; `global.game_version*` equalled `/game_versions`. The game version still comes only from `/game_versions` (R-CAP-3b).
23. **Answered:** (a) `/commodities` prices are commodity-wide averages per SCU, 0 = no value; per-environment averages do not exist. (b) In `commodities_prices` the untraded side is 0 in price, SCU and status; `date_modified` is one timestamp per row and is never equal to `date_added`. Which terminals and period feed the average stays undocumented ([DOC] index: "Default Average (Days)" 15).
24. **Partly answered [3P-OBS]:** `ids_reports` is an array of numeric strings, one per row, interleaved across concurrent submissions. Open (write): its order against `prices[]`, and how partial acceptance and the rejected entry are encoded ([RN 2026-08-26] confirms both exist). Decided 2026-10-09 (O-103): rows are mapped to IDs only by checking each ID through `data_info` (terminal, commodity, `date_added`), never by position alone, and sending is strictly serial, so IDs of concurrent submissions no longer interleave.
25. **Open (UEX):** no idempotency key or client report ID is documented; whether `service_unavailable`, `database_error`, `image_upload_error` and `image_storage_error` come before or after processing; whether HTTP 429 or `Retry-After` is ever sent. [3P-OBS]: a 15 s read timeout during UEX slowness did not stop processing, and no client has seen a 429 or `Retry-After`.
26. **Answered:** the traded side of a commodity at a terminal is the side with a price > 0 in `commodities_prices`; there is no capacity field (`scu_*_max` are reported maxima); `max_container_size` is the largest container size a terminal serves (0 = unknown), and no live row exceeds it; A22 (contiguous runs) holds on all 2,583 non-empty rows.
27. **Open (UEX):** `faction_affinity` – whether the terminal shows the affinity, whether reports at the 15 `is_affinity_influenceable` terminals must carry it, and how far it moves prices. Decided 2026-10-09 (O-93): optional user entry, sent only when entered, never inferred (R-SUB-13).
28. **Partly decided:** the per-type `notification` text (on 2026-10-09 for commodities: report the value shown in the terminal's "Local Market" view, since "Local Market Values" may differ from actual transaction prices). Decided 2026-10-09 (O-100): shown in the status area as untrusted plain text. Open: its bearing on which tab and price the app reads, checked with A21.
29. **New, open (UEX):** whether `data_remove` and `data_info` work on `is_production=0` submissions (`data_edit` returns `date_modified` 0 in test mode, which suggests test submissions are not stored), and whether `data_remove` by ID alone is unambiguous given that IDs repeat across partitions. Until this is confirmed, the release smoke test makes no withdrawal ([release process](../release-process.md), O-89).
