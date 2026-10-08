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
| **Report** | Die Zusammenfassung aller Scans eines Terminals und einer Seite (Kaufen = „Buy“, Verkaufen = „Local Market Value“) innerhalb eines Zeitfensters (Gruppierungsregel R-OCR-13). Der Report ist die Einheit, die an `POST /data_submit` geht. |
| **Sendeschwelle** | Mindestkonfidenz, ab der ein Feld ohne Bestätigung gesendet werden darf (Startwert 0.80, siehe [07](07-ocr-konzept.md) §2.6). Felder darunter müssen bestätigt oder korrigiert werden. |
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
| R-CAP-1 | **Nutzerdefinierte Bildordner:** beliebig viele Ordner hinzufügen, entfernen und (de)aktivieren. Pro Ordner einstellbar: Umgebung (**automatisch aus dem Pfad** – Standard – oder fest LIVE/PTU/EPTU/HOTFIX/TECH-PREVIEW oder **bei jedem Bild fragen**), Unterordner einbeziehen (ja/nein), Dateitypen (Standard `png`, `jpg`, `jpeg`), optional „nur Dateien neuer als …“. Die SC-Screenshot-Ordner werden beim Onboarding nur **vorgeschlagen**; der Nutzer bestätigt sie. | M |
| R-CAP-1a | **Manuell einlesen (Standard):** Ein Klick auf **„Einlesen“** (global für alle aktiven Ordner oder pro Ordner) liest alle Dateien ein, die noch nicht verarbeitet wurden. Ergebnis: „n neue Bilder übernommen, m übersprungen“. Optional gibt es eine Vorschau-Liste zum Abwählen einzelner Dateien. | M |
| R-CAP-1b | **Automatisch einlesen (opt-in pro Ordner):** Ist es aktiviert, werden neu angelegte Dateien sofort übernommen. Technik: `WatchService` (`ENTRY_CREATE`/`ENTRY_MODIFY`) plus Polling-Fallback (Standard 2 s), wenn der WatchService unzuverlässig ist (Wine-Prefixe, Netzlaufwerke) oder nicht registriert werden kann. Beim Start und nach dem Aktivieren läuft ein Nachhol-Scan für Dateien, die angelegt wurden, während die App geschlossen oder der Modus aus war (abschaltbar). | M |
| R-CAP-1c | **Erst lesen, wenn fertig geschrieben:** Eine Datei gilt als vollständig, wenn ihre Größe und `lastModified` für eine Ruhezeit (Standard 750 ms) stabil sind und sie sich als Bild dekodieren lässt. Teil-Dateien werden erneut versucht (mit Backoff, maximal 30 s) und danach als Fehler mit Grund angezeigt. | M |
| R-CAP-1d | **Verarbeitet-Register** (SQLite): Pfad, Größe, `lastModified` und Inhalts-Hash jeder eingelesenen Datei. Dadurch liest weder „Einlesen“ noch der Watcher dieselbe Datei doppelt, auch nach Umbenennen oder Kopieren (Hash). Pro Datei gibt es die Aktion „Erneut einlesen“. | M |
| R-CAP-1e | Status pro Ordner sichtbar: Modus (manuell/automatisch), überwacht ja/nein (und warum nicht, z. B. Ordner fehlt bzw. keine Rechte), Polling aktiv, letzte Datei, Zahl offener Dateien. Fehlt ein Ordner, wird er markiert und nicht stillschweigend entfernt. | M |
| R-CAP-2 | Weitere Eingangswege: Drag & Drop, Strg+V, Datei-Dialog (Mehrfachauswahl), optional Zwischenablage-Überwachung mit Fingerprint-Deduplizierung | M |
| R-CAP-3 | Die Umgebung wird aus dem Ordnerpfad abgeleitet (oder fest pro Ordner gesetzt), die Spielversion aus UEX (`game_versions`). Ist die Umgebung unbekannt, fragt die App nach; einen hartkodierten Fallback gibt es nicht. | M |
| R-CAP-3a | **Umgebungs-Mapping auf UEX:** UEX kennt nur `live` und `ptu`. Standard-Mapping: LIVE und HOTFIX → `live`, PTU/EPTU/TECH-PREVIEW → `ptu`. Konfigurierbar; das Mapping ist gegen UEX zu verifizieren (Annahme A9). Werden für das Ziel keine Reports angenommen (`is_accepting_*`), wird das Senden gesperrt. | M |
| R-CAP-3b | **Spielversion zum Aufnahmezeitpunkt:** Beim Einlesen wird die zum Aufnahmezeitpunkt gültige Version festgehalten (gecachter Stand von `game_versions`). Hat sich die Version zwischen Aufnahme und Senden geändert (Patch-Tag), warnt die App und verlangt eine Bestätigung bzw. verwirft den Report. | M |
| R-CAP-7 | **Arbeitskopien:** Die App speichert pro Capture die perspektivkorrigierten Panel-Ausschnitte (nicht den Vollbild-Screenshot) in ihrem Datenverzeichnis. Upload, KI-Nachprüfung und Review funktionieren damit auch, wenn das Original verschoben oder gelöscht wurde. Aufbewahrung: bis 7 Tage nach erfolgreichem Senden (einstellbar). | M |
| R-CAP-4 | Automatische Erkennung der SC-Installation: RSI-Launcher-Log `Launching Star Citizen … from (…)`, laufender Prozess `Bin64/StarCitizen.exe`, Standardpfade. Unter Linux zusätzlich Wine-/Proton-Prefixe (siehe Annahme A3). | S |
| R-CAP-5 | Aufnahmezeit aus dem Dateinamen (Regex), Fallback auf `lastModified`. Duplikate erkennt die App per Inhalts-Hash. | M |
| R-CAP-6 | Optionale Aufräumfunktion: Originale nach erfolgreichem Senden löschen oder archivieren. Standard ist **aus**. | S |

## Manuelle Erfassung (MAN)

| ID | Anforderung | Prio |
|---|---|---|
| R-MAN-1 | Terminal-Auswahl per Suchfeld (Fuzzy über Name, Nickname, Location, System), Filter nach Sternsystem, „zuletzt verwendet“ | M |
| R-MAN-2 | Nach der Terminalwahl erscheint die Liste der bekannten Commodities des Terminals (aus `commodities_prices`). Die aktuellen UEX-Werte werden **als Referenz neben dem Eingabefeld** angezeigt (inkl. Alter des Werts), **nicht als vorausgefüllter Wert**. Nur aktiv eingegebene oder per Tastendruck ausdrücklich übernommene Zeilen werden gesendet. So landen keine veralteten Werte ungeprüft als „frische“ Meldung bei UEX. | M |
| R-MAN-5 | Manuelle Reports können einen Screenshot anhängen. Ist ein Screenshot für den Nutzer Pflicht (Evaluationsphase neuer DataRunner, `screenshot_required`), wird das vorab angezeigt und das Senden ohne Bild gesperrt (Annahme A11). | M |
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
| R-OCR-13 | **Report-Gruppierung:** Scans gehören zu einem Report, wenn Terminal, Seite und Umgebung gleich sind und der zeitliche Abstand zum vorherigen Scan der Gruppe ≤ 10 min beträgt (einstellbar). Der Nutzer kann Reports im UI teilen und zusammenführen. Ein Scan mit unklarem Terminal bildet eine eigene Gruppe, bis das Terminal geklärt ist. | M |

## Optionale KI-Erkennung (VLM)

Die klassische OCR (oben) ist der **Standardweg** und läuft immer, auch während des Spiels. Zusätzlich lässt sich ein **lokales Vision-Language-Modell (VLM)** über [Ollama](https://ollama.com) zuschalten. Es braucht viel VRAM bzw. CPU und läuft deshalb standardmäßig **nur, wenn Star Citizen geschlossen ist**. So kann man im Spiel ohne KI arbeiten und die KI danach nachprüfen lassen.

| ID | Anforderung | Prio |
|---|---|---|
| R-VLM-1 | Einstellung „KI-Erkennung“ mit drei Werten: **Aus** (Standard), **Automatisch – nur wenn das Spiel geschlossen ist**, **Immer** (Experte, mit Warnung zu VRAM- und FPS-Einbußen) | S |
| R-VLM-2 | Spielerkennung: Der Prozess `StarCitizen.exe` wird zyklisch geprüft (Standard alle 5 s). Windows: Pfad endet auf `Bin64\StarCitizen.exe`. Linux: Kommandozeile oder Argumente des Wine-/Proton-Prozesses enthalten `StarCitizen.exe` (Annahme A7). Der RSI-Launcher allein zählt nicht als „Spiel läuft“. „Geschlossen“ gilt erst nach einer Hysterese (Standard 30 s), damit ein Neustart des Spiels die KI nicht startet. | S |
| R-VLM-3 | Spielstart während eines KI-Laufs: Der laufende Job wird sofort abgebrochen und das Modell entladen (`keep_alive: 0`). Betroffene Scans gehen zurück in den Zustand „KI ausstehend“; es gehen keine Daten verloren. | S |
| R-VLM-4 | Nachprüfen nach Spielende: Die App bietet an (oder führt automatisch aus, einstellbar), alle **noch nicht gesendeten** Reports mit dem VLM nachzulesen. Wählbar sind „nur Reports mit Warnungen“ (Standard) oder „alle“. Fortschritt und Abbruch sind jederzeit möglich. | S |
| R-VLM-5 | Fusion: VLM und klassische OCR sind **unabhängige Leser**. Übereinstimmung erhöht die Konfidenz („doppelt bestätigt“); Widerspruch erzeugt ein Finding `Ambiguous` mit beiden Kandidaten. VLM-Werte durchlaufen **dieselbe** Vokabular-Auflösung und Validierung (Prior, Toleranzen, Konsistenz). Kein Leser setzt allein einen Wert durch, der der Validierung widerspricht. | S |
| R-VLM-6 | Ausschließlich lokal: Standard-Host `http://localhost:11434`, konfigurierbar. Cloud-Modelle (Ollama-Cloud, Suffix `:cloud`) und Nicht-Localhost-Hosts nur nach expliziter Bestätigung mit Datenschutzhinweis. Bilder verlassen sonst nie den Rechner. | S |
| R-VLM-7 | Modellverwaltung: Ollama-Erreichbarkeit und -Version prüfen (`/api/version`), installierte Modelle anzeigen (`/api/tags`), empfohlenes Modell auf Wunsch laden (`/api/pull` mit Fortschritt). Ollama selbst wird **nicht** mitgeliefert; die App zeigt eine Installationsanleitung. | S |
| R-VLM-8 | Hardware-Hinweis: Nach dem Laden des Modells wird `size_vram` mit `size` verglichen (`/api/ps`). Läuft das Modell nicht vollständig auf der GPU, warnt die App („langsam, läuft auf CPU“). Die Geschwindigkeit pro Bild wird in der UI angezeigt. | S |
| R-VLM-9 | Eingabe ans VLM ist der **perspektivkorrigierte Panel-Ausschnitt** (Shop-Panel bzw. Location-Feld), nicht der ganze Screenshot. Das ist kleiner und schneller, und der Kontostand wird nicht mitgegeben. | S |
| R-VLM-10 | Ausgabeformat: Prompt als versionierte Ressource; Temperatur 0; Antwort als `KEY: value`-Zeilen plus Markdown-Tabelle, die ein deterministischer Parser verarbeitet (siehe [07](07-ocr-konzept.md) §2.7). Wird die Antwort abgeschnitten (`done_reason == "length"`), folgt genau ein Retry mit größerem Token-Limit. | S |
| R-VLM-11 | Das empfohlene Standardmodell wird per **Bake-off auf unserem Korpus** festgelegt (nicht nach Modellkarte). Kandidaten u. a. die bei basetool erprobten `qwen3-vl:8b-instruct`/`qwen3-vl:4b-instruct`; deren Eignung für Rohstoff-Terminals ist **unbelegt**. | S |
| R-VLM-12 | Die Funktion ist vollständig optional: Ohne Ollama ist die App uneingeschränkt nutzbar, ohne Fehlermeldungen beim Start. | M |

## Validierung (VAL)

| ID | Anforderung | Prio |
|---|---|---|
| R-VAL-1 | Regelbasierte Konfidenz pro Feld; Konfidenzwerte des Modells werden nicht direkt verwendet. Jede Warnung hat einen Grund-Code, der im UI angezeigt wird. | M |
| R-VAL-2 | Plausibilität gegen den UEX-Prior: Abweichung > `price_variation` % bzw. > `scu_variation` (aus `data_parameters`) führt zu einer Warnung, nicht zum Blockieren | M |
| R-VAL-2a | **Kein Prior vorhanden** (neue Commodity am Terminal, keine Historie): Ersatzweise wird der Commodity-weite Durchschnitt (`commodities.price_buy`/`price_sell`) mit doppelter Toleranz genutzt. Fehlt auch der, erfolgt keine Prior-basierte Reparatur; das Feld trägt das Finding „kein Referenzwert“ und liegt unter der Sendeschwelle, wenn es eine verwechselbare Ziffer enthält. | M |
| R-VAL-2b | **Reparaturen nur mit unabhängigem Zeugen automatisch:** Eine Reparatur, die sich nur auf den Prior stützt, liegt unter der Sendeschwelle (Bestätigung per Tastendruck). Automatisch übernommen wird sie nur, wenn ein unabhängiger Zeuge zustimmt (Glyph-Topologie, Zweitleser/VLM). Begründung: Der Prior ist ein möglicherweise veralteter Wert; sonst würden echte Preisänderungen still auf alte Werte „zurückrepariert“. | M |
| R-VAL-3 | Konsistenzregeln: Status ↔ SCU (z. B. „Out of Stock“ ⇒ 0 SCU), Seite ↔ Abschnitt, Commodity ist an diesem Terminal kauf- bzw. verkaufbar (`is_buyable`/`is_sellable`) | M |
| R-VAL-4 | Terminal-Ambiguität: Gibt es mehrere Kandidaten, ist die Auswahl Pflicht; die App wählt nichts still vor | M |
| R-VAL-5 | Fehlende Commodities: Commodities, die laut UEX am Terminal geführt werden, aber auf keinem Scan auftauchen, werden angezeigt. Optional kann der Nutzer sie als `is_missing` markieren. | S |

## Review-UI (UI)

| ID | Anforderung | Prio |
|---|---|---|
| R-UI-1 | Eingangsliste (Queue) mit Status: wartend / verarbeitet / prüfen / bereit / gesendet / Fehler | M |
| R-UI-2 | Report-Editor: Terminal (Pflicht-Dropdown bei Ambiguität), Seite, Spielversion, Tabelle der Commodity-Zeilen mit Name, Status, SCU, Preis/SCU und Container-Größen | M |
| R-UI-3 | Zeilen erscheinen in Bildschirmreihenfolge. Zu jeder Zeile gibt es den Bildausschnitt in lesbarer Größe; beim Fokus auf ein Feld wird die Quellregion im Screenshot hervorgehoben. | M |
| R-UI-4 | Kennzeichnung der **Erkennungssicherheit** (ok / bestätigen / auswählen / korrigieren) plus Text-Grund (nicht nur Farbe, wegen Barrierefreiheit). Visuell **getrennt** von der Abweichungsmarkierung (R-UI-10): Erkennungssicherheit als Symbol und Rahmen am Feld, Abweichung als Hintergrundfarbe plus Δ-Badge. So bleibt erkennbar, ob „schlecht gelesen“ oder „anders als bei UEX“ das Problem ist. | M |
| R-UI-5 | „Alle sicheren übernehmen“; Navigation von Problem zu Problem (F8/Shift+F8) | S |
| R-UI-6 | Manueller Zuschnitt und Ecken-Korrektur, wenn die Lokalisierung scheitert | M |
| R-UI-7 | Eigenes Theme (hell/dunkel), HiDPI, frei skalierbare Fenster | M |
| R-UI-8 | Startet sofort; Laden im Hintergrund mit Statuszeile; offline nutzbar mit dem gecachten Datenstand (Senden erst bei Verbindung) | M |
| R-UI-9 | Tray-Modus mit Benachrichtigung „n neue Screenshots“ | K |
| R-UI-10 | **Abweichungsmarkierung gegen UEX:** Jedes Feld (Preis, SCU, Status, Container-Größen) wird mit dem bisherigen UEX-Wert für dieselbe Kombination aus Terminal, Commodity und Seite verglichen und **farblich markiert**. Stufen:<br/>• **gleich** – keine Markierung<br/>• **abweichend innerhalb der Toleranz** – dezente Markierung (gelb), Hinweis „bitte gegenprüfen“<br/>• **stark abweichend** (> `price_variation` % bzw. > `scu_variation`, Status ≥ 2 Stufen entfernt) – kräftige Markierung (orange) **und Bestätigung erforderlich** (Sendesperre bis zur Bestätigung)<br/>• **kein Referenzwert** (neu an diesem Terminal) – Hinweis-Markierung (blau)<br/>Gilt für OCR- und manuelle Reports gleichermaßen. Schwellen kommen aus `data_parameters` bzw. den Einstellungen, nicht aus Konstanten. | M |
| R-UI-11 | **Gegenprüfen leicht machen:** Bei markierten Feldern sind sichtbar: UEX-Wert, Alter des UEX-Werts („vor 3 Tagen“), Differenz absolut und in % (z. B. „+12 %“), dazu der Bildausschnitt der Quelle. Die Markierung ist **nicht nur farbig**, sondern zusätzlich als Symbol und Text kodiert (Farbsehschwäche). Die Farben sind farbsehschwäche-tauglich gewählt (kein reines Rot/Grün) und im Theme konfigurierbar. Ein **veralteter** UEX-Wert (älter als einstellbar, Standard 7 Tage) schwächt die Markierung um eine Stufe ab und wird als „Referenz veraltet“ angezeigt. | M |
| R-UI-12 | **Zusammenfassung vor dem Senden:** Der Report zeigt oben die Anzahl abweichender und stark abweichender Felder; per Tastatur springt man von Abweichung zu Abweichung (gleiche Navigation wie R-UI-5). Eine Bestätigung gilt für genau den bestätigten Wert; wird der Wert danach geändert, ist erneut zu bestätigen. Zusätzlich werden **unerwartete Commodities** (an diesem Terminal bei UEX unbekannt) und **fehlende Commodities** (bei UEX geführt, im Scan nicht gefunden, R-VAL-5) markiert. | M |

## Übermittlung (SUB)

| ID | Anforderung | Prio |
|---|---|---|
| R-SUB-1 | `POST /data_submit` mit `id_terminal`, `type`, `is_production`, `game_version`, `prices[]` (getrennte Buy- und Sell-Zeilen), optional `container_sizes`, `screenshot`, `details` | M |
| R-SUB-2 | Sende-Queue: Rate-Limit-konform (120 req/min global, max. 500 Zeilen pro Report), begrenzte Parallelität, Retry mit Backoff nur bei 429/5xx/Netzfehlern. Die Queue ist **persistent** (SQLite): Reports, die offline oder vor einem Absturz eingereiht wurden, gehen nicht verloren und werden nach dem Neustart erst nach erneuter Freigabe gesendet. | M |
| R-SUB-3 | Sendesperre bei ungeklärten Feldern oder unklarem Terminal | M |
| R-SUB-4 | Persistente Historie (SQLite): Report-IDs (`ids_reports`), Zeitpunkt, Payload-Hash, 5-Minuten-Cooldown pro Terminal/Commodity/Umgebung (konservativ **ohne** Seite, bis die UEX-Regel geklärt ist – Annahme A12); Rückzug über `data_remove`. Optional zeigt die Historie den Bearbeitungsstatus bei UEX (`data_info`). | M |
| R-SUB-5 | Vor dem Senden: `data_parameters` prüfen (`is_accepting_reports` bzw. `is_accepting_ptu_reports`, `commodity.is_accepted`). Dafür wird ein Stand von höchstens 15 min verwendet (nicht der Tages-Cache), weil sich diese Flags am Patch-Tag ändern. | M |
| R-SUB-6 | Mapping aller bekannten UEX-Fehlercodes auf lokalisierte Meldungen mit Handlungsempfehlung | M |
| R-SUB-7 | Upload-Screenshot: perspektivkorrigierter Ausschnitt „SHOP INVENTORY“ plus Location-Feld, **Kontostand immer geschwärzt**, JPEG oder PNG < 10 MB (Ziel ~1–2 MP). Besteht ein Report aus mehreren gescrollten Captures, die API aber nur **ein** `screenshot`-Feld hat, werden die Shop-Ausschnitte vertikal zu einem Bild zusammengesetzt (Annahme A11: von UEX akzeptiert). | M |
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
| R-NF-1 | Windows 10/11 x64 und Linux x64 (glibc-Distros, X11; unter Wayland über XWayland – native Wayland-Unterstützung von JavaFX ist nicht belegt, XWayland-Betrieb ist in M4 zu testen) | M |
| R-NF-2 | Distribution: Windows MSI und portables ZIP; Linux `.deb` und portables `tar.gz` (App-Image von jpackage); jeweils mit gebündelter Runtime | M |
| R-NF-3 | Ressourcen: Heap begrenzt (Startwert `-Xmx512m`, durch Messung festzulegen), ONNX-Session wird nach Inaktivität freigegeben | S |
| R-NF-4 | Secret-Key nur im OS-Keystore (Windows Credential Manager, Linux Secret Service). Fallback ist eine Datei mit Rechten 0600 nach expliziter Warnung. Niemals in Logs. | M |
| R-NF-5 | Konfiguration: atomares Schreiben, Schema-Versionierung, sichtbare Fehler. Speicherorte: Windows `%APPDATA%`, Linux `$XDG_CONFIG_HOME`/`$XDG_DATA_HOME`/`$XDG_CACHE_HOME`. | M |
| R-NF-6 | Logging: rotierende Logdatei, Secrets maskiert, Diagnose-Export (Logs plus Systeminfo, ohne Secrets) | M |
| R-NF-7 | Update-Hinweis über GitHub Releases, höchstens einmal täglich, abschaltbar | S |
| R-NF-8 | Kein Eingriff ins Spiel: kein Zugriff auf Prozessspeicher, keine Input-Injektion, keine Overlays. Gelesen werden nur Dateien, die Zwischenablage und die **Prozessliste** (für Pfaderkennung und Spielerkennung, R-VLM-2). | M |

## Annahmen (zu verifizieren)

| ID | Annahme |
|---|---|
| A1 | Der Header für den Secret-Key heißt `secret_key`; die Doku-Zusammenfassungen sind widersprüchlich (`secret-key`). Muss gegen die Live-API mit `is_production=0` geprüft werden. |
| A2 | Unklar ist, ob für `data_submit` zusätzlich ein **App-Token** (`Authorization: Bearer`) nötig ist und wie ein Open-Source-Client es verteilt. Ein in die Distribution eingebettetes Token ist extrahierbar. Optionen: Rücksprache mit UEX oder ein vom Nutzer eingetragenes Token. |
| A3 | SC unter Linux läuft über Wine/Proton (z. B. LUG-Helper, Lutris). Screenshots landen dann im Prefix unter `drive_c/Program Files/Roberts Space Industries/StarCitizen/<CHANNEL>/screenshots`. Pfad und Struktur sind auf echten Linux-Installationen zu prüfen. |
| A4 | Der Windows-Standardpfad für Screenshots ist `…\StarCitizen\<CHANNEL>\screenshots`. Er wird beim Onboarding angezeigt und ist bestätigbar, nicht blind gesetzt. |
| A5 | Die Statusstufen 1–7 und ihre Namen kommen aus `commodities_status`. Die Zuordnung der Ingame-Texte (z. B. „Max Inventory“, „Out of Stock“ auf der Sell-Seite) zu den Codes ist am Korpus zu verifizieren. |
| A6 | Die Patch-City-Screenshots (2000×1125) sind möglicherweise skaliert. Ziel-Auflösungen müssen mit Original-Screenshots getestet werden. |
| A7 | Unter Linux ist das laufende Spiel über `ProcessHandle.info().command()`/`arguments()` des Wine-/Proton-Prozesses anhand von `StarCitizen.exe` erkennbar. Auf echten Installationen (Wine, Proton, Lutris) zu prüfen; Fallback ist ein Scan von `/proc/*/cmdline`. |
| A8 | Ein lokales VLM liest Rohstoff-Terminals genauer als die klassische OCR oder ergänzt sie sinnvoll. Belegt ist das bisher nur für Refinery-Panels (basetool); für unseren Fall wird es erst durch das Bake-off in M5 geprüft. Ist der Nutzen gering, bleibt das Feature klein oder entfällt. |
| A9 | UEX ordnet HOTFIX-Reports `live` und EPTU/TECH-PREVIEW-Reports `ptu` zu (bzw. akzeptiert diese überhaupt). |
| A10 | Auf der **Sell-Seite** entspricht die angezeigte „… SCU“-Zahl dem UEX-Feld `scu_sell`. UEX kennt zusätzlich `scu_sell_stock`; welches Feld die Zahl im Terminal meint, ist unklar. Auf der Buy-Seite entspricht „SHOP QUANTITY“ vermutlich `scu_buy`. |
| A11 | Ein aus mehreren Scroll-Ausschnitten zusammengesetzter Screenshot wird von UEX akzeptiert; manuelle Reports ohne Screenshot werden für etablierte DataRunner angenommen. |
| A12 | Die 5-Minuten-Duplikatsperre von UEX gilt pro (Terminal, Commodity) – ob die Seite (Buy/Sell) unterschieden wird, ist offen. |
