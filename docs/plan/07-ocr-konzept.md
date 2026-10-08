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

## 3. Übernommene Erkenntnisse aus basetool-sc-extractor (GPL-3.0 – nur Konzepte)

| Übernommen | Nicht übernommen |
|---|---|
| PP-OCRv6 small über ORT, ohne OpenCV | Lokales VLM (Ollama/Qwen3-VL): braucht 8–12 GB VRAM bzw. ~50 s/Bild auf der CPU und ist neben dem laufenden Spiel ungeeignet |
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
- **Metriken pro Feldtyp:**
  - exakt richtig
  - korrekt markiert (falsch, aber geflaggt)
  - **still falsch** (falsch und als sicher eingestuft) – die wichtigste Metrik, Ziel ≈ 0
  - Laufzeit
- **CI** prüft den öffentlichen, geschwärzten Teilkorpus. Der private Korpus wird lokal über eine Umgebungsvariable (`UEXDR_CORPUS_DIR`) eingebunden.
- Die **Patch-City-Screenshots** sind der erste Korpus-Eintrag. Sie müssen als Dateien eingecheckt werden (Kontostand vorher schwärzen) oder im privaten Korpus liegen.
