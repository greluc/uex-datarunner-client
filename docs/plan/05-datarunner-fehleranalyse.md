# Fehleranalyse SC-Datarunner-UEX und Fixes in unserem Client

## Quellenlage – bitte beachten

- SC-Datarunner-UEX ist **closed source**. Das Repo enthält nur README, Bilder und Pages-Konfiguration; die Binaries liegen unter „Releases“. Es gibt keinen Quellcode, den wir prüfen könnten. Die Binaries wurden bewusst **nicht** dekompiliert.
- Grundlage sind die öffentlichen Issues (#2–#38), die Release-Notes (v0.5.1–v0.12.0) und die README.
- Kommentare in den Issues waren ohne Login nicht sichtbar. Eine Zuordnung „Issue → Fix-Version“ ist deshalb oft nur **[abgeleitet]**, über Datumsabgleich.
- Zusätzlich gibt es **eigene Beobachtungen** an den vom Projektinhaber gelieferten Patch-City-Screenshots und am offiziellen App-Screenshot im README. Sie sind als **[beobachtet]** markiert.

## Übersicht: Problem → Ursache → Fix in unserem Client

Die Spalte „Req“ verweist auf Anforderungen in [01-anforderungen.md](01-anforderungen.md).

### OCR und Parsing

| # | Problem (Quelle) | Ursache (bekannt/vermutet) | Unser Fix | Req |
|---|---|---|---|---|
| F1 | **Währungssymbol `¤` wird als `9`/`@` gelesen** → „¤6,705“ wird zu „96705“ ([#32](https://github.com/Shebuka/SC-Datarunner-UEX/issues/32)) | Das Symbol steht direkt vor den Ziffern und wird als Teil der Zahl erkannt | 1. Preis-Parser nach dem Muster `<Symbol><Zahl>/SCU`: das Symbol wird geometrisch über die erste Glyphe im Preis-Token abgetrennt und nie als Ziffer gewertet. 2. Kandidatenbildung mit und ohne führende Glyphe, bewertet gegen den UEX-Preisprior (letzter Preis an diesem Terminal, `price_variation` aus `data_parameters`). 3. Ist das nicht eindeutig, wird das Feld markiert, nie still übernommen. | R-OCR-5, R-VAL-2 |
| F2 | **6↔8-Verwechslung** (bekannte Einschränkung seit v0.5.1, auch nach PaddleOCR) | HUD-Schrift, Unschärfe, chromatische Aberration | Confusable-Set {0,6,8,9} (Konzept aus basetool, neu implementiert). Reparatur nur bei eindeutigem Kandidaten; **automatisch nur mit unabhängigem Zeugen** (Glyph-Topologie, Zweitleser), sonst Bestätigung per Tastendruck (R-VAL-2b). Der topologische Glyph-Klassifikator (Löcher zählen) dient zusätzlich als Veto. Hinweis an Nutzer: chromatische Aberration auf 0 stellen. | R-OCR-6, R-VAL-2b |
| F3 | **Dezimal-/Tausendertrenner** – Locale-Verwirrung `123.456,67` vs. `123,456.78` ([#21](https://github.com/Shebuka/SC-Datarunner-UEX/issues/21)), Dezimalstellen ([#5](https://github.com/Shebuka/SC-Datarunner-UEX/issues/5)) | Parser kennt nur ein Format | Strukturbasierter Zahlparser: 3er-Gruppen gelten als Tausendertrenner; ein einzelner Trenner gefolgt von 1–2 Ziffern ist dezimal. Dezimalpreise werden unterstützt (UEX `price_*` ist float). Die Locale der OS-Einstellung wird ignoriert. | R-OCR-5 |
| F4 | **Preise um Faktor 1000 falsch** nach SC 4.7 (volle aUEC statt K/M) ([#19](https://github.com/Shebuka/SC-Datarunner-UEX/issues/19)) | Parser nahm K/M-Suffix an | Beide Formen parsen: Suffix K/M optional, Skalierung explizit. Der Plausibilitätscheck gegen den UEX-Prior fängt Faktor-1000-Fehler ab. | R-OCR-5, R-VAL-2 |
| F5 | **Mehrzeilige Namen** („Recycled Material“ + „Composite“, „Ship Ammunition“ + „Size 1“) (v0.8.1/v0.12.0) | Zeilenbasierte OCR ohne Vokabular | Namen werden **gegen das UEX-Vokabular** aufgelöst (inkl. lokalisierter Namen aus `global.ini`), nicht zeilenweise. Mehrere Text-Boxen einer Karte werden gemeinsam gematcht. | R-OCR-7 |
| F6 | **[beobachtet] Name und SCU-Menge überlappen**: „Recycled Material Compos**0 SCU**ite“ (Patch-City-Screenshots); im README-Screenshot fehlt bei genau diesem Eintrag die SCU-Zahl | Lange Namen laufen im Spiel in die rechtsbündige Mengenangabe | Regex `(\d[\d,.]*)\s*SCU` auch *innerhalb* zusammengeklebter Tokens; der Namensrest wird per Präfix-/Fuzzy-Match aufgelöst. Ist die Menge unlesbar, wird sie markiert (nicht leer gesendet). | R-OCR-7, R-VAL-1 |
| F7 | **„Failed to process image“** ([#3](https://github.com/Shebuka/SC-Datarunner-UEX/issues/3), [#8](https://github.com/Shebuka/SC-Datarunner-UEX/issues/8), [#18](https://github.com/Shebuka/SC-Datarunner-UEX/issues/18)); Ursache laut v0.6.8.1 ein zu strenges Template-Matching bei hellen Terminals | Ein fest vorgegebenes Referenzbild für die Perspektivkorrektur | Lokalisierung über **Text-Anker** („SHOP INVENTORY“, „YOUR INVENTORIES“, „COMMODITIES“) plus Farbanker – kein Pixel-Template. **Nie hart abbrechen:** Wenn die Lokalisierung scheitert, setzt der Nutzer die vier Ecken manuell bzw. wählt den Ausschnitt per Maus; danach läuft die normale Pipeline. | R-OCR-2, R-UI-6 |
| F8 | **Rote Terminals / Nyx-Layout / Pyro-Schrift** ([#23](https://github.com/Shebuka/SC-Datarunner-UEX/issues/23), [#30](https://github.com/Shebuka/SC-Datarunner-UEX/issues/30)) | Farbschwellen auf ein Theme abgestimmt | Theme-agnostische Vorverarbeitung (Luminanz/Max-Kanal statt fester Farbe). Mehrere Layout-Profile werden datengetrieben als JSON-Ressource geladen. Die Patch-City-Screenshots (rot/orange) gehören von Anfang an in den Testkorpus. | R-OCR-3, R-QA-2 |
| F9 | **Reihenfolge in der App ≠ Reihenfolge im Spiel** ([#37](https://github.com/Shebuka/SC-Datarunner-UEX/issues/37)) | Sortierung nach internem Kriterium | Die Review-Liste behält die **Bildschirmreihenfolge**, auch nach dem Stitching. Jede Zeile zeigt ihren Bildausschnitt in lesbarer Größe. | R-UI-3 |
| F10 | **Gesamtgenauigkeit, „halbe Daten manuell“** ([#32](https://github.com/Shebuka/SC-Datarunner-UEX/issues/32), [#37](https://github.com/Shebuka/SC-Datarunner-UEX/issues/37)) | Kombination aus F1–F9 | Mehrstufig: Vokabular-Constraint, UEX-Prior, Confusable-Repair, Zweitleser (Statusband-Farbe/Füllhöhe, Cargo-Size-Kästchen), messbar über den Golden-Korpus mit Regressionstests | R-QA-1..4 |

### Terminal und Location

| # | Problem (Quelle) | Ursache (bekannt/vermutet) | Unser Fix | Req |
|---|---|---|---|---|
| F11 | **Falsche Location/Terminal-Zuordnung** (Lorville CBD↔Rustville [#9](https://github.com/Shebuka/SC-Datarunner-UEX/issues/9), Admin↔CBD [#11](https://github.com/Shebuka/SC-Datarunner-UEX/issues/11), MIC-L5↔Ruin Station [#38](https://github.com/Shebuka/SC-Datarunner-UEX/issues/38), TDD↔MTP, Gateways) | Header-OCR plus Farbheuristik; Ambiguität wird zu leicht übersehen | 1. Primäre Quelle ist das Location-Feld unter „YOUR INVENTORIES“ (in den Patch-City-Screenshots: „PATCH CITY“), gematcht gegen alle Location-Namen und Nicknames aus UEX. 2. Kandidaten werden mit dem Sortiment abgeglichen: die erkannten Commodities müssen zu den bekannten Commodities des Terminals passen (`commodities_prices?id_terminal=`). 3. Kontext aus vorherigen Screenshots derselben Sitzung (gleiches Sternsystem). 4. Gibt es mehr als einen Kandidaten über dem Schwellwert, ist die Auswahl **Pflicht** (blockierend, prominent oben in der Karte); still vorausgewählt wird nichts. | R-OCR-8, R-VAL-4, R-UI-4 |
| F12 | **Gateway-Terminals mit gleichen Namen im Nachbarsystem** (offen seit v0.6.8) | Name allein nicht eindeutig | Disambiguierung über Sternsystem-Kontext (letzte Location/Sitzung); sonst Pflichtauswahl mit Anzeige des Systems im Dropdown | R-VAL-4 |
| F13 | **Low-Confidence-Werte werden trotzdem gesendet** (43 %/30 % → HTTP 200, [#38](https://github.com/Shebuka/SC-Datarunner-UEX/issues/38)) | Kein Sende-Gate | **Harte Sendesperre**: Felder unter dem Schwellwert müssen bestätigt oder korrigiert werden. „Send All“ überspringt solche Berichte und nennt den Grund. | R-SUB-3 |

### Umgebung, Konfiguration und Erfassung

| # | Problem (Quelle) | Ursache (bekannt/vermutet) | Unser Fix | Req |
|---|---|---|---|---|
| F14 | **HOTFIX-Umgebung „Unknown“ → Fallback auf Spielversion „4.1“ → Ablehnung** ([#27](https://github.com/Shebuka/SC-Datarunner-UEX/issues/27)) | Statischer Fallback | Ordnernamen-Mapping LIVE/PTU/EPTU/HOTFIX/TECH-PREVIEW konfigurierbar. Die Spielversion kommt **immer** aus `GET /game_versions` bzw. `data_parameters` (live/ptu). Ist die Umgebung unbekannt, muss der Nutzer wählen; es gibt **keinen hartkodierten Fallback**. Vor dem Senden wird geprüft, ob `is_accepting_reports` bzw. `is_accepting_ptu_reports` gesetzt ist. | R-CAP-3, R-SUB-5 |
| F15 | **Umgebung nicht erkannt in VM/Docker** ([#22](https://github.com/Shebuka/SC-Datarunner-UEX/issues/22)), separater PTU-Pfad ([#12](https://github.com/Shebuka/SC-Datarunner-UEX/issues/12)) | Autodetektion zwingend | Beliebig viele Überwachungsordner, je Ordner Umgebung manuell überschreibbar | R-CAP-1, R-CAP-3 |
| F16 | **Abhängigkeit von der Druck-Taste** – remappte Taste ([#33](https://github.com/Shebuka/SC-Datarunner-UEX/issues/33)), kein Ziffernblock ([#16](https://github.com/Shebuka/SC-Datarunner-UEX/issues/16)), keine Bilder importiert ([#35](https://github.com/Shebuka/SC-Datarunner-UEX/issues/35)), Strg+V gewünscht ([#26](https://github.com/Shebuka/SC-Datarunner-UEX/issues/26)) | Nur Ordnerüberwachung | Nutzerdefinierte Ordner, wahlweise per Klick auf „Einlesen“ oder automatisch beim Anlegen neuer Dateien (WatchService plus Polling-Fallback, z. B. für Wine/Netzlaufwerke; Lesen erst nach vollständigem Schreiben); dazu Drag & Drop, Strg+V, Datei-Import, Zwischenablage-Überwachung (optional). Diagnose-Anzeige „Ordner wird überwacht, letztes Bild: …“. | R-CAP-1, R-CAP-2 |
| F17 | **config.ini wird nicht angelegt / Einstellungen gehen verloren / App beendet sich nach Setup** ([#24](https://github.com/Shebuka/SC-Datarunner-UEX/issues/24), [#28](https://github.com/Shebuka/SC-Datarunner-UEX/issues/28), v0.12.0) | Fehlerhafte Persistenz | Atomares Schreiben (temp + `ATOMIC_MOVE`), Schema-Version und Migration, Fehler werden im UI angezeigt (nie still), Start-Selbsttest der Verzeichnisse | R-NF-5 |
| F18 | **Lokalisierungspfad veraltet nach Reinstall** (v0.10.0/v0.12.0) | Fester Pfad | `global.ini` über SC-Installationserkennung finden (RSI-Launcher-Log, Standardpfade, Wine-Prefix), bei Änderung (mtime/Hash) neu laden | R-L10N-2 |

### Übermittlung und Verbindung

| # | Problem (Quelle) | Ursache (bekannt/vermutet) | Unser Fix | Req |
|---|---|---|---|---|
| F19 | **Cooldown nur im Speicher**; Zeile löschen hebelt die 5-Minuten-Sperre aus (offen) | Kein persistenter Zustand | Submission-Historie in **SQLite**, Cooldown pro (Terminal, Commodity, Umgebung) persistent; konservativ ohne Seite, bis Annahme A12 geklärt ist. Rückzug über `data_remove` wird ebenfalls protokolliert. | R-SUB-4 |
| F20 | **Keine Verbindung zur API** in manchen Netzen ([#15](https://github.com/Shebuka/SC-Datarunner-UEX/issues/15), [#29](https://github.com/Shebuka/SC-Datarunner-UEX/issues/29), [#31](https://github.com/Shebuka/SC-Datarunner-UEX/issues/31)); Verdacht auf Zertifikat/Proxy | Proxy/TLS | 1. Unter Windows wird der **OS-Truststore** genutzt (`Windows-ROOT`, SunMSCAPI), dadurch funktionieren Firmen-/AV-Proxies mit eigener CA. 2. System-Proxy über `ProxySelector`. 3. Konfigurierbare Hosts mit Fallback (`api.uexcorp.space` → `api.uexcorp.uk`). 4. Diagnose-Dialog „Verbindung testen“ mit Status, HTTP-Code und Body; Secrets werden dabei maskiert. | R-API-3, R-NF-6 |
| F21 | **Senden hängt / ein Dialog pro Bericht** (v0.12.0) | UI-Thread, unbegrenzte Parallelität | Sende-Queue auf Virtual Threads, Semaphore (Standard 2 parallel), Token-Bucket für 120 req/min, nicht-modaler Fortschritt in der Liste | R-SUB-2 |
| F22 | **Fehler ohne Details** („Failed to send data:“ [#4](https://github.com/Shebuka/SC-Datarunner-UEX/issues/4), „error 28“ [#17](https://github.com/Shebuka/SC-Datarunner-UEX/issues/17)) | Fehlertexte nicht gemappt | Alle bekannten UEX-`status`-Codes werden auf verständliche, lokalisierte Meldungen mit Handlungsempfehlung gemappt. Unbekannte Codes erscheinen mit Rohcode. | R-SUB-6 |
| F23 | **„User is not a datarunner“** ([#2](https://github.com/Shebuka/SC-Datarunner-UEX/issues/2)), Secret-Key-Fehler ([#10](https://github.com/Shebuka/SC-Datarunner-UEX/issues/10)) | Keine Vorabprüfung | Beim Onboarding prüft `GET /user` den Key und liest `is_datarunner`/`is_datarunner_banned`. Bei Bedarf gibt es einen Link zur DataRunner-Anmeldung, und Senden wird vorab gesperrt. | R-API-4 |
| F24 | **Update-Check über unauthentifizierte GitHub-API (60 req/h)** | Zu häufiges Polling | Update-Check maximal einmal pro Tag, `ETag`/`If-None-Match`, abschaltbar | R-NF-7 |

### Plattform und UI

| # | Problem (Quelle) | Ursache (bekannt/vermutet) | Unser Fix | Req |
|---|---|---|---|---|
| F25 | **Nur Windows** | Entwurfsentscheidung | Windows und Linux, inklusive Wine-/Proton-Prefix-Erkennung | R-NF-1 |
| F26 | **Weißer Hintergrund/Lesbarkeit** ([#6](https://github.com/Shebuka/SC-Datarunner-UEX/issues/6)), zu große Tutorial-Fenster ([#25](https://github.com/Shebuka/SC-Datarunner-UEX/issues/25)) | Theme-/DPI-Handling | Eigenes CSS-Theme (hell/dunkel) unabhängig vom OS-Theme, alle Fenster skalierbar, HiDPI-Test bei 100/150/250 % | R-UI-7 |
| F27 | **SCU-Containergrößen fehlen** in Reports ([#13](https://github.com/Shebuka/SC-Datarunner-UEX/issues/13), offen) | Nicht ausgelesen | „AVAILABLE CARGO SIZE (SCU)“-Kästchen erkennen (Menge ⊆ {1,2,4,8,16,24,32}, aufsteigend) → Feld `container_sizes` | R-OCR-9 |
| F28 | **Kein Installer / Tray** ([#36](https://github.com/Shebuka/SC-Datarunner-UEX/issues/36), offen) | – | MSI (Windows) und `.deb` plus portables Archiv (Linux); Tray-Modus mit Benachrichtigung „n neue Screenshots“ als späteres Feature | R-NF-2, R-UI-9 |
| F29 | **Nur Commodities** ([#20](https://github.com/Shebuka/SC-Datarunner-UEX/issues/20), offen) | – | Datenmodell von Anfang an für `type` = `commodity`/`item`/`vehicle_buy`/`vehicle_rent`; Items und Fahrzeuge folgen nach 1.0 | R-SCOPE-2 |

### Datenschutz

| # | Problem (Quelle) | Ursache (bekannt/vermutet) | Unser Fix | Req |
|---|---|---|---|---|
| F30 | **[beobachtet] Kontostand im Upload-Screenshot.** „CURRENT BALANCE“ steht oben rechts im Terminal; ein unbeschnittener Screenshot würde ihn an UEX übertragen. Ob das Original das tut, ist unbekannt. | – | Der Upload-Screenshot enthält nur den Ausschnitt **„SHOP INVENTORY“ plus Location-Feld**. Der Kontostand wird **immer** geschwärzt; beides ist testabgedeckt. | R-SUB-7 |
