# Anforderungen

Prioritäten: **M** = Muss (1.0), **S** = Soll (1.x), **K** = Kann (später).

## Ziel

Spieler (UEX-DataRunner) erfassen mit dem Client Rohstoff-Terminaldaten aus Star Citizen und stellen sie UEX bereit. Dafür gibt es zwei Wege:

- **manuell:** schnelle, tastaturgetriebene Eingabe, vorbefüllt mit aktuellen UEX-Daten
- **per Screenshot:** OCR, Validierung, Review, Senden

Aktuelle UEX-Daten (Terminals, Commodities, letzte Preise, Statusstufen, Spielversion, Toleranzen) dienen als **Vorgabewerte und Constraints** für die Erkennung und die Auswahl.

## Begriffe

| Begriff | Bedeutung |
|---|---|
| **Capture** | Ein Screenshot |
| **Scan** | Ein Capture nach der OCR-Analyse |
| **Report** | Die Zusammenfassung aller Scans eines Terminals und einer Seite (Kaufen = „Buy“, Verkaufen = „Local Market Value“) innerhalb eines Zeitfensters. Der Report ist die Einheit, die an `POST /data_submit` geht. |
| **Prior** | Der UEX-Referenzwert (letzter oder durchschnittlicher Preis, SCU, Status) für die Kombination aus Terminal und Commodity |

## Scope

| ID | Anforderung | Prio |
|---|---|---|
| R-SCOPE-1 | Typ `commodity` (Kauf- und Verkaufsseite) | M |
| R-SCOPE-2 | Datenmodell und API-Schicht vorbereitet für `item`, `vehicle_buy`, `vehicle_rent` | M (Modell) / K (UI) |
| R-SCOPE-3 | **Nicht** im Scope von 1.0: Trade-Routen-Planer (Feature des Vorbilds), Fleet/Trades-Features | – |

## Erfassung (CAP)

| ID | Anforderung | Prio |
|---|---|---|
| R-CAP-1 | Mehrere Screenshot-Ordner überwachen (WatchService, zusätzlich Polling-Fallback für Wine-Prefixe und Netzlaufwerke). Pro Ordner lässt sich die Umgebung (LIVE/PTU/EPTU/HOTFIX/TECH-PREVIEW/manuell) festlegen. | M |
| R-CAP-2 | Weitere Eingangswege: Drag & Drop, Strg+V, Datei-Dialog (Mehrfachauswahl), optional Zwischenablage-Überwachung mit Fingerprint-Deduplizierung | M |
| R-CAP-3 | Die Umgebung wird aus dem Ordnerpfad abgeleitet, die Spielversion aus UEX (`game_versions`). Ist die Umgebung unbekannt, fragt die App nach; einen hartkodierten Fallback gibt es nicht. | M |
| R-CAP-4 | Automatische Erkennung der SC-Installation: RSI-Launcher-Log `Launching Star Citizen … from (…)`, laufender Prozess `Bin64/StarCitizen.exe`, Standardpfade. Unter Linux zusätzlich Wine-/Proton-Prefixe (siehe Annahme A3). | S |
| R-CAP-5 | Aufnahmezeit aus dem Dateinamen (Regex), Fallback auf `lastModified`. Duplikate erkennt die App per Inhalts-Hash. | M |
| R-CAP-6 | Optionale Aufräumfunktion: Originale nach erfolgreichem Senden löschen oder archivieren. Standard ist **aus**. | S |

## Manuelle Erfassung (MAN)

| ID | Anforderung | Prio |
|---|---|---|
| R-MAN-1 | Terminal-Auswahl per Suchfeld (Fuzzy über Name, Nickname, Location, System), Filter nach Sternsystem, „zuletzt verwendet“ | M |
| R-MAN-2 | Nach der Terminalwahl wird die Liste der bekannten Commodities des Terminals (aus `commodities_prices`) mit aktuellen UEX-Werten vorbefüllt. Der Nutzer bestätigt oder ändert nur die Abweichungen. | M |
| R-MAN-3 | Vollständige Tastaturbedienung: Tab/Enter, Zahlen-Shortcuts für die Statusstufen 1–7 | M |
| R-MAN-4 | Manuelle Reports durchlaufen dieselbe Validierung (VAL) wie OCR-Reports | M |

## Erkennung (OCR)

| ID | Anforderung | Prio |
|---|---|---|
| R-OCR-1 | Lokale OCR (keine Cloud) mit PaddleOCR-Modellen (Detektion + Erkennung) über ONNX Runtime; Modelle gebündelt, Lizenz- und Hash-Nachweis in `NOTICE` | M |
| R-OCR-2 | Lokalisierung des Terminal-Panels über Text- und Farbanker, Perspektivkorrektur. Scheitert sie, kann der Nutzer die Ecken bzw. den Ausschnitt manuell setzen; die App bricht nie hart ab. | M |
| R-OCR-3 | Theme-agnostisch: orange/rote (z. B. Patch City/Pyro), blaue und weitere Terminal-Themes. Layout-Profile sind datengetrieben. | M |
| R-OCR-4 | Erkennung von Seite (Buy / Local Market Value, aktiver Tab) und Abschnitt (IN STOCK / OUT OF STOCK / IN DEMAND / NO DEMAND / SELLABLE CARGO) | M |
| R-OCR-5 | Robuster Zahlparser: Währungssymbol `¤` abtrennen, Tausender- und Dezimaltrenner strukturell bestimmen, optionales K/M-Suffix, Suffix `/SCU` | M |
| R-OCR-6 | Reparatur verwechselbarer Ziffern ({0,6,8,9}, gegebenenfalls weitere aus dem Korpus), nur bei eindeutigem Kandidaten und mit Plausibilitätszeuge | M |
| R-OCR-7 | Commodity-Namen werden gegen das UEX-Vokabular und lokalisierte `global.ini`-Namen aufgelöst. Robust gegen Zeilenumbrüche und Überlappung mit der SCU-Angabe. | M |
| R-OCR-8 | Terminal- und Location-Erkennung primär aus dem Location-Feld unter „YOUR INVENTORIES“, abgeglichen mit dem Sortiment | M |
| R-OCR-9 | „AVAILABLE CARGO SIZE (SCU)“ → `container_sizes` | S |
| R-OCR-10 | Zweitleser für den Status: Farbe und Füllhöhe des vertikalen Lagerbalkens bzw. die Farbe des Statustexts, abgeglichen mit dem OCR-Statustext | S |
| R-OCR-11 | Stitching mehrerer gescrollter Screenshots eines Terminals bzw. einer Seite. Abgeschnittene Karten am Rand werden nur übernommen, wenn die Felder vollständig sind. Bei Konflikten wird der Wert bevorzugt, der weiter vom Viewport-Rand entfernt gelesen wurde. | M |
| R-OCR-12 | Verarbeitung im Hintergrund (begrenzte Parallelität), die UI blockiert nie. Ziel: < 3 s pro Screenshot auf einer Mittelklasse-CPU (**Zielwert, wird gemessen**). | M |

## Validierung (VAL)

| ID | Anforderung | Prio |
|---|---|---|
| R-VAL-1 | Regelbasierte Konfidenz pro Feld; Konfidenzwerte des Modells werden nicht direkt verwendet. Jede Warnung hat einen Grund-Code, der im UI angezeigt wird. | M |
| R-VAL-2 | Plausibilität gegen den UEX-Prior: Abweichung > `price_variation` % bzw. > `scu_variation` (aus `data_parameters`) führt zu einer Warnung, nicht zum Blockieren | M |
| R-VAL-3 | Konsistenzregeln: Status ↔ SCU (z. B. „Out of Stock“ ⇒ 0 SCU), Seite ↔ Abschnitt, Commodity ist an diesem Terminal kauf- bzw. verkaufbar (`is_buyable`/`is_sellable`) | M |
| R-VAL-4 | Terminal-Ambiguität: Gibt es mehrere Kandidaten, ist die Auswahl Pflicht; die App wählt nichts still vor | M |
| R-VAL-5 | Fehlende Commodities: Commodities, die laut UEX am Terminal geführt werden, aber auf keinem Scan auftauchen, werden angezeigt. Optional kann der Nutzer sie als `is_missing` markieren. | S |

## Review-UI (UI)

| ID | Anforderung | Prio |
|---|---|---|
| R-UI-1 | Eingangsliste (Queue) mit Status: wartend / verarbeitet / prüfen / bereit / gesendet / Fehler | M |
| R-UI-2 | Report-Editor: Terminal (Pflicht-Dropdown bei Ambiguität), Seite, Spielversion, Tabelle der Commodity-Zeilen mit Name, Status, SCU, Preis/SCU und Container-Größen | M |
| R-UI-3 | Zeilen erscheinen in Bildschirmreihenfolge. Zu jeder Zeile gibt es den Bildausschnitt in lesbarer Größe; beim Fokus auf ein Feld wird die Quellregion im Screenshot hervorgehoben. | M |
| R-UI-4 | Farbcodierung nach Konfidenz (ok / prüfen / Fehler) plus Text-Grund (nicht nur Farbe, wegen Barrierefreiheit) | M |
| R-UI-5 | „Alle sicheren übernehmen“; Navigation von Problem zu Problem (F8/Shift+F8) | S |
| R-UI-6 | Manueller Zuschnitt und Ecken-Korrektur, wenn die Lokalisierung scheitert | M |
| R-UI-7 | Eigenes Theme (hell/dunkel), HiDPI, frei skalierbare Fenster | M |
| R-UI-8 | Startet sofort; Laden im Hintergrund mit Statuszeile; offline nutzbar mit dem gecachten Datenstand (Senden erst bei Verbindung) | M |

## Übermittlung (SUB)

| ID | Anforderung | Prio |
|---|---|---|
| R-SUB-1 | `POST /data_submit` mit `id_terminal`, `type`, `is_production`, `game_version`, `prices[]` (getrennte Buy- und Sell-Zeilen), optional `container_sizes`, `screenshot`, `details` | M |
| R-SUB-2 | Sende-Queue: Rate-Limit-konform (120 req/min global, max. 500 Zeilen pro Report), begrenzte Parallelität, Retry mit Backoff nur bei 429/5xx/Netzfehlern | M |
| R-SUB-3 | Sendesperre bei ungeklärten Feldern oder unklarem Terminal | M |
| R-SUB-4 | Persistente Historie (SQLite): Report-IDs (`ids_reports`), Zeitpunkt, Payload-Hash, 5-Minuten-Cooldown pro Terminal/Commodity/Seite; Rückzug über `data_remove` | M |
| R-SUB-5 | Vor dem Senden: `data_parameters` prüfen (`is_accepting_reports` bzw. `is_accepting_ptu_reports`, `commodity.is_accepted`) | M |
| R-SUB-6 | Mapping aller bekannten UEX-Fehlercodes auf lokalisierte Meldungen mit Handlungsempfehlung | M |
| R-SUB-7 | Upload-Screenshot: perspektivkorrigierter Ausschnitt „SHOP INVENTORY“ plus Location-Feld, **Kontostand immer geschwärzt**, JPEG oder PNG < 10 MB (Ziel ~1–2 MP) | M |
| R-SUB-8 | Testmodus (`is_production=0`), umschaltbar in den Einstellungen und für Entwicklung und CI standardmäßig aktiv | M |

## UEX-Anbindung (API)

| ID | Anforderung | Prio |
|---|---|---|
| R-API-1 | Lokaler Cache der Referenzdaten (SQLite), TTL pro Endpoint (siehe [06](06-uex-api.md)), Refresh im Hintergrund, Anzeige des Datenstands | M |
| R-API-2 | Preis-Prior: `commodities_prices?id_terminal=` (bis zu 10 IDs pro Request) beim Öffnen eines Terminals; bei Bedarf `commodities_prices_all` (TTL 30 min) | M |
| R-API-3 | Konfigurierbarer Basis-Host mit Fallback, OS-Truststore, System-Proxy | M |
| R-API-4 | Secret-Key-Validierung und DataRunner-Status beim Onboarding und beim Start | M |
| R-API-5 | Einheitliche Envelope-Auswertung (`status`, `http_code`, `message`, `data`); defensives Parsen (Zahlen als String, 0/1-Flags) | M |

## Lokalisierung (L10N)

| ID | Anforderung | Prio |
|---|---|---|
| R-L10N-1 | UI-Sprachen mindestens Deutsch und Englisch (ResourceBundles), weitere einfach ergänzbar | M |
| R-L10N-2 | Spielsprache: Commodity- und Statusnamen sowie Tab-Labels aus der SC-`global.ini` (BOM entfernen, Formate `key=value` und `key,P=value`) | S |
| R-L10N-3 | Weitere OCR-Schriftsysteme (Kyrillisch, Koreanisch) über optional nachladbare Modelle | K |

## Qualitätssicherung (QA)

| ID | Anforderung | Prio |
|---|---|---|
| R-QA-1 | Golden-Korpus: echte Screenshots plus von Hand transkribierte Erwartungswerte (JSON). **Speicherort außerhalb des öffentlichen Repos**, falls die Bilder private Daten enthalten (Kontostand!). Gegebenenfalls nur geschwärzte Versionen einchecken. | M |
| R-QA-2 | Korpus deckt mindestens ab: Buy- und Sell-Tab, Scrollen, rote/orange Themes (Patch City), blaues Standard-Theme, Pyro, Nyx, Gateway, lange Namen, 4K/1440p/1080p/Ultrawide | M |
| R-QA-3 | Eval-CLI: Feldgenauigkeit pro Feldtyp, Anteil „still falsch“ (falsch und als sicher markiert) – **Ziel ≈ 0**, Anteil „markiert“, Laufzeit. Der Wert läuft in CI als Regressionstest (Schwellwerte, die nur steigen dürfen). | M |
| R-QA-4 | OCR-Digest-Test: Hash aller OCR-Rohausgaben über den Korpus, damit Modell- oder Runtime-Updates bewusst passieren | S |

## Nicht-funktional (NF)

| ID | Anforderung | Prio |
|---|---|---|
| R-NF-1 | Windows 10/11 x64 und Linux x64 (glibc-Distros, X11 und Wayland über XWayland) | M |
| R-NF-2 | Distribution: Windows MSI und portables ZIP; Linux `.deb` und portables `tar.gz` (App-Image von jpackage); jeweils mit gebündelter Runtime | M |
| R-NF-3 | Ressourcen: Heap begrenzt (Startwert `-Xmx512m`, durch Messung festzulegen), ONNX-Session wird nach Inaktivität freigegeben | S |
| R-NF-4 | Secret-Key nur im OS-Keystore (Windows Credential Manager, Linux Secret Service). Fallback ist eine Datei mit Rechten 0600 nach expliziter Warnung. Niemals in Logs. | M |
| R-NF-5 | Konfiguration: atomares Schreiben, Schema-Versionierung, sichtbare Fehler. Speicherorte: Windows `%APPDATA%`, Linux `$XDG_CONFIG_HOME`/`$XDG_DATA_HOME`/`$XDG_CACHE_HOME`. | M |
| R-NF-6 | Logging: rotierende Logdatei, Secrets maskiert, Diagnose-Export (Logs plus Systeminfo, ohne Secrets) | M |
| R-NF-7 | Update-Hinweis über GitHub Releases, höchstens einmal täglich, abschaltbar | S |
| R-NF-8 | Kein Eingriff ins Spiel: keine Prozess- oder Speicherzugriffe, keine Input-Injektion, keine Overlays. Nur Dateien und Zwischenablage werden gelesen. | M |

## Annahmen (zu verifizieren)

| ID | Annahme |
|---|---|
| A1 | Der Header für den Secret-Key heißt `secret_key`; die Doku-Zusammenfassungen sind widersprüchlich (`secret-key`). Muss gegen die Live-API mit `is_production=0` geprüft werden. |
| A2 | Unklar ist, ob für `data_submit` zusätzlich ein **App-Token** (`Authorization: Bearer`) nötig ist und wie ein Open-Source-Client es verteilt. Ein in die Distribution eingebettetes Token ist extrahierbar. Optionen: Rücksprache mit UEX oder ein vom Nutzer eingetragenes Token. |
| A3 | SC unter Linux läuft über Wine/Proton (z. B. LUG-Helper, Lutris). Screenshots landen dann im Prefix unter `drive_c/Program Files/Roberts Space Industries/StarCitizen/<CHANNEL>/screenshots`. Pfad und Struktur sind auf echten Linux-Installationen zu prüfen. |
| A4 | Der Windows-Standardpfad für Screenshots ist `…\StarCitizen\<CHANNEL>\screenshots`. Er wird beim Onboarding angezeigt und ist bestätigbar, nicht blind gesetzt. |
| A5 | Die Statusstufen 1–7 und ihre Namen kommen aus `commodities_status`. Die Zuordnung der Ingame-Texte (z. B. „Max Inventory“, „Out of Stock“ auf der Sell-Seite) zu den Codes ist am Korpus zu verifizieren. |
| A6 | Die Patch-City-Screenshots (2000×1125) sind möglicherweise skaliert. Ziel-Auflösungen müssen mit Original-Screenshots getestet werden. |
