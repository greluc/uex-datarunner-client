# Roadmap

Reihenfolge nach Risiko und Nutzen: zuerst werden die API-Unsicherheiten geklärt. Dann folgt **manuelle Erfassung plus Senden**; das ist schon ohne OCR ein nutzbares Produkt. Danach kommt die OCR. Aufwände werden bewusst nicht in Tagen geschätzt, weil Verfügbarkeit und Erfahrung des Teams unbekannt sind.

## M0 – Fundament und Verifikation

- [ ] Projektinhaber entscheidet: Projekt- und Paketname, Lizenz (GPL-3.0 bei Code-Port aus basetool), JDK 27 vs. 25 LTS
- [ ] **Toolchain-Check JDK 27:** Laufen Gradle 9.8.1 (Toolchain 27), Error Prone 2.50.0, NullAway, google-java-format, `org.beryx.jlink` und jpackage (WiX ≥ 4 unter Windows) mit JDK 27? Wenn nicht: JDK 25 LTS (siehe ADR)
- [ ] Gradle-Multiprojekt nach [02-architektur.md](02-architektur.md), `build-logic`, Version-Catalog, Spotless, Error Prone/NullAway, JUnit 6
- [ ] Qualitäts-Gates nach [09-engineering-prinzipien.md](09-engineering-prinzipien.md): ArchUnit-Regeln (Schichten, Zyklen, verbotene Abhängigkeiten), JaCoCo-Schwellen, Dependabot, PR-Vorlage mit Checkliste – **vor** dem ersten Fachcode, damit die Regeln von Anfang an greifen
- [ ] CI: GitHub Actions, Matrix Windows/Linux, `./gradlew check`
- [ ] **Lieferkette absichern, bevor die erste Abhängigkeit eingecheckt wird** ([10](10-supply-chain-security.md)): Repository-Beschränkung, Locking, `verification-metadata.xml` (SHA-256 + PGP) mit Keyring, Wrapper-Validierung, Actions per SHA gepinnt, minimale `permissions`, Dependabot mit Abkühlzeit, OSV-Scan, CODEOWNERS, Branch-Protection, `SECURITY.md`
- [ ] **API-Spike** (kleine CLI im Modul `adapter-uex`) gegen die Live-API mit `is_production=0`; klärt alle offenen Punkte aus [06-uex-api.md](06-uex-api.md) (Header, App-Token, `status_*`, `container_sizes`, `/user`)
- [ ] Mit UEX klären: App-Token für Open-Source-Clients, Nutzungsbedingungen, Annahmen A9–A12 (Umgebungs-Mapping, `scu_sell` vs. `scu_sell_stock`, zusammengesetzter Screenshot, Duplikatsperre)
- [ ] Korpus ausbauen: erster Eintrag `corpus/public/pyro-gateway-stanton-01` vorhanden (Transkription **vom Menschen zu verifizieren**); Patch-City-Screenshots als Dateien nachreichen; möglichst **Original-Screenshots** (verlustfrei, Originalauflösung) statt Chat-Uploads

**Abnahme:** `./gradlew check` ist auf beiden OS grün; ein Test-Report mit `is_production=0` wurde erfolgreich übermittelt und die Antwort dokumentiert. **Negativtests der Lieferkette:** Ein manipuliertes Artefakt (geänderter Hash) und eine dynamische Version lassen den Build nachweislich scheitern.

## M1 – Referenzdaten, manuelle Erfassung, Senden (erstes nutzbares Release 0.1)

- [ ] `domain`: Value Objects, Ports; `application`: Use-Cases Senden (Queue, Cooldown, Sendeschwelle), Abweichungsbewertung (R-UI-10), manuelle Erfassung – getestet mit Fakes der Ports
- [ ] `adapter-uex`: Client, DTO-Mapping, Envelope, Fehlercodes, Rate-Limiter, Host-Fallback; Kontrakt-Tests mit aufgezeichneten Antworten
- [ ] `adapter-storage`: SQLite, Migrationen, Repositories (Queue, Historie, Cache)
- [ ] `adapter-refdata`: Cache, TTL-Refresh, Vokabular-Indizes, Fuzzy-Matcher (property-getestet)
- [ ] `adapter-platform`: Secret-Store (FFM: Credential Manager / libsecret), Pfade, Truststore
- [ ] `ui`: Onboarding (Key → `/user`-Check), Einstellungen, **manuelle Erfassung** (R-MAN-*) mit Abweichungsmarkierung (R-UI-10..12), Historie, Diagnose
- [ ] `app`: Composition Root, Konfiguration
- [ ] Packaging: MSI / `.deb` / Archive aus CI, mit SHA256SUMS, SBOM und Build-Attestierung (R-SEC-6)

**Abnahme:** Ein Nutzer erfasst ein Terminal manuell in < 1 min (Zielwert), sendet es und sieht die Report-IDs. Ein Cooldown überlebt den Neustart der App.

## M2 – OCR-Kern und Eval

- [ ] `adapter-ocr`: ORT-Sessions, DB-Postprocessing, CTC-Decode, Bildoperationen, Homographie
- [ ] `tools/ocr-eval`: Korpus-Runner, Metriken, Crop-Dumps, Digest
- [ ] Baseline-Messung auf dem Korpus: Rohgenauigkeit pro Feldtyp, Laufzeit, RAM

**Abnahme:** Reproduzierbarer Eval-Bericht; die Rohgenauigkeit ist dokumentiert (noch ohne Zielwert).

## M3 – Pipeline und Review-UI (Release 0.5)

- [ ] `pipeline`: Locate, Layout, Feldparser, Auflösung, Validierung, Reparatur, Stitching, Konfidenz
- [ ] `adapter-capture` + `application`: nutzerdefinierte Ordner, Button „Einlesen“ (manuell, Standard), Auto-Watch opt-in pro Ordner (WatchService plus Polling-Fallback, Nachhol-Scan), Stable-File-Gate, Verarbeitet-Register, Drag & Drop, Strg+V, Aufnahmezeit
- [ ] `ui`: Queue-Ansicht, Report-Editor mit Bildausschnitten und Quell-Highlight, Abweichungsmarkierung gegen UEX (R-UI-10..12), Sendesperre, „Alle sicheren übernehmen“
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

- [ ] `adapter-platform`: `GameProcessMonitor` (Windows und Linux/Wine, Hysterese); Annahme A7 auf echten Systemen verifizieren
- [ ] `adapter-vlm`: `OllamaClient` (`/api/version`, `/api/tags`, `/api/ps`, `/api/pull`, `/api/chat`), Host-Allowlist, Abbruch und Entladen (`keep_alive: 0`)
- [ ] Prompt v1 plus deterministischer Antwort-Parser (Golden-Tests mit aufgezeichneten Antworten, ohne Ollama in CI)
- [ ] `pipeline`: Fusionsregeln (07 §2.7), property-getestet
- [ ] `application`: KI-Queue (persistiert), `RecognitionPolicy`; `ui`: Einstellungen (Aus / Automatisch / Immer), UI: Status „KI ausstehend / läuft / fertig“, Anzeige beider Kandidaten bei Widerspruch, Modellverwaltung mit Pull-Fortschritt
- [ ] **Bake-off** auf dem Korpus: Modelle (u. a. `qwen3-vl:8b-instruct`, `qwen3-vl:4b-instruct`), Markdown- vs. `format`-Ausgabe, Bildgröße; Metriken „nur OCR“ / „nur VLM“ / „Fusion“, Laufzeit, VRAM

**Abnahme:**

- Spielstart bricht einen KI-Lauf innerhalb von ≤ 5 s ab (2-s-Takt während eines Laufs plus Abbruch) und entlädt das Modell (Test mit simuliertem Prozess).
- Ohne Ollama gibt es keine Fehlermeldung.
- Die Fusion senkt „still falsch“ und „markiert“ auf dem Korpus messbar gegenüber „nur OCR“. **Ist das nicht der Fall, wird das Feature nicht ausgeliefert** (Annahme A8).

## Risiken

| Risiko | Auswirkung | Gegenmaßnahme / Prüfung |
|---|---|---|
| UEX verlangt ein App-Token, das ein Open-Source-Client nicht sicher verteilen kann (A2) | **Projektblocker** für das Senden | M0: früh mit UEX klären; Option „Nutzer trägt eigenes App-Token ein“ |
| Nutzungsbedingungen von UEX schließen Drittclients oder automatisierte Erfassung aus | Projektblocker | M0: Terms lesen und gegebenenfalls Freigabe einholen |
| PP-OCR ist als Primärleser für Terminals zu ungenau | Viel Handkorrektur; Kernnutzen sinkt | M2 misst früh; Gegenmaßnahmen: Fine-Tuning des Erkennungsmodells, VLM-Zweitleser (M5) |
| Spiel-Patch ändert das Terminal-Layout | Erkennung bricht | Datengetriebene Layout-Profile, Crop-Dumps, Korpus-Regressionstest; manueller Zuschnitt als Fallback |
| Toolchain unterstützt JDK 27 nicht | Build blockiert | M0-Check; Fallback JDK 25 LTS |
| Linux-Spezifika (Wine-Pfade, Spielerkennung, Secret Service, XWayland) weichen von den Annahmen ab | Linux-Funktionen eingeschränkt | Manuelle Konfiguration als Fallback überall; Tests auf echten Systemen in M4/M5 |
| Kompromittierte Abhängigkeit, CI-Aktion oder kompromittiertes Build-Artefakt | Schadcode bei Nutzern, Diebstahl des Secret-Keys | Maßnahmen S-1…S-31 in [10](10-supply-chain-security.md); Restrisiko „bösartige, korrekt signierte neue Version“ bleibt mittel |
| UEX-API ändert sich (Felder, Fehlercodes) | Senden schlägt fehl | Defensives Parsen, Kontrakt-Tests mit aufgezeichneten Antworten, verständliche Fehlermeldungen |

## Danach (1.x / später)

- Tray-Modus mit Benachrichtigung (#36)
- Items und Fahrzeugkauf/-miete (#20)
- Weitere OCR-Schriftsysteme (Kyrillisch/Koreanisch)
- Optional ein feinjustiertes Erkennungsmodell für die SC-HUD-Schrift
- Trade-Routen sind bewusst **nicht** geplant; dafür existieren spezialisierte Tools
