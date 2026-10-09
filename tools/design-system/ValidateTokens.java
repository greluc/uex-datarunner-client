// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.io.IOException;
import java.math.BigDecimal;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.regex.Pattern;

/**
 * Validates the repository token files against the conversion rules of the JavaFX handoff (§3).
 *
 * <p>Usage: {@code java ValidateTokens.java <token dir>}. Prints every violation and exits with
 * status 1 if there is any; nothing is fixed.
 */
final class ValidateTokens {

  private static final Pattern NAME = Pattern.compile("^[a-z0-9-]+$");
  private static final BigDecimal MIN_TARGET_PX = BigDecimal.valueOf(24);
  private static final Set<String> ROW_AND_TARGET_SIZES =
      Set.of("size-row-compact", "size-row-comfortable", "size-in-cell-control");

  private ValidateTokens() {}

  /**
   * Runs the validation.
   *
   * @param args the token directory
   * @throws IOException if a file cannot be read
   */
  public static void main(String[] args) throws IOException {
    Path dir = Path.of(args[0]);
    List<String> errors = validate(dir);
    if (errors.isEmpty()) {
      System.out.println("Token validation passed.");
      return;
    }
    errors.forEach(e -> System.out.println("ERROR " + e));
    System.out.println(errors.size() + " violation(s).");
    System.exit(1);
  }

  /**
   * Validates the token directory.
   *
   * @param dir the token directory
   * @return the violations, empty if there are none
   * @throws IOException if a file cannot be read
   */
  static List<String> validate(Path dir) throws IOException {
    List<String> errors = new ArrayList<>();
    Tokens tokens = Tokens.load(dir);

    Set<String> colorNames = null;
    Set<String> shadowNames = null;
    for (String theme : tokens.themes) {
      checkName("theme " + theme, theme, errors);
      Map<String, Object> colors = tokens.themeGroup(theme, "color");
      Map<String, Object> shadows = tokens.themeGroup(theme, "shadow");
      colorNames = sameNames("color", theme, colorNames, colors.keySet(), errors);
      shadowNames = sameNames("shadow", theme, shadowNames, shadows.keySet(), errors);
      for (String name : colors.keySet()) {
        checkName("colour", name, errors);
        Object value = Json.get(colors.get(name), "$value");
        if (value instanceof String s && !s.startsWith("{color.")) {
          errors.add("colour " + name + " in " + theme + " is a string that is not an alias: " + s);
          continue;
        }
        try {
          Tokens.Rgba rgba = tokens.color(theme, name);
          if (theme.equals("high-contrast") && rgba.alpha() < 1.0) {
            errors.add("colour " + name + " is translucent in high contrast");
          }
        } catch (IllegalArgumentException e) {
          errors.add(e.getMessage());
        }
      }
      for (Map.Entry<String, Object> shadow : shadows.entrySet()) {
        checkName("shadow", shadow.getKey(), errors);
        Object value = Json.get(shadow.getValue(), "$value");
        if (value instanceof List<?>) {
          errors.add("shadow " + shadow.getKey() + " in " + theme + " has more than one layer");
          continue;
        }
        boolean none = isNone(shadow.getValue());
        if (!none && decimal(Json.get(Json.get(value, "spread"), "value")).signum() != 0) {
          errors.add("shadow " + shadow.getKey() + " in " + theme + " has a spread");
        }
        if (!none && theme.equals("high-contrast")) {
          errors.add("shadow " + shadow.getKey() + " is not none in high contrast");
        }
      }
    }

    for (String group : tokens.baseGroups()) {
      checkName("group", group, errors);
      for (String name : tokens.baseGroup(group).keySet()) {
        checkName(group + " token", name, errors);
      }
    }
    Map<String, Object> faces = tokens.baseGroup("font-face");
    Map<String, Object> families = tokens.baseGroup("font-family");
    for (Map.Entry<String, Object> style : tokens.baseGroup("typography").entrySet()) {
      Object value = Json.get(style.getValue(), "$value");
      Map<?, ?> v = Json.object(value);
      for (Object key : v.keySet()) {
        if (!Set.of("fontFamily", "fontSize", "fontWeight", "letterSpacing", "lineHeight").contains(String.valueOf(key))) {
          errors.add("typography " + style.getKey() + " has unsupported property " + key);
        }
      }
      if (decimal(Json.get(Json.get(value, "letterSpacing"), "value")).signum() != 0) {
        errors.add("typography " + style.getKey() + " has letter spacing");
      }
      String familyRef = Json.string(Json.get(value, "fontFamily"));
      String familyKey = familyRef.replace("{font-family.", "").replace("}", "");
      if (!families.containsKey(familyKey)) {
        errors.add("typography " + style.getKey() + " references unknown family " + familyRef);
        continue;
      }
      String family = Json.string(Json.get(families.get(familyKey), "$value"));
      BigDecimal weight = decimal(Json.get(value, "fontWeight"));
      boolean listed = faces.values().stream().anyMatch(face ->
          Json.string(Json.get(face, "$value")).equals(family)
              && decimal(Json.get(Json.get(Json.get(face, "$extensions"), "space.uexdatarunner"), "weight")).compareTo(weight) == 0);
      if (!listed) {
        errors.add("typography " + style.getKey() + " uses weight " + weight + " without a listed " + family + " face");
      }
    }
    for (Map.Entry<String, Object> size : tokens.baseGroup("size").entrySet()) {
      if (ROW_AND_TARGET_SIZES.contains(size.getKey())) {
        BigDecimal px = decimal(Json.get(Json.get(size.getValue(), "$value"), "value"));
        if (px.compareTo(MIN_TARGET_PX) < 0) {
          errors.add(size.getKey() + " is below 24 px");
        }
      }
    }

    Object pairings = Json.parse(Files.readString(dir.resolve("pairings.json"), StandardCharsets.UTF_8));
    for (Object pair : Json.array(Json.get(pairings, "pairs"))) {
      String theme = Json.string(Json.get(pair, "theme"));
      if (!tokens.themes.contains(theme)) {
        errors.add("pairing names unknown theme " + theme);
        continue;
      }
      for (String role : List.of("foreground", "ground")) {
        String name = Json.string(Json.get(pair, role));
        if (colorNames != null && !colorNames.contains(name)) {
          errors.add("pairing " + role + " " + name + " is not a colour token");
        }
      }
    }

    Object states = Json.parse(Files.readString(dir.resolve("states.json"), StandardCharsets.UTF_8));
    Set<String> ids = new HashSet<>();
    for (Object state : Json.array(Json.get(states, "states"))) {
      String id = Json.string(Json.get(state, "id"));
      checkName("state id", id, errors);
      if (!ids.add(id)) {
        errors.add("state id " + id + " is defined twice");
      }
    }
    return errors;
  }

  private static boolean isNone(Object token) {
    Object ext = Json.object(token).get("$extensions");
    if (ext == null) {
      return false;
    }
    Object own = Json.object(ext).get("space.uexdatarunner");
    return own != null && Boolean.TRUE.equals(Json.object(own).get("none"));
  }

  private static Set<String> sameNames(String group, String theme, Set<String> expected, Set<String> actual, List<String> errors) {
    if (expected == null) {
      return Set.copyOf(actual);
    }
    for (String name : expected) {
      if (!actual.contains(name)) {
        errors.add(group + " " + name + " has no value in " + theme);
      }
    }
    for (String name : actual) {
      if (!expected.contains(name)) {
        errors.add(group + " " + name + " exists only in some themes, including " + theme);
      }
    }
    return expected;
  }

  private static void checkName(String kind, String name, List<String> errors) {
    if (!NAME.matcher(name).matches()) {
      errors.add(kind + " name '" + name + "' is not [a-z0-9-]");
    }
  }

  private static BigDecimal decimal(Object value) {
    if (value instanceof BigDecimal d) {
      return d;
    }
    throw new IllegalArgumentException("not a number: " + value);
  }
}
