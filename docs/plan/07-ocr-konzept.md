# OCR-Konzept

## 1. Beobachtungen an echten Screenshots (Patch City, Pyro)

Die fünf vom Projektinhaber gelieferten Screenshots (2000×1125, Buy- und Local-Market-Value-Tab, gescrollt) zeigen das aktuelle Terminal-Layout. Was wir daraus ableiten:

| Element | Beobachtung | Konsequenz |
|---|---|---|
| Kopf | „COMMODITIES“ oben links; „CURRENT BALANCE: ¤9,484,456 aUEC“ oben rechts | Kopf dient als Anker. **Kontostand nie übertragen**, im Upload schwärzen. |
| Linkes Panel „YOUR INVENTORIES“ | Dropdown mit der aktuellen Location („PATCH CITY“), darunter IN DEMAND / NO DEMAND / CANNOT SELL | **Location-Feld = beste Quelle für die Terminal-Zuordnung** (Text in Großbuchstaben, gespreizt) |
| Rechtes Panel „SHOP INVENTORY“ | Tabs „Buy“ und „Local Market Value“; der aktive Tab ist kräftig orange-rot gefüllt, der inaktive dunkel | Seite (BUY/SELL) über den Tab-Hintergrund bestimmen (Farbe bzw. Luminanz), zusätzlich über den OCR-Text |
| Abschnitte | Buy: „IN STOCK“ (aufgeklappt), unten „OUT OF STOCK“ (zugeklappt, „+“). Sell: „SELLABLE CARGO“, „IN DEMAND“ | Abschnitt pro Karte merken; Konsistenzregeln |
| Karte | Icon · vertikaler Statusbalken (Farbe/Füllhöhe) · Name · Statustext (farbig) · rechts „SHOP QUANTITY“ (nur Buy) · „333 SCU“ · „¤3,237/SCU“ · „AVAILABLE CARGO SIZE (SCU)“ mit Kästchen [1][2][4][8][16][24][32] | Feldpositionen relativ zur Karte; Zweitleser für den Status aus Balken und Farbe |
| Währungssymbol | `¤` steht direkt vor der Zahl, ohne Leerzeichen | Ursache für das „9“-Präfix im Original (F1): geometrisch abtrennen |
| Zahlen | Tausendertrenner Komma („6,000“, „¤36,000/SCU“), ganze aUEC | Strukturparser (F3/F4) |
| Lange Namen | „Recycled Material Composite“ überlappt mit „0 SCU“ → zusammengeklebt „Compos0iSCU“ | Namensauflösung per Fuzzy-Präfix, SCU-Regex innerhalb des Tokens (F6) |
| Scrollen | Am oberen und unteren Rand abgeschnittene Karten (z. B. nur „AVAILABLE CARGO SIZE“ ohne Namen; Karte ohne Cargo-Zeile) | Randkarten nur mit vollständigen Feldern übernehmen; Merge über Commodity-ID |
| Ausblendung am Rand | Untere Karten sind abgedunkelt („SHOP QUANTITY“ kaum sichtbar) | Bei Duplikaten den Wert der weiter vom Rand entfernten Lesung bevorzugen (basetool-Erkenntnis) |
| Hover-Hervorhebung | Eine Karte hat roten Hintergrund (Maus darüber) | Kontrastnormalisierung pro Karte, nicht global |
| Schrift | Ziffern im HUD-Stil (z. B. „16“ ähnelt „lb“, „4“ ähnelt „Ч“) | Cargo-Größen über eine geschlossene Menge {1,2,4,8,16,24,32} und die Reihenfolge klassifizieren, nicht frei lesen |
| Störungen | HUD-Elemente am Rand („94%“), Szene in der Mitte | Auf das Panel zuschneiden, bevor gelesen wird |
| Perspektive | Leicht schräg bzw. perspektivisch | Homographie auf Normgröße |

## 2. Pipeline im Detail

### 2.1 Vorverarbeitung und Locate

1. **Downscale per Box-Filter** auf ~1/4 für die Ankersuche. basetool hat gemessen, dass Bikubik schraffierte UI-Elemente verliert: 25 % vs. 83 % Trefferquote.
2. **Grob-OCR** (nur Detektion plus Erkennung auf dem Downscale bzw. auf Kacheln) findet die Text-Anker „SHOP INVENTORY“, „YOUR INVENTORIES“, „COMMODITIES“ und „AVAILABLE CARGO SIZE“. Ihre Positionen definieren die Panel-Geometrie.
3. **Panel-Rahmen:** Die orangefarbenen Rahmenlinien des Panels werden per Kantendetektion und Linien-Fit gesucht, um die vier Ecken für die Homographie zu bestimmen.
   - Fallback: Ecken aus den Text-Ankern plus Layout-Profil.
   - Letzter Fallback: manuell.
4. **Homographie** auf eine Normbreite (z. B. 1000 px für das Shop-Panel), dann bilineare Neuabtastung.
5. **Theme-agnostisch:** Für die Erkennung wird der Max-Kanal bzw. die Luminanz genutzt; Farbe dient nur für Tab, Status und Hover.

### 2.2 OCR

- **Modelle:** PaddleOCR **PP-OCRv6 small** Detektion (DBNet) und Erkennung (CTC), ONNX-Export von Hugging Face (`PaddlePaddle/PP-OCRv6_small_{det,rec}_onnx`), Apache-2.0. SHA-256 wird in `NOTICE` dokumentiert.
  - Laut basetool sind das ~10 MB + ~21 MB, das Wörterbuch hat 18.708 Einträge.
  - Die exakten Dateien werden beim Einbinden selbst heruntergeladen und gehasht.
- **Startparameter** (aus basetool, am eigenen Korpus nachzutunen):
  - kürzere Seite ≥ 736, auf ein Vielfaches von 32 gerundet
  - Binarisierung 0.2, Box-Schwelle 0.45, Unclip-Faktor 1.4
  - Erkennungshöhe 48
- **Prüfungen:**
  - Anzahl der Wörterbuchklassen gegen die Ausgabeform des Modells (Schutz vor falschem Wörterbuch)
  - Modell-Hash beim Laden
- **Laufzeit:** ORT-Session lazy, bei Inaktivität schließen; Intra-Op-Threads begrenzen (das Spiel läuft parallel).
- **Spätere Option** (nicht 1.0): ein eigenes, auf die SC-HUD-Schrift feinjustiertes Erkennungsmodell, falls der Korpus Bedarf zeigt.

### 2.3 Layout

- Karten über die wiederkehrende Struktur abgrenzen: Name links oben, „… SCU“ rechts oben, Preis „/SCU“ rechts darunter, Label „AVAILABLE CARGO SIZE“ unten.
- Zeilen-Clustering über vertikale Überlappung, Spalten über x-Zentren. Toleranzen werden **relativ** zur Normbreite angegeben, keine festen Pixel.
- Bildschirmreihenfolge (`screenOrder`) über die y-Position.

### 2.4 Feldparser (rein, property-getestet)

- **Preis:** `^[¤@9]?\s*(\d{1,3}(?:[,.\s]\d{3})*(?:[.,]\d{1,2})?)\s*([KkMm])?\s*/\s*SCU$`
  - Ein führendes Zeichen, das das Währungssymbol sein kann, wird **zweigleisig** behandelt: Kandidat mit und ohne dieses Zeichen.
  - Die Entscheidung trifft der Prior (§2.5).
- **SCU:** `(\d[\d,.\s]*)\s*SCU`, auch innerhalb verklebter Tokens.
- **Status:** gegen die Statusnamen aus `commodities_status` und die lokalisierten Namen; Zweitleser ist der Statusbalken (Füllhöhe ≈ Prozentband) bzw. die Textfarbe.
- **Cargo-Größen:** Anzahl der Kästchen plus OCR, abgeglichen mit den aufsteigenden Teilmengen von {1,2,4,8,16,24,32}.

### 2.5 Auflösung und Validierung mit UEX-Daten

1. **Terminal:**
   - Das Location-Feld wird gegen alle Location-Namen gematcht und liefert die Kandidaten-Terminals (`type=commodity`, `is_available_live`, nicht player-owned).
   - Bei mehreren Terminals an einer Location wird mit dem Sortiment abgeglichen (Jaccard der erkannten Commodity-IDs gegen das Sortiment aus `commodities_prices`).
   - Hinzu kommt der Sitzungskontext.
2. **Commodity:**
   - Zuerst wird gegen das **Sortiment des Terminals** gematcht, erst dann global.
   - Der Score kombiniert normalisiertes Levenshtein mit Präfix-Bonus.
   - Mindestabstand zum Zweitbesten; sonst `Ambiguous`.
3. **Preis-Prior:**
   - Kandidaten (roh, ohne Präfixzeichen, Confusable-Varianten, K/M-Skalierung) werden gegen den letzten Wert bzw. `price_*_avg_week` dieses Terminals und dieser Commodity bewertet, Toleranz `price_variation` %.
   - Liegt **genau ein** Kandidat in der Toleranz, wird er übernommen (Finding `Repaired`, falls nicht roh).
   - Liegen keiner oder mehrere in der Toleranz, wird der rohe Wert vorgeschlagen und als `OutOfTolerance`/`Ambiguous` markiert.
   - **Echte Preisänderungen** sind dadurch nicht blockiert: Der Nutzer bestätigt sie nur.
4. **Konsistenz:** „Out of Stock“ ⇒ SCU 0; Seite ↔ Abschnitt; `is_buyable`/`is_sellable`.
5. **Glyph-Topologie** (Lochzählung für 0/6/8/9, Konzept aus basetool, neu implementiert) arbeitet nur als **Veto** gegen Reparaturen.

### 2.6 Konfidenz (regelbasiert)

| Zustand | Konfidenz (Startwerte, am Korpus zu kalibrieren) |
|---|---|
| Sauber, im Prior-Band | 0.95 |
| Repariert (eindeutig, mit Zeuge) | 0.85 |
| Außerhalb der Toleranz, sonst plausibel | 0.70 → **prüfen** |
| Mehrdeutig / Konflikt zwischen Scans | 0.60 → **prüfen** |
| Unlesbar / unplausibel | 0.30 → **Fehler, Sendesperre** |

Die Report-Konfidenz entspricht dem schlechtesten Pflichtfeld, nicht dem Mittelwert, damit sich einzelne Fehler nicht „wegmitteln“.

### 2.7 Optionaler KI-Zweitleser (VLM über Ollama, nur bei geschlossenem Spiel)

**Warum zwei Leser?**

- basetool hat gemessen, dass **verschiedene** Leser unterschiedliche Fehler machen. Ihre zwei VLMs haben sich bei 8 von 430 Zellen widersprochen, aber nie auf denselben falschen Wert geeinigt.
- Dasselbe Modell zweimal laufen zu lassen bringt dagegen nichts (0 von 5 Fehlern gefunden, 2 neue erzeugt).
- Klassische OCR und VLM sind ein solches dekorreliertes Paar.

Die Zahlen stammen aus der Refinery-Domäne; ob sie übertragbar sind, prüft das Bake-off (Annahme A8).

**Ablauf:**

1. Während des Spiels liefert die klassische OCR sofort Ergebnisse. Reports mit Warnungen werden für die KI vorgemerkt; der Nutzer kann sie trotzdem jederzeit manuell korrigieren und senden.
2. Wenn das Spiel geschlossen ist (Hysterese), arbeitet die KI-Queue die vorgemerkten bzw. alle ungesendeten Reports ab. Eingabe sind die perspektivkorrigierten Panel-Ausschnitte (Shop-Panel, Location-Feld), auf eine Kante von ca. 1000–1500 px begrenzt.
3. Das Ergebnis läuft durch Parser, Vokabular-Auflösung und Validierung (identisch zur OCR), dann folgt die Fusion.
4. Startet das Spiel, wird der Request abgebrochen und das Modell entladen; die Jobs bleiben erhalten.

**Prompt** (`vlm/src/main/resources/prompts/shop_panel_v1.txt`, versioniert):

- beschreibt das Kartenlayout (Name, Statustext, Menge „… SCU“, Preis „¤…/SCU“, Cargo-Kästchen) und den aktiven Tab
- verlangt exakte Transkription („Ziffer für Ziffer, nichts korrigieren, `?` für Unlesbares“)
- Währungssymbol und Kontostand ausdrücklich ignorieren
- Antwortformat:

  ```
  TAB: Buy
  LOCATION: PATCH CITY
  | name | status | scu | price_per_scu | cargo_sizes |
  |---|---|---|---|---|
  | Omnapoxy | Medium Inventory | 333 | 3,237 | 1,2,4,8,16 |
  ```

**Warum Markdown statt JSON-Schema?** basetool hat freie Markdown-Ausgabe plus deterministischen Parser gegen schemaerzwungenes JSON gemessen: 0,9872 vs. 0,9821, mit weniger semantischen Fehlern. Das ist am eigenen Korpus zu bestätigen. Ollama unterstützt strukturierte Ausgabe (`format`); das ist eine Bake-off-Variante.

**Fusionsregeln pro Feld:**

| OCR | VLM | Ergebnis |
|---|---|---|
| Wert a | gleicher Wert a | a, Konfidenz 0.97 „doppelt bestätigt“ |
| a | b ≠ a, nur eine Confusable-Stelle verschieden | Glyph-Topologie und Prior entscheiden eindeutig, sonst `Ambiguous` (beide Kandidaten im UI) |
| a | b, stark verschieden | Der Kandidat im Prior-Band gewinnt nur, wenn genau einer drin liegt; sonst `Ambiguous` |
| unlesbar | b | b, wenn die Validierung ok ist, mit 0.85 (Ein-Leser-Wert); sonst `prüfen` |
| a | unlesbar | unverändert (OCR-Konfidenz) |
| Commodity-/Terminal-Auflösung verschieden | | immer `Ambiguous` → Pflichtauswahl |

Vom Nutzer bereits bestätigte oder korrigierte Felder überschreibt die KI **nie**; Abweichungen werden nur als Hinweis angezeigt.

**Grenzen (ehrlich):**

- basetool nennt für das 8B-Modell ~4 s/Bild auf einer RTX 5090 und ~53 s/Bild auf der CPU.
- Die Hardware-Stufen in basetool liegen bei ≥ 12 GB VRAM (8B) bzw. ≥ 8 GB (4B).
- Werte für unsere Panels müssen gemessen werden.
- Auf schwacher Hardware ist die KI ein „über Nacht“-Feature. Die klassische OCR bleibt deshalb der Primärweg.

## 3. Übernommene Erkenntnisse aus basetool-sc-extractor (GPL-3.0 – nur Konzepte)

| Übernommen | Nicht übernommen |
|---|---|
| PP-OCRv6 small über ORT, ohne OpenCV | Lokales VLM als **Primär**leser – bei uns nur **optionaler Zweitleser bei geschlossenem Spiel** (§2.7), weil es 8–12 GB VRAM bzw. ~50 s/Bild auf der CPU braucht |
| Box-Filter-Downscale für die Ankersuche | Refinery-spezifische Regeln und Farbkonstanten |
| Zahlen erst als Text lesen, dann deterministisch parsen | Fester 4K-Geometrie-Fallback (wir nutzen Text-Anker) |
| Confusable-Set und eindeutige Reparatur mit Zeugen | |
| Glyph-Topologie als Veto | |
| Regelbasierte Konfidenz statt Modell-Selbsteinschätzung | |
| Stitching: Randlesungen abwerten, Konflikte markieren | |
| Dekorrelierte Zweitleser (verschiedene Verfahren, nicht dasselbe Modell zweimal) | |
| Eval-Harness: Golden-Korpus außerhalb des Repos, Digest-Test, Kandidatenvergleich über das Pipeline-Ergebnis | |
| Crop-Dumps zur Sichtprüfung (ein Reskin brach dort die Lokalisierung unbemerkt) | |
| Hinweis an Nutzer: chromatische Aberration auf 0 | |

## 4. Mess- und Testkonzept

- **Korpus-Struktur:** `corpus/<id>/image.png` + `expected.json` (Terminal, Seite, Zeilen mit allen Feldern, `screenOrder`) + `meta.json` (Auflösung, Theme, Location, Spielversion).
- **Leser getrennt messen:** Die Eval läuft für „nur OCR“, „nur VLM (Modell X)“ und „Fusion“. Die Modellempfehlung folgt aus dem Fusionsergebnis auf dem Korpus, nicht aus der Modellkarte. VLM-Läufe sind opt-in (`UEXDR_VLM_HOST`), weil CI keine GPU hat.
- **Metriken pro Feldtyp:**
  - exakt richtig
  - korrekt markiert (falsch, aber geflaggt)
  - **still falsch** (falsch und als sicher eingestuft) – die wichtigste Metrik, Ziel ≈ 0
  - Laufzeit
- **CI** prüft den öffentlichen, geschwärzten Teilkorpus. Der private Korpus wird lokal über eine Umgebungsvariable (`UEXDR_CORPUS_DIR`) eingebunden.
- Die **Patch-City-Screenshots** sind der erste Korpus-Eintrag. Sie müssen als Dateien eingecheckt werden (Kontostand vorher schwärzen) oder im privaten Korpus liegen.
