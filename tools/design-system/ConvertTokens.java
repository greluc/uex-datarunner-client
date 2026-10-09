// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.io.IOException;
import java.math.BigDecimal;
import java.math.RoundingMode;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Converts a Claude Design system export into the repository token files.
 *
 * <p>Usage: {@code java ConvertTokens.java <export project dir> <output dir> <source label>}. The
 * input is the system's {@code tokens.json}, {@code accessibility-pairings.md} and {@code
 * state-language.md}; the output is a DTCG 2025.10 resolver document with one base set and one
 * set per theme, plus {@code pairings.json} and {@code states.json}.
 */
final class ConvertTokens {

  private static final String EXT = "space.uexdatarunner";
  private static final Pattern HEX = Pattern.compile("^#([0-9a-fA-F]{6})$");
  private static final Pattern RGBA =
      Pattern.compile("^rgba?\\(\\s*(\\d+)\\s*,\\s*(\\d+)\\s*,\\s*(\\d+)\\s*(?:,\\s*([0-9.]+)\\s*)?\\)$");
  private static final Pattern ALIAS = Pattern.compile("^\\{([a-z0-9-]+)\\}$");
  private static final Pattern OVER =
      Pattern.compile("^([a-z0-9-]+) over imagery \\((\\d+), (\\d+), (\\d+)\\)$");
  private static final Pattern ID_CELL = Pattern.compile("^`([a-z0-9-]+)`(?:\\s*\\((.*)\\))?$");
  private static final Pattern PX = Pattern.compile("^(-?\\d+(?:\\.\\d+)?)px$");
  private static final Pattern SHADOW =
      Pattern.compile("^(-?\\d+)(?:px)? (-?\\d+)(?:px)? (\\d+)(?:px)? (\\d+)(?:px)? (rgba?\\([^)]*\\)|#[0-9a-fA-F]{6})$");

  private ConvertTokens() {}

  /**
   * Runs the conversion.
   *
   * @param args export project directory, output directory, source label
   * @throws IOException if a file cannot be read or written
   */
  public static void main(String[] args) throws IOException {
    if (args.length != 3) {
      throw new IllegalArgumentException("usage: ConvertTokens <export dir> <output dir> <source label>");
    }
    Path in = Path.of(args[0]);
    Path out = Path.of(args[1]);
    String source = args[2];
    Object system = Json.parse(Files.readString(in.resolve("tokens.json"), StandardCharsets.UTF_8));
    Files.createDirectories(out);

    List<String> themes = new ArrayList<>();
    for (Object theme : Json.array(Json.get(Json.get(system, "color"), "themes"))) {
      themes.add(Json.string(Json.get(theme, "id")));
    }

    Map<String, Object> resolver = new LinkedHashMap<>();
    resolver.put("$schema", "https://www.designtokens.org/schemas/2025.10/resolver.json");
    resolver.put("version", "2025.10");
    resolver.put("name", "Tallyline");
    resolver.put("description", "Design tokens of UEX Datarunner Client, converted from " + source + ".");
    resolver.put("sets", Map.of("base", Map.of("sources", List.of(Map.of("$ref", "base.tokens.json")))));
    Map<String, Object> contexts = new LinkedHashMap<>();
    for (String theme : themes) {
      contexts.put(theme, List.of(Map.of("$ref", "theme-" + theme + ".tokens.json")));
    }
    Map<String, Object> themeModifier = new LinkedHashMap<>();
    themeModifier.put("description", "Colour theme");
    themeModifier.put("contexts", contexts);
    themeModifier.put("default", themes.getFirst());
    resolver.put("modifiers", Map.of("theme", themeModifier));
    resolver.put("resolutionOrder", List.of(Map.of("$ref", "#/sets/base"), Map.of("$ref", "#/modifiers/theme")));
    write(out.resolve("tokens.resolver.json"), resolver);

    write(out.resolve("base.tokens.json"), base(system, source));
    for (String theme : themes) {
      write(out.resolve("theme-" + theme + ".tokens.json"), theme(system, theme, source));
    }
    write(out.resolve("pairings.json"), pairings(Files.readString(in.resolve("accessibility-pairings.md"), StandardCharsets.UTF_8), source));
    write(out.resolve("states.json"), states(Files.readString(in.resolve("state-language.md"), StandardCharsets.UTF_8), source));
  }

  private static Map<String, Object> base(Object system, String source) {
    Map<String, Object> root = new LinkedHashMap<>();
    root.put("$description", "Theme-independent tokens, converted from " + source + "; lengths are px at the 14 px base.");
    Object type = Json.get(system, "type");

    Map<String, Object> faces = new LinkedHashMap<>();
    faces.put("$type", "fontFamily");
    for (Object font : Json.array(Json.get(type, "fonts"))) {
      String file = Json.string(Json.get(font, "file"));
      String name = file.substring(file.lastIndexOf('/') + 1, file.lastIndexOf('.')).toLowerCase(Locale.ROOT);
      Map<String, Object> token = new LinkedHashMap<>();
      token.put("$value", Json.string(Json.get(font, "family")));
      Map<String, Object> ext = new LinkedHashMap<>();
      ext.put("file", file);
      ext.put("weight", new BigDecimal(Json.string(Json.get(font, "weight"))));
      ext.put("style", Json.string(Json.get(font, "style")));
      token.put("$extensions", Map.of(EXT, ext));
      faces.put(name, token);
    }
    root.put("font-face", faces);

    Map<String, Object> families = new LinkedHashMap<>();
    families.put("$type", "fontFamily");
    for (Map.Entry<?, ?> family : Json.object(Json.get(type, "families")).entrySet()) {
      String first = Json.string(family.getValue()).split(",")[0].trim().replace("\"", "");
      families.put(String.valueOf(family.getKey()), Map.of("$value", first));
    }
    root.put("font-family", families);

    Map<String, Object> styles = new LinkedHashMap<>();
    styles.put("$type", "typography");
    for (Object group : Json.array(Json.get(type, "groups"))) {
      String family = Json.string(Json.get(group, "family"));
      for (Object style : Json.array(Json.get(group, "styles"))) {
        BigDecimal size = px(Json.string(Json.get(style, "fontSize")));
        BigDecimal line = px(Json.string(Json.get(style, "lineHeight")));
        Map<String, Object> value = new LinkedHashMap<>();
        value.put("fontFamily", "{font-family." + family + "}");
        value.put("fontSize", dimension(size));
        value.put("fontWeight", Json.get(style, "fontWeight"));
        value.put("letterSpacing", dimension(BigDecimal.ZERO));
        value.put("lineHeight", line.divide(size, 4, RoundingMode.HALF_EVEN));
        Map<String, Object> token = new LinkedHashMap<>();
        token.put("$value", value);
        token.put("$description", Json.string(Json.get(style, "usage")));
        Map<String, Object> ext = new LinkedHashMap<>();
        ext.put("group", Json.string(Json.get(group, "name")));
        ext.put("lineHeightPx", line);
        token.put("$extensions", Map.of(EXT, ext));
        styles.put(Json.string(Json.get(style, "name")), token);
      }
    }
    root.put("typography", styles);

    for (String family : List.of("spacing", "radius", "width", "size")) {
      Map<String, Object> group = new LinkedHashMap<>();
      group.put("$type", "dimension");
      for (Object token : Json.array(Json.get(Json.get(system, family), "tokens"))) {
        Map<String, Object> t = new LinkedHashMap<>();
        t.put("$value", dimension(px(Json.string(Json.get(token, "value")))));
        t.put("$description", Json.string(Json.get(token, "usage")));
        group.put(Json.string(Json.get(token, "name")), t);
      }
      root.put(family, group);
    }

    Map<String, Object> motion = new LinkedHashMap<>();
    motion.put("$description", "From the system README, section Motion; the system's tokens.json has no motion family.");
    motion.put("duration-hover", ordered("$type", "duration", "$value", ordered("value", BigDecimal.valueOf(120), "unit", "ms"), "$description", "Hover and pressed colour transitions."));
    motion.put("duration-select", ordered("$type", "duration", "$value", ordered("value", BigDecimal.valueOf(160), "unit", "ms"), "$description", "Selection and panel-open colour transitions."));
    motion.put("easing-standard", ordered("$type", "cubicBezier", "$value", List.of(BigDecimal.ZERO, BigDecimal.ZERO, new BigDecimal("0.58"), BigDecimal.ONE), "$description", "CSS ease-out."));
    root.put("motion", motion);
    return root;
  }

  private static Map<String, Object> theme(Object system, String theme, String source) {
    Map<String, Object> root = new LinkedHashMap<>();
    root.put("$description", "Colour and shadow tokens of the " + theme + " theme, converted from " + source + ".");
    Map<String, Object> color = new LinkedHashMap<>();
    color.put("$type", "color");
    for (Object token : Json.array(Json.get(Json.get(system, "color"), "tokens"))) {
      String raw = Json.string(Json.get(Json.get(token, "value"), theme)).trim();
      Map<String, Object> t = new LinkedHashMap<>();
      Matcher alias = ALIAS.matcher(raw);
      t.put("$value", alias.matches() ? "{color." + alias.group(1) + "}" : color(raw));
      t.put("$description", Json.string(Json.get(token, "usage")));
      color.put(Json.string(Json.get(token, "name")), t);
    }
    root.put("color", color);

    Map<String, Object> shadow = new LinkedHashMap<>();
    shadow.put("$type", "shadow");
    for (Object token : Json.array(Json.get(Json.get(system, "shadow"), "tokens"))) {
      String raw = Json.string(Json.get(Json.get(token, "value"), theme)).trim();
      Map<String, Object> t = new LinkedHashMap<>();
      if (raw.equals("none")) {
        Map<String, Object> value = new LinkedHashMap<>();
        value.put("color", color("rgba(0, 0, 0, 0)"));
        value.put("offsetX", dimension(BigDecimal.ZERO));
        value.put("offsetY", dimension(BigDecimal.ZERO));
        value.put("blur", dimension(BigDecimal.ZERO));
        value.put("spread", dimension(BigDecimal.ZERO));
        t.put("$value", value);
        t.put("$extensions", Map.of(EXT, Map.of("none", Boolean.TRUE)));
      } else {
        Matcher m = SHADOW.matcher(raw);
        if (!m.matches()) {
          throw new IllegalArgumentException("unsupported shadow " + raw);
        }
        Map<String, Object> value = new LinkedHashMap<>();
        value.put("color", color(m.group(5)));
        value.put("offsetX", dimension(new BigDecimal(m.group(1))));
        value.put("offsetY", dimension(new BigDecimal(m.group(2))));
        value.put("blur", dimension(new BigDecimal(m.group(3))));
        value.put("spread", dimension(new BigDecimal(m.group(4))));
        t.put("$value", value);
      }
      t.put("$description", Json.string(Json.get(token, "usage")));
      shadow.put(Json.string(Json.get(token, "name")), t);
    }
    root.put("shadow", shadow);
    return root;
  }

  private static Map<String, Object> pairings(String markdown, String source) {
    Map<String, Object> root = new LinkedHashMap<>();
    root.put("source", source);
    List<Object> cooccurrence = new ArrayList<>();
    List<Object> pairs = new ArrayList<>();
    String section = "";
    for (String line : markdown.split("\n")) {
      if (line.startsWith("## ")) {
        section = line.substring(3).trim();
        continue;
      }
      if (section.startsWith("Co-occurrence") && line.startsWith("- ")) {
        cooccurrence.add(line.substring(2).trim());
      }
      if (section.startsWith("Declared") && line.startsWith("| `")) {
        String[] cells = line.split("\\|");
        Map<String, Object> pair = new LinkedHashMap<>();
        pair.put("foreground", cells[1].trim().replace("`", ""));
        String ground = cells[2].trim().replace("`", "");
        Matcher over = OVER.matcher(ground);
        if (over.matches()) {
          pair.put("ground", over.group(1));
          pair.put("over", List.of(new BigDecimal(over.group(2)), new BigDecimal(over.group(3)), new BigDecimal(over.group(4))));
        } else {
          pair.put("ground", ground);
        }
        pair.put("theme", cells[3].trim());
        pair.put("required", new BigDecimal(cells[4].trim().replace(":1", "")));
        pairs.add(pair);
      }
    }
    if (pairs.isEmpty()) {
      throw new IllegalArgumentException("no declared pairs found");
    }
    root.put("cooccurrence", cooccurrence);
    root.put("pairs", pairs);
    return root;
  }

  private static Map<String, Object> states(String markdown, String source) {
    Map<String, Object> root = new LinkedHashMap<>();
    root.put("source", source);
    List<Object> states = new ArrayList<>();
    String group = "";
    List<String> header = List.of();
    for (String line : markdown.split("\n")) {
      if (line.startsWith("#")) {
        group = line.replaceFirst("^#+ ", "").trim();
        header = List.of();
        continue;
      }
      if (group.equals("Generic feedback") && line.startsWith("`")) {
        listedIds(group, line, states);
        continue;
      }
      if (!line.startsWith("|")) {
        continue;
      }
      List<String> cells = new ArrayList<>();
      for (String cell : line.substring(1, line.length() - (line.endsWith("|") ? 1 : 0)).split("\\|", -1)) {
        cells.add(cell.trim());
      }
      if (cells.getFirst().equalsIgnoreCase("State id")
          || cells.getFirst().equalsIgnoreCase("Id")
          || cells.getFirst().equalsIgnoreCase("Group")) {
        header = cells.stream().map(c -> c.toLowerCase(Locale.ROOT)).toList();
        continue;
      }
      if (header.size() > 1 && header.get(1).startsWith("state ids")) {
        if (!cells.getFirst().startsWith("---")) {
          listedIds(group + " / " + cells.getFirst(), cells.get(1), states);
        }
        continue;
      }
      if (header.isEmpty() || cells.getFirst().startsWith("---") || !cells.getFirst().startsWith("`")) {
        continue;
      }
      Map<String, Object> state = new LinkedHashMap<>();
      state.put("group", group);
      for (int i = 0; i < Math.min(header.size(), cells.size()); i++) {
        String key = header.get(i).equals("state id") ? "id" : header.get(i).replace(' ', '-');
        String value = cells.get(i);
        if (key.equals("id")) {
          Matcher id = ID_CELL.matcher(value);
          if (!id.matches()) {
            throw new IllegalArgumentException("unexpected state id cell " + value);
          }
          state.put("id", id.group(1));
          if (id.group(2) != null) {
            state.put("qualifier", id.group(2));
          }
        } else {
          state.put(key, value);
        }
      }
      states.add(state);
    }
    if (states.isEmpty()) {
      throw new IllegalArgumentException("no states found");
    }
    root.put("states", mergeSummaries(states));
    return root;
  }

  private static List<Object> mergeSummaries(List<Object> states) {
    Map<String, Map<?, ?>> full = new LinkedHashMap<>();
    for (Object state : states) {
      Map<?, ?> s = Json.object(state);
      if (s.containsKey("icon")) {
        if (full.put(Json.string(s.get("id")), s) != null) {
          throw new IllegalArgumentException("state " + s.get("id") + " is defined twice");
        }
      }
    }
    Map<String, Object> summaryGroups = new LinkedHashMap<>();
    for (Object state : states) {
      Map<?, ?> s = Json.object(state);
      String id = Json.string(s.get("id"));
      if (!s.containsKey("icon") && full.containsKey(id)) {
        summaryGroups.put(id, s.get("group"));
      }
    }
    List<Object> merged = new ArrayList<>();
    for (Object state : states) {
      Map<?, ?> s = Json.object(state);
      String id = Json.string(s.get("id"));
      if (!s.containsKey("icon") && full.containsKey(id)) {
        continue;
      }
      if (summaryGroups.containsKey(id)) {
        Map<String, Object> copy = new LinkedHashMap<>();
        s.forEach((k, v) -> copy.put(String.valueOf(k), v));
        copy.put("summary-group", summaryGroups.get(id));
        merged.add(copy);
      } else {
        merged.add(state);
      }
    }
    return merged;
  }

  private static void listedIds(String group, String cell, List<Object> states) {
    Matcher m = Pattern.compile("`([a-z][a-z0-9-]*)`(?:\\s*\\(([^)]*)\\))?").matcher(cell);
    while (m.find()) {
      if (m.group(1).contains("_") || m.start() > 0 && cell.charAt(m.start() - 1) == '(') {
        continue;
      }
      Map<String, Object> state = new LinkedHashMap<>();
      state.put("group", group);
      state.put("id", m.group(1));
      if (m.group(2) != null && !m.group(2).startsWith("`")) {
        state.put("label", m.group(2).trim());
      }
      states.add(state);
    }
  }

  private static Map<String, Object> ordered(Object... keysAndValues) {
    Map<String, Object> map = new LinkedHashMap<>();
    for (int i = 0; i < keysAndValues.length; i += 2) {
      map.put((String) keysAndValues[i], keysAndValues[i + 1]);
    }
    return map;
  }

  private static Map<String, Object> color(String raw) {
    int r;
    int g;
    int b;
    BigDecimal alpha = BigDecimal.ONE;
    Matcher hex = HEX.matcher(raw);
    Matcher rgba = RGBA.matcher(raw);
    if (hex.matches()) {
      int v = Integer.parseInt(hex.group(1), 16);
      r = v >> 16 & 0xff;
      g = v >> 8 & 0xff;
      b = v & 0xff;
    } else if (rgba.matches()) {
      r = Integer.parseInt(rgba.group(1));
      g = Integer.parseInt(rgba.group(2));
      b = Integer.parseInt(rgba.group(3));
      if (rgba.group(4) != null) {
        alpha = new BigDecimal(rgba.group(4));
      }
    } else {
      throw new IllegalArgumentException("unsupported colour " + raw);
    }
    Map<String, Object> value = new LinkedHashMap<>();
    value.put("colorSpace", "srgb");
    value.put("components", List.of(unit(r), unit(g), unit(b)));
    value.put("alpha", alpha);
    value.put("hex", String.format("#%02x%02x%02x", r, g, b));
    return value;
  }

  private static BigDecimal unit(int channel) {
    return BigDecimal.valueOf(channel).divide(BigDecimal.valueOf(255), 6, RoundingMode.HALF_EVEN);
  }

  private static BigDecimal px(String raw) {
    Matcher m = PX.matcher(raw.trim());
    if (!m.matches()) {
      throw new IllegalArgumentException("not a px length: " + raw);
    }
    return new BigDecimal(m.group(1));
  }

  private static Map<String, Object> dimension(BigDecimal value) {
    Map<String, Object> d = new LinkedHashMap<>();
    d.put("value", value);
    d.put("unit", "px");
    return d;
  }

  private static void write(Path file, Object value) throws IOException {
    Files.writeString(file, Json.write(value), StandardCharsets.UTF_8);
  }
}
