# Review-table prototype (issue #22)

Spike code for the M0 review-table prototype and the icon rendering check ([04](../../docs/plan/04-roadmap.md) M0; ADR-0005 D3, D5; R-UI-3, R-UI-4, R-UI-10; O-107, O-112, O-113). It lives on the branch `prototype/review-table` and never merges into `main`; only the result is carried into the documents that own the facts.

## What it is

A standalone crate (its own `[workspace]` and `Cargo.lock`) on `egui`/`eframe` 0.36.2 with the wgpu backend and `egui_extras` 0.36.2:

- 200 rows of deterministic sample data covering every recognition-confidence state and every deviation level;
- a virtualised `egui_extras::TableBuilder` table with our own cell focus, painted cells and in-cell editors;
- in-cell editing: a single-line text editor (price, SCU, container sizes) and a searchable combo box (commodity, status) with prefix, word-start and substring ranking;
- keyboard navigation: Tab / Shift+Tab, arrows, Enter or F2 to edit, Enter to commit, Esc to cancel, Space to confirm, F8 / Shift+F8 to the next / previous field that needs attention; the table scrolls the target row into view;
- the Tallyline cell states from `ui/design-tokens/` (dark theme, design-system version 1791542897-a5d3): recognition confidence as icon and border, deviation as background colour, icon and Δ badge (outlined for minor, filled for major, dashed border for no reference), and every state as text in the status line and the tooltip;
- a scripted measurement mode and two PowerShell scripts.

**Deviation from the issue, decided by @greluc on 2026-10-14:** the `tallyline-egui` crate is not in the repository yet (O-112), so the widgets are our own on plain egui with the token values, and the icons are stand-in glyphs from egui's default fonts (NotoEmoji, emoji-icon-font). The check of the real Tallyline icon font stays open until the crate is imported.

## Commands

```powershell
cargo test --locked --manifest-path prototype/review-table/Cargo.toml
cargo test --locked --manifest-path prototype/review-table/Cargo.toml --features wgpu-tests --test icons
cargo build --release --locked --manifest-path prototype/review-table/Cargo.toml
prototype/review-table/target/release/review-table-spike --scale 125
prototype/review-table/scripts/measure-frame-time.ps1
prototype/review-table/scripts/measure-footprint.ps1 -Label "<3D application running>"
```

`--scale` is the effective display scale (physical pixels per point × 100); the app reaches it through egui zoom whatever the OS scale of the monitor. The `wgpu-tests` feature writes the icon sheets (`icons-<scale>.png`, magnified `-x8`) and table renders to `target/icon-check/`.

## Tests

| File | What it pins |
|---|---|
| `tests/logic.rs` | sample data covers every state; Tab order and arrow moves stop at the edges; F8 / Shift+F8 find the next / previous problem and wrap; only a user confirmation settles a deviation; filter ranking; confidence draws icon and border only, deviation draws background, icon and badge; the two dimensions are independent; every marked cell has a text reason |
| `tests/table.rs` | through `egui_kittest`: Tab, Shift+Tab, arrows, Enter-type-Enter commits as "entered by you", Esc discards, Tab commits and moves, 200 F8 jumps each scroll the row into the drawn range, the combo box filters as you type and Enter / arrows pick, an untouched combo box never picks a candidate (no silent preselection), Space confirms, only the visible rows are drawn |
| `tests/measure.rs` | the frame plan and the nearest-rank statistics |
| `tests/icons.rs` | no stand-in icon falls back to the missing-glyph box (and the probe sees a known gap); every icon and every table text quad sits on the physical pixel grid at 100, 125 and 150 %; with `wgpu-tests` the renders for the visual check |

## Results (2026-10-14, Windows 11, monitor at 150 % OS scale and 144 Hz)

### Frame time

Release build, window 1280 × 800 logical points at the OS scale (1920 × 1200 physical pixels in every run). The scales are reached through egui zoom on top of the monitor's 150 %, so the OS DPI path (winit scale factor, window sizing) ran at 150 % only; scripted run of 600 scroll frames, 300 F8 frames and 300 combo-box frames per scale. CPU time per frame is eframe's `cpu_usage` (UI pass plus rendering, without the vsync wait).

| Scale | Phase | Median ms | p95 ms | Max ms | Frame interval median ms |
|---|---|---|---|---|---|
| 100 % | scroll / F8 / combo box | 1.43 / 1.12 / 1.17 | 1.94 / 1.61 / 1.98 | 3.41 / 2.15 / 7.06 | 6.93 |
| 125 % | scroll / F8 / combo box | 0.99 / 0.95 / 0.88 | 1.37 / 1.23 / 1.25 | 1.79 / 1.59 / 1.89 | 6.94 |
| 150 % | scroll / F8 / combo box | 0.73 / 0.71 / 0.73 | 0.95 / 0.96 / 1.14 | 1.26 / 1.20 / 1.52 | 6.95 |
| 200 % | scroll / F8 / combo box | 1.18 / 1.32 / 0.83 | 2.11 / 2.29 / 1.47 | 7.26 / 4.13 / 2.09 | 6.92 |
| 250 % | scroll / F8 / combo box | 0.64 / 0.54 / 0.60 | 1.07 / 0.92 / 1.19 | 3.11 / 2.36 / 2.14 | 6.93 |

This is the run on the committed code; the machine was in use during it (see idle below), and two earlier runs on the code before the review fixes gave medians of 0.4 to 1.0 ms and a single worst frame of 9.8 ms. The frame-interval median stayed at the 144 Hz vsync interval in every run, the p95 at most 9.6 ms. CPU time per frame stays below one vsync interval at every scale with a wide margin; it falls with scale mainly because fewer rows fit the same physical window.

### Idle CPU

After the scripted run the window stays idle for 30 s; the script samples the process CPU time over 24 s of that window (Windows reports it in 15.6 ms steps).

| Scale | Idle CPU, % of one core | Frames drawn while idle |
|---|---|---|
| 100 % | 8.39 | 867 (307 with input) |
| 125 % | 0.20 | 8 (0 with input) |
| 150 % | 0.26 | 8 (0 with input) |
| 200 % | 0.33 | 8 (0 with input) |
| 250 % | 0.78 | 71 (60 with input) |

Without input the window draws 8 frames in the idle window: the settle frames after the last state change and the closing frame; there is no continuous repaint (02 §7). Input reached the window at 100 % and 250 % in this run (the machine was in use), and the frames without new events after it are egui's follow-up frames (tooltip delay and fade after the pointer stops). Earlier runs without input gave 0.07 to 0.39 % of one core at 100 % and 250 % with 8 and 9 frames. A clean measurement needs a machine nobody touches during the run.

### wgpu footprint (script check only)

One run of `measure-footprint.ps1` with no 3D application running, at 100 %: binary 13.0 MB (release, unstripped), first frame 0.84 s after the process start (the marker is written at the start of the first UI pass, just before that frame is presented), idle CPU 0 %, working set 200 MB, private memory 557 MB, GPU dedicated memory 289 MB, GPU shared memory 46 MB, GPU engine load 0 %. The measurement next to a running 3D application is issue #23 and needs a tester with such an application; the script reads the non-localised WMI GPU performance classes, so it works on a non-English Windows. On Linux it reports memory and CPU only.

### Icon rendering check (stand-in glyphs)

- **Pixel grid:** every glyph quad of the icon sheet and of the whole table lands on the physical pixel grid at 100, 125 and 150 % (`tests/icons.rs`; epaint rounds text rows to pixels and places glyphs with sub-pixel binning).
- **Crispness, seen in the magnified renders:** at 100 % straight strokes of one font unit are one crisp pixel. At 125 % and 150 % some vertical strokes of outline icons (the box of `☑`, the edges of `🆕`) fall on two pixel columns with partial coverage and look softer; fills and diagonal shapes look the same at every scale. Fine detail inside a 16 pt glyph (the lettering of `🆕`) is soft at every scale. egui does no hinting, so this depends on the font's outlines: the Tallyline icons with their 1.0 px strokes (O-107) must be checked the same way once the crate is imported; the test and the render harness are ready for it.
- **Missing glyphs:** `⑂`, `✎` and `⇄` are not in egui's default fonts and render as the missing-glyph box; they were replaced. `Fonts::has_glyph` of epaint 0.36.2 reported `✔`, `✏`, `☑` and `⚠` as missing although they render, so it cannot be used as the check; the test compares the glyph's atlas rectangle with that of the missing-glyph box instead.

## Verdict

**The toolkit choice holds.** Every acceptance item of #22 that can be done without the `tallyline-egui` crate is met on egui 0.36.2: 200 rows, in-cell editing, a searchable combo box, the full keyboard navigation and both cell-state dimensions work, the frame CPU time stays far below one vsync interval at 100 to 250 %, and the idle window does not repaint continuously. No result reopens ADR-0005 D3.

Open, and not a failure of the toolkit:

- the crispness check of the real Tallyline icon font (O-112 import, then rerun `tests/icons.rs` with the `wgpu-tests` feature);
- the wgpu footprint next to a running 3D application (#23, with `scripts/measure-footprint.ps1`); the 289 MB of dedicated GPU memory while idle is worth a look there;
- Linux was not measured; the crate builds with the `wayland` and `x11` features but has only run on Windows;
- the result still has to be recorded as dated sections in 04 (M0 items), O-107, O-112 and O-113 on `main`;
- what the prototype showed we must build ourselves: an elided cell text (long status values are clipped at the badge), the deviation recalculation after an edit, the detail strip, and a keyboard model that takes Tab and the arrows away from egui's own focus traversal (`Memory::move_focus(FocusDirection::None)` while no editor is open, `EventFilter` on the editor).
