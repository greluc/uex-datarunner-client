# ADR-001: Implementation language – Java vs. Rust

- **Status:** Accepted (2026-10-08)
- **Decision:** **Java 27 + JavaFX 27**, build with Gradle 9.8.1 (Kotlin DSL)
- **Context:** Desktop client for Windows and Linux that reads Star Citizen terminal screenshots via OCR or allows manual input, validates against UEX reference data and submits to the UEX API.

## 1. What the app needs technically (weighted)

| # | Criterion | Weight | Why |
|---|---|---|---|
| K1 | GUI for data-heavy review/edit forms (tables, comboboxes with search, image crops with overlays, keyboard operation) | **5** | Core of the user experience. Most bugs of the original are UX/review problems (see [05](05-datarunner-bug-analysis.md)). |
| K2 | ONNX Runtime for PaddleOCR (detection + recognition) | **5** | Core of the recognition |
| K3 | Image processing (crop, color, homography/perspective, downscale) | 3 | Own implementation remains manageable |
| K4 | HTTP/JSON, caching, rate limit | 2 | Trivial in both languages |
| K5 | Packaging/distribution Windows + Linux | 3 | Installers or portable builds |
| K6 | Resource consumption next to the running game | 3 | Star Citizen needs a lot of RAM/VRAM |
| K7 | Reusability of existing reference solutions (basetool) | 3 | Shortens OCR development |
| K8 | Secure storage of the UEX secret key in the OS keystore | 2 | Security requirement |
| K9 | Development speed/iteration cycle (UI tuning, OCR heuristics) | 4 | OCR heuristics need many iterations |

## 2. Facts (checked on 2026-10-08, not from memory)

### Java

- **JDK 27** is GA (mid-September 2026, not LTS, six months of support). The latest LTS is **JDK 25**.
- **JavaFX 27** is GA on Maven Central (`org.openjfx:javafx-controls:27`).
- **ONNX Runtime Java 1.30.0** (`com.microsoft.onnxruntime:onnxruntime`) is official and stable.
  - The JAR already contains the natives for `win-x64`, `linux-x64`, `linux-aarch64` and `osx-aarch64`; checked by listing the JAR contents.
  - It ships an `Automatic-Module-Name: com.microsoft.onnxruntime` → in the jlink image, `org.beryx.jlink` merges it into the merged module (explicit `mergedModuleName`), so native access is granted to that module (see [02](02-architecture.md) §8).
- **basetool-sc-extractor** (Kotlin/JVM, JDK 25) uses exactly this ONNX Runtime version with PP-OCRv6 small models.
  - The image processing is written entirely on `BufferedImage`, without OpenCV.
  - This shows that PP-OCR via ORT on the JVM works **technically** with the SC HUD font.
  - **Limitation:** In basetool, PP-OCR is only the **second reader for digits** (refinery panels); the primary reader there is a VLM. How good PP-OCR is as the **primary reader** for commodity terminals is unproven and will be measured in M2.
- Further building blocks:
  - **Gradle 9.8.1**
  - **Jackson 3.2.3** (`tools.jackson.core`)
  - **sqlite-jdbc 3.53.4.0**
  - **JUnit 6.1.3**
  - **Error Prone 2.50.0** + **NullAway 0.14.2** + **JSpecify 1.0.1**
  - **Spotless 8.10.3**
  - Version list in [CLAUDE.md](../../CLAUDE.md)

### Rust

- **Rust 1.99.0** stable (build of 2026-09-28 according to `channel-rust-stable.toml`, release approx. October 1, 2026). Stable edition **2024** (checked with rustc 1.97; an edition 2027 is not stable).
- **`ort` (ONNX Runtime binding): latest version `2.0.0-rc.13`.**
  - There is **no stable 2.x version**, only release candidates.
  - The library is widely used (> 21 million downloads), but is formally an RC.
- **`oar-ocr` 0.10.0** (2026-10-03) offers a ready-made PaddleOCR pipeline in Rust based on `ort`. That is a real advantage for Rust.
- **`ocrs` 0.13.1 / `rten` 0.27.0** are a pure Rust OCR alternative without native ONNX Runtime, but with their own models instead of PaddleOCR.
- **`image` 0.25.10 + `imageproc` 0.27.0** already include projective transformations.
- GUI options:
  - **`egui` 0.36.2**: immediate mode, tables via `egui_extras`
  - **`slint` 1.18.1**: declarative, `StandardTableView`
  - **`iced` 0.14.0** (December 2025): has had a `table` widget since 0.14 ("Display tables", cells are arbitrary widgets). Editable cells are therefore possible, but editing logic, validation and focus navigation have to be built yourself. Corrected after checking the crate source code of `iced_widget` 0.14.2; an earlier version wrongly claimed that iced has no table widget.
  - The assessments of egui and slint (extent of inline editing) do **not** come from a code check. They are to be read as an assessment; decisive for the decision is that JavaFX ships editable tables with cell editors out of the box.
- **`keyring` 4.2.0** offers cross-platform access to the OS keystore. That is an advantage over Java.

## 3. Rating (1 = poor, 5 = very good)

| Criterion | Wt. | Java | Rust | Rationale |
|---|---|---|---|---|
| K1 GUI | 5 | **5** | 2–3 | JavaFX ships `TableView` with cell editors, `ComboBox`, `Canvas`, CSS theming, accessibility and HiDPI. In Rust, editable tables with validation highlighting are considerably more custom work. |
| K2 ONNX | 5 | **5** | 4 | Java is official and stable, natives are in the JAR. Rust: `ort` is only an RC; in return there is the ready-made pipeline `oar-ocr`. |
| K3 Image processing | 3 | 3 | **4** | Java needs its own homography (~100 lines); `imageproc` has it ready-made. |
| K4 HTTP/JSON | 2 | 5 | 5 | `java.net.http` + Jackson 3 vs. `reqwest` + `serde`: equivalent |
| K5 Distribution | 3 | 3 | **5** | `jpackage` creates one package per OS with runtime (considerably larger, build per OS required). Rust: one binary plus the ONNX Runtime lib. |
| K6 Resources | 3 | 3 | **5** | The JVM needs more RAM. It can be limited via `-Xmx`, G1 and Compact Object Headers (JEP 534, default from JDK 27), but stays above Rust. **Concrete numbers must be measured; they are not estimated here.** |
| K7 Reuse | 3 | **5** | 3 | basetool runs on the JVM with the same ORT Java API: concepts (not code, see license) can be transferred directly. Taking over code requires GPL-3.0 (see below). |
| K8 Secret store | 2 | 3 | **5** | Java has no cross-platform keyring. Solution: a small custom binding via the **FFM API** to Windows Credential Manager and libsecret. Rust: `keyring` ready-made. |
| K9 Iteration | 4 | **5** | 3 | Incremental compilation, hot reload of CSS and fast UI prototyping favor Java. Rust compile times with ORT plus GUI are noticeably longer. |
| **Total (weighted)** | | **128** | **113–118** | |

Calculation Java: 25+25+9+10+9+9+15+6+20 = 128.
Calculation Rust: (10–15)+20+12+10+15+15+9+10+12 = 113–118.

The exact score is less important than the two criteria with the highest weight: **GUI (K1)** and **OCR runtime (K2)**.

## 4. Decision and honest assessment

**Chosen: Java.** The implementation is easiest in Java, because

1. the most laborious part (the review UI with editable, validated tables and image overlays) is standard with JavaFX and would be custom work in Rust;
2. the ONNX Runtime is officially stable in Java and ships the natives for both target OSes in the JAR;
3. basetool-sc-extractor already uses the technical chain (PP-OCRv6 + ORT + `BufferedImage`) on the JVM for the same game font – there, however, only as a second reader for digits; suitability as the primary reader will only be measured in M2.

**What speaks against it – deliberately accepted:**

- **Package size and RAM** are worse than with Rust. Countermeasures:
  - `jlink` with only the required modules
  - heap limit
  - load models lazily and close the ORT session after inactivity
- **JDK 27 is not an LTS.** In line with the requirement "latest version", we develop on JDK 27. Its support ends when JDK 28 ships (March 2027), so the move to JDK 28 must be **released** before the April 2027 JDK security update.
  - Because `jpackage` ships the runtime, users get JDK security fixes only through our releases: every quarterly JDK security update means a PATCH release ([release-process.md](../release-process.md) 'Security release').
  - **Alternative:** JDK 25 LTS, if less upgrade effort is desired (the quarterly rebuilds remain).
- **No preview features** (e.g. Structured Concurrency, JEP 533 – still preview in JDK 27). Production code uses only final features.
- **Toolchain risk JDK 27:** Error Prone, NullAway, google-java-format and the jlink plugin hook deeply into javac or the JDK. Whether the current versions support JDK 27 is **not checked**; that happens in M0. If one of them does not support JDK 27, the fallback is JDK 25 LTS (the decision for Java remains unaffected).

**When Rust would be the better choice:** if a small single binary and minimal memory footprint were the top priority, or if the team knows Rust considerably better than Java. The architecture (see [02](02-architecture.md)) is cut in a language-neutral way, so a later port of the OCR core would be possible.

**Addendum (optional AI recognition):** The local VLM is connected via the Ollama HTTP API (`java.net.http` + Jackson). This is equally easy in both languages and does not change the rating. Game detection uses the JDK (`ProcessHandle`, `/proc`) plus a small FFM call on Windows (Toolhelp snapshot, R-VLM-2); in Rust it would need a crate such as `sysinfo`.

## 5. License notice (consequence for reuse)

basetool-sc-extractor is licensed under **GPL-3.0-or-later**.

- **Taking over or translating code** makes our client a derivative work that must be published under GPL-3.0-compatible terms.
- **Ideas, thresholds and measurement results** may be freely re-implemented.
- The **PP-OCRv6 models** are licensed under Apache-2.0 and may be used directly.

**Open decision for the project owner:** determine the project's license (recommendation: GPL-3.0-or-later if code from basetool is to be ported; otherwise freely selectable). Until then we implement **only by concept** and do not copy any code.
