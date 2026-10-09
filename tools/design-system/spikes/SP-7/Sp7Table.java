// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.awt.image.BufferedImage;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.IntStream;
import javafx.animation.AnimationTimer;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.beans.property.SimpleStringProperty;
import javafx.css.PseudoClass;
import javafx.css.TransitionEvent;
import javafx.scene.Scene;
import javafx.scene.control.TableCell;
import javafx.scene.control.TableColumn;
import javafx.scene.control.TableView;
import javafx.scene.control.skin.VirtualFlow;
import javafx.scene.image.Image;
import javafx.scene.image.PixelReader;
import javafx.stage.Stage;
import javax.imageio.ImageIO;

/** SP-7: measures frame timing of a 200-row state-styled TableView while scrolling. */
public final class Sp7Table extends Application {

  private static final List<PseudoClass> CONFIDENCE = List.of(PseudoClass.getPseudoClass("confidence-confirm"),
      PseudoClass.getPseudoClass("confidence-correct"), PseudoClass.getPseudoClass("confirmed"));
  private static final List<PseudoClass> DEVIATION = List.of(PseudoClass.getPseudoClass("deviation-minor"),
      PseudoClass.getPseudoClass("deviation-major"), PseudoClass.getPseudoClass("no-reference"));
  private static final int FRAMES = 400;

  private static int transitionStarts;
  private static int cellTransitionStarts;

  /**
   * Starts the program.
   *
   * @param args stylesheet directory, output directory, variant (focus-effect, effect-all, no-effect), reduced motion
   */
  public static void main(String[] args) {
    launch(args);
  }

  /**
   * Builds the table and scrolls it.
   *
   * @param stage the primary stage
   */
  @Override
  public void start(Stage stage) {
    Path cssDir = Path.of(getParameters().getRaw().get(0));
    Path out = Path.of(getParameters().getRaw().get(1));
    String variant = getParameters().getRaw().get(2);
    boolean reduced = Boolean.parseBoolean(getParameters().getRaw().get(3));
    TableView<Integer> table = new TableView<>();
    table.getStyleClass().add(variant);
    for (int c = 0; c < 6; c++) {
      int col = c;
      TableColumn<Integer, String> tc = new TableColumn<>("Field " + c);
      tc.setPrefWidth(140);
      tc.setCellValueFactory(cd -> new SimpleStringProperty(String.format("%,d", cd.getValue() * 37 + col)));
      tc.setCellFactory(_ -> new StateCell(col));
      table.getColumns().add(tc);
    }
    table.getItems().addAll(IntStream.range(0, 200).boxed().toList());
    table.getSelectionModel().setCellSelectionEnabled(true);
    table.addEventFilter(TransitionEvent.START, _ -> transitionStarts++);
    Scene scene = new Scene(table, 900, 700);
    scene.getStylesheets().add(cssDir.resolve("sp7.css").toUri().toString());
    scene.getPreferences().setReducedMotion(reduced);
    stage.setScene(scene);
    stage.setTitle("SP-7 " + variant);
    stage.show();
    table.requestFocus();
    System.out.println("variant=" + variant + " reducedMotion=" + scene.getPreferences().isReducedMotion()
        + " prism.order=" + System.getProperty("prism.order"));

    List<Double> deltas = new ArrayList<>();
    new AnimationTimer() {
      private long last;
      private int frame;

      @Override
      public void handle(long now) {
        if (last != 0) {
          deltas.add((now - last) / 1e6);
        }
        last = now;
        VirtualFlow<?> flow = (VirtualFlow<?>) table.lookup(".virtual-flow");
        if (frame == 30) {
          System.out.println("MEASURE-START");
          deltas.clear();
          transitionStarts = 0;
          cellTransitionStarts = 0;
        }
        if (frame >= 30) {
          flow.scrollPixels(frame < 30 + FRAMES / 2 ? 12 : -12);
          int first = flow.getFirstVisibleCell() == null ? 0 : flow.getFirstVisibleCell().getIndex();
          table.getFocusModel().focus(first + 5, table.getColumns().get(frame % 6));
        }
        if (frame == 30 + FRAMES) {
          stop();
          save(scene.snapshot(null), out.resolve(variant + (reduced ? "-reduced" : "") + "-"
              + System.getProperty("prism.order", "default") + ".png"));
          System.out.println("MEASURE-END");
          double[] d = deltas.stream().mapToDouble(Double::doubleValue).sorted().toArray();
          double mean = java.util.Arrays.stream(d).average().orElse(0);
          System.out.printf("frames=%d meanDelta=%.2fms p50=%.2fms p95=%.2fms max=%.2fms fps=%.1f transitionStarts=%d cellHandlerStarts=%d%n",
              d.length, mean, d[d.length / 2], d[(int) (d.length * 0.95)], d[d.length - 1], 1000 / mean, transitionStarts, cellTransitionStarts);
          cellTransitionStarts = 0;
          int flipped = 0;
          for (var n : table.lookupAll(".table-cell")) {
            if (n instanceof StateCell sc && sc.isVisible() && !sc.isEmpty()) {
              sc.pseudoClassStateChanged(DEVIATION.get(1), !sc.getPseudoClassStates().contains(DEVIATION.get(1)));
              flipped++;
            }
          }
          int flippedCells = flipped;
          javafx.animation.PauseTransition p = new javafx.animation.PauseTransition(javafx.util.Duration.millis(400));
          p.setOnFinished(_ -> {
            System.out.println("in-place flip of deviation-major on " + flippedCells
                + " visible cells -> TransitionEvents on cells=" + cellTransitionStarts);
            Platform.exit();
          });
          p.play();
        }
        frame++;
      }
    }.start();
  }

  private static final class StateCell extends TableCell<Integer, String> {
    private final int col;

    StateCell(int col) {
      addEventHandler(TransitionEvent.ANY, _ -> cellTransitionStarts++);
      this.col = col;
    }

    @Override
    protected void updateItem(String item, boolean empty) {
      super.updateItem(item, empty);
      setText(empty ? null : item);
      int row = getIndex();
      for (int i = 0; i < 3; i++) {
        pseudoClassStateChanged(CONFIDENCE.get(i), !empty && (row * 7 + col * 3) % 6 == i);
        pseudoClassStateChanged(DEVIATION.get(i), !empty && (row + col * 5) % 5 == i);
      }
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
