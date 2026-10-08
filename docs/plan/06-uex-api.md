# UEX API 2.0 – Arbeitsnotizen

> **Quellenlage, ehrlich:** Die offizielle Doku (`https://uexcorp.space/api/documentation/`) war aus der Recherche-Umgebung **nicht erreichbar** (Netzwerk-Policy bzw. DNS). Die Angaben stammen daher aus drei Quellen:
>
> - **[DOC-SNIPPET]:** Ausschnitte der offiziellen Doku in Suchergebnissen
> - **[DOC-KOPIE]:** von Dritten eingecheckte Kopien der Doku-Seiten (Stand ca. Anfang 2025)
> - **[3P]:** Open-Source-Clients, nämlich `dolejska-daniel/uexcorp-openapi`, `dolejska-daniel/starcitizen-kioskprobe`, `Zamotic/UexCorpDataRunnerClient`, `Hybris95/UEX-Trader`, `ByteCollectiveIO/sc-nav` (Doku-Zusammenfassung vom 2026-07-25) und PyPI `uex-lib`
>
> **Alles hier ist vor der Implementierung gegen die offizielle Doku und die Live-API (`is_production=0`) zu verifizieren** – siehe Meilenstein M0 in [04-roadmap.md](04-roadmap.md).

## Basis

| Thema | Wert | Quelle |
|---|---|---|
| Base-URL | `https://api.uexcorp.space/2.0/`, Mirror `https://api.uexcorp.uk/2.0/` | 3P, DOC-SNIPPET |
| App-Auth | `Authorization: Bearer <app token>` (App unter „My Apps“ anlegen) | DOC-SNIPPET |
| User-Auth | Header `secret_key: <40 Zeichen>`, **Schreibweise unsicher** (`secret-key` in manchen Quellen) | DOC-KOPIE, 3P |
| Envelope | `{"status":"ok","http_code":200,"message":"","data":…}`, Fehler als String-Code in `status` | 3P |
| Rate-Limit | 172.800 Requests/Tag (≈ 120/min); bei Überschreitung `status: requests_limit_reached` bzw. 429 | DOC-SNIPPET |
| Datentypen | Zahlen teils als String, Flags als 0/1 → defensiv parsen | 3P |

## Einreichen: `POST /data_submit`

Doku-Slug: `post_data_submit`.

```json
{
  "id_terminal": 123,
  "type": "commodity",
  "is_production": 0,
  "game_version": "4.x.y",
  "container_sizes": "1,2,4,8,16,24,32",
  "screenshot": "<base64 ohne data:-Präfix, ≤ 10 MB>",
  "details": "optional",
  "prices": [
    {"id_commodity": 7, "price_buy": 3237, "scu_buy": 333, "status_buy": 4},
    {"id_commodity": 9, "price_sell": 36000, "scu_sell": 0, "status_sell": 1}
  ]
}
```

- `type`: `commodity | item | vehicle_buy | vehicle_rent`
- Zeilenfelder: `id_commodity`, `price_buy`/`price_sell` (UEC pro SCU), `scu_buy`/`scu_sell`, `status_buy`/`status_sell` (1–7), `is_missing` (0/1), optional `quality` (0–1000)
- `container_sizes`: kommagetrennte Teilmenge von {1,2,4,8,16,24,32} – **offen:** ob pro Report oder pro Zeile; die Quellen zeigen nur pro Report
- **Screenshot ist Pflicht** für neue DataRunner während der Evaluationsphase (`evaluation_period_days`, z. B. 90)
- Limits [3P/sc-nav]: 500 Zeilen pro Report; 1000 Reports pro 30 min; kein Duplikat (Terminal + Item) innerhalb von 5 min
- Erfolg: `data: {"ids_reports": ["…"], "date_added": <unix>, "username": "…"}`; die Reihenfolge von `ids_reports` entspricht `prices[]`
- Fehlercodes:
  - **gesichert:** `missing_id_terminal`, `terminal_not_found`, `invalid_type`, `user_not_found`, `user_not_allowed`
  - **laut Zusammenfassung:** `invalid_input`, `missing_secret_key`, `invalid_secret_key`, `max_rows_exceeded`, `duplicated_report`, `screenshot_required`, `invalid_game_version`, `user_disabled`
- Verwandte Endpoints:
  - `GET /data_info` (Status eigener Reports)
  - `data_edit`
  - `data_remove` (zurückziehen; hebt die Duplikatsperre auf)

## Referenzdaten

| Endpoint | Nutzen | TTL (Vorschlag) |
|---|---|---|
| `GET /commodities` | Vokabular: `id`, `name`, `code`, `slug`, `is_buyable`, `is_sellable`, `is_available_live`, `is_visible`, … | 1 h |
| `GET /terminals?type=commodity` | Terminals: `id`, `name`, `nickname`, `displayname`, `code`, Location-IDs/-Namen, `is_available_live`, `is_player_owned`, `max_container_size` | 12 h |
| `GET /commodities_prices?id_terminal=a,b,…` (≤ 10) | **Prior** pro Terminal: aktuelle Werte plus min/max/avg (Woche/Monat), `status_*`, `container_sizes` | 30 min |
| `GET /commodities_prices_all` | Kompakter Komplettstand (Fallback/Offline) | 30 min |
| `GET /commodities_status` | Statusstufen 1–7 je Seite (`buy[]`/`sell[]`) mit Namen, Kürzel, Prozentband, Farben | 1 Tag |
| `GET /game_versions` | `{"live":"…","ptu":"…"}` | 1 Tag |
| `GET /data_parameters` | `is_accepting_reports`, `is_accepting_ptu_reports`, `commodity.price_variation`, `commodity.scu_variation`, `evaluation_period_days` (**Einheiten unbestätigt**) | 1 Tag |
| `GET /star_systems`, `/planets`, `/moons`, `/orbits`, `/space_stations`, `/cities`, `/outposts` | Location-Namen für das Feld „YOUR INVENTORIES“ und die Disambiguierung | 1 Tag |
| `GET /user` | `is_datarunner`, `is_datarunner_banned`, `username` – Key-Prüfung (Verhalten nur mit `secret_key` **unsicher**) | – |

## Offene Punkte (M0-Verifikation)

1. Header-Schreibweise `secret_key` vs. `secret-key`
2. Wird für `data_submit` zusätzlich ein App-Bearer-Token benötigt? Wenn ja: Wie verteilt ein Open-Source-Client es (Nutzer-Token vs. eingebettet)? Am besten mit UEX klären.
3. Ist `status_*` Pflicht? Ist `container_sizes` pro Report oder pro Zeile?
4. Welcher Host nimmt aktuell POSTs an?
5. Verhalten von `GET /user` nur mit Secret-Key
6. Aktuelle Fehlercode-Liste und Einheiten von `price_variation`, `scu_variation` und `ttl`
7. Nutzungsbedingungen (`https://uexcorp.space/about/legal`) für Drittclients, User-Agent-Konvention
8. **Sell-Seite:** Bedeutet die „… SCU“-Zahl im Terminal `scu_sell` oder `scu_sell_stock`? (Annahme A10) – **Indiz** aus den Pyro-Gateway-Screenshots: Die Zahl steht zusammen mit einem Lagerstatus (1,482 SCU, VERY LOW INVENTORY), spricht also eher für den Lagerbestand (`scu_sell_stock`).
9. **`container_sizes`:** Im Spiel unterscheiden sich die Größen **pro Commodity** (Patch City: Omnapoxy 1–16, Human Food Bars 8–32). Gibt es das Feld nur pro Report, muss entschieden werden, ob es dann weggelassen wird (Vorschlag, statt Informationen zu vermischen) oder ob pro Commodity einzeln gesendet wird.
10. **Screenshot:** Wird ein aus mehreren Scroll-Ausschnitten zusammengesetztes Bild akzeptiert? Werden manuelle Reports ohne Screenshot angenommen? (Annahme A11)
11. **Duplikatsperre:** Gilt die 5-Minuten-Sperre pro (Terminal, Commodity) oder zusätzlich pro Seite? (Annahme A12)
12. **`is_missing`:** Welche Felder sind bei `is_missing = 1` nötig bzw. erlaubt?
13. **Umgebungen:** Wie ordnet UEX HOTFIX/EPTU/TECH-PREVIEW zu? (Annahme A9)
14. **Mischen in einem Payload:** Ein Payload kann laut Beispielen Buy- und Sell-Zeilen enthalten. Wir senden trotzdem **einen Report pro Seite**, weil jeder Report einen eigenen Screenshot der jeweiligen Seite hat.
