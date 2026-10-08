# Review des Plans (2026-10-08)

Geprüft wurden alle Dokumente unter `docs/plan/`, dazu `CLAUDE.md` und `README.md`. Kriterien waren Vollständigkeit, sachliche Fehler, Logiklücken und Widersprüche zwischen den Dokumenten. Die Befunde sind direkt in den Dokumenten korrigiert. Diese Liste hält fest, **was** falsch oder lückenhaft war, damit die Änderungen nachvollziehbar bleiben.

## A. Sachliche Fehler (korrigiert)

| # | Befund | Korrektur |
|---|---|---|
| A-1 | `sealed interface Reader permits OcrReader, VlmReader` in `domain`, die Implementierungen aber in den Modulen `ocr` und `vlm`. **Kompiliert nicht:** In benannten JPMS-Modulen müssen permittierte Subtypen im selben Modul liegen. | `Reader` ist ein normales Interface; `ReaderKind` als Enum (02 §4b, CLAUDE.md-Regel) |
| A-2 | Die Preis-Regex in 07 §2.4 lehnte genau den Fall ab, für den sie gedacht war (`96705/SCU`), und alle Zahlen ohne Tausendertrenner | Prozeduraler Parser mit Kandidatenbildung (07 §2.4) |
| A-3 | ADR: „iced hat kein ausgereiftes Tabellen-Widget“ – **falsch**. `iced_widget` 0.14 enthält `table.rs` (geprüft im Crate-Quellcode). | Aussage korrigiert; die Einschätzungen zu egui und slint sind als unverifiziert gekennzeichnet. Die Entscheidung ändert sich nicht. |
| A-4 | ADR: basetool „beweist“, dass der OCR-Ansatz funktioniert – **überzogen**. Dort ist PP-OCR nur Ziffern-Zweitleser, Primärleser ist ein VLM. | Abgeschwächt; die Eignung als Primärleser wird in M2 gemessen |
| A-5 | `InventoryStatus` prüfte hart auf 1–7. Das widerspricht der eigenen Regel „Statusstufen aus der API, nicht aus Konstanten“. | Prüfung gegen `ReferenceSnapshot` (02 §3) |
| A-6 | Thread-Pool `min(2, cores/2)` ergibt 0 Threads auf 1-Kern-Systemen | `max(1, min(2, cores / 2))` |
| A-7 | R-NF-1 behauptete Wayland-Betrieb über XWayland als gegeben | Als Annahme bzw. Testpunkt markiert; native Wayland-Unterstützung von JavaFX ist nicht belegt |
| A-8 | Rust-Releasedatum: Das Build-Datum (28.09.) wurde als Releasedatum angegeben | Präzisiert |

## B. Logiklücken und Widersprüche (geschlossen)

| # | Befund | Lösung | Ort |
|---|---|---|---|
| B-1 | `pipeline → refdata → uex-api` und `pipeline → ocr`: Die „reine“ Pipeline hing an Modulen mit Netz- und Datei-I/O, im Widerspruch zu Leitprinzip 3 | `pipeline` hängt nur noch an `domain`. Sie bekommt `ReferenceSnapshot`, `TextDetector` und `ReaderResult` übergeben; `app` verdrahtet alles. | 02 §2, §4 |
| B-2 | Fusionsreihenfolge widersprüchlich (Diagramm: Fusion → Validierung; Text: Validierung → Fusion) | Festgelegt: pro Leser Parsen und Auflösen → Fusion → **einmal** validieren → Stitching | 02 §4b, 07 §2.7 |
| B-3 | **Sendeschwelle** nirgends definiert; Konfidenzwerte ohne Bedeutung für das Sende-Gate | Sendeschwelle 0.80 als Begriff und in der Konfidenztabelle | 01 Begriffe, 07 §2.6 |
| B-4 | Prior-basierte Reparatur konnte **echte Preisänderungen still auf alte Werte zurückreparieren**, und das mit 0.85, also sendefähig | Reparaturen nur mit unabhängigem Zeugen automatisch, sonst 0.75 → bestätigen | R-VAL-2b, 07 §2.5 |
| B-5 | Kein Verhalten definiert, wenn **kein Prior** existiert (neue Commodity am Terminal) | Fallback auf den Commodity-Durchschnitt, sonst keine Reparatur | R-VAL-2a |
| B-6 | Manuelle Erfassung **füllte UEX-Werte vor**. Damit lassen sich veraltete Werte per Durchklicken als frische Meldung senden; ein Datenqualitätsproblem für UEX. | Werte werden als Referenz angezeigt, nicht vorbefüllt; nur aktiv übernommene Zeilen werden gesendet | R-MAN-2 |
| B-7 | Manuelle Reports haben keinen Screenshot, neue DataRunner brauchen aber einen (`screenshot_required`) | Screenshot anhängbar, Pflicht wird vorab angezeigt | R-MAN-5, A11 |
| B-8 | Ein Report besteht aus mehreren Scroll-Captures, die API nimmt aber nur **einen** Screenshot | Shop-Ausschnitte vertikal zusammensetzen (Annahme A11) | R-SUB-7 |
| B-9 | „Zeitfenster“ für Reports war nicht definiert | Gruppierungsregel: gleiches Terminal, gleiche Seite und Umgebung, ≤ 10 min Abstand; Teilen und Zusammenführen im UI | R-OCR-13 |
| B-10 | UEX kennt nur `live`/`ptu`; die Zuordnung von HOTFIX, EPTU und TECH-PREVIEW fehlte | Standard-Mapping plus Annahme A9 | R-CAP-3a |
| B-11 | **Patch-Tag:** Screenshot vor dem Patch, Senden danach → falsche Spielversion | Version zum Aufnahmezeitpunkt festhalten, bei Wechsel warnen | R-CAP-3b |
| B-12 | `data_parameters` wurde 1 Tag gecacht, die Annahme-Flags ändern sich aber am Patch-Tag | Vor dem Senden ein Stand von höchstens 15 min | R-SUB-5 |
| B-13 | Die Sende-Queue war nicht persistent: Offline-Reports gingen beim Beenden verloren | Persistente Queue; nach einem Neustart erst nach Freigabe senden | R-SUB-2, 02 §6 |
| B-14 | Cooldown-Schlüssel uneinheitlich (01: mit Seite; 02: mit Seite und Umgebung); die UEX-Regel ist unbekannt | Einheitlich und konservativ (Terminal, Commodity, Umgebung); `duplicated_report` wird als Cooldown behandelt; Annahme A12 | R-SUB-4, 02 §6 |
| B-15 | Wird das Original gelöscht (Aufräumfunktion oder Nutzer), fehlte das Bild für Review, KI und Upload | Arbeitskopien der Panel-Ausschnitte mit Aufbewahrungsfrist | R-CAP-7 |
| B-16 | „manuell“ als Umgebungswert pro Ordner war unklar | Optionen: automatisch aus Pfad / fest / bei jedem Bild fragen | R-CAP-1 |
| B-17 | Diagramm: Manuelle Reports umgingen die Validierung (Widerspruch zu R-MAN-4) | Pfeil ergänzt | 02 §1 |
| B-18 | Locate braucht Grob-OCR, die Pipeline darf `ocr` aber nicht kennen | Interface `TextDetector` wird injiziert | 02 §4 |
| B-19 | M5-Abnahme „≤ 5 s Abbruch“ war mit einem 5-s-Prüftakt nicht sicher erreichbar | 2-s-Takt während eines KI-Laufs | 02 §4b, 04 M5 |
| B-20 | Verweise auf nicht existierende Anforderungen (`R-SCOPE`, Tray ohne Anforderung) | `R-SCOPE-2`, neu `R-UI-9` | 05 F28/F29 |
| B-21 | CLAUDE.md nannte „Annahmen A1–A6“, es gibt inzwischen A1–A12 | Aktualisiert | CLAUDE.md |

## C. Fehlende Teile (ergänzt)

| # | Fehlte | Ergänzt in |
|---|---|---|
| C-1 | Risikoregister (u. a. App-Token als möglicher Projektblocker, UEX-Nutzungsbedingungen, OCR-Genauigkeit, Layout-Patches) | 04 „Risiken“ |
| C-2 | Prüfung, ob die Toolchain (Error Prone, NullAway, palantir-java-format, jlink-Plugin, jpackage/WiX) mit **JDK 27** läuft | 04 M0, ADR §4 |
| C-3 | Offene API-Fragen: Sell-SCU-Feld (`scu_sell` vs. `scu_sell_stock`), `container_sizes` pro Commodity vs. pro Report, `is_missing`-Pflichtfelder, Screenshot-Komposit | 06 Offene Punkte 8–14, A10–A12 |
| C-4 | Bearbeitungsstatus der eigenen Reports bei UEX (`data_info`) in der Historie | R-SUB-4 |

## D. Bewusst offen gelassen (Entscheidung oder Verifikation nötig)

- **Vom Projektinhaber zu entscheiden:** Projekt- und Paketname, Lizenz, JDK 27 vs. 25 LTS.
- **Gegen die Live-API bzw. mit UEX zu klären:** A1, A2, A9–A12 sowie die offenen Punkte in 06. Das gilt besonders für `container_sizes`: Die Größen unterscheiden sich im Spiel pro Commodity. Gibt es das Feld nur pro Report, sollten wir es eher weglassen als verfälschen.
- **Am Korpus zu messen:** alle Schwellwerte (Sendeschwelle, Konfidenzstufen, Toleranzen, Gruppierungsfenster), Laufzeit- und RAM-Ziele, die VLM-Eignung (A8).
- **Auf echten Systemen zu testen:** Linux/Wine-Pfade (A3), Spielerkennung (A7), XWayland, Secret Service. Für libsecret über FFM ist zu beachten, dass die Store-API variadisch ist; die FFM-API unterstützt das, es erhöht aber den Aufwand. Alternative wäre D-Bus direkt.

## E. Restbewertung (ehrlich)

- Der Plan ist für den Start von M0 vollständig genug. Die größten Unsicherheiten liegen **außerhalb** unseres Einflusses: App-Token und Nutzungsbedingungen von UEX sowie die exakte Semantik von `data_submit`. Diese Punkte sollten vor jeder größeren Implementierung geklärt werden, weil sie das Projekt im schlimmsten Fall blockieren.
- Die zweitgrößte Unsicherheit ist die **reale OCR-Genauigkeit** von PP-OCR auf Terminal-Screenshots. Deshalb kommt M2 (Messung) vor dem Bau der kompletten Pipeline.
- Die Zahlen aus basetool (VLM-Geschwindigkeit, Markdown- vs. JSON-Genauigkeit, Erkennungsraten) stammen aus einer anderen Domäne (Refinery). Sie sind als Startwerte zu verstehen, nicht als Zusage.

## F. Nachtrag: Modularisierung, Wartbarkeit und Abweichungsmarkierung

Eine zweite Prüfung mit Blick auf Modularisierung, Clean Code und Best Practices hat Folgendes ergeben (Modulnamen in den Abschnitten A–E beziehen sich noch auf den alten Schnitt):

| # | Befund | Änderung |
|---|---|---|
| F-1 | `app` vereinte UI, Secret-Store, KI-Steuerung und Verdrahtung; `submission` und `capture` mischten Fachlogik (Queue, Cooldown, Policy) mit Technik (HTTP, SQLite, Dateisystem). Use-Cases waren dadurch nur mit UI bzw. Technik testbar. | Neuer Schnitt nach **Ports & Adapters**: Kern `domain`/`pipeline`/`application`, Adapter `adapter-*`, Präsentation `ui`, Composition Root `app` (02 §2) |
| F-2 | Mehrere Module hätten eigene SQLite-Zugriffe gehabt (Duplikation, verstreutes SQL) | Ein Modul `adapter-storage` mit Repositories; SQL nur dort (09 §6) |
| F-3 | OS-Integration (Secret-Store, Prozess-Monitor, Installationserkennung, Truststore) lag verteilt in `app` und `capture` | Gebündelt in `adapter-platform` |
| F-4 | Best Practices waren genannt, aber nicht **durchgesetzt** | 09: Regeln mit Durchsetzung (JPMS, ArchUnit, Error Prone/NullAway, JaCoCo-Gate, Dependabot, PR-Checkliste, Definition of Done); M0 richtet die Gates vor dem ersten Fachcode ein |
| F-5 | Zeitabhängige Logik (Cooldown, Hysterese, Gruppierung) war nicht deterministisch testbar | `java.time.Clock` als injizierter Port |
| F-6 | Abweichungen vom bisherigen UEX-Wert waren nur indirekt über die Konfidenz sichtbar. Ein **sauber gelesener, aber falscher** Wert (Zahlendreher in der Quelle, falsche Zeile) wäre unmarkiert geblieben. | Eigene Dimension „Abweichung“ mit farblicher Markierung, Δ-Anzeige, Alter der Referenz und Bestätigungspflicht bei starker Abweichung (R-UI-10..12, 07 §2.6, 02 §7) |
