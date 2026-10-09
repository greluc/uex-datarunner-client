// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.ArrayList;
import java.util.List;
import java.util.spi.ToolProvider;
import javax.tools.JavaCompiler;

/** SP-8: builds a small named module with a stylesheet and fonts in a scratch directory and runs it. */
public final class Sp8Jpms {

  private static final String MAIN = """
      package sp.theme;

      import java.io.InputStream;
      import java.nio.file.Files;
      import java.nio.file.Path;
      import javafx.application.Application;
      import javafx.application.Platform;
      import javafx.scene.Scene;
      import javafx.scene.control.Label;
      import javafx.scene.layout.VBox;
      import javafx.scene.text.Font;
      import javafx.stage.Stage;

      public final class Main extends Application {
        public static void main(String[] args) { launch(args); }

        static void tmp(String when) throws Exception {
          try (var s = Files.list(Path.of(System.getProperty("java.io.tmpdir")))) {
            System.out.println("tmp +JXF " + when + ": " + s.filter(p -> p.getFileName().toString().contains("JXF")).count());
          }
        }

        @Override
        public void start(Stage stage) throws Exception {
          System.out.println("module=" + Main.class.getModule() + " named=" + Main.class.getModule().isNamed());
          var cssUrl = Main.class.getResource("theme.css");
          System.out.println("Class.getResource(theme.css)=" + cssUrl);
          System.out.println("ClassLoader.getResource(sp/theme/theme.css)=" + Main.class.getClassLoader().getResource("sp/theme/theme.css"));
          System.out.println("Thread context loader getResource(sp/theme/theme.css)=" + Thread.currentThread().getContextClassLoader().getResource("sp/theme/theme.css"));
          tmp("start");
          var fontUrl = Main.class.getResource("fonts/IBMPlexSans-Regular.ttf");
          System.out.println("font resource URL=" + fontUrl);
          Font byUrl = Font.loadFont(fontUrl.toExternalForm(), 14);
          System.out.println("Font.loadFont(url) -> " + (byUrl == null ? "null" : byUrl.getName()));
          tmp("after loadFont(url)");
          try (InputStream in = Main.class.getResourceAsStream("fonts/IBMPlexSans-Bold.ttf")) {
            Font byStream = Font.loadFont(in, 14);
            System.out.println("Font.loadFont(stream) -> " + (byStream == null ? "null" : byStream.getName()));
          }
          tmp("after loadFont(stream)");
          Label a = new Label("styled by URL");
          a.getStyleClass().add("probe");
          Label b = new Label("semibold via @font-face");
          b.getStyleClass().add("probe-face");
          Scene scene = new Scene(new VBox(a, b), 300, 100);
          scene.getStylesheets().add(cssUrl.toExternalForm());
          stage.setScene(scene);
          stage.show();
          scene.getRoot().applyCss();
          System.out.println("URL sheet: label textFill=" + a.getTextFill() + " (expected 0xff9440ff)");
          System.out.println("URL sheet: @font-face label font=" + b.getFont().getName() + " (expected IBM Plex Sans SmBld)");
          tmp("after stylesheet with relative @font-face");
          Label c = new Label("bare path");
          c.getStyleClass().add("probe");
          Scene bare = new Scene(new VBox(c), 100, 50);
          bare.getStylesheets().add("sp/theme/theme.css");
          bare.getRoot().applyCss();
          System.out.println("bare-path sheet 'sp/theme/theme.css': label textFill=" + c.getTextFill());
          Platform.exit();
        }
      }
      """;

  private static final String CSS = """
      @font-face {
          src: url("fonts/IBMPlexSans-SemiBold.ttf");
      }

      .probe {
          -fx-text-fill: #ff9440;
      }

      .probe-face {
          -fx-font-family: "IBM Plex Sans SmBld";
      }
      """;

  private Sp8Jpms() {
  }

  /**
   * Builds and runs the module variants.
   *
   * @param args output directory, JavaFX jar directory, font directory, java executable
   * @throws Exception when a build or run step fails
   */
  public static void main(String[] args) throws Exception {
    Path out = Path.of(args[0]);
    String jfx = args[1];
    Path fonts = Path.of(args[2]);
    String java = args[3];
    for (String variant : List.of("closed", "open")) {
      Path base = out.resolve(variant);
      Path src = base.resolve("src");
      Path pkg = src.resolve("sp/theme");
      Files.createDirectories(pkg.resolve("fonts"));
      String info = (variant.equals("open") ? "open " : "") + "module sp.theme {\n  requires javafx.controls;\n  exports sp.theme;\n}\n";
      Files.writeString(src.resolve("module-info.java"), info);
      Files.writeString(pkg.resolve("Main.java"), MAIN);
      Path classes = base.resolve("classes");
      Path res = classes.resolve("sp/theme");
      Files.createDirectories(res.resolve("fonts"));
      Files.writeString(res.resolve("theme.css"), CSS, StandardCharsets.UTF_8);
      for (String f : List.of("IBMPlexSans-Regular.ttf", "IBMPlexSans-SemiBold.ttf", "IBMPlexSans-Bold.ttf")) {
        Files.copy(fonts.resolve(f), res.resolve("fonts").resolve(f), StandardCopyOption.REPLACE_EXISTING);
      }
      JavaCompiler javac = javax.tools.ToolProvider.getSystemJavaCompiler();
      int rc = javac.run(null, null, null, "--module-path", jfx, "-d", classes.toString(),
          src.resolve("module-info.java").toString(), pkg.resolve("Main.java").toString());
      System.out.println(variant + ": javac rc=" + rc);
      Path jar = base.resolve("sp.theme.jar");
      Files.deleteIfExists(jar);
      int jrc = ToolProvider.findFirst("jar").orElseThrow().run(System.out, System.err, "--create",
          "--file", jar.toString(), "--main-class", "sp.theme.Main", "-C", classes.toString(), ".");
      System.out.println(variant + ": jar rc=" + jrc);
      for (String layout : List.of("exploded", "jar")) {
        Path tmp = base.resolve("tmp-" + layout);
        Files.createDirectories(tmp);
        String mp = jfx + File.pathSeparator + (layout.equals("jar") ? jar : classes);
        List<String> cmd = new ArrayList<>(List.of(java, "-Djava.io.tmpdir=" + tmp,
            "--enable-native-access=javafx.graphics", "--module-path", mp, "-m", "sp.theme/sp.theme.Main"));
        Process p = new ProcessBuilder(cmd).redirectErrorStream(true).start();
        String text = new String(p.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        int prc = p.waitFor();
        System.out.println("===== " + variant + " / " + layout + " (exit " + prc + ")");
        text.lines().filter(l -> !l.startsWith("WARNING: ") || l.contains("CSS") || l.contains("Resource"))
            .forEach(System.out::println);
        try (var s = Files.list(tmp)) {
          System.out.println("tmp +JXF files left after exit: " + s.count());
        }
      }
    }
  }
}
