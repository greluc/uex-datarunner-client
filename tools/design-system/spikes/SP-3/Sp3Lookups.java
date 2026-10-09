// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import javafx.animation.PauseTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.beans.property.SimpleStringProperty;
import javafx.collections.FXCollections;
import javafx.geometry.Bounds;
import javafx.geometry.Orientation;
import javafx.geometry.Side;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.ComboBox;
import javafx.scene.control.ContextMenu;
import javafx.scene.control.Label;
import javafx.scene.control.MenuItem;
import javafx.scene.control.ScrollBar;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.control.TextField;
import javafx.scene.control.Tooltip;
import javafx.scene.control.skin.ComboBoxListViewSkin;
import javafx.scene.image.Image;
import javafx.scene.image.PixelReader;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import javafx.util.Duration;
import javax.imageio.ImageIO;

/** SP-3: checks prefixed lookups, colour-name collisions and stock controls without Modena. */
public final class Sp3Lookups extends Application {

  private static final List<String> PROBES = List.of("probe-named-red", "probe-bare-bead",
      "probe-bare-fade", "probe-tl-red", "probe-tl-bead", "probe-digit", "probe-case-exact",
      "probe-case-lower", "probe-alias", "probe-derive", "probe-ladder", "probe-missing");

  /**
   * Starts the program.
   *
   * @param args mode (modena, empty, min, null), stylesheet directory, output directory
   */
  public static void main(String[] args) {
    launch(args);
  }

  /**
   * Builds the probe scene and prints computed values.
   *
   * @param stage the primary stage
   */
  @Override
  public void start(Stage stage) {
    String mode = getParameters().getRaw().get(0);
    Path cssDir = Path.of(getParameters().getRaw().get(1));
    Path outDir = Path.of(getParameters().getRaw().get(2));
    switch (mode) {
      case "modena" -> { }
      case "empty" -> Application.setUserAgentStylesheet(cssDir.resolve("sp3-ua-empty.css").toUri().toString());
      case "min" -> Application.setUserAgentStylesheet(cssDir.resolve("sp3-ua-min.css").toUri().toString());
      case "null" -> {
        Application.setUserAgentStylesheet(cssDir.resolve("sp3-ua-empty.css").toUri().toString());
        Application.setUserAgentStylesheet(null);
      }
      default -> throw new IllegalArgumentException(mode);
    }
    System.out.println("mode=" + mode + " getUserAgentStylesheet()=" + Application.getUserAgentStylesheet());

    FlowPane probes = new FlowPane(4, 4);
    for (String p : PROBES) {
      Region r = new Region();
      r.getStyleClass().add(p);
      r.setPrefSize(24, 24);
      probes.getChildren().add(r);
    }
    Button mapped = new Button("Modena rule + -tl");
    mapped.getStyleClass().add("probe-modena-rule");
    Button plain = new Button("Plain button");
    Label text = new Label("Probe text");
    text.getStyleClass().add("probe-text");
    TextField field = new TextField("Text field");
    CheckBox check = new CheckBox("Check");
    check.setSelected(true);
    ComboBox<String> combo = new ComboBox<>(FXCollections.observableArrayList("Alpha", "Beta", "Gamma"));
    combo.getSelectionModel().select(0);
    TableView<String[]> table = new TableView<>();
    for (int c = 0; c < 3; c++) {
      int col = c;
      TableColumn<String[], String> tc = new TableColumn<>("Column " + c);
      tc.setCellValueFactory(cd -> new SimpleStringProperty(cd.getValue()[col]));
      tc.setPrefWidth(110);
      table.getColumns().add(tc);
    }
    for (int r = 0; r < 60; r++) {
      table.getItems().add(new String[] {"Row " + r, "Value " + r, "Δ −" + r});
    }
    table.setPrefSize(360, 220);
    table.getSelectionModel().select(2);
    Label tipOwner = new Label("Tooltip owner");
    Tooltip tip = new Tooltip("Tooltip text");
    tipOwner.setTooltip(tip);
    ContextMenu menu = new ContextMenu(new MenuItem("Menu item one"), new MenuItem("Menu item two"));

    VBox root = new VBox(8, probes, new HBox(8, mapped, plain, check), text, field, combo, table, tipOwner);
    root.setStyle("-fx-padding: 8;");
    Scene scene = new Scene(root, 420, 560);
    scene.getStylesheets().add(cssDir.resolve("sp3-author.css").toUri().toString());
    stage.setScene(scene);
    stage.setTitle("SP-3 " + mode);
    stage.setX(60);
    stage.setY(60);
    stage.show();

    PauseTransition wait = new PauseTransition(Duration.millis(500));
    wait.setOnFinished(_ -> {
      for (Node n : probes.getChildren()) {
        Region r = (Region) n;
        System.out.println("probe " + r.getStyleClass().getFirst() + " fill="
            + (r.getBackground() == null ? "none" : r.getBackground().getFills().getLast().getFill()));
      }
      describe("button probe-modena-rule", mapped);
      System.out.println("   textFill=" + mapped.getTextFill());
      describe("plain button", plain);
      System.out.println("   textFill=" + plain.getTextFill() + " font=" + plain.getFont());
      System.out.println("label probe-text textFill=" + text.getTextFill());
      describe("text field", field);
      describe("check box", check);
      describe("check box .box", check.lookup(".box"));
      describe("check box .mark", check.lookup(".mark"));
      describe("combo box", combo);
      describe("combo .arrow", combo.lookup(".arrow"));
      describe("table header background", table.lookup(".column-header-background"));
      describe("first table-row-cell", table.lookup(".table-row-cell"));
      for (Node sb : table.lookupAll(".scroll-bar")) {
        if (sb instanceof ScrollBar s && s.getOrientation() == Orientation.VERTICAL && s.isVisible()) {
          describe("vertical scroll bar", s);
          describe("vertical thumb", s.lookup(".thumb"));
          describe("vertical increment arrow", s.lookup(".increment-arrow"));
        }
      }
      save(scene.snapshot(null), outDir.resolve(mode + "-scene.png"));
      tip.show(stage, stage.getX() + 40, stage.getY() + 600);
      menu.show(plain, Side.BOTTOM, 0, 0);
      combo.show();
      PauseTransition popups = new PauseTransition(Duration.millis(500));
      popups.setOnFinished(_ -> {
        Node tipNode = tip.getSkin().getNode();
        describe("tooltip", tipNode);
        describe("context menu", menu.getSkin().getNode());
        Node list = ((ComboBoxListViewSkin<?>) combo.getSkin()).getPopupContent();
        describe("combo popup list", list);
        save(tipNode.getScene().snapshot(null), outDir.resolve(mode + "-tooltip.png"));
        save(menu.getSkin().getNode().getScene().snapshot(null), outDir.resolve(mode + "-contextmenu.png"));
        save(list.getScene().snapshot(null), outDir.resolve(mode + "-combopopup.png"));
        tip.hide();
        menu.hide();
        combo.hide();
        PauseTransition end = new PauseTransition(Duration.millis(300));
        end.setOnFinished(_ -> Platform.exit());
        end.play();
      });
      popups.play();
    });
    wait.play();
  }

  private static void describe(String name, Node n) {
    if (n == null) {
      System.out.println(name + ": not found");
      return;
    }
    Bounds b = n.getLayoutBounds();
    String fill = "none";
    String pad = "";
    if (n instanceof Region r) {
      if (r.getBackground() != null && !r.getBackground().getFills().isEmpty()) {
        fill = r.getBackground().getFills().size() + " fills, last=" + r.getBackground().getFills().getLast().getFill();
      }
      pad = " padding=" + r.getPadding() + " shape=" + (r.getShape() != null);
    }
    System.out.printf("%s: size=%.1fx%.1f bg=%s%s%n", name, b.getWidth(), b.getHeight(), fill, pad);
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
      Files.createDirectories(file.getParent());
      ImageIO.write(bi, "png", file.toFile());
    } catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }
}
