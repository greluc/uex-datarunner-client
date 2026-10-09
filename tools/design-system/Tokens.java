// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.io.IOException;
import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * Loads the repository token files through their DTCG 2025.10 resolver document.
 *
 * <p>Only the subset of the resolver format that the repository uses is supported: one base set
 * and one {@code theme} modifier whose contexts each reference one file.
 */
final class Tokens {

  private static final Pattern ALIAS = Pattern.compile("^\\{([a-z0-9.-]+)\\}$");

  /** The theme ids in resolver order. */
  final List<String> themes;

  private final Object base;
  private final Map<String, Object> themeFiles;

  private Tokens(List<String> themes, Object base, Map<String, Object> themeFiles) {
    this.themes = themes;
    this.base = base;
    this.themeFiles = themeFiles;
  }

  /**
   * Loads the token files.
   *
   * @param dir the directory holding {@code tokens.resolver.json}
   * @return the loaded tokens
   * @throws IOException if a file cannot be read
   */
  static Tokens load(Path dir) throws IOException {
    Object resolver = read(dir.resolve("tokens.resolver.json"));
    if (!"2025.10".equals(Json.get(resolver, "version"))) {
      throw new IllegalArgumentException("resolver version must be 2025.10");
    }
    Object baseRef = Json.get(Json.array(Json.get(Json.get(Json.get(resolver, "sets"), "base"), "sources")).getFirst(), "$ref");
    Object base = read(dir.resolve(Json.string(baseRef)));
    Map<?, ?> contexts = Json.object(Json.get(Json.get(Json.get(resolver, "modifiers"), "theme"), "contexts"));
    List<String> themes = new ArrayList<>();
    Map<String, Object> files = new LinkedHashMap<>();
    for (Map.Entry<?, ?> context : contexts.entrySet()) {
      String theme = String.valueOf(context.getKey());
      List<?> sources = Json.array(context.getValue());
      if (sources.size() != 1) {
        throw new IllegalArgumentException("theme " + theme + " must reference exactly one file");
      }
      themes.add(theme);
      files.put(theme, read(dir.resolve(Json.string(Json.get(sources.getFirst(), "$ref")))));
    }
    return new Tokens(List.copyOf(themes), base, files);
  }

  /**
   * Returns the tokens of one group of a theme file, with their raw values.
   *
   * @param theme the theme id
   * @param group the group name, for example {@code color}
   * @return token name to token object, without {@code $}-prefixed group properties
   */
  Map<String, Object> themeGroup(String theme, String group) {
    return tokensOf(Json.get(themeFiles.get(theme), group));
  }

  /**
   * Returns the tokens of one group of the base file.
   *
   * @param group the group name
   * @return token name to token object
   */
  Map<String, Object> baseGroup(String group) {
    return tokensOf(Json.get(base, group));
  }

  /**
   * Returns the base file's group names.
   *
   * @return the group names, without {@code $}-prefixed members
   */
  List<String> baseGroups() {
    return Json.object(base).keySet().stream().map(String::valueOf).filter(k -> !k.startsWith("$")).toList();
  }

  /**
   * Resolves a colour token of a theme to straight sRGB channels and alpha.
   *
   * @param theme the theme id
   * @param name the colour token name
   * @return the resolved colour
   * @throws IllegalArgumentException on an unknown token, an alias cycle or a malformed value
   */
  Rgba color(String theme, String name) {
    return color(theme, name, new ArrayList<>());
  }

  private Rgba color(String theme, String name, List<String> seen) {
    if (seen.contains(name)) {
      seen.add(name);
      throw new IllegalArgumentException("alias cycle " + String.join(" -> ", seen));
    }
    seen.add(name);
    Map<String, Object> colors = themeGroup(theme, "color");
    if (!colors.containsKey(name)) {
      throw new IllegalArgumentException("unknown colour " + name + " in " + theme);
    }
    Object value = Json.get(colors.get(name), "$value");
    if (value instanceof String alias) {
      Matcher m = ALIAS.matcher(alias);
      if (!m.matches() || !m.group(1).startsWith("color.")) {
        throw new IllegalArgumentException("bad alias " + alias + " in " + name);
      }
      return color(theme, m.group(1).substring("color.".length()), seen);
    }
    if (!"srgb".equals(Json.get(value, "colorSpace"))) {
      throw new IllegalArgumentException(name + " in " + theme + " is not srgb");
    }
    List<?> c = Json.array(Json.get(value, "components"));
    String hex = Json.string(Json.get(value, "hex"));
    int rgb = Integer.parseInt(hex.substring(1), 16);
    Rgba rgba = new Rgba(rgb >> 16 & 0xff, rgb >> 8 & 0xff, rgb & 0xff, ((BigDecimal) Json.get(value, "alpha")).doubleValue());
    for (int i = 0; i < 3; i++) {
      double expected = rgba.channel(i) / 255.0;
      if (Math.abs(((BigDecimal) c.get(i)).doubleValue() - expected) > 0.00001) {
        throw new IllegalArgumentException(name + " in " + theme + ": components disagree with hex");
      }
    }
    return rgba;
  }

  private static Map<String, Object> tokensOf(Object group) {
    Map<String, Object> tokens = new LinkedHashMap<>();
    for (Map.Entry<?, ?> entry : Json.object(group).entrySet()) {
      String key = String.valueOf(entry.getKey());
      if (!key.startsWith("$")) {
        tokens.put(key, entry.getValue());
      }
    }
    return tokens;
  }

  private static Object read(Path file) throws IOException {
    return Json.parse(Files.readString(file, StandardCharsets.UTF_8));
  }

  /**
   * A straight (non-premultiplied) sRGB colour.
   *
   * @param r red, 0–255
   * @param g green, 0–255
   * @param b blue, 0–255
   * @param alpha opacity, 0–1
   */
  record Rgba(int r, int g, int b, double alpha) {

    /**
     * Returns one channel.
     *
     * @param index 0 for red, 1 for green, 2 for blue
     * @return the channel value, 0–255
     */
    int channel(int index) {
      return switch (index) {
        case 0 -> r;
        case 1 -> g;
        default -> b;
      };
    }

    /**
     * Composites this colour over an opaque ground.
     *
     * @param ground the opaque ground
     * @return the opaque result, rounded to whole channel values
     */
    Rgba over(Rgba ground) {
      return new Rgba(
          (int) Math.round(r * alpha + ground.r * (1 - alpha)),
          (int) Math.round(g * alpha + ground.g * (1 - alpha)),
          (int) Math.round(b * alpha + ground.b * (1 - alpha)),
          1.0);
    }

    /**
     * Returns the WCAG 2.2 relative luminance of an opaque colour.
     *
     * @return the relative luminance, 0–1
     */
    double luminance() {
      return 0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b);
    }

    /**
     * Returns the colour as {@code #rrggbb}, followed by its alpha if it is translucent.
     *
     * @return the text form
     */
    String text() {
      String hex = String.format("#%02x%02x%02x", r, g, b);
      return alpha < 1.0 ? hex + " @" + alpha : hex;
    }

    private static double linear(int channel) {
      double c = channel / 255.0;
      return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
    }
  }
}
