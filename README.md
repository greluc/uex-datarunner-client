# UEX Datarunner Client

Plattformübergreifender Desktop-Client (Windows + Linux) für Star-Citizen-DataRunner. Er erfasst Daten von Rohstoff-Terminals – **manuell oder per Screenshot-OCR** – und übermittelt sie an [UEX Corp](https://uexcorp.space). Aktuelle UEX-Daten dienen dabei als Vorgabewerte und Plausibilitätsgrenzen für die Erkennung.

Während des Spiels arbeitet eine schlanke, lokale OCR. Optional prüft ein lokales KI-Modell (über Ollama) die Ergebnisse nach, sobald das Spiel geschlossen ist.

> Status: **Planung**. Es gibt noch keinen lauffähigen Code.

## Plan

| Dokument | Inhalt |
|---|---|
| [01 – Anforderungen](docs/plan/01-anforderungen.md) | Funktionale und nicht-funktionale Anforderungen, Annahmen |
| [02 – Architektur](docs/plan/02-architektur.md) | Module, Datenfluss, Domänenmodell, Plattform-Integration |
| [03 – Sprachentscheidung (ADR)](docs/plan/03-sprachentscheidung.md) | Java vs. Rust, Bewertung, Entscheidung: Java 27 + JavaFX 27 |
| [04 – Roadmap](docs/plan/04-roadmap.md) | Meilensteine M0–M5 mit Abnahmekriterien, Risiken |
| [05 – Fehleranalyse SC-Datarunner-UEX](docs/plan/05-datarunner-fehleranalyse.md) | Bekannte Fehler des Vorbilds und unsere Fixes |
| [06 – UEX API](docs/plan/06-uex-api.md) | Endpoints, Payload, offene Verifikationspunkte |
| [07 – OCR-Konzept](docs/plan/07-ocr-konzept.md) | Pipeline, Screenshot-Beobachtungen, Lehren aus basetool-sc-extractor |
| [08 – Review](docs/plan/08-review.md) | Prüfung des Plans: korrigierte Fehler, geschlossene Lücken, offene Punkte |
| [09 – Engineering-Prinzipien](docs/plan/09-engineering-prinzipien.md) | Modularisierung, Clean Code, Tests, Definition of Done – mit Durchsetzung |

## Hinweis

Inoffizielles Community-Projekt, nicht mit UEX Corp oder Cloud Imperium Games / Roberts Space Industries verbunden.
