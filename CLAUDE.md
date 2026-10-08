# CLAUDE.md – UEX Datarunner Client

Desktop-Client (Windows + Linux) zum Erfassen von Star-Citizen-Rohstoff-Terminaldaten – **manuell oder per Screenshot-OCR** – und zum Übermitteln an die **UEX API 2.0** (`POST /data_submit`). Aktuelle UEX-Daten (Terminals, Commodities, letzte Preise, Statusstufen, Toleranzen) sind **Constraints und Vorgabewerte** für Erkennung und Auswahl.

**Status:** Planungsphase. Es existiert noch kein Code. Ausgangspunkt ist Meilenstein M0 in `docs/plan/04-roadmap.md`.

## Pflichtlektüre vor Änderungen

| Datei | Inhalt |
|---|---|
| `docs/plan/01-anforderungen.md` | Anforderungen mit IDs (`R-…`) und **Annahmen A1–A6** (unverifiziert!) |
| `docs/plan/02-architektur.md` | Module, Abhängigkeitsrichtung, Domänenmodell, Threading |
| `docs/plan/03-sprachentscheidung.md` | ADR Java vs. Rust, inklusive Lizenzhinweis zu basetool (GPL-3.0) |
| `docs/plan/05-datarunner-fehleranalyse.md` | Bekannte Fehler des Vorbilds (F1–F30) und unsere Fixes |
| `docs/plan/06-uex-api.md` | API-Notizen; **ungesicherte Punkte sind markiert** |
| `docs/plan/07-ocr-konzept.md` | OCR-Pipeline und Screenshot-Beobachtungen |

Wenn eine Änderung einer Anforderung oder Architekturentscheidung widerspricht, aktualisiere das Dokument im selben Commit oder frage nach.

## Arbeitsregeln

- **Nicht raten.** Versionen, API-Felder und Bibliotheks-APIs werden vor der Nutzung in der Quelle geprüft (Maven Central, offizielle Doku, Quellcode). Bei Unsicherheit: als Annahme markieren und nachfragen.
- **UEX-API:** Neue oder geänderte Felder und Fehlercodes werden erst gegen die Live-API mit `is_production=0` verifiziert, dann wird `docs/plan/06-uex-api.md` aktualisiert. Gegen die API wird **nie** mit `is_production=1` getestet.
- **basetool-sc-extractor ist GPL-3.0.** Code nicht kopieren oder übersetzen, solange die Projektlizenz nicht geklärt ist; nur Konzepte nachimplementieren. Die PP-OCR-Modelle sind Apache-2.0 und dürfen gebündelt werden (mit `NOTICE` und SHA-256).
- SC-Datarunner-UEX ist closed source: **nicht dekompilieren**.
- Keine Secrets (UEX-Secret-Key, App-Token) in Code, Tests, Logs, Fixtures oder Commits.
- Screenshots für den Korpus: Kontostand („CURRENT BALANCE“) schwärzen, bevor sie ins Repo kommen.

## Tech-Stack (Stand 2026-10-08, geprüft)

| Bereich | Wahl | Version |
|---|---|---|
| Sprache/JDK | Java (Toolchain via Foojay) | **27** (kein LTS; Wechsel auf 28 im März 2027 einplanen) |
| UI | JavaFX (`org.openjfx:javafx-controls`/`-fxml`, Plattform-Classifier `win`/`linux`) | 27 |
| Build | Gradle, Kotlin DSL, Version-Catalog `gradle/libs.versions.toml` | 9.8.1 |
| OCR-Runtime | `com.microsoft.onnxruntime:onnxruntime` (Natives win-x64/linux-x64 im JAR) | 1.30.0 |
| OCR-Modelle | PaddleOCR PP-OCRv6 small det + rec (ONNX, Hugging Face `PaddlePaddle/…`) | – |
| JSON | Jackson 3: `tools.jackson.core:jackson-databind` (Packages `tools.jackson.*`; **Annotationen bleiben** `com.fasterxml.jackson.annotation`) | 3.2.3 |
| HTTP | `java.net.http.HttpClient` (JDK) | – |
| DB | `org.xerial:sqlite-jdbc` | 3.53.4.0 |
| Logging | SLF4J API + Logback | 2.0.20 / 1.6.5 |
| Nullness | JSpecify + NullAway (über Error Prone) | 1.0.1 / 0.14.2 / 2.50.0 |
| Format | Spotless (palantir-java-format) | Plugin 8.10.3 |
| Tests | JUnit Jupiter, AssertJ, Mockito, WireMock, jqwik, TestFX | 6.1.3 / 3.27.7 / 5.24.0 / 3.13.2 / 1.10.1 / 4.0.18 |
| Gradle-Plugins | `net.ltgt.errorprone` 5.1.1, `org.beryx.jlink` 4.1.1, `com.github.ben-manes.versions` 0.65.0, `org.gradle.toolchains.foojay-resolver-convention` 1.0.0 | |

Regeln für den Stack:

- Nur **stabile** Releases, keine Alpha/Beta/RC/Milestone-Versionen. Ausnahme nur mit Begründung im Commit.
- Neue Abhängigkeit = Version auf Maven Central prüfen, in den Catalog eintragen, Lizenz prüfen (Apache/MIT/BSD/EPL/LGPL ok; GPL nur nach Lizenzentscheidung).
- Keine schwergewichtigen Frameworks (kein Spring, kein ORM, kein DI-Container); Konstruktor-Injektion.
- Kein OpenCV/JavaCV; Bildoperationen selbst auf `BufferedImage` bzw. `int[]`-Rastern.

## Befehle

Gelten ab M0, sobald der Build existiert:

```bash
./gradlew check                 # Format-Check, Error Prone/NullAway, alle Tests
./gradlew spotlessApply         # formatieren
./gradlew :app:run              # App starten
./gradlew test --tests '*ParserTest'          # gezielte Tests
./gradlew :tools:ocr-eval:run --args="--corpus $UEXDR_CORPUS_DIR"   # OCR-Auswertung
./gradlew dependencyUpdates     # verfügbare Updates (nur stabile übernehmen)
./gradlew :app:jpackage         # Installer für das aktuelle OS
```

Vor jedem Commit muss `./gradlew check` grün sein.

## Java-Konventionen (modernes Java, nur finale Features)

- **Keine Preview- oder Incubator-Features** (kein `--enable-preview`; z. B. Structured Concurrency in JDK 27 noch Preview).
- **Datentypen:** Records für Werte und DTOs; `sealed` Interfaces plus Records für Ergebnis- und Fehlertypen.
- **Kontrollfluss:** `switch` mit Pattern Matching bzw. Record-Patterns statt `instanceof`-Ketten; erschöpfende Switches ohne `default` bei sealed Types; Unnamed Variables `_` für ungenutzte Bindungen.
- **Nullness:** Nullness explizit: `@NullMarked` auf `package-info.java` jedes Packages, `@Nullable` nur wo nötig. Keine `Optional`-Felder und -Parameter; `Optional` nur als Rückgabewert.
- **Nebenläufigkeit:**
  - **Virtual Threads** für I/O (`Executors.newVirtualThreadPerTaskExecutor()`).
  - CPU-lastige OCR läuft in einem **begrenzten** Plattform-Thread-Pool.
  - `ScopedValue` statt `ThreadLocal` für Kontext.
  - UI-Updates nur über `Platform.runLater`.
- **Collections und Streams:** Sequenced Collections (`getFirst()`/`getLast()`/`reversed()`), Stream-Gatherers, wo sie Code klarer machen. Unveränderliche Collections (`List.of`, `Stream.toList()`).
- **Native-Zugriff:** **FFM-API** (`java.lang.foreign`) für Native-Calls (Credential Manager, libsecret); kein JNA, kein JNI.
- **Geld und Einheiten:**
  - Geld als `BigDecimal`, niemals `double`.
  - Zeiten als `java.time` (`Instant` intern, `Duration` für TTL und Cooldown).
- **JPMS:** Jedes Modul hat ein `module-info.java`; nur API-Packages exportieren.
- **Fehlerbehandlung:**
  - Erwartbare Fehler (API-Status, Parse-Fehler) als Rückgabetyp (sealed `Result`-Typen), nicht als Exceptions.
  - Exceptions nur für Programmierfehler und echte I/O-Ausnahmen.
  - Nie Exceptions verschlucken.
- **Logging:** SLF4J mit Platzhaltern; Secrets über einen Masking-Filter; keine Bild- oder Payload-Dumps auf INFO.
- **Lokalisierung:** UI-Texte nur über ResourceBundles (`messages_de.properties`, `messages_en.properties`); keine Strings im Code.
- **Kommentare und Bezeichner** auf Englisch; Doku unter `docs/` auf Deutsch.

## Architektur-Leitplanken

- Abhängigkeiten nur in Pfeilrichtung (siehe `02-architektur.md`); `domain` hat keine Abhängigkeiten außer JSpecify.
- Pipeline-Stufen sind **reine Funktionen** auf unveränderlichen Records, ohne UI-, Netz- oder Dateisystemzugriff. Dadurch sind sie golden-testbar.
- **Nichts still raten:**
  - Jeder unsichere Wert trägt ein `Finding` mit Grund-Code.
  - Das Sende-Gate blockiert Felder unter dem Schwellwert und unklare Terminals.
  - Wenn es mehrere Kandidaten gibt, wählt die App nichts still vor.
- **UEX-Werte nicht hartkodieren:** Spielversion, Statusstufen und Toleranzen kommen aus der API bzw. dem Cache, niemals aus Konstanten (siehe F14).
- Freie OCR-Strings erreichen nie die API; nur aufgelöste IDs aus dem UEX-Vokabular.
- Der Upload-Screenshot enthält nur das Shop-Panel und das Location-Feld; **der Kontostand wird immer geschwärzt** (Test Pflicht).

## Tests

- **Unit:** Parser (Preis, SCU, Status, Cargo-Größen) mit jqwik-Property-Tests; Fuzzy-Matcher; Validierungsregeln; Stitching.
- **API:** WireMock mit aufgezeichneten (anonymisierten) Antworten. Ein Live-Test läuft nur manuell, opt-in über `UEXDR_LIVE_TEST=1` und mit `is_production=0`.
- **OCR:**
  - Synthetische Bilder für Locate und Layout.
  - Der Golden-Korpus über `UEXDR_CORPUS_DIR`; der Test ist ohne Variable übersprungen, nicht grün-gelogen (`Assumptions.assumeTrue`).
  - Digest-Test für Modell- und Runtime-Updates.
- **Metrik „still falsch“** (falsch und als sicher markiert): Regressionen sind ein Blocker.
- **UI:** TestFX nur für kritische Flows (Onboarding, Sendesperre).
- Bugfixes immer mit einem Test, der den Fehler vorher reproduziert.

## Git

- Kleine, fokussierte Commits; Nachrichten auf Englisch im Imperativ (`Add price parser for currency glyph`).
- Keine generierten Artefakte, Modelle > 50 MB, privaten Screenshots oder Secrets einchecken. Modelle mit Hash in `NOTICE`.
