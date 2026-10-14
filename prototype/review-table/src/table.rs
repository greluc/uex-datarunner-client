// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The review table: a virtualised `egui_extras` table with its own cell focus, in-cell editors and a searchable combo box.

use std::ops::Range;

use egui::text::{CCursor, CCursorRange};
use egui::{
    Align, Align2, Area, CornerRadius, EventFilter, FocusDirection, FontId, Frame, Id, Key, Layout,
    Modifiers, Order, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
    vec2,
};
use egui_extras::{Column as TableColumn, TableBuilder};

use crate::filter::filter_candidates;
use crate::model::{Column, Confidence, EditorKind, Row};
use crate::nav::{CellPos, Direction, Move, next_problem, step};
use crate::style::{
    BorderStyle, CellLook, ICON_SIZE, Palette, RADIUS, ROW_HEIGHT, StateIcon, WIDTH_HAIRLINE,
    WIDTH_STATE, look,
};

const COLUMN_WIDTHS: [f32; 5] = [240.0, 160.0, 140.0, 220.0, 220.0];
const CELL_PADDING: f32 = 6.0;
const ICON_GAP: f32 = 4.0;
const BADGE_FONT_SIZE: f32 = 12.0;
const BODY_FONT_SIZE: f32 = 14.0;
const POPUP_MAX_ROWS: usize = 8;
const DASH_LENGTH: f32 = 4.0;
const GAP_LENGTH: f32 = 3.0;
const CELL_INSET: f32 = 1.0;
const BADGE_PADDING: Vec2 = vec2(8.0, 2.0);

/// The open in-cell editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Editor {
    /// A text field with its buffer.
    Text {
        /// The text typed so far.
        buffer: String,
    },
    /// A searchable combo box with its query and highlighted candidate.
    Choice {
        /// The filter text typed so far.
        query: String,
        /// The index of the candidate chosen with the arrow keys, if any.
        highlighted: Option<usize>,
    },
}

impl Editor {
    /// The text the editor's field edits: the buffer or the query.
    #[must_use]
    pub const fn text(&self) -> &String {
        match self {
            Self::Text { buffer } => buffer,
            Self::Choice { query, .. } => query,
        }
    }

    /// The highlighted candidate: the one chosen with the arrow keys, else the top match once a query is typed, else none.
    #[must_use]
    pub fn highlighted(&self) -> Option<usize> {
        match self {
            Self::Text { .. } => None,
            Self::Choice { query, highlighted } => {
                highlighted.or_else(|| (!query.trim().is_empty()).then_some(0))
            }
        }
    }
}

/// How a cell was clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Click {
    Single,
    Double,
}

#[derive(Clone, Debug)]
struct Editing {
    pos: CellPos,
    editor: Editor,
    focus_pending: bool,
}

/// A user command on the table; keyboard, mouse and the measurement driver all go through it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Move the cell focus.
    Move(Move),
    /// Jump to the next or previous field that needs attention (F8, Shift+F8).
    Problem(Direction),
    /// Focus a cell directly.
    Focus(CellPos),
    /// Open the editor of the focused cell.
    BeginEdit,
    /// Write the editor's value back and close it.
    Commit,
    /// Close the editor without a change.
    Cancel,
    /// Confirm the focused field as it is.
    Confirm,
    /// Move the highlight in the candidate list by the given number of entries.
    Highlight(isize),
    /// Pick the candidate at the given index of the filtered list.
    Pick(usize),
}

/// The review table with its rows, cell focus and editor.
pub struct ReviewTable {
    rows: Vec<Row>,
    focus: CellPos,
    editing: Option<Editing>,
    scroll_to: Option<usize>,
    scroll_offset: Option<f32>,
    drawn_rows: Range<usize>,
    palette: Palette,
}

impl ReviewTable {
    /// A table over `rows` with the focus on the first cell.
    #[must_use]
    pub fn new(rows: Vec<Row>) -> Self {
        Self {
            rows,
            focus: CellPos::new(0, Column::Commodity),
            editing: None,
            scroll_to: None,
            scroll_offset: None,
            drawn_rows: 0..0,
            palette: Palette::DARK,
        }
    }

    /// The rows.
    #[must_use]
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// The focused cell.
    #[must_use]
    pub const fn focus(&self) -> CellPos {
        self.focus
    }

    /// The open editor, if any.
    #[must_use]
    pub fn editor(&self) -> Option<&Editor> {
        self.editing.as_ref().map(|editing| &editing.editor)
    }

    /// The rows drawn in the last pass.
    #[must_use]
    pub fn drawn_rows(&self) -> Range<usize> {
        self.drawn_rows.clone()
    }

    /// The focused cell as text: row, column, value, state and Δ.
    #[must_use]
    pub fn focus_description(&self) -> String {
        let Some(row) = self.rows.get(self.focus.row) else {
            return String::new();
        };
        let field = row.field(self.focus.column);
        let cell_look = look(field, &self.palette);
        let mut parts = vec![
            format!("Row {}", self.focus.row + 1),
            self.focus.column.title().to_owned(),
            field.value.clone(),
        ];
        if !cell_look.reason.is_empty() {
            parts.push(cell_look.reason);
        }
        if let Some(badge) = cell_look.badge {
            parts.push(badge.text);
        }
        parts.join(" · ")
    }

    /// The number of fields that need attention.
    #[must_use]
    pub fn problem_count(&self) -> usize {
        self.rows
            .iter()
            .flat_map(|row| Column::ALL.map(|column| row.field(column).needs_attention()))
            .filter(|attention| *attention)
            .count()
    }

    /// Pins the vertical scroll offset in points for the measurement driver; `None` hands scrolling back to the user.
    pub const fn set_scroll_offset(&mut self, offset: Option<f32>) {
        self.scroll_offset = offset;
    }

    /// The candidates of the open choice editor, filtered by its query.
    #[must_use]
    pub fn candidates(&self) -> Vec<&'static str> {
        match &self.editing {
            Some(Editing {
                pos,
                editor: Editor::Choice { query, .. },
                ..
            }) => {
                let vocabulary = pos.column.vocabulary();
                filter_candidates(vocabulary, query)
                    .into_iter()
                    .filter_map(|index| vocabulary.get(index).copied())
                    .collect()
            }
            Some(Editing {
                editor: Editor::Text { .. },
                ..
            })
            | None => Vec::new(),
        }
    }

    /// Applies `command` to the table state.
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Move(movement) => self.move_focus(step(self.focus, movement, self.rows.len())),
            Command::Problem(direction) => {
                if let Some(pos) = next_problem(&self.rows, self.focus, direction) {
                    self.move_focus(pos);
                }
            }
            Command::Focus(pos) => {
                if self
                    .editing
                    .as_ref()
                    .is_some_and(|editing| editing.pos != pos)
                {
                    self.apply(Command::Commit);
                }
                self.move_focus(pos);
            }
            Command::BeginEdit => self.begin_edit(),
            Command::Commit => self.commit(),
            Command::Cancel => self.editing = None,
            Command::Confirm => self.confirm(),
            Command::Highlight(delta) => self.highlight(delta),
            Command::Pick(index) => {
                if let Some(Editing {
                    editor: Editor::Choice { highlighted, .. },
                    ..
                }) = &mut self.editing
                {
                    *highlighted = Some(index);
                }
                self.commit();
            }
        }
    }

    fn move_focus(&mut self, pos: CellPos) {
        self.focus = pos;
        self.scroll_to = Some(pos.row);
    }

    fn begin_edit(&mut self) {
        let Some(row) = self.rows.get(self.focus.row) else {
            return;
        };
        let editor = match self.focus.column.editor() {
            EditorKind::Text => Editor::Text {
                buffer: row.field(self.focus.column).value.clone(),
            },
            EditorKind::Choice => Editor::Choice {
                query: String::new(),
                highlighted: None,
            },
        };
        self.editing = Some(Editing {
            pos: self.focus,
            editor,
            focus_pending: true,
        });
    }

    fn commit(&mut self) {
        let candidates = self.candidates();
        let Some(editing) = self.editing.take() else {
            return;
        };
        let new_value = match &editing.editor {
            Editor::Text { buffer } => Some(buffer.trim().to_owned()),
            Editor::Choice { .. } => editing
                .editor
                .highlighted()
                .and_then(|index| candidates.get(index))
                .map(|candidate| (*candidate).to_owned()),
        };
        let Some(field) = self
            .rows
            .get_mut(editing.pos.row)
            .map(|row| row.field_mut(editing.pos.column))
        else {
            return;
        };
        if let Some(value) = new_value
            && !value.is_empty()
            && value != field.value
        {
            field.value = value;
            field.confidence = Confidence::UserEntered;
        }
    }

    fn confirm(&mut self) {
        let Some(field) = self
            .rows
            .get_mut(self.focus.row)
            .map(|row| row.field_mut(self.focus.column))
        else {
            return;
        };
        field.confidence = Confidence::Confirmed;
    }

    fn highlight(&mut self, delta: isize) {
        let count = self.candidates().len();
        if let Some(Editing { editor, .. }) = &mut self.editing {
            let next = editor.highlighted().map_or(0, |current| {
                current
                    .saturating_add_signed(delta)
                    .min(count.saturating_sub(1))
            });
            if let Editor::Choice { highlighted, .. } = editor {
                *highlighted = Some(next);
            }
        }
    }

    fn read_commands(&self, ui: &Ui) -> Vec<Command> {
        let editing = self
            .editing
            .as_ref()
            .map(|editing| editing.pos.column.editor());
        ui.input_mut(|input| {
            let mut commands = Vec::new();
            let mut take = |modifiers: Modifiers, key: Key, command: &[Command]| {
                if input.consume_key(modifiers, key) {
                    commands.extend_from_slice(command);
                }
            };
            match editing {
                None => {
                    take(Modifiers::SHIFT, Key::Tab, &[Command::Move(Move::Previous)]);
                    take(Modifiers::NONE, Key::Tab, &[Command::Move(Move::Next)]);
                    take(Modifiers::NONE, Key::ArrowUp, &[Command::Move(Move::Up)]);
                    take(
                        Modifiers::NONE,
                        Key::ArrowDown,
                        &[Command::Move(Move::Down)],
                    );
                    take(
                        Modifiers::NONE,
                        Key::ArrowLeft,
                        &[Command::Move(Move::Left)],
                    );
                    take(
                        Modifiers::NONE,
                        Key::ArrowRight,
                        &[Command::Move(Move::Right)],
                    );
                    take(
                        Modifiers::SHIFT,
                        Key::F8,
                        &[Command::Problem(Direction::Backward)],
                    );
                    take(
                        Modifiers::NONE,
                        Key::F8,
                        &[Command::Problem(Direction::Forward)],
                    );
                    take(Modifiers::NONE, Key::Enter, &[Command::BeginEdit]);
                    take(Modifiers::NONE, Key::F2, &[Command::BeginEdit]);
                    take(Modifiers::NONE, Key::Space, &[Command::Confirm]);
                }
                Some(kind) => {
                    take(Modifiers::NONE, Key::Enter, &[Command::Commit]);
                    take(Modifiers::NONE, Key::Escape, &[Command::Cancel]);
                    take(
                        Modifiers::SHIFT,
                        Key::Tab,
                        &[Command::Commit, Command::Move(Move::Previous)],
                    );
                    take(
                        Modifiers::NONE,
                        Key::Tab,
                        &[Command::Commit, Command::Move(Move::Next)],
                    );
                    match kind {
                        EditorKind::Choice => {
                            take(Modifiers::NONE, Key::ArrowDown, &[Command::Highlight(1)]);
                            take(Modifiers::NONE, Key::ArrowUp, &[Command::Highlight(-1)]);
                        }
                        EditorKind::Text => {}
                    }
                }
            }
            commands
        })
    }

    /// Reads the keyboard, applies the commands and draws the table.
    pub fn ui(&mut self, ui: &mut Ui) {
        self.handle_input(ui);
        self.draw(ui);
    }

    /// Consumes the table's keys and applies their commands; call it before any other widget reads the keyboard.
    pub fn handle_input(&mut self, ui: &Ui) {
        let editing = self.editing.is_some();
        for command in self.read_commands(ui) {
            self.apply(command);
        }
        if !editing {
            ui.memory_mut(|memory| memory.move_focus(FocusDirection::None));
        }
    }

    /// Draws the table and applies the mouse commands.
    pub fn draw(&mut self, ui: &mut Ui) {
        let mut clicked: Option<(CellPos, Click)> = None;
        let mut editor_rect: Option<Rect> = None;
        let mut drawn: Option<Range<usize>> = None;

        let mut builder = TableBuilder::new(ui)
            .id_salt("review-table")
            .striped(false)
            .resizable(false)
            .animate_scrolling(false)
            .cell_layout(Layout::left_to_right(Align::Center));
        for width in COLUMN_WIDTHS {
            builder = builder.column(TableColumn::exact(width));
        }
        if let Some(row) = self.scroll_to.take() {
            builder = builder.scroll_to_row(row, None);
        }
        if let Some(offset) = self.scroll_offset {
            builder = builder.vertical_scroll_offset(offset);
        }

        let Self {
            rows,
            focus,
            editing,
            palette,
            ..
        } = self;
        let palette = *palette;
        let focus = *focus;

        builder
            .header(ROW_HEIGHT, |mut header| {
                for column in Column::ALL {
                    header.col(|ui| {
                        ui.label(
                            egui::RichText::new(column.title())
                                .color(palette.text_secondary)
                                .size(BODY_FONT_SIZE),
                        );
                    });
                }
            })
            .body(|body| {
                body.rows(ROW_HEIGHT, rows.len(), |mut table_row| {
                    let index = table_row.index();
                    drawn = Some(match drawn.take() {
                        Some(range) => range.start.min(index)..range.end.max(index + 1),
                        None => index..index + 1,
                    });
                    let Some(row) = rows.get(index) else {
                        return;
                    };
                    for column in Column::ALL {
                        let pos = CellPos::new(index, column);
                        table_row.col(|ui| {
                            let cell_editing = editing.as_mut().filter(|e| e.pos == pos);
                            let outcome = cell_ui(ui, row, pos, focus, cell_editing, &palette);
                            if let Some(rect) = outcome.editor_rect {
                                editor_rect = Some(rect);
                            }
                            if let Some(click) = outcome.clicked {
                                clicked = Some((pos, click));
                            }
                        });
                    }
                });
            });

        self.drawn_rows = drawn.unwrap_or(0..0);

        if let Some(rect) = editor_rect
            && let Some(index) = self.candidate_popup(ui, rect)
        {
            self.apply(Command::Pick(index));
        }
        if let Some((pos, click)) = clicked {
            self.apply(Command::Focus(pos));
            match click {
                Click::Single => {}
                Click::Double => self.apply(Command::BeginEdit),
            }
        }
    }

    fn candidate_popup(&self, ui: &Ui, cell: Rect) -> Option<usize> {
        let Some(Editing {
            editor: editor @ Editor::Choice { .. },
            ..
        }) = &self.editing
        else {
            return None;
        };
        let highlighted = editor.highlighted();
        let candidates = self.candidates();
        let palette = self.palette;
        let mut picked = None;
        Area::new(Id::new("review-table-candidates"))
            .order(Order::Foreground)
            .fixed_pos(cell.left_bottom())
            .show(ui.ctx(), |ui| {
                Frame::popup(ui.style())
                    .fill(palette.surface_raised)
                    .show(ui, |ui| {
                        ui.set_width(cell.width());
                        let height = ROW_HEIGHT * POPUP_MAX_ROWS as f32;
                        egui::ScrollArea::vertical()
                            .max_height(height)
                            .show(ui, |ui| {
                                if candidates.is_empty() {
                                    ui.label(
                                        egui::RichText::new("No match")
                                            .color(palette.text_secondary),
                                    );
                                }
                                for (index, candidate) in candidates.iter().enumerate() {
                                    let selected = highlighted == Some(index);
                                    let response = ui.selectable_label(selected, *candidate);
                                    if selected {
                                        response.scroll_to_me(None);
                                    }
                                    if response.clicked() {
                                        picked = Some(index);
                                    }
                                }
                            });
                    });
            });
        picked
    }
}

struct CellOutcome {
    editor_rect: Option<Rect>,
    clicked: Option<Click>,
}

fn paint_border(ui: &Ui, rect: Rect, look: &CellLook) {
    let Some(border) = look.border else {
        return;
    };
    let stroke = Stroke::new(border.width, border.colour);
    match border.style {
        BorderStyle::Solid => {
            ui.painter()
                .rect_stroke(rect, CornerRadius::same(RADIUS), stroke, StrokeKind::Inside);
        }
        BorderStyle::Dashed => {
            let inset = rect.shrink(border.width / 2.0);
            let outline = [
                inset.left_top(),
                inset.right_top(),
                inset.right_bottom(),
                inset.left_bottom(),
                inset.left_top(),
            ];
            ui.painter().extend(Shape::dashed_line(
                &outline,
                stroke,
                DASH_LENGTH,
                GAP_LENGTH,
            ));
        }
    }
}

/// Paints `icon` with its left edge at `left`, vertically centred on `centre_y`.
pub fn paint_icon(ui: &Ui, left: f32, centre_y: f32, icon: StateIcon) {
    ui.painter().text(
        Pos2::new(left, centre_y),
        Align2::LEFT_CENTER,
        icon.icon.glyph(),
        FontId::proportional(ICON_SIZE),
        icon.colour,
    );
}

fn paint_badge(ui: &Ui, right: f32, centre_y: f32, badge: &crate::style::Badge) -> f32 {
    let galley = ui.painter().layout_no_wrap(
        badge.text.clone(),
        FontId::proportional(BADGE_FONT_SIZE),
        badge.colour,
    );
    let size = galley.size() + BADGE_PADDING;
    let rect = Rect::from_min_size(Pos2::new(right - size.x, centre_y - size.y / 2.0), size);
    match badge.fill {
        Some(fill) => {
            ui.painter()
                .rect_filled(rect, CornerRadius::same(RADIUS), fill);
        }
        None => {
            ui.painter().rect_stroke(
                rect,
                CornerRadius::same(RADIUS),
                Stroke::new(WIDTH_HAIRLINE, badge.colour),
                StrokeKind::Inside,
            );
        }
    }
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, badge.colour);
    rect.left()
}

fn cell_ui(
    ui: &mut Ui,
    row: &Row,
    pos: CellPos,
    focus: CellPos,
    editing: Option<&mut Editing>,
    palette: &Palette,
) -> CellOutcome {
    let field = row.field(pos.column);
    let rect = ui.max_rect().shrink(CELL_INSET);
    let response = ui.allocate_rect(rect, Sense::hover());
    let cell_look = look(field, palette);
    let focused_row = pos.row == focus.row;
    let fill = cell_look.background.unwrap_or(if focused_row {
        palette.selection_row_bg
    } else {
        palette.surface
    });
    ui.painter()
        .rect_filled(rect, CornerRadius::same(RADIUS), fill);
    paint_border(ui, rect, &cell_look);

    let centre_y = rect.center().y;
    let mut left = rect.left() + CELL_PADDING;
    if let Some(icon) = cell_look.confidence_icon {
        paint_icon(ui, left, centre_y, icon);
    }
    left += ICON_SIZE + ICON_GAP;

    let mut right = rect.right() - CELL_PADDING;
    if let Some(badge) = &cell_look.badge {
        right = paint_badge(ui, right, centre_y, badge) - ICON_GAP;
    }
    if let Some(icon) = cell_look.deviation_icon {
        right -= ICON_SIZE;
        paint_icon(ui, right, centre_y, icon);
        right -= ICON_GAP;
    }

    let value_rect = Rect::from_min_max(
        Pos2::new(left, rect.top()),
        Pos2::new(right.max(left), rect.bottom()),
    );
    let editor_rect = match editing {
        Some(editing) => {
            editor_ui(ui, value_rect, editing, &field.value, palette);
            Some(rect)
        }
        None => {
            let text_colour = palette.text;
            ui.painter().with_clip_rect(value_rect).text(
                Pos2::new(value_rect.left(), centre_y),
                Align2::LEFT_CENTER,
                &field.value,
                FontId::proportional(BODY_FONT_SIZE),
                text_colour,
            );
            None
        }
    };

    if pos == focus {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(RADIUS),
            Stroke::new(WIDTH_STATE, palette.focus_ring),
            StrokeKind::Outside,
        );
    }

    if !cell_look.reason.is_empty() {
        response.clone().on_hover_text(&cell_look.reason);
    }
    let clicked = if response.hovered() {
        ui.input(|input| {
            if input
                .pointer
                .button_double_clicked(egui::PointerButton::Primary)
            {
                Some(Click::Double)
            } else if input.pointer.primary_clicked() {
                Some(Click::Single)
            } else {
                None
            }
        })
    } else {
        None
    };
    CellOutcome {
        editor_rect,
        clicked,
    }
}

fn editor_ui(ui: &mut Ui, rect: Rect, editing: &mut Editing, current: &str, palette: &Palette) {
    let filter = EventFilter {
        tab: true,
        horizontal_arrows: true,
        vertical_arrows: true,
        escape: true,
    };
    let id = Id::new((
        "review-table-editor",
        editing.pos.row,
        editing.pos.column.index(),
    ));
    let (buffer, hint) = match &mut editing.editor {
        Editor::Text { buffer } => (buffer, ""),
        Editor::Choice { query, .. } => (query, current),
    };
    let output = ui
        .scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            TextEdit::singleline(buffer)
                .id(id)
                .frame(Frame::NONE)
                .hint_text(hint)
                .text_color(palette.text)
                .font(FontId::proportional(BODY_FONT_SIZE))
                .desired_width(rect.width())
                .event_filter(filter)
                .show(ui)
        })
        .inner;
    if editing.focus_pending {
        editing.focus_pending = false;
        output.response.request_focus();
        let mut state = output.state;
        let end = CCursor::new(editing.editor.text().chars().count());
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(0), end)));
        state.store(ui.ctx(), output.response.id);
    }
}
