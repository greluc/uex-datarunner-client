// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import javafx.animation.PauseTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.beans.property.ReadOnlyObjectWrapper;
import javafx.css.PseudoClass;
import javafx.geometry.Bounds;
import javafx.geometry.Point2D;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.SnapshotParameters;
import javafx.scene.control.ContentDisplay;
import javafx.scene.control.Label;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableRow;
import javafx.scene.control.TableView;
import javafx.scene.control.skin.VirtualFlow;
import javafx.scene.image.Image;
import javafx.scene.image.PixelReader;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.robot.Robot;
import javafx.scene.text.Font;
import javafx.scene.text.Text;
import javafx.scene.transform.Transform;
import javafx.stage.Screen;
import javafx.stage.Stage;
import javafx.util.Duration;
import javax.imageio.ImageIO;

/** SP-9: renders the review-table cell state stack and checks focus, ellipsis and scrolling. */
public final class Sp9CellStates extends Application {

  private static final List<String> STATES = List.of("confidence-confirm", "confidence-select",
      "confidence-correct", "user-entered", "confirmed", "double-confirmed", "deviation-minor",
      "deviation-major", "no-reference", "reference-outdated", "needs-confirmation");

  /**
   * One field of a row.
   *
   * @param value the displayed value
   * @param states the pseudo-class names to set
   * @param delta the delta badge text, or an empty string
   */
  public record Field(String value, Set<String> states, String delta) {
  }

  /**
   * One commodity row.
   *
   * @param name the commodity name
   * @param flag whether the row marker is shown
   * @param price the price field
   * @param scu the SCU field
   * @param status the status field
   */
  public record RowData(String name, boolean flag, Field price, Field scu, Field status) {
  }

  private TableView<RowData> table;
  private Stage stage;
  private Path out;
  private String tag;

  /**
   * Starts the program.
   *
   * @param args stylesheet directory, font directory, output directory, selection mode (cell or row)
   */
  public static void main(String[] args) {
    launch(args);
  }

  /**
   * Builds the table and runs the checks.
   *
   * @param primary the primary stage
   */
  @Override
  public void start(Stage primary) {
    stage = primary;
    Path cssDir = Path.of(getParameters().getRaw().get(0));
    Path fonts = Path.of(getParameters().getRaw().get(1));
    out = Path.of(getParameters().getRaw().get(2));
    String mode = getParameters().getRaw().get(3);
    for (String f : List.of("IBMPlexSans-Regular.ttf", "IBMPlexSans-SemiBold.ttf", "IBMPlexSans-Bold.ttf",
        "JetBrainsMonoNL-Regular.ttf")) {
      Font.loadFont(fonts.resolve(f).toUri().toString(), 14);
    }
    String scale = System.getProperty("glass.win.uiScale", "default");
    tag = mode + "-scale-" + scale;

    table = new TableView<>();
    table.getStyleClass().add("review-table");
    TableColumn<RowData, RowData> slot = column("", 8, "slot");
    TableColumn<RowData, RowData> marker = column("", 24, "marker");
    TableColumn<RowData, RowData> name = column("COMMODITY", 170, "name");
    TableColumn<RowData, RowData> price = column("PRICE", 190, "num");
    TableColumn<RowData, RowData> scu = column("SCU", 130, "num");
    TableColumn<RowData, RowData> status = column("STATUS", 150, "num");
    table.getColumns().addAll(List.of(slot, marker, name, price, scu, status));
    table.getItems().addAll(rows());
    table.getSelectionModel().setCellSelectionEnabled(mode.equals("cell"));

    Label summary = new Label("Summary header (outside the table)");
    summary.setStyle("-fx-text-fill: #e8eef4; -fx-padding: 6 12;");
    VBox root = new VBox(summary, table);
    VBox.setVgrow(table, Priority.ALWAYS);
    Scene scene = new Scene(root, 700, 420);
    scene.getStylesheets().add(cssDir.resolve("sp9.css").toUri().toString());
    stage.setScene(scene);
    stage.setTitle("SP-9 " + tag);
    stage.setX(40);
    stage.setY(40);
    stage.show();
    System.out.println("tag=" + tag + " stage outputScale=" + stage.getOutputScaleX() + "x" + stage.getOutputScaleY()
        + " renderScale=" + stage.getRenderScaleX() + " screen outputScale=" + Screen.getPrimary().getOutputScaleX()
        + " screen bounds=" + Screen.getPrimary().getBounds());

    table.getSelectionModel().clearAndSelect(1, mode.equals("cell") ? price : null);
    table.requestFocus();
    table.getFocusModel().focus(3, price);

    after(600, () -> {
      Robot robot = new Robot();
      TableRow<?> hoverRow = row(2);
      Bounds hb = hoverRow.localToScreen(hoverRow.getBoundsInLocal());
      robot.mouseMove(new Point2D(hb.getMinX() + 300, hb.getCenterY()));
      after(400, () -> {
        System.out.println("table pseudo-classes=" + table.getPseudoClassStates());
        for (int r = 0; r < 4; r++) {
          System.out.println("row " + r + " pseudo-classes=" + row(r).getPseudoClassStates());
        }
        TableCell<?, ?> fc = cell(3, price);
        System.out.println("focused cell (3, price) pseudo-classes=" + fc.getPseudoClassStates()
            + " isFocused()=" + fc.isFocused() + " focusVisible=" + fc.isFocusVisible());
        printBorder("focused cell (3, price) confirm+major+focus", fc);
        printBorder("cell (1, scu) confirmed+no-reference", cell(1, scu));
        printBorder("cell (0, status) user-entered", cell(0, status));
        printBorder("cell (3, scu) double-confirmed", cell(3, scu));
        for (int r : new int[] {1, 3}) {
          TableCell<?, ?> nc = cell(r, name);
          Text t = (Text) nc.lookup(".text");
          System.out.println("name cell row " + r + " item='" + ((RowData) nc.getItem()).name() + "' displayed='"
              + (t == null ? "null" : t.getText()) + "' overrun=" + nc.getTextOverrun()
              + " ellipsisString='" + nc.getEllipsisString() + "' cellHeight=" + nc.getHeight());
        }
        save("focus-a-confirm-major");
        table.getFocusModel().focus(0, scu);
        after(300, () -> {
          printBorder("focused cell (0, scu) no state", cell(0, scu));
          save("focus-b-no-state");
          table.getFocusModel().focus(1, scu);
          after(300, () -> {
            printBorder("focused cell (1, scu) confirmed+no-reference", cell(1, scu));
            save("focus-c-confirmed-noref");
            keyboardAndScroll(robot, price);
          });
        });
      });
    });
  }

  private void keyboardAndScroll(Robot robot, TableColumn<RowData, RowData> price) {
    stage.toFront();
    stage.requestFocus();
    table.requestFocus();
    table.getFocusModel().focus(3, price);
    after(300, () -> {
      robot.keyPress(KeyCode.DOWN);
      robot.keyRelease(KeyCode.DOWN);
      after(300, () -> {
        var pos = table.getFocusModel().getFocusedCell();
        TableCell<?, ?> c = cell(pos.getRow(), pos.getTableColumn());
        System.out.println("after keyboard DOWN: focused position row=" + pos.getRow() + " table.isFocusVisible="
            + table.isFocusVisible() + " table pseudo=" + table.getPseudoClassStates());
        if (c != null) {
          System.out.println("   focused cell pseudo-classes=" + c.getPseudoClassStates() + " focusVisible="
              + c.isFocusVisible());
        }
        System.out.println("   row " + pos.getRow() + " pseudo-classes=" + row(pos.getRow()).getPseudoClassStates());
        Node header = table.lookup(".column-header-background");
        double headerBottom = header.localToScene(header.getBoundsInLocal()).getMaxY();
        VirtualFlow<?> flow = (VirtualFlow<?>) table.lookup(".virtual-flow");
        Bounds flowB = flow.localToScene(flow.getBoundsInLocal());
        System.out.println("header bottom (scene y)=" + headerBottom + " flow top=" + flowB.getMinY() + " flow bottom="
            + flowB.getMaxY());
        table.scrollTo(25);
        after(200, () -> {
          report("table.scrollTo(25)", 25, headerBottom, flowB);
          table.scrollTo(0);
          after(200, () -> {
            flow.scrollTo(22);
            after(200, () -> {
              report("VirtualFlow.scrollTo(22) from top", 22, headerBottom, flowB);
              flow.scrollTo(4);
              after(200, () -> {
                report("VirtualFlow.scrollTo(4) from below", 4, headerBottom, flowB);
                table.getFocusModel().focus(4, price);
                save("scrollto-row4");
                Platform.exit();
              });
            });
          });
        });
      });
    });
  }

  private void report(String what, int index, double headerBottom, Bounds flowB) {
    TableRow<?> r = row(index);
    if (r == null) {
      System.out.println(what + ": row " + index + " not realised");
      return;
    }
    Bounds b = r.localToScene(r.getBoundsInLocal());
    System.out.printf("%s: row %d scene y %.1f..%.1f; below header=%s; fully inside viewport=%s%n", what, index,
        b.getMinY(), b.getMaxY(), b.getMinY() >= headerBottom - 0.01, b.getMinY() >= flowB.getMinY() - 0.01
            && b.getMaxY() <= flowB.getMaxY() + 0.01);
  }

  private void printBorder(String what, TableCell<?, ?> c) {
    StringBuilder sb = new StringBuilder(what + ": bg=");
    sb.append(c.getBackground() == null ? "none" : c.getBackground().getFills().getLast().getFill());
    if (c.getBorder() != null) {
      for (var s : c.getBorder().getStrokes()) {
        sb.append(" | stroke ").append(s.getTopStroke()).append(" w=").append(s.getWidths().getTop())
            .append(" inset=").append(s.getInsets().getTop()).append(" style=")
            .append(s.getTopStyle().getDashArray().isEmpty() ? "solid" : "dashed" + s.getTopStyle().getDashArray());
      }
    }
    System.out.println(sb);
  }

  private TableRow<?> row(int index) {
    for (Node n : table.lookupAll(".table-row-cell")) {
      if (n instanceof TableRow<?> r && r.getIndex() == index && r.isVisible()) {
        return r;
      }
    }
    return null;
  }

  private TableCell<?, ?> cell(int index, TableColumn<?, ?> col) {
    for (Node n : table.lookupAll(".table-cell")) {
      if (n instanceof TableCell<?, ?> c && c.getIndex() == index && c.getTableColumn() == col && c.isVisible()) {
        return c;
      }
    }
    return null;
  }

  private void save(String name) {
    SnapshotParameters sp = new SnapshotParameters();
    sp.setTransform(Transform.scale(stage.getOutputScaleX(), stage.getOutputScaleY()));
    sp.setFill(Color.BLACK);
    Image img = stage.getScene().getRoot().snapshot(sp, null);
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
      ImageIO.write(bi, "png", out.resolve(tag + "-" + name + ".png").toFile());
    } catch (IOException e) {
      throw new UncheckedIOException(e);
    }
    System.out.println("saved " + tag + "-" + name + ".png " + w + "x" + h);
  }

  private static TableColumn<RowData, RowData> column(String title, double width, String kind) {
    TableColumn<RowData, RowData> c = new TableColumn<>(title);
    c.setPrefWidth(width);
    c.setSortable(false);
    c.setReorderable(false);
    c.setCellValueFactory(cd -> new ReadOnlyObjectWrapper<>(cd.getValue()));
    c.setCellFactory(_ -> new StateCell(kind, title));
    return c;
  }

  private static final class StateCell extends TableCell<RowData, RowData> {
    private final String kind;
    private final String title;
    private final Region lead = new Region();
    private final Label value = new Label();
    private final Label delta = new Label();
    private final HBox box;
    private final Region flag = new Region();

    StateCell(String kind, String title) {
      this.kind = kind;
      this.title = title;
      getStyleClass().add(kind);
      lead.getStyleClass().add("lead-icon");
      value.getStyleClass().add("value-text");
      delta.getStyleClass().add("delta-badge");
      Region spacer = new Region();
      HBox.setHgrow(spacer, Priority.ALWAYS);
      box = new HBox(4, lead, spacer, value, delta);
      box.setAlignment(Pos.CENTER_LEFT);
      box.prefWidthProperty().bind(widthProperty().subtract(24));
      box.setMinWidth(0);
      value.setMinWidth(0);
      flag.getStyleClass().add("flag");
    }

    @Override
    protected void updateItem(RowData item, boolean empty) {
      super.updateItem(item, empty);
      for (String s : STATES) {
        pseudoClassStateChanged(PseudoClass.getPseudoClass(s), false);
      }
      setText(null);
      setGraphic(null);
      if (empty || item == null) {
        return;
      }
      switch (kind) {
        case "name" -> setText(item.name());
        case "marker" -> setGraphic(item.flag() ? flag : null);
        case "num" -> {
          Field f = switch (title) {
            case "PRICE" -> item.price();
            case "SCU" -> item.scu();
            default -> item.status();
          };
          for (String s : f.states()) {
            pseudoClassStateChanged(PseudoClass.getPseudoClass(s), true);
          }
          value.setText(f.value());
          delta.setText(f.delta());
          delta.setVisible(!f.delta().isEmpty());
          delta.setManaged(!f.delta().isEmpty());
          setGraphic(box);
          setContentDisplay(ContentDisplay.GRAPHIC_ONLY);
        }
        default -> { }
      }
    }
  }

  private static List<RowData> rows() {
    List<RowData> rows = new ArrayList<>();
    rows.add(new RowData("Agricium (Ore)", true,
        new Field("1,315", Set.of("confidence-confirm", "deviation-major"), "Δ +273"),
        new Field("2,368", Set.of(), ""),
        new Field("Medium", Set.of("user-entered"), "")));
    rows.add(new RowData("Recycled Material Composite Extra Long Name (Raw)", true,
        new Field("1,450", Set.of("confidence-correct", "deviation-minor"), "Δ +12"),
        new Field("3,128", Set.of("confirmed", "no-reference"), ""),
        new Field("Low", Set.of("confidence-select"), "")));
    rows.add(new RowData("Laranite", false,
        new Field("2,710", Set.of("confirmed", "deviation-minor"), "Δ −8"),
        new Field("860", Set.of("confidence-confirm"), ""),
        new Field("High", Set.of(), "")));
    rows.add(new RowData("Quantanium (Raw) With An Exceptionally Long Commodity Name", true,
        new Field("1,315", Set.of("confidence-confirm", "deviation-major"), "Δ +273"),
        new Field("3,340", Set.of("double-confirmed"), ""),
        new Field("Out of stock", Set.of("no-reference"), "")));
    for (int i = 4; i < 40; i++) {
      rows.add(new RowData("Commodity " + i, false, new Field(String.format("%,d", 1000 + i * 37), Set.of(), ""),
          new Field(String.valueOf(i * 12), Set.of(), ""), new Field("Medium", Set.of(), "")));
    }
    return rows;
  }

  private static void after(double ms, Runnable r) {
    PauseTransition p = new PauseTransition(Duration.millis(ms));
    p.setOnFinished(_ -> r.run());
    p.play();
  }
}
