# ADR-001: Implementierungssprache – Java vs. Rust

- **Status:** Akzeptiert (2026-10-08)
- **Entscheidung:** **Java 27 + JavaFX 27**, Build mit Gradle 9.8.1 (Kotlin DSL)
- **Kontext:** Desktop-Client für Windows und Linux, der Star-Citizen-Terminal-Screenshots per OCR ausliest bzw. manuelle Eingaben erlaubt, gegen UEX-Referenzdaten validiert und an die UEX-API übermittelt.

## 1. Was die App technisch braucht (gewichtet)

| # | Kriterium | Gewicht | Warum |
|---|---|---|---|
| K1 | GUI für datenlastige Review-/Editier-Masken (Tabellen, Comboboxen mit Suche, Bildausschnitte mit Overlays, Tastaturbedienung) | **5** | Kern der Nutzererfahrung. Die meisten Bugs des Vorbilds sind UX-/Review-Probleme (siehe [05](05-datarunner-fehleranalyse.md)). |
| K2 | ONNX-Runtime für PaddleOCR (Detektion + Erkennung) | **5** | Kern der Erkennung |
| K3 | Bildverarbeitung (Crop, Farbe, Homographie/Perspektive, Downscale) | 3 | Eigenimplementierung bleibt überschaubar |
| K4 | HTTP/JSON, Caching, Rate-Limit | 2 | In beiden Sprachen trivial |
| K5 | Paketierung/Distribution Windows + Linux | 3 | Installer bzw. portable Builds |
| K6 | Ressourcenverbrauch neben dem laufenden Spiel | 3 | Star Citizen braucht viel RAM/VRAM |
| K7 | Wiederverwendbarkeit vorhandener Referenz-Lösungen (basetool) | 3 | Verkürzt die OCR-Entwicklung |
| K8 | Sichere Ablage des UEX-Secret-Keys im OS-Keystore | 2 | Sicherheitsanforderung |
| K9 | Entwicklungsgeschwindigkeit/Iterationszyklus (UI-Tuning, OCR-Heuristiken) | 4 | OCR-Heuristiken brauchen viele Iterationen |

## 2. Faktenlage (am 2026-10-08 geprüft, nicht aus dem Gedächtnis)

### Java

- **JDK 27** ist GA (Mitte September 2026, kein LTS, sechs Monate Support). Das letzte LTS ist **JDK 25**.
- **JavaFX 27** ist GA auf Maven Central (`org.openjfx:javafx-controls:27`).
- **ONNX Runtime Java 1.30.0** (`com.microsoft.onnxruntime:onnxruntime`) ist offiziell und stabil.
  - Das JAR enthält bereits die Natives für `win-x64`, `linux-x64`, `linux-aarch64` und `osx-aarch64`; geprüft durch Auflisten des JAR-Inhalts.
  - Es bringt einen `Automatic-Module-Name: com.microsoft.onnxruntime` mit.
- **basetool-sc-extractor** (Kotlin/JVM, JDK 25) nutzt genau diese ONNX-Runtime-Version mit PP-OCRv6-small-Modellen.
  - Die Bildverarbeitung ist komplett auf `BufferedImage` geschrieben, ohne OpenCV.
  - Damit ist gezeigt, dass PP-OCR über ORT auf der JVM mit der SC-HUD-Schrift **technisch** funktioniert.
  - **Einschränkung:** In basetool ist PP-OCR nur der **Zweitleser für Ziffern** (Refinery-Panels); Primärleser ist dort ein VLM. Wie gut PP-OCR als **Primärleser** für Rohstoff-Terminals ist, ist unbelegt und wird in M2 gemessen.
- Weitere Bausteine:
  - **Gradle 9.8.1**
  - **Jackson 3.2.3** (`tools.jackson.core`)
  - **sqlite-jdbc 3.53.4.0**
  - **JUnit 6.1.3**
  - **Error Prone 2.50.0** + **NullAway 0.14.2** + **JSpecify 1.0.1**
  - **Spotless 8.10.3**
  - Versionsliste in [CLAUDE.md](../../CLAUDE.md)

### Rust

- **Rust 1.99.0** stable (Build vom 2026-09-28 laut `channel-rust-stable.toml`, Release ca. 1. Oktober 2026). Stabile Edition **2024** (geprüft mit rustc 1.97; eine Edition 2027 ist nicht stabil).
- **`ort` (ONNX-Runtime-Binding): neueste Version `2.0.0-rc.13`.**
  - Es gibt **keine stabile 2.x-Version**, nur Release Candidates.
  - Die Bibliothek wird breit genutzt (> 21 Mio. Downloads), ist aber formal ein RC.
- **`oar-ocr` 0.10.0** (2026-10-03) bietet eine fertige PaddleOCR-Pipeline in Rust auf Basis von `ort`. Das ist ein echter Vorteil für Rust.
- **`ocrs` 0.13.1 / `rten` 0.27.0** sind eine reine Rust-OCR-Alternative ohne native ONNX Runtime, aber mit eigenen Modellen statt PaddleOCR.
- **`image` 0.25.10 + `imageproc` 0.27.0** bringen projektive Transformationen bereits mit.
- GUI-Optionen:
  - **`egui` 0.36.2**: Immediate Mode, Tabellen über `egui_extras`
  - **`slint` 1.18.1**: deklarativ, `StandardTableView`
  - **`iced` 0.14.0** (Dezember 2025): hat seit 0.14 ein `table`-Widget („Display tables“, Zellen sind beliebige Widgets). Editierbare Zellen sind also möglich, Editier-Logik, Validierung und Fokus-Navigation muss man aber selbst bauen. Korrigiert nach Prüfung des Crate-Quellcodes von `iced_widget` 0.14.2; eine frühere Fassung behauptete fälschlich, iced habe kein Tabellen-Widget.
  - Die Einschätzungen zu egui und slint (Umfang der Inline-Bearbeitung) stammen **nicht** aus einer Code-Prüfung. Sie sind als Einschätzung zu lesen; für die Entscheidung ausschlaggebend ist, dass JavaFX editierbare Tabellen mit Cell-Editoren fertig mitbringt.
- **`keyring` 4.2.0** bietet plattformübergreifenden Zugriff auf den OS-Keystore. Das ist ein Vorteil gegenüber Java.

## 3. Bewertung (1 = schlecht, 5 = sehr gut)

| Kriterium | Gew. | Java | Rust | Begründung |
|---|---|---|---|---|
| K1 GUI | 5 | **5** | 2–3 | JavaFX bringt `TableView` mit Cell-Editoren, `ComboBox`, `Canvas`, CSS-Theming, Accessibility und HiDPI mit. In Rust sind editierbare Tabellen mit Validierungs-Highlighting deutlich mehr Eigenbau. |
| K2 ONNX | 5 | **5** | 4 | Java ist offiziell und stabil, Natives sind im JAR. Rust: `ort` ist nur RC; dafür gibt es die fertige Pipeline `oar-ocr`. |
| K3 Bildverarbeitung | 3 | 3 | **4** | Java braucht eine eigene Homographie (~100 Zeilen); `imageproc` hat sie fertig. |
| K4 HTTP/JSON | 2 | 5 | 5 | `java.net.http` + Jackson 3 vs. `reqwest` + `serde`: gleichwertig |
| K5 Distribution | 3 | 3 | **5** | `jpackage` erzeugt pro OS ein Paket mit Runtime (deutlich größer, Build pro OS nötig). Rust: ein Binary plus ONNX-Runtime-Lib. |
| K6 Ressourcen | 3 | 3 | **5** | Die JVM braucht mehr RAM. Sie ist per `-Xmx`, G1 und Compact Object Headers (JEP 534, ab JDK 27 Default) begrenzbar, bleibt aber über Rust. **Konkrete Zahlen müssen gemessen werden, sie werden hier nicht geschätzt.** |
| K7 Wiederverwendung | 3 | **5** | 3 | basetool läuft auf der JVM mit derselben ORT-Java-API: Konzepte (nicht Code, siehe Lizenz) lassen sich direkt übertragen. Code-Übernahme setzt GPL-3.0 voraus (siehe unten). |
| K8 Secret-Store | 2 | 3 | **5** | Java hat keinen plattformübergreifenden Keyring. Lösung: kleine eigene Anbindung per **FFM-API** an Windows Credential Manager und libsecret. Rust: `keyring` fertig. |
| K9 Iteration | 4 | **5** | 3 | Inkrementelle Kompilierung, Hot-Reload von CSS und schnelles UI-Prototyping sprechen für Java. Rust-Compile-Zeiten mit ORT plus GUI sind spürbar länger. |
| **Summe (gewichtet)** | | **128** | **113–118** | |

Rechnung Java: 25+25+9+10+9+9+15+6+20 = 128.
Rechnung Rust: (10–15)+20+12+10+15+15+9+10+12 = 113–118.

Die genaue Punktzahl ist weniger wichtig als die zwei Kriterien mit dem höchsten Gewicht: **GUI (K1)** und **OCR-Runtime (K2)**.

## 4. Entscheidung und ehrliche Einordnung

**Gewählt: Java.** Die Umsetzung ist in Java am einfachsten, weil

1. der aufwändigste Teil (die Review-UI mit editierbaren, validierten Tabellen und Bild-Overlays) mit JavaFX Standard ist und in Rust Eigenbau wäre;
2. die ONNX Runtime in Java offiziell stabil ist und die Natives für beide Ziel-OS im JAR mitbringt;
3. basetool-sc-extractor die technische Kette (PP-OCRv6 + ORT + `BufferedImage`) auf der JVM für dieselbe Spielschrift bereits nutzt – dort allerdings nur als Ziffern-Zweitleser; die Eignung als Primärleser misst erst M2.

**Was dagegen spricht – bewusst in Kauf genommen:**

- **Paketgröße und RAM** sind schlechter als bei Rust. Gegenmaßnahmen:
  - `jlink` nur mit den benötigten Modulen
  - Heap-Limit
  - Modelle lazy laden und die ORT-Session nach Inaktivität schließen
- **JDK 27 ist kein LTS.** Gemäß Vorgabe „aktuellste Version“ entwickeln wir auf JDK 27 und ziehen im März 2027 auf JDK 28 nach.
  - Für Endnutzer ist das egal, weil `jpackage` die Runtime mitliefert.
  - **Alternative:** JDK 25 LTS, falls weniger Upgrade-Aufwand gewünscht ist.
- **Keine Preview-Features** (z. B. Structured Concurrency, JEP 533 – in JDK 27 noch Preview). Produktivcode verwendet nur finale Features.
- **Toolchain-Risiko JDK 27:** Error Prone, NullAway, palantir-java-format und das jlink-Plugin greifen tief in javac bzw. das JDK ein. Ob die aktuellen Versionen JDK 27 unterstützen, ist **nicht geprüft**; das passiert in M0. Unterstützt eines davon JDK 27 nicht, ist der Fallback JDK 25 LTS (die Entscheidung Java bleibt davon unberührt).

**Wann Rust die bessere Wahl wäre:** wenn ein kleines Einzel-Binary und minimaler Speicherbedarf oberste Priorität hätten oder wenn das Team Rust deutlich besser beherrscht als Java. Die Architektur (siehe [02](02-architektur.md)) ist sprachneutral geschnitten, ein späterer Port des OCR-Kerns wäre also möglich.

**Nachtrag (optionale KI-Erkennung):** Das lokale VLM wird über die HTTP-API von Ollama angebunden (`java.net.http` + Jackson). Das ist in beiden Sprachen gleich einfach und ändert die Bewertung nicht. Die Spielerkennung über `ProcessHandle` ist im JDK enthalten; in Rust bräuchte es eine Crate wie `sysinfo`.

## 5. Lizenzhinweis (Konsequenz für die Wiederverwendung)

basetool-sc-extractor steht unter **GPL-3.0-or-later**.

- **Code übernehmen oder übersetzen** macht unseren Client zu einem abgeleiteten Werk, das unter GPL-3.0-kompatiblen Bedingungen veröffentlicht werden muss.
- **Ideen, Schwellwerte und Messergebnisse** dürfen frei neu implementiert werden.
- Die **PP-OCRv6-Modelle** stehen unter Apache-2.0 und dürfen direkt verwendet werden.

**Offene Entscheidung für den Projektinhaber:** Lizenz des Projekts festlegen (Empfehlung: GPL-3.0-or-later, wenn Code aus basetool portiert werden soll; sonst frei wählbar). Bis dahin implementieren wir **nur nach Konzept** und kopieren keinen Code.
