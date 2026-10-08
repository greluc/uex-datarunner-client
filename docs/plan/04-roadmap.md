# Roadmap

Reihenfolge nach Risiko und Nutzen: zuerst werden die API-Unsicherheiten geklärt. Dann folgt **manuelle Erfassung plus Senden**; das ist schon ohne OCR ein nutzbares Produkt. Danach kommt die OCR. Aufwände werden bewusst nicht in Tagen geschätzt, weil Verfügbarkeit und Erfahrung des Teams unbekannt sind.

## M0 – Fundament und Verifikation

- [ ] Projektinhaber entscheidet: Projekt- und Paketname, Lizenz (GPL-3.0 bei Code-Port aus basetool), JDK 27 vs. 25 LTS
- [ ] Gradle-Multiprojekt nach [02-architektur.md](02-architektur.md), `build-logic`, Version-Catalog, Spotless, Error Prone/NullAway, JUnit 6
- [ ] CI: GitHub Actions, Matrix Windows/Linux, `./gradlew check`
- [ ] **API-Spike** (kleine CLI im Modul `uex-api`) gegen die Live-API mit `is_production=0`; klärt alle offenen Punkte aus [06-uex-api.md](06-uex-api.md) (Header, App-Token, `status_*`, `container_sizes`, `/user`)
- [ ] Mit UEX klären, wie ein Open-Source-Client das App-Token handhaben soll
- [ ] Korpus anlegen: Patch-City-Screenshots plus weitere Terminals/Themes; Erwartungswerte transkribieren

**Abnahme:** `./gradlew check` ist auf beiden OS grün; ein Test-Report mit `is_production=0` wurde erfolgreich übermittelt und die Antwort dokumentiert.

## M1 – Referenzdaten, manuelle Erfassung, Senden (erstes nutzbares Release 0.1)

- [ ] `uex-api`: Client, Envelope, Fehlercodes, Rate-Limiter, Host-Fallback, OS-Truststore
- [ ] `refdata`: SQLite-Cache, TTL-Refresh, Vokabular-Indizes, Fuzzy-Matcher (property-getestet)
- [ ] `submission`: Payload, Queue, Cooldown, Historie, `data_remove`
- [ ] `app`: Onboarding (Key → `/user`-Check), Secret-Store (FFM: Credential Manager / libsecret), Einstellungen, **manuelle Erfassung** (R-MAN-*), Historie, Diagnose
- [ ] Packaging: MSI / `.deb` / Archive aus CI

**Abnahme:** Ein Nutzer erfasst ein Terminal manuell in < 1 min (Zielwert), sendet es und sieht die Report-IDs. Ein Cooldown überlebt den Neustart der App.

## M2 – OCR-Kern und Eval

- [ ] `ocr`: ORT-Sessions, DB-Postprocessing, CTC-Decode, Bildoperationen, Homographie
- [ ] `tools/ocr-eval`: Korpus-Runner, Metriken, Crop-Dumps, Digest
- [ ] Baseline-Messung auf dem Korpus: Rohgenauigkeit pro Feldtyp, Laufzeit, RAM

**Abnahme:** Reproduzierbarer Eval-Bericht; die Rohgenauigkeit ist dokumentiert (noch ohne Zielwert).

## M3 – Pipeline und Review-UI (Release 0.5)

- [ ] `pipeline`: Locate, Layout, Feldparser, Auflösung, Validierung, Reparatur, Stitching, Konfidenz
- [ ] `capture`: nutzerdefinierte Ordner, Button „Einlesen“ (manuell, Standard), Auto-Watch opt-in pro Ordner (WatchService plus Polling-Fallback, Nachhol-Scan), Stable-File-Gate, Verarbeitet-Register, Drag & Drop, Strg+V, Aufnahmezeit
- [ ] `app`: Queue-Ansicht, Report-Editor mit Bildausschnitten und Quell-Highlight, Sendesperre, „Alle sicheren übernehmen“
- [ ] Upload-Screenshot: Zuschnitt, Kontostand schwärzen (Test!)

**Abnahme:** Auf dem Korpus ist „still falsch“ ≈ 0. Die Feldgenauigkeit nach Validierung ist gemessen, und die Zielwerte werden **auf Basis der Messung** festgelegt und in CI eingefroren.

## M4 – Robustheit und Plattform (Release 1.0)

- [ ] Weitere Themes/Layouts (blau Standard, Nyx, Gateways, rote Scrapyard-Terminals), Ultrawide/1080p/4K
- [ ] Spiel-Lokalisierung (`global.ini`) und SC-Installationserkennung inklusive Wine/Proton (Annahmen A3/A4 verifiziert)
- [ ] Zweitleser für Status (Balken/Farbe) und Cargo-Größen
- [ ] HiDPI, Barrierefreiheit (Tastatur, Kontrast), Lokalisierung DE/EN
- [ ] Ressourcenmessung neben dem laufenden Spiel; Heap- und Thread-Limits festlegen

## M5 – Optionale KI-Erkennung bei geschlossenem Spiel (Release 1.1, vorziehbar)

Setzt M3 voraus (gemeinsame Auflösung, Validierung und Stitching). Kann parallel zu M4 laufen, wenn Kapazität da ist.

- [ ] `capture`: `GameProcessMonitor` (Windows und Linux/Wine, Hysterese); Annahme A7 auf echten Systemen verifizieren
- [ ] `vlm`: `OllamaClient` (`/api/version`, `/api/tags`, `/api/ps`, `/api/pull`, `/api/chat`), Host-Allowlist, Abbruch und Entladen (`keep_alive: 0`)
- [ ] Prompt v1 plus deterministischer Antwort-Parser (Golden-Tests mit aufgezeichneten Antworten, ohne Ollama in CI)
- [ ] `pipeline`: Leser-Abstraktion und Fusionsregeln (07 §2.7), property-getestet
- [ ] KI-Queue (persistiert), `RecognitionPolicy`, Einstellungen (Aus / Automatisch / Immer), UI: Status „KI ausstehend / läuft / fertig“, Anzeige beider Kandidaten bei Widerspruch, Modellverwaltung mit Pull-Fortschritt
- [ ] **Bake-off** auf dem Korpus: Modelle (u. a. `qwen3-vl:8b-instruct`, `qwen3-vl:4b-instruct`), Markdown- vs. `format`-Ausgabe, Bildgröße; Metriken „nur OCR“ / „nur VLM“ / „Fusion“, Laufzeit, VRAM

**Abnahme:**

- Spielstart bricht einen KI-Lauf innerhalb von ≤ 5 s ab und entlädt das Modell (Test mit simuliertem Prozess).
- Ohne Ollama gibt es keine Fehlermeldung.
- Die Fusion senkt „still falsch“ und „markiert“ auf dem Korpus messbar gegenüber „nur OCR“. **Ist das nicht der Fall, wird das Feature nicht ausgeliefert** (Annahme A8).

## Danach (1.x / später)

- Tray-Modus mit Benachrichtigung (#36)
- Items und Fahrzeugkauf/-miete (#20)
- Weitere OCR-Schriftsysteme (Kyrillisch/Koreanisch)
- Optional ein feinjustiertes Erkennungsmodell für die SC-HUD-Schrift
- Trade-Routen sind bewusst **nicht** geplant; dafür existieren spezialisierte Tools
