# Domain-Driven Design und Test-Driven Development

Wie wir fachlich modellieren (DDD) und wie wir Code entstehen lassen (TDD). Beides ergänzt [02-architektur.md](02-architektur.md) (Ports & Adapters) und [09-engineering-prinzipien.md](09-engineering-prinzipien.md) (Durchsetzung).

**Grundsatz „soweit möglich“:**

- **DDD** lohnt sich dort, wo echte Fachregeln liegen: Report, Sende-Gate, Abweichungen, Erkennung, Cooldown. In technischen Randbereichen (Dateiüberwachung, OS-Integration) wäre es Zeremonie.
- **TDD** gilt für den Kern ohne Ausnahme. Bei OCR-Schwellwerten, UI-Layout und nativer OS-Integration ist es nur eingeschränkt sinnvoll; dafür gibt es unten ausdrücklich andere Regeln, statt TDD vorzutäuschen.

## Teil A – Domain-Driven Design

### A1. Ubiquitous Language

Die Fachbegriffe sind verbindlich, in Doku (Deutsch) und Code (Englisch) jeweils gleich. Ein Begriff = eine Bedeutung. Neue Begriffe kommen zuerst hierher, dann in den Code.

| Begriff (Doku) | Code | Bedeutung |
|---|---|---|
| Capture | `Capture` | ein eingelesener Screenshot (Datei-Identität, Quelle, Umgebung, Aufnahmezeit, Spielversion zur Aufnahmezeit) |
| Scan | `Scan` | Ergebnis der Erkennung eines Captures: gelesene Karten mit Findings |
| Karte | `CardReading` | eine gelesene Commodity-Karte (Rohwerte pro Leser) |
| Leser | `Reader`, `ReaderKind` | Verfahren, das ein Panel liest (OCR, VLM) |
| Report | `Report` | Meldung für **ein** Terminal, **eine** Seite, **eine** Umgebung; Einheit für `data_submit` |
| Zeile | `ReportRow` | eine Commodity im Report mit Preis, SCU, Status, Kistengrößen |
| Seite | `TradeSide` | BUY („Buy“-Tab) / SELL („Local Market Value“-Tab) |
| Prior | `PricePrior` | bisheriger UEX-Wert für (Terminal, Commodity, Seite) samt Alter |
| Finding | `Finding` | begründeter Hinweis auf Unsicherheit (Grund-Code) |
| Konfidenz | `Confidence` | wie sicher **gelesen** wurde |
| Abweichung | `Deviation` | wie stark ein Wert vom Prior abweicht (`EQUAL`, `MINOR`, `MAJOR`, `NO_REFERENCE`) |
| Feldbewertung | `FieldAssessment` | Konfidenz + Abweichung + Referenz eines Felds |
| Bestätigung | `Confirmation` | ausdrückliche Freigabe eines Felds durch den Nutzer, an genau diesen Wert gebunden |
| Sendeschwelle | `SendThreshold` | Mindestkonfidenz für Senden ohne Bestätigung |
| Sende-Gate | `SubmissionGate` | Regelwerk, ob ein Report gesendet werden darf |
| Cooldown | `Cooldown` | Sperrzeit nach erfolgreicher Meldung für (Terminal, Commodity, Umgebung) |
| Umgebung | `GameEnvironment` | LIVE, PTU, EPTU, HOTFIX, TECH-PREVIEW |
| Referenzdaten | `ReferenceSnapshot` | unveränderlicher Stand der UEX-Daten für eine Entscheidung |

### A2. Bounded Contexts

| Kontext | Verantwortung | Kernmodell | Beziehung |
|---|---|---|---|
| **Erfassung** (Capture) | Bilder aus Ordnern/Zwischenablage aufnehmen, Duplikate vermeiden, Umgebung und Version zum Aufnahmezeitpunkt festhalten | `Capture`, `WatchedFolder` | liefert `CaptureImported` an Erkennung |
| **Erkennung** (Recognition) | Panel finden, lesen, auflösen, fusionieren, validieren | `Scan`, `CardReading`, `Finding`, Domain-Services | nutzt Referenzdaten (lesend); liefert `CaptureScanned` |
| **Meldung** (Reporting) | Reports bilden, prüfen, bestätigen, freigeben – **Kern der Fachregeln** | Aggregat `Report` | nutzt Erkennung und Referenzdaten; liefert `ReportReleased` |
| **Übermittlung** (Submission) | Senden, Queue, Cooldown, Historie, Rückzug | Aggregate `SubmissionJob`, `Cooldown` | Customer der UEX-API über eine **Anti-Corruption-Layer** |
| **Referenzdaten** (Reference) | UEX-Stammdaten und Preise als Lesemodell | `ReferenceSnapshot` und Value Objects | **Conformist** zu UEX (wir übernehmen deren Modell), abgeschirmt durch die ACL in `adapter-uex` |
| **Spielumgebung** (Game) | Umgebung, Version, „Spiel läuft“, Spiel-Lokalisierung | `GameEnvironment`, `GameVersion`, `GameState` | Shared Kernel mit allen |

**Umsetzung im Code:**

- Die Kontexte sind **Packages** in `domain` und `application` (`…domain.reporting`, `…application.reporting` usw.); die Erkennungslogik liegt im Modul `pipeline`.
- Gemeinsame Typen (IDs, `PricePerScu`, `ScuQuantity`, `GameEnvironment`) liegen im **Shared Kernel** `…domain.shared`. Er wird klein gehalten; Änderungen dort brauchen besondere Sorgfalt.
- **Regel:** Kontexte referenzieren andere Aggregate nur per ID und kommunizieren über Domain-Events oder Application-Services, nie über direkte Objektreferenzen. Durchgesetzt wird das per ArchUnit.

### A3. Aggregate und Invarianten

**`Report`** (Aggregate Root, Kontext Meldung) – hier liegen die wichtigsten Fachregeln:

| Invariante | Regel |
|---|---|
| I1 | Pro Report gibt es jede Commodity höchstens einmal. |
| I2 | Freigabe (`release()`) ist nur möglich, wenn: das Terminal aufgelöst ist; jedes Pflichtfeld ≥ Sendeschwelle **oder** bestätigt ist; jede `MAJOR`-Abweichung bestätigt ist; Umgebung und Spielversion zum UEX-Annahmestand passen. |
| I3 | Eine Bestätigung gilt für genau einen Wert. `correct(field, newValue)` hebt die Bestätigung auf. |
| I4 | Ein gesendeter Report ist unveränderlich; nur `withdraw()` ist möglich (führt zu `data_remove`). |
| I5 | Die Spielversion ist die zum Aufnahmezeitpunkt gültige; ein Versionswechsel macht den Report „zu prüfen“. |

Zustände als sealed Typen: `Draft → Released → Queued → Submitted | Rejected`, `Submitted → Withdrawn`. Ungültige Übergänge liefern `Outcome.Refused` mit Grund-Code; es gibt keine Status-Strings.

Weitere Aggregate:

- **`Capture`** (Erfassung): Identität = Inhalts-Hash; Zustand `Imported → Scanned | Failed`; Umgebung und Version unveränderlich nach dem Import.
- **`SubmissionJob`** (Übermittlung): ein Sendeversuch eines freigegebenen Reports; Retry-Zähler, letzter Fehler; persistiert.
- **`Cooldown`** (Übermittlung): Schlüssel (Terminal, Commodity, Umgebung), `until`-Zeitpunkt; Invariante: Ein Job für einen Schlüssel im Cooldown wird nicht gesendet.

**Größe:** Aggregate bleiben klein. Ein Report kennt Captures nur per ID, Terminals und Commodities nur per ID.

### A4. Value Objects

Immutable Records mit Validierung im kompakten Konstruktor, Gleichheit über Werte, keine primitiven Typen in fachlichen Signaturen (gegen „Primitive Obsession“):

`TerminalId`, `CommodityId`, `CaptureId`, `ReportId`, `PricePerScu` (BigDecimal ≥ 0), `ScuQuantity` (≥ 0), `InventoryStatus`, `ContainerSizes` (Teilmenge der erlaubten Größen, sortiert), `GameVersion`, `Confidence` (0..1), `Deviation`, `Percent`, `FileFingerprint`, `Confirmation`.

### A5. Domain-Services

Zustandslose Fachlogik, die zu keinem einzelnen Aggregat gehört (reine Funktionen):

- **Erkennung** (in `pipeline`): `CommodityResolver`, `TerminalResolver`, `PriceCandidateEvaluator`, `ReaderFusion`, `Stitcher`
- **Meldung**: `DeviationAssessor` (→ `FieldAssessment`), `ReportGrouper`, `SubmissionGate`

### A6. Domain-Events

`CaptureImported`, `CaptureScanned`, `ReportDraftCreated`, `ReportReleased`, `ReportSubmitted`, `SubmissionRejected`, `ReportWithdrawn`, `GameStateChanged`, `ReferenceDataRefreshed`.

- Events sind unveränderliche Records in `domain`.
- Zugestellt werden sie **in-process und synchron** über einen kleinen Dispatcher in `application`, ohne Framework.
- **Aggregate sind unveränderlich** (Records): Ein Befehl wie `report.confirm(…)` liefert `Outcome.Ok(neuerZustand, events)` oder `Outcome.Refused(grund)`. Der Application-Service speichert den neuen Zustand über das Repository und veröffentlicht danach die Events. So sind Aggregate ohne Mocks testbar und thread-sicher.
- Events machen die Abläufe testbar: „Given Events / When Command / Then Events“.

### A7. Repositories und Anti-Corruption-Layer

- **Ein Repository pro Aggregat** (Port in `domain`, Implementierung in `adapter-storage`). Es gibt keine Repositories für Entities innerhalb eines Aggregats.
- **ACL zu UEX** (`adapter-uex`):
  - UEX-DTOs (snake_case, Zahlen als Strings, 0/1-Flags, Status-Strings) werden an der Grenze in Value Objects übersetzt.
  - UEX-Fehlercodes werden zu sealed `SubmissionError`-Typen.
  - Kein UEX-Begriff „leckt“ in den Kern.
- **ACL zum Spiel:** `global.ini` und Ingame-Texte werden über Mapping-Tabellen in Fachbegriffe übersetzt (`InventoryStatus`, `TradeSide`).
- **ACL zu Ollama:** Die Modellantwort wird zu `CardReading`.

### A8. Markierung und Durchsetzung

- Eigene, abhängigkeitsfreie Marker-Annotationen in `domain` (`@AggregateRoot`, `@ValueObject`, `@DomainEvent`, `@DomainService`).
- **Kein jMolecules:** Das wäre eine zusätzliche Abhängigkeit im Kern (geprüft: `org.jmolecules:jmolecules-ddd` 2.0.1). Fünf eigene Annotationen leisten hier dasselbe.
- **ArchUnit-Regeln:**
  - `@ValueObject` und `@DomainEvent` sind Records.
  - `@AggregateRoot`-Klassen werden nur über ihr Repository geladen und gespeichert.
  - Aggregate referenzieren fremde Aggregate nur über `*Id`-Typen.
  - Domain-Typen haben keine öffentlichen Setter.
  - Kontext-Packages greifen nicht auf Interna anderer Kontexte zu.

## Teil B – Test-Driven Development

### B1. Wo TDD gilt

| Bereich | Vorgehen |
|---|---|
| `domain`, `pipeline`, `application` | **TDD verpflichtend:** Red → Green → Refactor. Kein Produktionscode ohne einen vorher fehlschlagenden Test. |
| Adapter (`adapter-uex`, `adapter-storage`, `adapter-refdata`, `adapter-capture`) | **Test-first, soweit möglich:** Kontrakt- und Integrationstests (WireMock, echte SQLite-Datei, echtes Temp-Verzeichnis) zuerst, dann die Implementierung |
| `adapter-ocr`, `adapter-vlm` und OCR-Schwellwerte | **Spike & Stabilize:** Exploration im Eval-Harness ist erlaubt (Ergebnis: Messwerte, kein Merge). Vor dem Merge wird das gewünschte Verhalten mit Golden- und Unit-Tests festgeschrieben, die ohne die Änderung fehlschlagen. |
| `ui` | ViewModels per TDD (ohne Fenster); Views (Layout, CSS) ohne TDD, kritische Flows per TestFX nach der Umsetzung |
| `adapter-platform` (FFM, Prozessliste) | Schmale Wrapper hinter Ports; der Port wird per Fake getestet, die native Seite durch Integrationstests auf dem Ziel-OS (CI-Matrix) |
| Bugfix (überall) | Zuerst ein Test, der den Fehler reproduziert |

### B2. Outside-in pro Use-Case

1. **Akzeptanztest** auf Ebene `application` (fachliche Sprache, Fakes für alle Ports), abgeleitet aus der Anforderung. Der Testname bzw. das Tag verweist auf die Anforderungs-ID, z. B. `@Tag("R-UI-10")`.
2. Daraus folgen Unit-Tests für Aggregate, Value Objects und Domain-Services.
3. Zum Schluss kommen die Adapter mit ihren Kontrakttests.

Beispiel (Struktur, nicht finaler Code):

```java
@Test @Tag("R-UI-10") @Tag("R-SUB-3")
void majorPriceDeviationBlocksReleaseUntilConfirmed() {
  var report = aDraftReport().at(PYRO_GATEWAY_STANTON).side(BUY)
      .row(HYDROGEN_FUEL, price("782"), prior("520", ageDays(1)))   // > price_variation
      .build();

  assertThat(report.release(gate)).isEqualTo(refused(UNCONFIRMED_MAJOR_DEVIATION));

  var confirmed = report.confirm(HYDROGEN_FUEL, PRICE).orThrow();
  assertThat(confirmed.release(gate)).satisfies(isOkWithState(Released.class));
}
```

### B3. Werkzeuge und Regeln

- **Fakes vor Mocks:** Für jeden Port gibt es eine handgeschriebene Fake-Implementierung im Gradle-`java-test-fixtures`-Source-Set des jeweiligen Moduls. Diese Fixtures nutzen alle Tests gemeinsam. Mockito nur für Interaktionsprüfungen, die ein Fake nicht sinnvoll abbildet.
- **Test-Data-Builder** (`aDraftReport()`, `aCapture()`, `aSnapshot()`) in denselben Test-Fixtures, statt Testdaten zu kopieren.
- **Property-based Tests** (jqwik) für Parser, Fuzzy-Matcher, Fusion, `DeviationAssessor` und die Invarianten des `Report` (z. B. „nach `correct` ist ein Feld nie bestätigt“).
- **Golden-Tests** für die Pipeline gegen den Korpus (`corpus/`).
- **Mutation-Testing** (PIT) für `domain`, `pipeline` und `application`, um zu prüfen, ob die Tests Fehler wirklich finden – bei TDD die ehrliche Kontrolle gegen „Tests ohne Aussage“:
  - Startwert für den Mutation-Score ≥ 70 %; er wird nach der ersten Messung festgelegt und darf nur steigen.
  - Läuft im PR für geänderte Klassen und nightly vollständig.
  - Versionen geprüft: PIT 1.30.0, Gradle-Plugin `info.solidsoft.pitest` 1.19.0, `pitest-junit5-plugin` 1.2.3. **Dass das JUnit-5-Plugin mit JUnit 6 und JDK 27 zusammenarbeitet, ist nicht geprüft** → M0.
- **Rot nie auf `main`:** Red/Green findet lokal bzw. im Feature-Branch statt; jeder Commit auf `main` ist grün.
- **Refactor-Schritt ist Pflicht:** Nach Grün wird aufgeräumt (Namen nach Ubiquitous Language, Duplikate entfernen), solange die Tests grün sind.

### B4. Rückverfolgbarkeit

- Jede M-Anforderung aus 01 hat mindestens einen Akzeptanztest mit `@Tag("<R-ID>")`.
- Ein kleiner Report im Build listet R-IDs ohne Test (Gradle-Task über JUnit-Tags). In M0 ist das eine Warnung, ab M3 ein Fehler für M-Anforderungen.
