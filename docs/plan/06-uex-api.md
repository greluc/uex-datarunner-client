# UEX API 2.0 – working notes

> **Doc type:** Living spec — binding. Last reviewed: 2026-10-08.

> **Sources, honestly:** The official documentation (`https://uexcorp.space/api/documentation/`) was **not reachable** from the research environment (network policy or DNS). The information therefore comes from three sources:
>
> - **[DOC-SNIPPET]:** excerpts of the official documentation in search results
> - **[DOC-COPY]:** copies of the documentation pages checked in by third parties (as of roughly early 2025)
> - **[3P]:** open-source clients, namely `dolejska-daniel/uexcorp-openapi`, `dolejska-daniel/starcitizen-kioskprobe`, `Zamotic/UexCorpDataRunnerClient`, `Hybris95/UEX-Trader`, `ByteCollectiveIO/sc-nav` (documentation summary from 2026-07-25) and PyPI `uex-lib`
>
> **Everything here must be verified against the official documentation and the live API (`is_production=0`) before implementation** – see milestone M0 in [04-roadmap.md](04-roadmap.md).

## Basics

| Topic | Value | Source |
|---|---|---|
| Base URL | `https://api.uexcorp.space/2.0/`, mirror `https://api.uexcorp.uk/2.0/` (operator unverified, open point 4) | 3P, DOC-SNIPPET |
| App auth | `Authorization: Bearer <app token>` (create the app under "My Apps") | DOC-SNIPPET |
| User auth | Header `secret_key: <40 characters>`, **spelling uncertain** (`secret-key` in some sources) | DOC-COPY, 3P |
| Envelope | `{"status":"ok","http_code":200,"message":"","data":…}`, errors as a string code in `status` | 3P |
| Rate limit | 172,800 requests/day (≈ 120/min); when exceeded `status: requests_limit_reached` or 429 | DOC-SNIPPET |
| Data types | Numbers partly as strings, flags as 0/1 → parse defensively | 3P |

## Submitting: `POST /data_submit`

Documentation slug: `post_data_submit`.

```json
{
  "id_terminal": 123,
  "type": "commodity",
  "is_production": 0,
  "game_version": "4.x.y",
  "container_sizes": "1,2,4,8,16,24,32",
  "screenshot": "<base64 without data: prefix, ≤ 10 MB>",
  "details": "optional",
  "prices": [
    {"id_commodity": 7, "price_buy": 3237, "scu_buy": 333, "status_buy": 4},
    {"id_commodity": 9, "price_sell": 36000, "scu_sell": 0, "status_sell": 1}
  ]
}
```

- `type`: `commodity | item | vehicle_buy | vehicle_rent`
- Row fields: `id_commodity`, `price_buy`/`price_sell` (UEC per SCU), `scu_buy`/`scu_sell`, `status_buy`/`status_sell` (1–7), `is_missing` (0/1), optional `quality` (0–1000)
- `container_sizes`: comma-separated subset of {1,2,4,8,16,24,32} – **open:** whether per report or per row; the sources only show per report
- **Screenshot is mandatory** for new DataRunners during the evaluation period (`evaluation_period_days`, e.g. 90)
- Limits [3P/sc-nav]: 500 rows per report; 1000 reports per 30 min; no duplicate (terminal + item) within 5 min
- Success: `data: {"ids_reports": ["…"], "date_added": <unix>, "username": "…"}`; the order of `ids_reports` reportedly matches `prices[]` (unverified; R-SUB-9 maps by position only when the counts match, open point 24)
- Error codes:
  - **confirmed:** `missing_id_terminal`, `terminal_not_found`, `invalid_type`, `user_not_found`, `user_not_allowed`
  - **according to the summary:** `invalid_input`, `missing_secret_key`, `invalid_secret_key`, `max_rows_exceeded`, `duplicated_report`, `screenshot_required`, `invalid_game_version`, `user_disabled`
- Related endpoints:
  - `GET /data_info` (status of own reports)
  - `data_edit`
  - `data_remove` (withdraw; lifts the duplicate block)

Error classes (R-SUB-11; every known code has exactly one class, whatever the HTTP status; codes not listed count as permanent):

| Code / answer | Class | Handling |
|---|---|---|
| connect-phase failure, HTTP 429, `requests_limit_reached` (HTTP 200) | transient (not processed, R-SUB-9) | retry with backoff; wait for the row budget or `Retry-After` |
| `duplicated_report` | cooldown | cooldown for every row key, the job waits and is sent again; after an unknown outcome see R-SUB-9 |
| `missing_secret_key`, `invalid_secret_key`, `user_not_found`, `user_not_allowed`, `user_disabled`; HTTP 401/403 without a known code (assumption, open point 6) | account | pause the whole queue; persistent "re-authentication required" state with an action (R-UI-16) |
| `is_accepting_reports`/`is_accepting_ptu_reports` = 0, `commodity.is_accepted` = 0 (pre-send check, R-SUB-5) | acceptance closed | hold, re-check every 15 min (R-VAL-6 applies) |
| `screenshot_required`, `max_rows_exceeded`, `terminal_not_found` | report-fixable | back to Draft with a finding (`terminal_not_found`: select the terminal again, refresh the vocabulary) |
| `invalid_game_version` | permanent (not logged as a client bug) | Rejected (final); "duplicate as new draft" is not offered, because a duplicate keeps the capture-time version (R-CAP-3b, open point 21, A18) |
| `missing_id_terminal`, `invalid_type`, `invalid_input` | permanent (suspected client bug, logged with the app version) | Rejected, with "duplicate as new draft" |
| any HTTP 5xx without a known code; request timeout, connection reset or other I/O failure after connecting; empty or unparseable answer | – (unknown outcome, R-SUB-9) | never retried automatically |

## Reference data

| Endpoint | Use | TTL (proposal) |
|---|---|---|
| `GET /commodities` | Vocabulary: `id`, `name`, `code`, `slug`, `is_buyable`, `is_sellable`, `is_available_live` (filters the LIVE vocabulary only; PTU: open point 19), `is_visible`, `price_buy`/`price_sell` (assumed commodity-wide average in UEC/SCU; 0 = no value in the [3P] uex-lib fixture; unverified, open point 23; used by R-VAL-2a), … | 1 h |
| `GET /terminals?type=commodity` | Terminals: `id`, `name`, `nickname`, `displayname`, `code`, location IDs/names, `is_available_live`, `is_player_owned`, `max_container_size`; `is_available_live` filters the LIVE vocabulary only; PTU availability: open point 19 | 12 h |
| `GET /commodities_prices?id_terminal=a,b,…` (≤ 10) | **Prior** per terminal: current values plus min/max/avg (week/month), `status_*`, `container_sizes`, `date_modified` (age of the prior; only the current values are used as the prior, [11](11-ddd-and-tdd.md) §A1; semantics: open point 23) | 30 min |
| `GET /commodities_prices_all` | Compact complete snapshot (fallback/offline) | 30 min |
| `GET /commodities_status` | Status levels 1–7 per side (`buy[]`/`sell[]`) with names, abbreviations, percentage band, colors | 1 day |
| `GET /game_versions` | `{"live":"…","ptu":"…"}`; `ptu` can be `null` when no PTU is running [3P: uex-lib fixture]; the delay between a patch and the new value, and whether every patch that can change the terminal UI changes the version string, are unverified (A17; measured on the next patch day, result recorded here) | 1 day; additionally fetched at startup, before an import batch is versioned, on game start and ≤ 15 min before every send attempt (R-CAP-3b, R-SUB-5); every successful fetch updates the observed version history |
| `GET /data_parameters` | Nested per the [3P] uex-lib fixture and the traffic-derived `dolejska-daniel/uexcorp-openapi` spec (unverified; structure: open point 22): `global.is_accepting_reports`, `global.is_accepting_ptu_reports`, `global.evaluation_period_days`, `global.game_version`/`game_version_ptu`; one object per report type (`commodity`, `item`, `vehicle_buy`, `vehicle_rent`) with its own `is_accepted` – a flag **per report type**, not per commodity; for `commodity`: `is_accepted` (0/1, reports of type `commodity` accepted), `price_variation`, `scu_variation`, `ttl`, `is_temporary_enabled` (**units unconfirmed**, A15) | 1 day for tolerances; **≤ 15 min before every send attempt** for `global.is_accepting_*` and `commodity.is_accepted` (R-SUB-5) |
| `GET /star_systems`, `/planets`, `/moons`, `/orbits`, `/space_stations`, `/cities`, `/outposts` | Location names for the "YOUR INVENTORIES" field and disambiguation | 1 day |
| `GET /user` | `is_datarunner`, `is_datarunner_banned`, `username` – key check (behavior with only `secret_key` **uncertain**) | – |

## Open points (M0 verification)

1. Header spelling `secret_key` vs. `secret-key`
2. Is an app bearer token additionally required for `data_submit`? If so: how does an open-source client distribute it (user token vs. embedded)? Best clarified with UEX.
3. Is `status_*` mandatory? Is `container_sizes` per report or per row? The answer updates the "Mandatory field" definition in [11](11-ddd-and-tdd.md) §A1 in the same commit.
4. Which hosts are operated by UEX (`api.uexcorp.space`, `api.uexcorp.uk`), and which of them accept POSTs? Confirm with UEX or the official documentation, not with third-party clients. Also record per endpoint whether `secret_key` and the app token are required (R-API-3).
5. Behavior of `GET /user` with only the secret key
6. Current error code list and units of `price_variation`, `scu_variation` and `ttl`; does UEX answer HTTP 401/403 without a status code (classified as account until verified, R-SUB-11); is `commodity.is_accepted` a flag for commodity reports as a whole (as the [3P] spec suggests; see point 22)?
7. Terms of use (`https://uexcorp.space/about/legal`) for third-party clients, user agent convention; whether recorded UEX responses may be published in the public corpus and in test fixtures ([07](07-ocr-concept.md) §4)
8. **Sell side:** Does the "… SCU" number in the terminal mean `scu_sell` or `scu_sell_stock`? (assumption A10) – **Indication** from the Pyro Gateway screenshots: the number appears together with a stock status (1,482 SCU, VERY LOW INVENTORY), so it rather points to the stock level (`scu_sell_stock`).
9. **`container_sizes`:** In the game the sizes differ **per commodity** (Patch City: Omnapoxy 1–16, Human Food Bars 8–32). If the field exists only per report, it must be decided whether it is then omitted (proposal, instead of mixing information) or whether each commodity is sent individually. The answer updates the "Mandatory field" definition in [11](11-ddd-and-tdd.md) §A1 in the same commit.
10. **Screenshot:** Is an image stitched together from several scroll crops accepted? Are manual reports without a screenshot accepted? (assumption A11) How can a client learn before sending that a screenshot is required: does `GET /user` expose a DataRunner start date or an evaluation flag? Does a test submission (`is_production=0`) without a screenshot return `screenshot_required` during the evaluation period? If so, the onboarding test submission (R-SUB-8) can detect it (R-MAN-5).
11. **Duplicate block:** Does the 5-minute block apply per (terminal, commodity) or additionally per side? Is it scoped per user, does it also apply to `is_production=0`, and are live and PTU reports distinguished? (assumption A12)
12. **`is_missing`:** Which fields are required or allowed with `is_missing = 1`? Must be verified before R-VAL-5 offers the mark.
13. **Environments:** How does UEX map HOTFIX/EPTU/TECH-PREVIEW? (assumption A9)
14. **Mixing in one payload:** According to examples, a payload can contain buy and sell rows. We nevertheless send **one report per side**, because each report has its own screenshot of the respective side.
15. **`data_remove` / `data_info`:** exact parameters (report ID list? per row?) and whether `data_remove` is allowed for every own report or only within a time window – needed for withdrawal (I4) and the history status. Can `data_info` find a user's own reports without `ids_reports` (e.g. by terminal, commodity and time window)? Without that, an `OutcomeUnknown` report can only be resolved by the user (R-SUB-9).
16. **Rate budgets:** is the 1000-reports-per-30-min limit counted per price row (as `ids_reports` suggests) and per user or per app token?
17. **`data_edit`:** parameters and semantics (edit a submitted row in place?); decides whether R-SUB-10 offers an in-place edit with a new confirmation or only withdraw (`data_remove`) plus duplicate as a new draft (see also I4 in [11-ddd-and-tdd.md](11-ddd-and-tdd.md)).
18. **Observation time:** does `data_submit` accept a timestamp of the observation (relevant for R-VAL-6, R-VAL-8 and catch-up imports)? **Indication** [3P]: a UEX dev note (#151, 2025-05-14, quoted in the traffic-derived `dolejska-daniel/uexcorp-openapi` spec) says `data_submit` got an optional `date_added` parameter "to allow setting a custom report creation date". To verify with `is_production=0`: the name, format and allowed range, and whether UEX then orders "latest" values by it. Until verified, UEX presumably treats the most recently submitted report as the latest value. If `date_added` is verified, the client sends `observedAt` there, and R-VAL-8 is reassessed.
19. **PTU data:** does UEX provide PTU prices/terminals (separate endpoint, flag or parameter)? Needed for R-API-6.
20. **User-Agent:** is there a convention, and can UEX block a faulty client version by User-Agent? (R-API-7)
21. **Older game version:** does `data_submit` accept a `game_version` that differs from the current `game_versions` value of the environment (e.g. the previous version shortly after a patch), and what exactly triggers `invalid_game_version`? Check with `is_production=0` (R-CAP-3b, R-SUB-11, assumption A18).
22. **`data_parameters` structure:** confirm the nesting seen in the [3P] uex-lib fixture and the traffic-derived `dolejska-daniel/uexcorp-openapi` spec – `global.*` for the acceptance flags, game version and evaluation period; one object per report type (`commodity`, `item`, `vehicle_buy`, `vehicle_rent`) with `is_accepted`, `price_variation`, `ttl`, … – and that `commodity.is_accepted` is the acceptance of report type `commodity`, not a per-commodity flag (R-SUB-5, R-SUB-11, A15). Also check whether `global.game_version`/`game_version_ptu` always equal `GET /game_versions`. Until verified, the game version and the observed version history come only from `/game_versions` (R-CAP-3b); `data_parameters` is not a version source.
23. **Prior and average semantics:** (a) `/commodities` `price_buy`/`price_sell`: a commodity-wide average over which terminals and period, LIVE only or per environment, and does 0 mean "no value" (the [3P] fixture shows 0 for a buyable commodity)? Needed for R-VAL-2a and R-API-6. (b) `commodities_prices`: how a side without data is encoded (the [3P] fixture shows 0 for that side's price, SCU and status) and whether `date_modified` is the time of the last accepted report (one timestamp per row covers both sides) – needed for the prior's age ([11](11-ddd-and-tdd.md) §A1, R-UI-11). Until verified, a price of 0 or a status outside the `commodities_status` levels counts as "no prior" for that side, never as a value; `adapter-uex` maps it so.
24. **Partial acceptance:** can `ids_reports` contain fewer IDs than `prices[]` has rows? If so, how are the accepted rows identified – is the order kept, and does `data_info` return `id_commodity` and the side for each report ID? Needed for the per-row mapping of `PartiallyAccepted` (R-SUB-9; see point 15).
25. **Retry safety of `data_submit`:** does it accept an idempotency key or a client-side report ID? Does UEX answer 503 (with `Retry-After`) only before processing a request? Until verified, every 5xx without a known status code counts as an unknown outcome (R-SUB-9).
26. **Fields used by recognition checks:**
    - How does `commodities_prices` encode whether a commodity is offered on the buy or the sell side of a terminal? Needed for the side-specific assortment check ([07](07-ocr-concept.md) §2.5 item 2).
    - Which fields hold the SCU maximum per terminal, commodity and side (e.g. `scu_buy_max`; names unverified)? Needed for the MAX INVENTORY check (07 §1b).
    - What does `terminals.max_container_size` mean? Needed for the cargo-size check (07 §2.4).
