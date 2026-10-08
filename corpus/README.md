# OCR-Testkorpus

Dieses Verzeichnis enthält den **öffentlichen** Teil des Golden-Korpus (siehe `docs/plan/07-ocr-konzept.md` §4). Der private Teil liegt außerhalb des Repos und wird über `UEXDR_CORPUS_DIR` eingebunden.

## Regeln

- Kontostand („CURRENT BALANCE“) ist **immer geschwärzt**, bevor ein Bild hier landet. Keine Spielernamen, keine Chat-Fenster.
- Pro Eintrag gibt es ein Verzeichnis `<location>-<nr>/` mit den Bildern und `expected.json`.
- `expected.json` bleibt auf `"verified": false`, bis ein Mensch die Transkription unabhängig geprüft hat. Nicht verifizierte Einträge zählen in der Eval nur als „Smoke“, nicht als Golden-Metrik.
- Bevorzugt werden **Original-Screenshots** (verlustfrei, Originalauflösung). Per Chat hochgeladene Bilder sind vermutlich skaliert und neu komprimiert; das wird in `source` vermerkt.

## Einträge

| Eintrag | Inhalt | Status |
|---|---|---|
| `pyro-gateway-stanton-01` | Pyro Gateway (Stanton), blaues Theme: 4× Buy (gescrollt, ohne Überlappung), 1× Sell | transkribiert, nicht verifiziert |
