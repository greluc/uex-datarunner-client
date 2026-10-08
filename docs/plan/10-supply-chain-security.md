# Supply-Chain-Sicherheit

Ziel: Weder eine kompromittierte Abhängigkeit, ein manipuliertes Build-Werkzeug oder eine CI-Aktion noch ein unterwandertes Release-Artefakt soll unbemerkt Code auf die Rechner der Nutzer bringen. Der Client verarbeitet einen UEX-Secret-Key und läuft neben dem Spiel – ein kompromittierter Build wäre ein direkter Angriff auf die Nutzer.

**Ehrliche Grenze:** Supply-Chain-Sicherheit ist Risikoreduktion, keine Garantie. Die folgenden Maßnahmen verhindern vor allem *unbemerkte* Änderungen (manipulierte Artefakte, umgehängte Tags, still nachgezogene Versionen). Gegen einen Maintainer, der eine bösartige, korrekt signierte Version veröffentlicht, helfen nur Abkühlzeit, Review und eine kleine Zahl von Abhängigkeiten.

## 1. Bedrohungsmodell

| # | Angriffsweg | Beispiel |
|---|---|---|
| T1 | Manipuliertes oder ausgetauschtes Artefakt im Repository bzw. auf dem Transportweg | Artefakt wird nach der Veröffentlichung ersetzt; Mirror liefert andere Bytes |
| T2 | Bösartige **neue Version** einer echten Abhängigkeit (Account-Übernahme beim Maintainer) | Patch-Release mit Schadcode wird automatisch übernommen |
| T3 | Typosquatting oder Dependency-Confusion über zusätzliche Repositories | gleichnamiges Paket in JitPack oder `mavenLocal` |
| T4 | Manipulierter **Gradle-Wrapper** (`gradle-wrapper.jar`) bzw. manipulierte Gradle-Distribution | geänderte JAR im PR |
| T5 | Kompromittierte **GitHub Action** (Tag wird auf bösartigen Commit umgehängt) | Vorfall um `tj-actions/changed-files` (März 2025) |
| T6 | Manipulierte **Binär-Assets**: ONNX-Modelle, native Bibliotheken, JDK des Builds | ausgetauschtes Modell von Hugging Face; JDK-Download |
| T7 | Manipulation **nach** dem Build (Release-Dateien ersetzt) | Installer im Release wird ausgetauscht |
| T8 | Native Bibliotheken, die zur Laufzeit in ein **gemeinsames Temp-Verzeichnis** entpackt werden (DLL-/SO-Planting) | sqlite-jdbc und ONNX Runtime entpacken Natives standardmäßig nach `java.io.tmpdir` |
| T9 | Laufzeit-Nachladen von Code oder Modellen aus unversionierten Quellen | Modell-Download „latest“; Auto-Update ohne Signaturprüfung |
| T10 | Optionale KI-Modelle über Ollama (vom Nutzer geladen) | manipuliertes Modell in einer fremden Registry |

## 2. Maßnahmen

### 2.1 Abhängigkeiten auflösen (T1–T3)

| ID | Maßnahme | Umsetzung |
|---|---|---|
| S-1 | **Nur zwei Repositories:** Maven Central für Bibliotheken, Gradle Plugin Portal nur für Plugins. Kein `mavenLocal()`, kein JitPack, keine Snapshot-Repos. | `settings.gradle.kts`: `dependencyResolutionManagement { repositoriesMode = RepositoriesMode.FAIL_ON_PROJECT_REPOS }`; Repository-Content-Filter (`exclusiveContent`/`content { includeGroup… }`) für das Plugin Portal |
| S-2 | **Nur feste Versionen** aus dem Version-Catalog; keine dynamischen Versionen (`+`, `latest.release`, Ranges), keine `SNAPSHOT`s | `resolutionStrategy { failOnDynamicVersions(); failOnChangingVersions() }` in `build-logic` für alle Konfigurationen |
| S-3 | **Dependency-Locking** für alle Konfigurationen inklusive Buildscript-Classpath; transitive Versionen sind eingefroren | `dependencyLocking { lockAllConfigurations() }`; `gradle.lockfile` eingecheckt; Änderung nur per `--write-locks` in einem eigenen Commit |
| S-4 | **Dependency-Verification:** SHA-256-Prüfsumme **und** PGP-Signatur jedes Artefakts (Bibliotheken, Plugins, transitive) | `gradle/verification-metadata.xml` mit `verify-metadata` und `verify-signatures` = `true`; vertrauenswürdige Schlüssel in `gradle/verification-keyring.keys` (eingecheckt). Ein Build mit unbekanntem Artefakt oder Schlüssel **bricht ab**. |
| S-5 | **Neue Schlüssel oder Prüfsummen nur mit Review:** Aktualisierung über `./gradlew --write-verification-metadata pgp,sha256 --export-keys help` in einem eigenen PR. Der Diff zeigt jeden neuen Schlüssel bzw. Hash; neue Schlüssel-Fingerprints werden gegen die Projektseite bzw. die Release-Ankündigung des Herausgebers geprüft und im PR dokumentiert. | `CODEOWNERS`: `gradle/verification-metadata.xml`, `gradle/verification-keyring.keys`, `gradle/libs.versions.toml`, `*.lockfile`, `.github/workflows/**` → Review durch den Maintainer Pflicht |
| S-6 | Artefakte ohne Signatur (falls unvermeidbar) nur mit Prüfsumme und **expliziter Begründung** in `verification-metadata.xml` (Kommentar „warum kein PGP“) | Review |

### 2.2 Updates (T2)

| ID | Maßnahme | Umsetzung |
|---|---|---|
| S-7 | **Abkühlzeit für neue Versionen:** Dependabot schlägt Versions-Updates frühestens **7 Tage** nach Veröffentlichung vor. Kompromittierte Releases werden erfahrungsgemäß oft innerhalb von Tagen entdeckt. **Sicherheits-Updates** sind davon ausgenommen. | `.github/dependabot.yml`: `cooldown: default-days: 7` (ohne Angabe gilt laut GitHub-Doku ein Standard von 3 Tagen); Ökosysteme `gradle` und `github-actions` |
| S-8 | Updates einzeln oder in kleinen Gruppen, nie automatisch gemergt; jedes Update aktualisiert Lockfile **und** Verification-Metadata im selben PR (Dependabot kann das nicht selbst → manueller Folge-Commit) | Review-Checkliste in 09 §11 erweitert |
| S-9 | Bei jedem Update kurz prüfen: Changelog, neue transitive Abhängigkeiten (Diff im Lockfile), neuer Signaturschlüssel? | PR-Vorlage |

### 2.3 Build-Werkzeuge (T4, T6)

| ID | Maßnahme | Umsetzung |
|---|---|---|
| S-10 | **Gradle-Wrapper absichern:** `distributionSha256Sum` in `gradle/wrapper/gradle-wrapper.properties`; CI prüft `gradle-wrapper.jar` gegen die offiziellen Prüfsummen | `gradle/actions/wrapper-validation` (gradle/actions v6.4.0, per Commit-SHA gepinnt) in jedem Workflow vor dem ersten Gradle-Aufruf |
| S-11 | **JDK des Release-Builds** nicht über den Toolchain-Auto-Download beziehen, sondern explizit über `actions/setup-java` (Distribution Temurin, feste Version) | `org.gradle.java.installations.auto-download=false` in CI; lokal darf Foojay genutzt werden, Releases entstehen **nur** in CI |
| S-12 | Keine Build-Skripte aus dem Netz (`apply(from = "https://…")`), keine Plugins außerhalb des Catalogs | ArchUnit kann das nicht prüfen → Review plus CI-Grep auf `apply(from` mit URL |

### 2.4 CI/CD (T5, T7)

| ID | Maßnahme | Umsetzung |
|---|---|---|
| S-13 | **Actions per vollständigem Commit-SHA pinnen** (Version als Kommentar), nie per Tag oder Branch | `uses: actions/checkout@<40-hex-sha> # v7.0.1`; Dependabot (`github-actions`) aktualisiert SHA und Kommentar |
| S-14 | **Minimale Rechte:** `permissions: contents: read` als Workflow-Default; Schreibrechte nur im Release-Job (`contents: write`, `id-token: write`, `attestations: write`) | Workflow-Review; kein `pull_request_target` mit Checkout von PR-Code |
| S-15 | Release-Builds nur aus **geschützten Tags** auf geschütztem `main`; Release-Job in einer GitHub-Environment mit Freigabe | Branch-/Tag-Protection, Environment „release“ |
| S-16 | **Build-Provenienz (SLSA):** Jedes Release-Artefakt bekommt eine signierte Attestierung | `actions/attest-build-provenance` (v4.2.2, SHA-gepinnt); Nutzer können mit `gh attestation verify` prüfen |
| S-17 | **Prüfsummen** (`SHA256SUMS`) für alle Artefakte; Anleitung zur Prüfung im README | Release-Job |
| S-18 | **SBOM** (CycloneDX) für jedes Release, als Release-Asset | Gradle-Plugin `org.cyclonedx.bom` 3.5.0 |
| S-19 | **Schwachstellen-Scan** in jedem PR und täglich: Lockfiles gegen OSV | `google/osv-scanner` v2.6.0 auf `gradle.lockfile`/`verification-metadata.xml`; bekannte kritische/hohe Schwachstellen in Laufzeit-Abhängigkeiten **blockieren** den Merge (Ausnahmen nur dokumentiert mit Ablaufdatum) |
| S-20 | Dependency-Graph an GitHub melden, damit Dependabot-Alerts auch transitive Gradle-Abhängigkeiten sehen | `gradle/actions/dependency-submission` |
| S-21 | Optional: Code-Signing der Windows-Installer (Authenticode) gegen SmartScreen-Warnungen und Manipulation | Kostet ein Zertifikat → Entscheidung des Projektinhabers; bis dahin Attestierung plus Prüfsummen |

### 2.5 Binär-Assets und Laufzeit (T6, T8, T9, T10)

| ID | Maßnahme | Umsetzung |
|---|---|---|
| S-22 | **ONNX-Modelle** werden mit fester Hugging-Face-Revision (Commit-Hash, nicht `main`) bezogen. SHA-256 steht in `NOTICE` und in einer Prüfdatei; ein Gradle-Task prüft vor dem Packen, die App prüft beim Laden. Abweichung = Abbruch mit klarer Meldung. | `adapter-ocr` (Laden), `build-logic` (Build-Prüfung) |
| S-23 | **Keine Laufzeit-Downloads von Code oder OCR-Modellen.** Optional nachladbare Modelle (z. B. weitere Schriftsysteme, R-L10N-3) nur aus fest konfigurierten URLs mit fest hinterlegtem SHA-256 | Review; `adapter-ocr` |
| S-24 | **Natives nicht ins gemeinsame Temp entpacken:** sqlite-jdbc und ONNX Runtime laden ihre Natives aus einem app-eigenen Verzeichnis (im jlink-Image mitgeliefert bzw. in ein nutzereigenes, nicht weltbeschreibbares Verzeichnis entpackt) | System-Properties für die Native-Pfade (z. B. `org.sqlite.tmpdir`); **die exakten Property-Namen für ONNX Runtime in M0 prüfen** |
| S-25 | **Update-Hinweis ohne Auto-Update:** Die App zeigt nur „neue Version verfügbar“ mit Link zur GitHub-Release-Seite; sie lädt und startet nichts selbst | `adapter-platform` |
| S-26 | **TLS überall**, kein Abschalten der Zertifikatsprüfung (auch nicht „zum Debuggen“) | ArchUnit-Regel: Implementierungen von `X509TrustManager`/`HostnameVerifier` nur in der einen, reviewten Composite-Klasse für `Windows-ROOT` |
| S-27 | **Ollama-Modelle (optional):** Die App zeigt den Modell-Digest aus `/api/tags` an. Der Nutzer kann ein Modell auf einen Digest **pinnen**; ein abweichender Digest erzeugt eine Warnung. Modelle nur aus der offiziellen Ollama-Registry empfehlen. Die Integrität des Downloads selbst liegt bei Ollama, nicht bei uns. | `adapter-vlm`, R-VLM-7 |

### 2.6 Repository und Prozess

| ID | Maßnahme |
|---|---|
| S-28 | Branch-Protection für `main`: PR-Pflicht, grüne CI, Review durch CODEOWNERS, kein Force-Push; 2FA für alle Maintainer |
| S-29 | **Wenige Abhängigkeiten** (09 §9): Jede neue Abhängigkeit braucht eine Begründung, eine Lizenzprüfung und einen Blick auf den Wartungszustand. Gibt es einen PGP-Schlüssel auf Central? Wie viele Maintainer? |
| S-30 | **Reproduzierbarkeit:** Archive mit `isPreserveFileTimestamps = false` und `isReproducibleFileOrder = true`. Ehrlich: jpackage-Installer (MSI/deb) sind nicht bitgenau reproduzierbar; die Attestierung (S-16) ersetzt das teilweise. |
| S-31 | Sicherheitsmeldungen: `SECURITY.md` mit Meldeweg (privat via GitHub Security Advisories) |

## 3. Was das konkret bringt

| Bedrohung | Abgedeckt durch | Restrisiko |
|---|---|---|
| T1 manipuliertes Artefakt | S-3, S-4 (Hash + Signatur) | gering |
| T2 bösartige neue Version | S-7 Abkühlzeit, S-8/S-9 Review, S-19 Scan | **mittel** – korrekt signierte Schadversion eines echten Maintainers wird nur durch Zeit und Review erkannt |
| T3 Typosquatting/Confusion | S-1, S-2, S-5 | gering |
| T4 Wrapper/Gradle | S-10 | gering |
| T5 Actions | S-13, S-14 | gering |
| T6 Modelle/Natives/JDK | S-4, S-11, S-22 | gering |
| T7 Manipulation nach Build | S-16, S-17 | gering, sofern Nutzer prüfen; ohne Code-Signing (S-21) prüfen das die wenigsten |
| T8 Native-Planting | S-24 | gering nach Umsetzung |
| T9 Laufzeit-Nachladen | S-23, S-25 | gering |
| T10 Ollama-Modelle | S-27 | **mittel** – liegt außerhalb unserer Kontrolle; Funktion ist optional und lokal |
