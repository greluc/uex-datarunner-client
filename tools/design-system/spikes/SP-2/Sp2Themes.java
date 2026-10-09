// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import javafx.animation.PauseTransition;
import javafx.application.Application;
import javafx.application.ColorScheme;
import javafx.application.Platform;
import javafx.collections.ListChangeListener;
import javafx.collections.MapChangeListener;
import javafx.geometry.Side;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.control.Alert;
import javafx.scene.control.Button;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ContextMenu;
import javafx.scene.control.Label;
import javafx.scene.control.MenuItem;
import javafx.scene.control.Tooltip;
import javafx.scene.control.skin.ComboBoxListViewSkin;
import javafx.scene.image.Image;
import javafx.scene.image.PixelReader;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HeaderBar;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.robot.Robot;
import javafx.stage.Modality;
import javafx.stage.Popup;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.stage.Window;
import javafx.util.Duration;
import javax.imageio.ImageIO;

/** SP-2: compares theme-switching mechanisms across windows, popups and title bars. */
public final class Sp2Themes extends Application {

  private String mode;
  private Path cssDir;
  private Path outDir;
  private ColorScheme current = ColorScheme.DARK;
  private Stage stage;
  private Stage extended;
  private Tooltip tooltip;
  private ContextMenu contextMenu;
  private ComboBox<String> combo;
  private Popup popup;
  private Region popupPanel;
  private Alert alert;
  private Button button;
  private Label owner;

  /**
   * Starts the program.
   *
   * @param args mode (A0, A, B1, B2, C1, C2, HC), stylesheet directory, output directory
   */
  public static void main(String[] args) {
    launch(args);
  }

  /**
   * Builds the windows and runs the switch sequence.
   *
   * @param primary the primary stage
   */
  @Override
  public void start(Stage primary) {
    List<String> raw = getParameters().getRaw();
    mode = raw.get(0);
    cssDir = Path.of(raw.get(1));
    outDir = Path.of(raw.get(2));
    stage = primary;
    printPlatformPreferences();

    owner = new Label("Tooltip owner");
    tooltip = new Tooltip("Tooltip text");
    owner.setTooltip(tooltip);
    button = new Button("Context menu owner");
    contextMenu = new ContextMenu(new MenuItem("Item one"), new MenuItem("Item two"));
    button.setContextMenu(contextMenu);
    combo = new ComboBox<>();
    combo.getItems().addAll("Alpha", "Beta", "Gamma");
    VBox content = new VBox(12, owner, button, combo, new Label("Body label"));
    content.setStyle("-fx-padding: 16;");
    stage.setScene(new Scene(content, 360, 260));
    stage.setTitle("SP-2 " + mode);
    stage.setX(80);
    stage.setY(80);

    popupPanel = new Region();
    popupPanel.getStyleClass().add("popup-panel");
    popupPanel.setPrefSize(120, 60);
    popup = new Popup();
    popup.getContent().add(popupPanel);

    alert = new Alert(Alert.AlertType.INFORMATION, "Alert body");
    alert.initOwner(stage);
    alert.initModality(Modality.NONE);

    extended = new Stage(StageStyle.EXTENDED);
    HeaderBar headerBar = new HeaderBar();
    headerBar.setCenter(new Label("HeaderBar"));
    headerBar.getStyleClass().add("header-bar");
    BorderPane extRoot = new BorderPane(new StackPane(new Label("EXTENDED body")));
    extRoot.setTop(headerBar);
    extended.setScene(new Scene(extRoot, 420, 160));
    extended.setX(480);
    extended.setY(80);
    extended.setTitle("SP-2 EXTENDED");

    if (!mode.equals("A0")) {
      Window.getWindows().addListener((ListChangeListener<Window>) c -> {
        while (c.next()) {
          for (Window w : c.getAddedSubList()) {
            prepare(w);
          }
        }
      });
    }
    prepare(stage);
    prepare(extended);
    applyScheme(ColorScheme.DARK);
    stage.show();
    extended.show();
    stage.toFront();
    showPopups();
    after(800, () -> {
      report("dark");
      applyScheme(ColorScheme.LIGHT);
      after(800, () -> {
        report("light");
        listWindows();
        tooltip.hide();
        contextMenu.hide();
        combo.hide();
        popup.hide();
        alert.close();
        after(400, () -> {
          showPopups();
          after(800, () -> {
            report("light-reshown");
            tooltip.hide();
            contextMenu.hide();
            combo.hide();
            popup.hide();
            alert.close();
            after(300, Platform::exit);
          });
        });
      });
    });
  }

  private void prepare(Window w) {
    if (w.getScene() != null) {
      prepareScene(w.getScene());
    }
    w.sceneProperty().addListener((_, _, s) -> {
      if (s != null) {
        prepareScene(s);
      }
    });
  }

  private void prepareScene(Scene scene) {
    if (mode.equals("A0") && scene != stage.getScene()) {
      return;
    }
    if (mode.equals("HC") && scene != stage.getScene()) {
      return;
    }
    List<String> sheets = new ArrayList<>();
    sheets.add(uri("sp2-components.css"));
    switch (mode) {
      case "A0", "A" -> sheets.add(uri(current == ColorScheme.DARK ? "sp2-dark.css" : "sp2-light.css"));
      case "B1", "B2" -> sheets.add(uri("sp2-media.css"));
      case "C1", "C2" -> sheets.add(uri("sp2-import.css"));
      case "HC" -> {
        if (Boolean.getBoolean("sp2.authorSheet")) {
          sheets.add(uri("sp2-dark.css"));
        } else {
          sheets.clear();
        }
      }
      default -> throw new IllegalArgumentException(mode);
    }
    scene.getStylesheets().setAll(sheets);
    if (mode.equals("B2") || mode.equals("C2")) {
      scene.getPreferences().setColorScheme(current);
    }
  }

  private void applyScheme(ColorScheme scheme) {
    current = scheme;
    System.out.println("== applying " + scheme + " in mode " + mode);
    switch (mode) {
      case "A0" -> prepareScene(stage.getScene());
      case "A" -> Window.getWindows().forEach(w -> {
        if (w.getScene() != null) {
          prepareScene(w.getScene());
        }
      });
      case "B1", "C1" -> {
        stage.getScene().getPreferences().setColorScheme(scheme);
        extended.getScene().getPreferences().setColorScheme(scheme);
      }
      case "B2", "C2" -> Window.getWindows().forEach(w -> {
        if (w.getScene() != null) {
          w.getScene().getPreferences().setColorScheme(scheme);
        }
      });
      case "HC" -> { }
      default -> throw new IllegalArgumentException(mode);
    }
  }

  private void showPopups() {
    alert.show();
    alert.getDialogPane().getScene().getWindow().setX(80);
    alert.getDialogPane().getScene().getWindow().setY(420);
    stage.toFront();
    stage.requestFocus();
    tooltip.show(stage, stage.getX() + 40, stage.getY() + 300);
    popup.show(stage, stage.getX() + 200, stage.getY() + 300);
    combo.show();
    contextMenu.show(button, Side.BOTTOM, 0, 0);
  }

  private void report(String label) {
    listWindows();
    System.out.println("-- " + label + ": platform scheme=" + Platform.getPreferences().getColorScheme()
        + " primary scene scheme=" + stage.getScene().getPreferences().getColorScheme());
    Node tipNode = tooltip.getSkin().getNode();
    Node cmNode = contextMenu.getSkin().getNode();
    Node listNode = ((ComboBoxListViewSkin<?>) combo.getSkin()).getPopupContent();
    line("primary root", stage.getScene().getRoot());
    line("button", button);
    System.out.println("   button textFill=" + button.getTextFill());
    System.out.println("   body label textFill="
        + ((Label) ((VBox) stage.getScene().getRoot()).getChildren().get(3)).getTextFill());
    line("tooltip", tipNode);
    System.out.println("   tooltip textFill=" + ((Label) tipNode).getTextFill() + " scene sheets="
        + tipNode.getScene().getStylesheets().size() + " scene scheme="
        + tipNode.getScene().getPreferences().getColorScheme());
    line("context menu", cmNode);
    System.out.println("   context menu scene sheets=" + cmNode.getScene().getStylesheets().size()
        + " scene scheme=" + cmNode.getScene().getPreferences().getColorScheme());
    line("combo popup list", listNode);
    line("popup panel", popupPanel);
    System.out.println("   popup scene sheets=" + popupPanel.getScene().getStylesheets().size()
        + " scene scheme=" + popupPanel.getScene().getPreferences().getColorScheme());
    line("alert dialog pane", alert.getDialogPane());
    System.out.println("   alert scene sheets=" + alert.getDialogPane().getScene().getStylesheets().size()
        + " scene scheme=" + alert.getDialogPane().getScene().getPreferences().getColorScheme());
    line("extended root", extended.getScene().getRoot());
    System.out.println("   HeaderBar.getSystemColorScheme(extended)="
        + HeaderBar.getSystemColorScheme(extended));
    Robot robot = new Robot();
    double tx = stage.getX() + 60;
    double ty = stage.getY() + 12;
    Color titlePixel = robot.getPixelColor(tx, ty);
    System.out.println("   DECORATED title bar pixel at (" + tx + "," + ty + ") = " + titlePixel);
    save(robot.getScreenCapture(null, stage.getX(), stage.getY(), stage.getWidth(), stage.getHeight()),
        outDir.resolve(mode + "-" + label + "-decorated.png"));
    save(robot.getScreenCapture(null, extended.getX(), extended.getY(), extended.getWidth(),
        extended.getHeight()), outDir.resolve(mode + "-" + label + "-extended.png"));
    Window aw = alert.getDialogPane().getScene().getWindow();
    save(robot.getScreenCapture(null, aw.getX(), aw.getY(), aw.getWidth(), aw.getHeight()),
        outDir.resolve(mode + "-" + label + "-alert.png"));
    save(tipNode.getScene().snapshot(null), outDir.resolve(mode + "-" + label + "-tooltip.png"));
    save(cmNode.getScene().snapshot(null), outDir.resolve(mode + "-" + label + "-contextmenu.png"));
    save(listNode.getScene().snapshot(null), outDir.resolve(mode + "-" + label + "-combopopup.png"));
  }

  private void listWindows() {
    for (Window w : Window.getWindows()) {
      System.out.println("   window " + w.getClass().getName() + " scene=" + System.identityHashCode(w.getScene())
          + " sheets=" + (w.getScene() == null ? "-" : w.getScene().getStylesheets()));
    }
    System.out.println("   context menu scene id=" + System.identityHashCode(contextMenu.getSkin().getNode().getScene())
        + " contextMenu.getScene id=" + System.identityHashCode(contextMenu.getScene())
        + " in getWindows=" + Window.getWindows().contains(contextMenu)
        + " contextMenu.isShowing=" + contextMenu.isShowing() + " combo.isShowing=" + combo.isShowing()
        + " tooltip.isShowing=" + tooltip.isShowing() + " popup.isShowing=" + popup.isShowing());
  }

  private static void line(String name, Node node) {
    if (node == null) {
      return;
    }
    String fill = "none";
    if (node instanceof Region r && r.getBackground() != null && !r.getBackground().getFills().isEmpty()) {
      var fills = r.getBackground().getFills();
      fill = fills.getLast().getFill() + " (" + fills.size() + " fills)";
    }
    System.out.println("   " + name + " background=" + fill + " styleClass=" + node.getStyleClass());
  }

  private void printPlatformPreferences() {
    Map<String, Object> sorted = new TreeMap<>();
    Platform.getPreferences().forEach((k, v) -> sorted.put(k, v));
    sorted.forEach((k, v) -> {
      if (k.startsWith("Windows.SPI") || k.startsWith("Windows.UIColor.Accent") || k.contains("HighContrast")
          || k.startsWith("Windows.UISettings")) {
        System.out.println("pref " + k + " = " + v);
      }
    });
    System.out.println("pref count=" + sorted.size() + " colorScheme=" + Platform.getPreferences().getColorScheme()
        + " reducedMotion=" + Platform.getPreferences().isReducedMotion());
    Platform.getPreferences().addListener((MapChangeListener<String, Object>) c ->
        System.out.println("preference change: " + c.getKey() + " -> " + c.getValueAdded()));
    System.out.println("MapChangeListener registered on Platform.getPreferences()");
    System.out.println("sys com.sun.javafx.highContrastTheme=" + System.getProperty("com.sun.javafx.highContrastTheme"));
  }

  private String uri(String name) {
    return cssDir.resolve(name).toUri().toString();
  }

  private static void after(double ms, Runnable r) {
    PauseTransition p = new PauseTransition(Duration.millis(ms));
    p.setOnFinished(_ -> r.run());
    p.play();
  }

  static void save(Image img, Path file) {
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
      Files.createDirectories(file.getParent());
      ImageIO.write(bi, "png", file.toFile());
    } catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }
}
