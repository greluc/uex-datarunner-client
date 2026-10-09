// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import javafx.animation.PauseTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.css.PseudoClass;
import javafx.css.TransitionEvent;
import javafx.scene.Scene;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.stage.Stage;
import javafx.util.Duration;

/** SP-1: checks whether size, radius and duration lookups resolve, including em values. */
public final class Sp1Lookups extends Application {

  private static final PseudoClass HOVER_SIM = PseudoClass.getPseudoClass("hover-sim");

  private final Map<String, Long> startNanos = new LinkedHashMap<>();

  /**
   * Starts the program.
   *
   * @param args the stylesheet directory
   */
  public static void main(String[] args) {
    launch(args);
  }

  /**
   * Builds the scene and prints the resolved values.
   *
   * @param stage the primary stage
   */
  @Override
  public void start(Stage stage) {
    Path cssDir = Path.of(getParameters().getRaw().getFirst());
    Region px = region("pad-px");
    Region mixed = region("pad-mixed");
    Region alias = region("pad-alias");
    Region emRoot = region("pad-em-root-defined");
    Region emLocal = region("pad-em-local-defined");
    Region emLiteral = region("pad-em-literal");
    Label fontLookup = new Label("font lookup");
    fontLookup.getStyleClass().add("font-lookup");
    VBox spacing = new VBox(new Label("a"), new Label("b"));
    spacing.getStyleClass().add("box-spacing");
    VBox compact = new VBox();
    compact.getStyleClass().add("density-compact");
    Region compactPx = region("pad-px");
    VBox compactSpacing = new VBox(new Label("c"), new Label("d"));
    compactSpacing.getStyleClass().add("box-spacing");
    compact.getChildren().addAll(compactPx, compactSpacing);
    Region tShort = region("trans-shorthand");
    Region tLong = region("trans-longhand");
    Region tLit = region("trans-literal");
    for (Region r : List.of(tShort, tLong, tLit)) {
      r.setPrefSize(50, 20);
      r.addEventHandler(TransitionEvent.ANY, e -> onTransition(r, e));
    }
    Label tipOwner = new Label("tooltip owner");
    Tooltip tip = new Tooltip("tip");
    tipOwner.setTooltip(tip);

    VBox root = new VBox(4, px, mixed, alias, emRoot, emLocal, emLiteral, fontLookup, spacing,
        compact, tShort, tLong, tLit, tipOwner);
    Scene scene = new Scene(root, 400, 700);
    scene.getStylesheets().add(cssDir.resolve("sp1.css").toUri().toString());
    stage.setScene(scene);
    stage.show();

    root.applyCss();
    root.layout();
    System.out.println("root font size (unstyled label) = " + tipOwner.getFont().getSize());
    print("pad-px", px);
    print("pad-mixed", mixed);
    print("pad-alias", alias);
    print("pad-em-root-defined (node font 20px, root 10px)", emRoot);
    print("pad-em-local-defined (node font 20px)", emLocal);
    print("pad-em-literal (node font 20px)", emLiteral);
    System.out.println("font-lookup label font size = " + fontLookup.getFont().getSize());
    System.out.println("box-spacing VBox spacing = " + spacing.getSpacing());
    print("pad-px inside .density-compact", compactPx);
    System.out.println("box-spacing inside .density-compact spacing = " + compactSpacing.getSpacing());
    Region single = region("probe-single-sizes");
    Region padFour = region("probe-pad-four");
    Label gapRoot = new Label("g");
    gapRoot.getStyleClass().add("probe-gap-root-em");
    Label gapLocal = new Label("g");
    gapLocal.getStyleClass().add("probe-gap-local-em");
    Label labelPad = new Label("p");
    labelPad.getStyleClass().add("probe-label-padding");
    javafx.scene.control.ListView<String> list = new javafx.scene.control.ListView<>();
    list.getStyleClass().add("probe-fixed-cell");
    root.getChildren().addAll(single, padFour, gapRoot, gapLocal, labelPad, list);
    root.applyCss();
    System.out.println("probe-single-sizes prefWidth=" + single.getPrefWidth() + " minHeight="
        + single.getMinHeight());
    System.out.println("probe-pad-four padding=" + padFour.getPadding());
    System.out.println("probe-gap-root-em (label 20px, root 10px, lookup 1.5em on root) gap="
        + gapRoot.getGraphicTextGap() + " font=" + gapRoot.getFont().getSize());
    System.out.println("probe-gap-local-em (label 20px, lookup 1.5em on the label) gap="
        + gapLocal.getGraphicTextGap());
    System.out.println("probe-label-padding labelPadding=" + labelPad.getLabelPadding());
    System.out.println("probe-fixed-cell ListView fixedCellSize=" + list.getFixedCellSize());

    PauseTransition before = new PauseTransition(Duration.millis(300));
    before.setOnFinished(_ -> {
      for (Region r : List.of(tShort, tLong, tLit)) {
        startNanos.put(r.getStyleClass().getFirst(), System.nanoTime());
        r.pseudoClassStateChanged(HOVER_SIM, true);
      }
      PauseTransition mid = new PauseTransition(Duration.millis(200));
      mid.setOnFinished(_ -> {
        for (Region r : List.of(tShort, tLong, tLit)) {
          System.out.println("at ~200ms " + r.getStyleClass().getFirst() + " fill = "
              + r.getBackground().getFills().getFirst().getFill());
        }
      });
      mid.play();
      PauseTransition after = new PauseTransition(Duration.millis(900));
      after.setOnFinished(_ -> {
        for (Region r : List.of(tShort, tLong, tLit)) {
          System.out.println("at ~900ms " + r.getStyleClass().getFirst() + " fill = "
              + r.getBackground().getFills().getFirst().getFill());
        }
        tip.show(stage, stage.getX() + 50, stage.getY() + 50);
        PauseTransition tipWait = new PauseTransition(Duration.millis(300));
        tipWait.setOnFinished(_ -> {
          System.out.println("tooltip showDelay = " + tip.getShowDelay()
              + ", showDuration = " + tip.getShowDuration()
              + ", hideDelay = " + tip.getHideDelay()
              + ", showDuration indefinite? " + tip.getShowDuration().isIndefinite());
          Tooltip plain = new Tooltip("plain");
          System.out.println("unstyled tooltip defaults: showDelay = " + plain.getShowDelay()
              + ", showDuration = " + plain.getShowDuration() + ", hideDelay = "
              + plain.getHideDelay());
          tip.hide();
          Platform.exit();
        });
        tipWait.play();
      });
      after.play();
    });
    before.play();
  }

  private void onTransition(Region r, TransitionEvent e) {
    String key = r.getStyleClass().getFirst();
    long ms = (System.nanoTime() - startNanos.getOrDefault(key, System.nanoTime())) / 1_000_000;
    System.out.println("TransitionEvent " + key + " " + e.getEventType() + " property="
        + e.getPropertyName() + " elapsed=" + e.getElapsedTime() + " wall=" + ms + "ms");
  }

  private static Region region(String styleClass) {
    Region r = new Region();
    r.getStyleClass().add(styleClass);
    r.setMinHeight(10);
    return r;
  }

  private static void print(String name, Region r) {
    StringBuilder sb = new StringBuilder(name).append(": padding=").append(r.getPadding());
    if (r.getBackground() != null) {
      var fills = r.getBackground().getFills();
      for (int i = 0; i < fills.size(); i++) {
        sb.append(" | fill[").append(i).append("] ").append(fills.get(i).getFill())
            .append(" radii=").append(fills.get(i).getRadii().getTopLeftHorizontalRadius())
            .append(" insets=").append(fills.get(i).getInsets());
      }
    }
    if (r.getBorder() != null) {
      var stroke = r.getBorder().getStrokes().getFirst();
      sb.append(" | border widths=").append(stroke.getWidths()).append(" radius=")
          .append(stroke.getRadii().getTopLeftHorizontalRadius()).append(" colour=")
          .append(stroke.getTopStroke());
    }
    sb.append(" | prefWidth=").append(r.getPrefWidth());
    System.out.println(sb);
  }
}
