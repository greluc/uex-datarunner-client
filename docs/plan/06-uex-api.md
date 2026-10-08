# UEX API 2.0 – working notes

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
| Base URL | `https://api.uexcorp.space/2.0/`, mirror `https://api.uexcorp.uk/2.0/` | 3P, DOC-SNIPPET |
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
- Success: `data: {"ids_reports": ["…"], "date_added": <unix>, "username": "…"}`; the order of `ids_reports` matches `prices[]`
- Error codes:
  - **confirmed:** `missing_id_terminal`, `terminal_not_found`, `invalid_type`, `user_not_found`, `user_not_allowed`
  - **according to the summary:** `invalid_input`, `missing_secret_key`, `invalid_secret_key`, `max_rows_exceeded`, `duplicated_report`, `screenshot_required`, `invalid_game_version`, `user_disabled`
- Related endpoints:
  - `GET /data_info` (status of own reports)
  - `data_edit`
  - `data_remove` (withdraw; lifts the duplicate block)

## Reference data

| Endpoint | Use | TTL (proposal) |
|---|---|---|
| `GET /commodities` | Vocabulary: `id`, `name`, `code`, `slug`, `is_buyable`, `is_sellable`, `is_available_live`, `is_visible`, … | 1 h |
| `GET /terminals?type=commodity` | Terminals: `id`, `name`, `nickname`, `displayname`, `code`, location IDs/names, `is_available_live`, `is_player_owned`, `max_container_size` | 12 h |
| `GET /commodities_prices?id_terminal=a,b,…` (≤ 10) | **Prior** per terminal: current values plus min/max/avg (week/month), `status_*`, `container_sizes` | 30 min |
| `GET /commodities_prices_all` | Compact complete snapshot (fallback/offline) | 30 min |
| `GET /commodities_status` | Status levels 1–7 per side (`buy[]`/`sell[]`) with names, abbreviations, percentage band, colors | 1 day |
| `GET /game_versions` | `{"live":"…","ptu":"…"}` | 1 day |
| `GET /data_parameters` | `is_accepting_reports`, `is_accepting_ptu_reports`, `commodity.price_variation`, `commodity.scu_variation`, `evaluation_period_days` (**units unconfirmed**) | 1 day |
| `GET /star_systems`, `/planets`, `/moons`, `/orbits`, `/space_stations`, `/cities`, `/outposts` | Location names for the "YOUR INVENTORIES" field and disambiguation | 1 day |
| `GET /user` | `is_datarunner`, `is_datarunner_banned`, `username` – key check (behavior with only `secret_key` **uncertain**) | – |

## Open points (M0 verification)

1. Header spelling `secret_key` vs. `secret-key`
2. Is an app bearer token additionally required for `data_submit`? If so: how does an open-source client distribute it (user token vs. embedded)? Best clarified with UEX.
3. Is `status_*` mandatory? Is `container_sizes` per report or per row?
4. Which host currently accepts POSTs?
5. Behavior of `GET /user` with only the secret key
6. Current error code list and units of `price_variation`, `scu_variation` and `ttl`
7. Terms of use (`https://uexcorp.space/about/legal`) for third-party clients, user agent convention
8. **Sell side:** Does the "… SCU" number in the terminal mean `scu_sell` or `scu_sell_stock`? (assumption A10) – **Indication** from the Pyro Gateway screenshots: the number appears together with a stock status (1,482 SCU, VERY LOW INVENTORY), so it rather points to the stock level (`scu_sell_stock`).
9. **`container_sizes`:** In the game the sizes differ **per commodity** (Patch City: Omnapoxy 1–16, Human Food Bars 8–32). If the field exists only per report, it must be decided whether it is then omitted (proposal, instead of mixing information) or whether each commodity is sent individually.
10. **Screenshot:** Is an image stitched together from several scroll crops accepted? Are manual reports without a screenshot accepted? (assumption A11)
11. **Duplicate block:** Does the 5-minute block apply per (terminal, commodity) or additionally per side? (assumption A12)
12. **`is_missing`:** Which fields are required or allowed with `is_missing = 1`?
13. **Environments:** How does UEX map HOTFIX/EPTU/TECH-PREVIEW? (assumption A9)
14. **Mixing in one payload:** According to examples, a payload can contain buy and sell rows. We nevertheless send **one report per side**, because each report has its own screenshot of the respective side.
