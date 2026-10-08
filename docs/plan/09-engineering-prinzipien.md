# Engineering-Prinzipien: Modularisierung, Wartbarkeit, Clean Code

Dieses Dokument macht „Best Practices“ **überprüfbar**: Jede Regel nennt, *wie* sie durchgesetzt wird. Wo möglich, geschieht das automatisch (Compiler, JPMS, ArchUnit, Error Prone, CI), sonst über Review-Checkliste und Definition of Done.

Regeln ohne Durchsetzung sind Wunschdenken. Deshalb gibt es bewusst wenige Regeln, und diese sind verbindlich.

## 1. Qualitätsziele (Priorität absteigend)

1. **Korrektheit der gesendeten Daten:** keine still falschen Werte an UEX.
2. **Wartbarkeit:** SC-Patches ändern das Terminal-Layout, UEX ändert die API. Solche Änderungen sollen lokal in einem Modul bleiben.
3. **Testbarkeit:** Fachlogik ohne UI, Netz, Datenbank und Spiel testbar.
4. **Sicherheit und Datenschutz:** Secret-Key, Kontostand.
5. **Portabilität:** Windows und Linux.
6. **Ressourcenschonung** neben dem laufenden Spiel.

## 2. Modularisierung

| Regel | Durchsetzung |
|---|---|
| Ports & Adapters: Kern (`domain`, `pipeline`, `application`) kennt keine Technik | JPMS: Kernmodule `requires` nur `domain` bzw. JSpecify. **ArchUnit:** keine Klassen aus `javafx..`, `java.net.http..`, `java.sql..`, `ai.onnxruntime..`, `tools.jackson..` im Kern; kein `java.nio.file.Files` in `domain`/`pipeline`/`application`. |
| Abhängigkeitsrichtung wie in [02 §2](02-architektur.md), zyklenfrei | Gradle-Projektabhängigkeiten plus JPMS (Zyklen kompilieren nicht); ArchUnit `slices().should().beFreeOfCycles()` auch für Packages **innerhalb** eines Moduls |
| Jedes Modul hat eine schmale öffentliche API | `module-info.java` exportiert nur `…<modul>.api` (bzw. bewusst gewählte Packages); Implementierung liegt in `…<modul>.internal`; ArchUnit: kein Zugriff auf fremde `internal`-Packages |
| Adapter sprechen nicht miteinander, außer den in 02 genannten Kanten | Gradle-Abhängigkeiten in `build-logic` zentral erlaubt; ArchUnit-Regel pro Adapter |
| Kein Durchreichen von Technik-Typen über Modulgrenzen (DTOs, `ResultSet`, `OrtSession`, JavaFX-Typen) | Ports verwenden nur `domain`-Typen; Adapter mappen an der Grenze (ArchUnit: Port-Signaturen nur mit `domain`-Typen) |
| Package-by-Feature innerhalb eines Moduls (z. B. `pipeline.locate`, `pipeline.parse`, `pipeline.stitch`) statt Package-by-Layer | Review-Checkliste |
| `app` enthält nur Verdrahtung und Start | ArchUnit: keine Klasse in `app` außer `Main`, `*Wiring`/`*Module` und Konfigurationsladern; Zeilenbudget als Review-Hinweis |
| Kein Service-Locator, keine statischen Singletons, keine globalen veränderlichen Zustände | ArchUnit: keine nicht-finalen `static` Felder; `static final` nur für Logger und unveränderliche Konstanten |

## 3. Clean Code

| Regel | Durchsetzung |
|---|---|
| **Sprechende Namen** in der Fachsprache des Glossars (01 „Begriffe“): `Capture`, `Scan`, `Report`, `Prior`, `Finding` – überall gleich | Review; Glossar ist verbindlich |
| **Eine Verantwortung** pro Klasse und Methode. Richtwerte: Methoden ≤ ~30 Zeilen, Klassen ≤ ~300 Zeilen. Überschreitungen brauchen einen Grund. | Review-Checkliste (Richtwert, kein Dogma) |
| **Keine magischen Zahlen:** Schwellwerte (Sendeschwelle, Toleranzen, Ruhezeiten, Hysterese, Gruppierungsfenster) liegen in typisierten Settings-Records mit dokumentierten Defaults, nicht verstreut im Code. UEX-Werte kommen aus `ReferenceSnapshot`. | Review; Error Prone; Tests prüfen die Defaults an einer Stelle |
| **Unveränderlich als Standard:** Records, `List.copyOf`, keine Setter in `domain`/`pipeline` | ArchUnit: Klassen in `domain` sind Records, Enums, sealed Interfaces oder Ports (Interfaces) |
| **Keine Booleschen Steuerparameter** in öffentlichen APIs (`process(x, true)`) → Enums oder eigene Methoden | Review |
| **Null-frei:** JSpecify `@NullMarked`, NullAway auf Fehler-Level | Build bricht bei Verstößen |
| **Fehler sind Werte:** Erwartbare Fehler als sealed `Result`/`Finding`-Typen, keine Exceptions für Kontrollfluss; nie Exceptions schlucken | Error Prone (`CatchAndPrintStackTrace`, ungenutzte Rückgabewerte über `@CheckReturnValue`), Review |
| **Kommentare erklären das Warum**, nicht das Was. Öffentliche Ports und Module haben Javadoc. | Javadoc-Lint für exportierte Packages (`-Xdoclint` auf `api`-Packages) |
| **Kein toter Code, keine auskommentierten Blöcke**; `TODO` nur mit Issue-Nummer | Error Prone (`UnusedVariable`, `UnusedMethod`); CI-Schritt prüft `TODO` ohne `#<Issue>` |
| **Einheitliches Format** | Spotless mit google-java-format (Google Java Style) im `check`; Formatierung wird nie von Hand diskutiert |
| **Kleine, testbare reine Funktionen** in `pipeline`; Seiteneffekte nur in Adaptern | Modulschnitt plus ArchUnit (siehe §2) |

## 4. Fehlerbehandlung, Logging, Beobachtbarkeit

- **Fehlerkategorien:**
  - fachlich (`Finding`, UEX-Statuscodes) → werden dem Nutzer erklärt
  - technisch erwartbar (Netz weg, Datei gesperrt) → Retry oder Hinweis
  - Programmierfehler → Exception, Log auf ERROR, Diagnose-Export
- **Nutzertexte** nur über i18n-Schlüssel; Fehlercodes werden zentral auf Schlüssel gemappt (eine Tabelle, testabgedeckt: jeder bekannte UEX-Code hat einen Text).
- **Logging:** SLF4J, Kontext über MDC (`captureId`, `reportId`), kein Logging von Secrets, Bilddaten oder kompletten Payloads auf INFO. Ein Test belegt, dass der Masking-Filter greift.

## 5. Nebenläufigkeit

- Geteilter Zustand ist unveränderlich (Records). Veränderlicher Zustand hat **genau einen Besitzer** (z. B. Queue-Service) und wird nur über dessen API geändert.
- Executor werden injiziert und beim Beenden geordnet geschlossen (`AutoCloseable`, `try-with-resources` in `app`). Es gibt keine selbst erzeugten Threads in Fachcode.
- Abbruch ist ein regulärer Pfad: KI-Lauf, Einlesen und Senden sind abbrechbar, und das ist getestet.
- Zeit kommt immer über `java.time.Clock` (injiziert): Cooldown, Hysterese und Gruppierung sind damit deterministisch testbar.

## 6. Persistenz

- `adapter-storage` besitzt die Verbindung. Jede fachliche Tabelle gehört genau einem Repository.
- **Kein SQL außerhalb der Repositories** (ArchUnit: `java.sql..` nur in `adapter-storage`).
- Schema-Migrationen sind versioniert, nur vorwärts und haben je einen Test (leere DB → aktuelle Version; Vorversion mit Testdaten → aktuelle Version).
- Konfigurationsdateien haben eine Schema-Version und eine Migration (siehe F17).

## 7. UI (MVVM)

- **Views** sind passiv (Layout, Binding, CSS) und enthalten keine Logik.
- **ViewModels** halten den UI-Zustand und rufen Use-Cases auf; sie sind ohne Fenster unit-testbar.
- Formatierung (Zahlen, Δ %, „vor 3 Tagen“) liegt an **einer** Stelle (Formatter-Klasse) und ist lokalisiert.
- Abweichungs- und Konfidenzdarstellung entsteht über CSS-Pseudoklassen aus `FieldAssessment` (02 §7); keine Farbwerte im Java-Code.

## 8. Tests (Testpyramide)

| Ebene | Was | Werkzeug |
|---|---|---|
| Unit (Basis, die meisten) | `domain`, `pipeline`, `application` mit Fakes der Ports | JUnit 6, AssertJ, jqwik (Parser, Fuzzy-Matcher, Fusion, Abweichungsbewertung) |
| Architektur | Regeln aus §2/§3/§6 | ArchUnit |
| Adapter-Integration | SQLite (echte Datei-DB im Temp-Ordner), UEX-Client gegen WireMock, Ordner-Watcher gegen echtes Temp-Verzeichnis | JUnit, WireMock |
| Kontrakt | UEX-Antworten (aufgezeichnet, anonymisiert) gegen unsere DTOs | JUnit |
| Golden/Korpus | OCR- und Pipeline-Ergebnis gegen Erwartungswerte, Metrik „still falsch“ | `tools/ocr-eval`, opt-in über `UEXDR_CORPUS_DIR` |
| UI (wenige) | Onboarding, Sendesperre, Abweichungsbestätigung | TestFX |

Weitere Testregeln:

- **Testnamen** beschreiben das Verhalten (`rejectsSubmissionWhenMajorDeviationUnconfirmed`); Aufbau Given/When/Then; Testdaten über Builder statt Copy-Paste.
- **Flaky Tests** werden nicht toleriert: Ursache beheben, nicht deaktivieren.
- **Coverage-Gate (JaCoCo):** Startwert ≥ 85 % Zeilen für `domain`, `pipeline`, `application`. Für Adapter gibt es keine Quote, dafür Integrationstests. Die Quote dient der Lückenfindung, nicht als Selbstzweck.

## 9. Abhängigkeiten und Build

- Versionen nur im Version-Catalog; Convention-Plugins in `build-logic` statt Copy-Paste in `build.gradle.kts`.
- **Dependabot** (GitHub-nativ) für Gradle und GitHub Actions; Updates nur auf stabile Versionen, jeweils mit grüner CI.
- Neue Abhängigkeit nur mit Begründung im PR. Zu prüfen sind Lizenz, Wartungszustand (letztes Release, Maintainer) und Größe. Lieber 50 Zeilen eigener Code als eine schwere Bibliothek für eine Funktion.
- Reproduzierbare Builds: Gradle-Wrapper mit Checksumme, Dependency-Locking.

## 10. Dokumentation und Entscheidungen

- **ADRs** für Architekturentscheidungen unter `docs/adr/NNNN-titel.md`. ADR-001 ist die Sprachentscheidung ([03](03-sprachentscheidung.md)); weitere folgen, z. B. für Ports & Adapters, Abweichungsmodell und Lizenz.
- `docs/plan/` bleibt die fachliche Wahrheit. Code, der davon abweicht, ändert das Dokument im selben PR.
- `CHANGELOG.md` nach „Keep a Changelog“; Versionierung nach **SemVer**. Persistierte Daten (DB, Config) gelten als öffentliche Schnittstelle: Bricht sich das Format, gibt es eine Migration.

## 11. Prozess: Definition of Done und Review-Checkliste

**Definition of Done** für jede Änderung:

- [ ] `./gradlew check` grün auf Windows und Linux (CI-Matrix)
- [ ] Tests für neues Verhalten; Bugfix mit reproduzierendem Test
- [ ] Keine neuen ArchUnit-, Error-Prone- oder NullAway-Verstöße; keine Unterdrückung ohne Kommentar mit Begründung
- [ ] Nutzertexte in DE und EN
- [ ] Betroffene Plan- oder ADR-Dokumente aktualisiert
- [ ] Keine Secrets, privaten Screenshots oder großen Binärdateien im Diff

**Review-Checkliste** (PR-Vorlage `.github/pull_request_template.md`):

- Liegt der Code im richtigen Modul und Package? Neue Modulkante nötig – und erlaubt?
- Ist die Fachlogik frei von Technik?
- Gibt es magische Zahlen, Boolean-Parameter, Null-Rückgaben oder verschluckte Exceptions?
- Sind Namen glossar-konform?
- Sind Fehlerfälle und Abbruch behandelt und getestet?
- Ist die Änderung so klein wie möglich (eine Sache pro PR)?

## 12. Werkzeug-Versionen (geprüft am 2026-10-08)

| Werkzeug | Version | Hinweis |
|---|---|---|
| ArchUnit (`com.tngtech.archunit:archunit-junit5`) | 1.5.1 | Unterstützung für Class-Files von JDK 27 in M0 prüfen |
| JaCoCo | 0.8.15 (Juni 2026) | Unterstützung für JDK-27-Class-Files in M0 prüfen; sonst Coverage-Gate bis zum Update aussetzen (nicht den JDK wechseln) |
| Übrige | siehe [CLAUDE.md](../../CLAUDE.md) | |
