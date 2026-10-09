// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.io.IOException;
import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

/**
 * Checks WCAG 2.2 contrast for every declared pairing and the derived checks of the JavaFX handoff
 * (§7), in every theme, and writes a Markdown report.
 *
 * <p>Usage: {@code java Contrast.java --self-test}, or {@code java Contrast.java <token dir>
 * <report file> <inputs label>}. Exits with status 1 if a self-test or a check fails; nothing is
 * waived.
 */
final class Contrast {

  private static final List<String> SURFACES =
      List.of("surface-0", "surface-1", "surface-2", "surface-3", "surface-sunken");
  private static final List<String> STATE_GROUNDS =
      List.of("deviation-minor-bg", "deviation-major-bg", "no-reference-bg", "selection-row-bg", "row-hover-bg");
  private static final List<String> CONFIDENCE_BORDERS =
      List.of(
          "confidence-confirm-border",
          "confidence-select-border",
          "confidence-correct-border",
          "user-entered-border",
          "confirmed-border",
          "double-confirmed-border");
  private static final double LARGE_SURFACE_MAX_LUMINANCE = 0.04;
  private static final double PRIMARY_TEXT_MIN = 7.0;
  private static final double PRIMARY_TEXT_MAX = 16.0;

  private Contrast() {}

  /**
   * Runs the self-test or the check.
   *
   * @param args {@code --self-test}, or the token directory, the report file and an inputs label
   * @throws IOException if a file cannot be read or written
   */
  public static void main(String[] args) throws IOException {
    if (args.length == 1 && args[0].equals("--self-test")) {
      System.exit(selfTest() ? 0 : 1);
    }
    if (!selfTest()) {
      System.exit(1);
    }
    Path dir = Path.of(args[0]);
    Tokens tokens = Tokens.load(dir);
    Object pairings = Json.parse(Files.readString(dir.resolve("pairings.json"), StandardCharsets.UTF_8));
    List<Result> results = new ArrayList<>();
    for (Object pair : Json.array(Json.get(pairings, "pairs"))) {
      String theme = Json.string(Json.get(pair, "theme"));
      Object over = Json.object(pair).get("over");
      List<?> imagery = over == null ? null : Json.array(over);
      results.add(check(tokens, theme, "declared",
          Json.string(Json.get(pair, "foreground")), Json.string(Json.get(pair, "ground")), imagery,
          ((BigDecimal) Json.get(pair, "required")).doubleValue()));
    }
    for (String theme : tokens.themes) {
      double text = theme.equals("high-contrast") ? 7.0 : 4.5;
      for (String ground : concat(SURFACES, STATE_GROUNDS)) {
        results.add(check(tokens, theme, "focus ring", "focus-ring", ground, null, 3.0));
      }
      for (String border : CONFIDENCE_BORDERS) {
        for (String ground : STATE_GROUNDS) {
          results.add(check(tokens, theme, "state on tint", border, ground, null, 3.0));
        }
      }
      results.add(check(tokens, theme, "delta badge", "deviation-major-badge-fg", "deviation-major-badge-bg", null, text));
      results.add(check(tokens, theme, "fill focus", "focus-ring-on-fill", "accent", null, 3.0));
      results.add(check(tokens, theme, "fill focus", "focus-ring-on-critical", "critical-bg-strong", null, 3.0));
      for (String fg : List.of("text-primary", "text-secondary", "text-muted")) {
        for (String ground : concat(SURFACES, STATE_GROUNDS)) {
          results.add(check(tokens, theme, "text floor", fg, ground, null, text));
        }
      }
    }
    List<String> band = new ArrayList<>();
    for (String surface : SURFACES) {
      double luminance = tokens.color("dark", surface).luminance();
      band.add(String.format(Locale.ROOT, "| %s | %.4f | %s |", surface, luminance,
          luminance <= LARGE_SURFACE_MAX_LUMINANCE ? "pass" : "FAIL"));
      if (luminance > LARGE_SURFACE_MAX_LUMINANCE) {
        results.add(new Result("dark", "luminance band", surface, "-", luminance, LARGE_SURFACE_MAX_LUMINANCE, false));
      }
    }
    double primary = ratio(tokens.color("dark", "text-primary"), tokens.color("dark", "surface-0"));
    boolean primaryInBand = primary >= PRIMARY_TEXT_MIN && primary <= PRIMARY_TEXT_MAX;
    if (!primaryInBand) {
      results.add(new Result("dark", "luminance band", "text-primary", "surface-0", primary, PRIMARY_TEXT_MIN, false));
    }
    String section = report(tokens, results, band, primary, primaryInBand, args[2]);
    Path reportFile = Path.of(args[1]);
    String existing = Files.exists(reportFile) ? Files.readString(reportFile, StandardCharsets.UTF_8) : "";
    Files.writeString(reportFile, replaceSection(existing, section), StandardCharsets.UTF_8);
    long failures = results.stream().filter(r -> !r.pass()).count();
    System.out.println(results.size() + " checks, " + failures + " failure(s).");
    System.exit(failures == 0 ? 0 : 1);
  }

  /**
   * Returns the WCAG contrast ratio of two opaque colours.
   *
   * @param a the first colour
   * @param b the second colour
   * @return the ratio, 1–21
   */
  static double ratio(Tokens.Rgba a, Tokens.Rgba b) {
    double la = a.luminance();
    double lb = b.luminance();
    return (Math.max(la, lb) + 0.05) / (Math.min(la, lb) + 0.05);
  }

  private static boolean selfTest() {
    double black = ratio(new Tokens.Rgba(0, 0, 0, 1), new Tokens.Rgba(255, 255, 255, 1));
    double grey = ratio(new Tokens.Rgba(0x77, 0x77, 0x77, 1), new Tokens.Rgba(255, 255, 255, 1));
    Tokens.Rgba half = new Tokens.Rgba(0, 0, 0, 0.5).over(new Tokens.Rgba(255, 255, 255, 1));
    boolean ok = Math.abs(black - 21.0) < 0.001 && Math.abs(grey - 4.48) < 0.005 && half.r() == 128;
    System.out.println(String.format(Locale.ROOT,
        "Self-test %s: #000 on #fff = %.3f (expected 21.0); #777 on #fff = %.3f (expected about 4.48); 50%% black over white = %d (expected 128).",
        ok ? "passed" : "FAILED", black, grey, half.r()));
    return ok;
  }

  private static Result check(Tokens tokens, String theme, String kind, String fg, String ground, List<?> imagery, double required) {
    Tokens.Rgba g = tokens.color(theme, ground);
    List<Tokens.Rgba> grounds = new ArrayList<>();
    if (imagery != null) {
      Tokens.Rgba image = new Tokens.Rgba(
          ((BigDecimal) imagery.get(0)).intValue(), ((BigDecimal) imagery.get(1)).intValue(), ((BigDecimal) imagery.get(2)).intValue(), 1.0);
      grounds.add(g.over(image));
    } else if (g.alpha() < 1.0) {
      for (String surface : SURFACES) {
        grounds.add(g.over(tokens.color(theme, surface)));
      }
    } else {
      grounds.add(g);
    }
    Tokens.Rgba f = tokens.color(theme, fg);
    double worst = Double.MAX_VALUE;
    for (Tokens.Rgba opaqueGround : grounds) {
      Tokens.Rgba opaqueFg = f.alpha() < 1.0 ? f.over(opaqueGround) : f;
      worst = Math.min(worst, ratio(opaqueFg, opaqueGround));
    }
    String groundLabel = imagery == null ? ground : ground + " over (" + imagery.get(0) + ", " + imagery.get(1) + ", " + imagery.get(2) + ")";
    return new Result(theme, kind, fg, groundLabel, worst, required, worst + 1e-9 >= required);
  }

  private static String report(Tokens tokens, List<Result> results, List<String> band, double primary, boolean primaryInBand, String inputs) {
    StringBuilder out = new StringBuilder();
    out.append("### Contrast (generated)\n\n");
    out.append("Generated by `tools/design-system/Contrast.java` from `ui/design-tokens/` (").append(inputs).append("). ");
    out.append("WCAG 2.2 relative luminance; translucent colours are composited over each ground, and a translucent ground over every surface. ");
    out.append("A ratio is shown rounded down to two decimals; the pass/fail decision uses the exact value.\n\n");
    out.append("| Theme | Checks | Failures | Closest to its floor |\n|---|---|---|---|\n");
    for (String theme : tokens.themes) {
      List<Result> mine = results.stream().filter(r -> r.theme().equals(theme)).toList();
      Result lowest = mine.stream().min((a, b) -> Double.compare(a.ratio() - a.required(), b.ratio() - b.required())).orElseThrow();
      out.append("| ").append(theme).append(" | ").append(mine.size()).append(" | ")
          .append(mine.stream().filter(r -> !r.pass()).count()).append(" | ")
          .append(String.format(Locale.ROOT, "%s on %s: %s (needs %s) |", lowest.fg(), lowest.ground(), floor2(lowest.ratio()), floor2(lowest.required())))
          .append('\n');
    }
    out.append("\n**Luminance band (dark theme):** large surfaces at or below ").append(LARGE_SURFACE_MAX_LUMINANCE)
        .append(String.format(Locale.ROOT, "; `text-primary` on `surface-0` = %s (band %s–%s: %s).\n\n",
            floor2(primary), floor2(PRIMARY_TEXT_MIN), floor2(PRIMARY_TEXT_MAX), primaryInBand ? "pass" : "FAIL"));
    out.append("| Surface | Relative luminance | Result |\n|---|---|---|\n");
    band.forEach(line -> out.append(line).append('\n'));
    List<Result> failed = results.stream().filter(r -> !r.pass()).toList();
    out.append("\n**Failures:** ").append(failed.isEmpty() ? "none." : failed.size() + "").append("\n\n");
    if (!failed.isEmpty()) {
      out.append("| Theme | Check | Foreground | Ground | Ratio | Required |\n|---|---|---|---|---|---|\n");
      failed.forEach(r -> out.append(row(r)));
      out.append('\n');
    }
    out.append("<details><summary>All ").append(results.size()).append(" checks</summary>\n\n");
    out.append("| Theme | Check | Foreground | Ground | Ratio | Required |\n|---|---|---|---|---|---|\n");
    results.forEach(r -> out.append(row(r)));
    out.append("\n</details>\n");
    return out.toString();
  }

  private static String replaceSection(String document, String section) {
    String heading = "### Contrast (generated)";
    int start = document.indexOf(heading);
    if (start < 0) {
      return document.isEmpty() ? section : document.stripTrailing() + "\n\n" + section;
    }
    int end = document.indexOf("\n### ", start + heading.length());
    int endHeading = document.indexOf("\n## ", start + heading.length());
    if (end < 0 || endHeading >= 0 && endHeading < end) {
      end = endHeading;
    }
    String tail = end < 0 ? "" : "\n" + document.substring(end + 1);
    return document.substring(0, start) + section + tail;
  }

  private static String row(Result r) {
    return "| " + r.theme() + " | " + r.kind() + " | `" + r.fg() + "` | `" + r.ground() + "` | " + floor2(r.ratio()) + " | " + floor2(r.required()) + (r.pass() ? "" : " (FAIL)") + " |\n";
  }

  private static String floor2(double value) {
    return String.format(Locale.ROOT, "%.2f", Math.floor(value * 100) / 100);
  }

  private static List<String> concat(List<String> a, List<String> b) {
    List<String> all = new ArrayList<>(a);
    all.addAll(b);
    return all;
  }

  private record Result(String theme, String kind, String fg, String ground, double ratio, double required, boolean pass) {}
}
