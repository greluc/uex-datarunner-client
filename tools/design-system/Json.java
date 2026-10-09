// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.math.BigDecimal;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Minimal JSON reader and writer for the design-system scripts.
 *
 * <p>Objects become insertion-ordered {@link Map}s, arrays {@link List}s, numbers {@link
 * BigDecimal}s, and {@code null} becomes {@link Json#NULL}.
 */
final class Json {

  /** Marker for a JSON {@code null}. */
  static final Object NULL = new Object();

  private final String text;
  private int pos;

  private Json(String text) {
    this.text = text;
  }

  /**
   * Parses a JSON document.
   *
   * @param text the document
   * @return the root value
   * @throws IllegalArgumentException if the text is not valid JSON
   */
  static Object parse(String text) {
    Json parser = new Json(text);
    parser.skipWhitespace();
    Object value = parser.value();
    parser.skipWhitespace();
    if (parser.pos != text.length()) {
      throw parser.error("trailing content");
    }
    return value;
  }

  /**
   * Writes a value as indented JSON with two spaces and a final line break.
   *
   * @param value a map, list, string, number, boolean or {@link #NULL}
   * @return the JSON text
   */
  static String write(Object value) {
    StringBuilder out = new StringBuilder();
    write(value, out, 0);
    return out.append('\n').toString();
  }

  /**
   * Returns the member of an object, failing if it is absent.
   *
   * @param object the object
   * @param key the member name
   * @return the member value
   * @throws IllegalArgumentException if the value is not an object or the member is absent
   */
  static Object get(Object object, String key) {
    Map<?, ?> map = object(object);
    if (!map.containsKey(key)) {
      throw new IllegalArgumentException("missing member '" + key + "'");
    }
    return map.get(key);
  }

  /**
   * Returns a value as an object.
   *
   * @param value the value
   * @return the value as a map
   * @throws IllegalArgumentException if the value is not an object
   */
  static Map<?, ?> object(Object value) {
    if (value instanceof Map<?, ?> map) {
      return map;
    }
    throw new IllegalArgumentException("not an object: " + value);
  }

  /**
   * Returns a value as an array.
   *
   * @param value the value
   * @return the value as a list
   * @throws IllegalArgumentException if the value is not an array
   */
  static List<?> array(Object value) {
    if (value instanceof List<?> list) {
      return list;
    }
    throw new IllegalArgumentException("not an array: " + value);
  }

  /**
   * Returns a value as a string.
   *
   * @param value the value
   * @return the string
   * @throws IllegalArgumentException if the value is not a string
   */
  static String string(Object value) {
    if (value instanceof String s) {
      return s;
    }
    throw new IllegalArgumentException("not a string: " + value);
  }

  private Object value() {
    char c = peek();
    return switch (c) {
      case '{' -> parseObject();
      case '[' -> parseArray();
      case '"' -> parseString();
      case 't' -> literal("true", Boolean.TRUE);
      case 'f' -> literal("false", Boolean.FALSE);
      case 'n' -> literal("null", NULL);
      default -> number();
    };
  }

  private Map<String, Object> parseObject() {
    Map<String, Object> map = new LinkedHashMap<>();
    pos++;
    skipWhitespace();
    if (peek() == '}') {
      pos++;
      return map;
    }
    while (true) {
      skipWhitespace();
      String key = parseString();
      skipWhitespace();
      expect(':');
      skipWhitespace();
      if (map.put(key, value()) != null) {
        throw error("duplicate member '" + key + "'");
      }
      skipWhitespace();
      if (peek() == ',') {
        pos++;
        continue;
      }
      expect('}');
      return map;
    }
  }

  private List<Object> parseArray() {
    List<Object> list = new ArrayList<>();
    pos++;
    skipWhitespace();
    if (peek() == ']') {
      pos++;
      return list;
    }
    while (true) {
      skipWhitespace();
      list.add(value());
      skipWhitespace();
      if (peek() == ',') {
        pos++;
        continue;
      }
      expect(']');
      return list;
    }
  }

  private String parseString() {
    expect('"');
    StringBuilder out = new StringBuilder();
    while (true) {
      char c = next();
      if (c == '"') {
        return out.toString();
      }
      if (c != '\\') {
        out.append(c);
        continue;
      }
      char e = next();
      switch (e) {
        case '"', '\\', '/' -> out.append(e);
        case 'b' -> out.append('\b');
        case 'f' -> out.append('\f');
        case 'n' -> out.append('\n');
        case 'r' -> out.append('\r');
        case 't' -> out.append('\t');
        case 'u' -> {
          out.append((char) Integer.parseInt(text.substring(pos, pos + 4), 16));
          pos += 4;
        }
        default -> throw error("bad escape");
      }
    }
  }

  private BigDecimal number() {
    int start = pos;
    while (pos < text.length() && "+-0123456789.eE".indexOf(text.charAt(pos)) >= 0) {
      pos++;
    }
    if (start == pos) {
      throw error("unexpected character");
    }
    return new BigDecimal(text.substring(start, pos));
  }

  private Object literal(String word, Object value) {
    if (!text.startsWith(word, pos)) {
      throw error("expected " + word);
    }
    pos += word.length();
    return value;
  }

  private void skipWhitespace() {
    while (pos < text.length() && Character.isWhitespace(text.charAt(pos))) {
      pos++;
    }
  }

  private char peek() {
    if (pos >= text.length()) {
      throw error("unexpected end");
    }
    return text.charAt(pos);
  }

  private char next() {
    char c = peek();
    pos++;
    return c;
  }

  private void expect(char c) {
    if (next() != c) {
      pos--;
      throw error("expected '" + c + "'");
    }
  }

  private IllegalArgumentException error(String message) {
    return new IllegalArgumentException(message + " at offset " + pos);
  }

  private static void write(Object value, StringBuilder out, int indent) {
    switch (value) {
      case Map<?, ?> map when map.isEmpty() -> out.append("{}");
      case Map<?, ?> map -> {
        out.append("{\n");
        int i = 0;
        for (Map.Entry<?, ?> entry : map.entrySet()) {
          pad(out, indent + 2);
          quote(String.valueOf(entry.getKey()), out);
          out.append(": ");
          write(entry.getValue(), out, indent + 2);
          out.append(++i < map.size() ? ",\n" : "\n");
        }
        pad(out, indent);
        out.append('}');
      }
      case List<?> list when list.isEmpty() -> out.append("[]");
      case List<?> list when list.stream().allMatch(e -> e instanceof Number) -> {
        out.append('[');
        for (int i = 0; i < list.size(); i++) {
          out.append(i == 0 ? "" : ", ");
          write(list.get(i), out, indent);
        }
        out.append(']');
      }
      case List<?> list -> {
        out.append("[\n");
        for (int i = 0; i < list.size(); i++) {
          pad(out, indent + 2);
          write(list.get(i), out, indent + 2);
          out.append(i + 1 < list.size() ? ",\n" : "\n");
        }
        pad(out, indent);
        out.append(']');
      }
      case String s -> quote(s, out);
      case BigDecimal d -> out.append(d.stripTrailingZeros().toPlainString());
      case Number n -> out.append(new BigDecimal(n.toString()).stripTrailingZeros().toPlainString());
      case Boolean b -> out.append(b);
      default -> {
        if (value != NULL) {
          throw new IllegalArgumentException("cannot write " + value.getClass());
        }
        out.append("null");
      }
    }
  }

  private static void pad(StringBuilder out, int indent) {
    out.append(" ".repeat(indent));
  }

  private static void quote(String s, StringBuilder out) {
    out.append('"');
    for (char c : s.toCharArray()) {
      switch (c) {
        case '"' -> out.append("\\\"");
        case '\\' -> out.append("\\\\");
        case '\n' -> out.append("\\n");
        case '\r' -> out.append("\\r");
        case '\t' -> out.append("\\t");
        default -> {
          if (c < 0x20) {
            out.append(String.format("\\u%04x", (int) c));
          } else {
            out.append(c);
          }
        }
      }
    }
    out.append('"');
  }
}
