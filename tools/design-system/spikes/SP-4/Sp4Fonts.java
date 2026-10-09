// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.stream.Stream;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.image.PixelReader;
import javafx.scene.layout.VBox;
import javafx.scene.text.Font;
import javafx.scene.text.FontWeight;
import javafx.scene.text.Text;
import javafx.stage.Stage;
import javax.imageio.ImageIO;

/** SP-4: checks font family and face names, weight mapping, glyph coverage and temp copies. */
public final class Sp4Fonts extends Application {

  private static final List<String> FILES = List.of("IBMPlexSans-Regular.ttf", "IBMPlexSans-SemiBold.ttf",
      "IBMPlexSans-Bold.ttf", "JetBrainsMonoNL-Regular.ttf", "JetBrainsMonoNL-Bold.ttf");

  private static final String GLYPHS = "Δ−●→←▲▼✓⚠×… ";

  private Path tmp;

  /**
   * Starts the program.
   *
   * @param args font directory, output directory, jar with fonts (for the jar: test)
   */
  public static void main(String[] args) {
    launch(args);
  }

  /**
   * Runs the font checks.
   *
   * @param stage the primary stage
   */
  @Override
  public void start(Stage stage) throws Exception {
    Path fonts = Path.of(getParameters().getRaw().get(0));
    Path out = Path.of(getParameters().getRaw().get(1));
    Path jar = Path.of(getParameters().getRaw().get(2));
    tmp = Path.of(System.getProperty("java.io.tmpdir"));
    System.out.println("java.io.tmpdir=" + tmp);
    System.out.println("com.sun.javafx.fontSize=" + System.getProperty("com.sun.javafx.fontSize"));
    Font def = Font.getDefault();
    System.out.println("Font.getDefault(): name=" + def.getName() + " family=" + def.getFamily() + " size=" + def.getSize());
    System.out.println("families before load: " + Font.getFamilies().size());
    listTmp("before any load");

    for (String f : FILES) {
      Font loaded = Font.loadFont(fonts.resolve(f).toUri().toString(), 14);
      System.out.println("loadFont(file:) " + f + " -> name='" + loaded.getName() + "' family='"
          + loaded.getFamily() + "' style='" + loaded.getStyle() + "'");
    }
    listTmp("after Font.loadFont(file: URL) x5");
    for (String fam : List.of("IBM Plex Sans", "IBM Plex Sans SmBld", "IBM Plex Sans SemiBold", "JetBrains Mono NL")) {
      System.out.println("getFontNames('" + fam + "') = " + Font.getFontNames(fam));
    }
    System.out.println("families containing Plex/JetBrains: "
        + Font.getFamilies().stream().filter(s -> s.contains("Plex") || s.contains("JetBrains")).toList());
    for (FontWeight w : FontWeight.values()) {
      Font f = Font.font("IBM Plex Sans", w, 14);
      Font m = Font.font("JetBrains Mono NL", w, 14);
      System.out.println("Font.font(Plex, " + w + "=" + w.getWeight() + ") -> " + f.getName() + " | Mono -> " + m.getName());
    }
    System.out.println("Font.font('IBM Plex Sans SmBld', NORMAL) -> " + Font.font("IBM Plex Sans SmBld", FontWeight.NORMAL, 14).getName());
    System.out.println("Font.font('IBM Plex Sans SmBld', BOLD) -> " + Font.font("IBM Plex Sans SmBld", FontWeight.BOLD, 14).getName());
    System.out.println("Font.font('IBM Plex Sans SemiBold') -> " + Font.font("IBM Plex Sans SemiBold", 14).getName());
    System.out.println("new Font('IBM Plex Sans SemiBold', 14) -> " + new Font("IBM Plex Sans SemiBold", 14).getName());

    for (String f : FILES) {
      java.awt.Font awt = java.awt.Font.createFont(java.awt.Font.TRUETYPE_FONT, fonts.resolve(f).toFile());
      StringBuilder sb = new StringBuilder("cmap (AWT canDisplay) " + f + ":");
      GLYPHS.codePoints().forEach(cp -> sb.append(String.format(" U+%04X=%s", cp, awt.canDisplay(cp) ? "yes" : "NO")));
      System.out.println(sb);
    }

    VBox root = new VBox(6);
    root.setStyle("-fx-padding: 10; -fx-background-color: white;");
    for (String spec : List.of("IBM Plex Sans|NORMAL", "IBM Plex Sans SmBld|NORMAL", "IBM Plex Sans|BOLD",
        "JetBrains Mono NL|NORMAL", "JetBrains Mono NL|BOLD", "System|NORMAL")) {
      String[] p = spec.split("\\|");
      Text t = new Text(p[0] + " " + p[1] + "  1,315 Δ −273 ● → ▲ ✓ ⚠ × …");
      t.setFont(Font.font(p[0], FontWeight.valueOf(p[1]), 20));
      root.getChildren().add(t);
    }
    String[] cssCases = {
        "-fx-font-family: 'IBM Plex Sans'; -fx-font-weight: normal;",
        "-fx-font-family: 'IBM Plex Sans'; -fx-font-weight: 500;",
        "-fx-font-family: 'IBM Plex Sans'; -fx-font-weight: 600;",
        "-fx-font-family: 'IBM Plex Sans'; -fx-font-weight: 700;",
        "-fx-font-family: 'IBM Plex Sans'; -fx-font-weight: bold;",
        "-fx-font-family: 'IBM Plex Sans SmBld';",
        "-fx-font-family: 'IBM Plex Sans SmBld'; -fx-font-weight: 600;",
        "-fx-font-family: 'JetBrains Mono NL'; -fx-font-weight: bold;",
        "-fx-font-family: 'IBM Plex Sans', 'Segoe UI';",
        "-fx-font-family: 'No Such Font', 'IBM Plex Sans';"
    };
    List<Label> cssLabels = new java.util.ArrayList<>();
    for (String css : cssCases) {
      Label l = new Label(css);
      l.setStyle(css + " -fx-font-size: 16px; -fx-text-fill: black;");
      cssLabels.add(l);
      root.getChildren().add(l);
    }
    Path cssFile = out.resolve("sp4-fontface.css");
    String jarUrl = "jar:" + jar.toUri() + "!/fonts/JetBrainsMonoNL-Regular.ttf";
    Files.writeString(cssFile, "@font-face { src: url('" + fonts.resolve("IBMPlexSans-SemiBold.ttf").toUri() + "'); }\n"
        + "@font-face { src: url('" + jarUrl + "'); }\n"
        + ".ff-semibold { -fx-font-family: 'IBM Plex Sans SmBld'; -fx-font-size: 16px; }\n");
    Scene scene = new Scene(root, 900, 560);
    stage.setScene(scene);
    stage.show();
    root.applyCss();
    for (int i = 0; i < cssCases.length; i++) {
      System.out.println("CSS [" + cssCases[i] + "] -> " + cssLabels.get(i).getFont().getName());
    }
    save(scene.snapshot(null), out.resolve("glyphs-and-weights.png"));

    listTmp("before @font-face");
    scene.getStylesheets().add(cssFile.toUri().toString());
    Label ff = new Label("font-face");
    ff.getStyleClass().add("ff-semibold");
    root.getChildren().add(ff);
    root.applyCss();
    System.out.println("@font-face (file: + jar:) applied; .ff-semibold label font=" + ff.getFont().getName());
    listTmp("after @font-face with file: and jar: src");

    try (InputStream in = Files.newInputStream(fonts.resolve("IBMPlexSans-Bold.ttf"))) {
      Font s = Font.loadFont(in, 14);
      System.out.println("loadFont(InputStream) -> " + s.getName());
    }
    listTmp("after Font.loadFont(InputStream)");
    Font j = Font.loadFont("jar:" + jar.toUri() + "!/fonts/JetBrainsMonoNL-Bold.ttf", 14);
    System.out.println("loadFont(jar: URL) -> " + (j == null ? "null" : j.getName()));
    listTmp("after Font.loadFont(jar: URL)");
    Platform.exit();
  }

  private void listTmp(String when) throws IOException {
    try (Stream<Path> s = Files.list(tmp)) {
      List<String> names = s.map(p -> p.getFileName().toString()).filter(n -> n.contains("JXF")).toList();
      System.out.println("tmp +JXF files " + when + ": " + names.size() + " " + names);
    }
  }

  private static void save(Image img, Path file) {
    int w = (int) img.getWidth();
    int h = (int) img.getHeight();
    BufferedImage bi = new BufferedImage(w, h, BufferedImage.TYPE_INT_ARGB);
    PixelReader pr = img.getPixelReader();
    for (int y = 0; y < h; y++) {
      for (int x = 0; x < w; x++) {
        bi.setRGB(x, y, pr.getArgb(x, y));
      }
    }
    try {
      ImageIO.write(bi, "png", file.toFile());
    } catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }
}
