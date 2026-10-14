// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keyboard navigation over the cell grid: Tab order, arrows and F8 / Shift+F8.

use crate::model::{Column, Row};

/// The position of a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellPos {
    /// The row index.
    pub row: usize,
    /// The column.
    pub column: Column,
}

impl CellPos {
    /// The cell at `row` and `column`.
    #[must_use]
    pub const fn new(row: usize, column: Column) -> Self {
        Self { row, column }
    }

    const fn linear(self) -> usize {
        self.row * Column::ALL.len() + self.column.index()
    }

    fn from_linear(index: usize) -> Option<Self> {
        let column = Column::from_index(index % Column::ALL.len())?;
        Some(Self::new(index / Column::ALL.len(), column))
    }
}

/// A focus move by one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// Tab: the next cell in reading order.
    Next,
    /// Shift+Tab: the previous cell in reading order.
    Previous,
    /// The cell above.
    Up,
    /// The cell below.
    Down,
    /// The cell to the left.
    Left,
    /// The cell to the right.
    Right,
}

/// The search direction of F8 (forward) and Shift+F8 (backward).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// Towards the end of the table.
    Forward,
    /// Towards the start of the table.
    Backward,
}

/// The cell after applying `movement` to `from` in a table of `row_count` rows; the move stops at the grid edges.
#[must_use]
pub fn step(from: CellPos, movement: Move, row_count: usize) -> CellPos {
    let last_row = row_count.saturating_sub(1);
    let last_linear = row_count
        .saturating_mul(Column::ALL.len())
        .saturating_sub(1);
    let target = match movement {
        Move::Next => CellPos::from_linear((from.linear() + 1).min(last_linear)),
        Move::Previous => CellPos::from_linear(from.linear().saturating_sub(1)),
        Move::Up => Some(CellPos::new(from.row.saturating_sub(1), from.column)),
        Move::Down => Some(CellPos::new((from.row + 1).min(last_row), from.column)),
        Move::Left => Column::from_index(from.column.index().saturating_sub(1))
            .map(|column| CellPos::new(from.row, column)),
        Move::Right => {
            Column::from_index(from.column.index() + 1).map(|column| CellPos::new(from.row, column))
        }
    };
    target.unwrap_or(from)
}

/// The next cell after `from` in `direction` that needs attention, wrapping around; `from` itself is found last.
#[must_use]
pub fn next_problem(rows: &[Row], from: CellPos, direction: Direction) -> Option<CellPos> {
    let total = rows.len() * Column::ALL.len();
    if total == 0 {
        return None;
    }
    let start = from.linear() % total;
    (1..=total)
        .map(|offset| match direction {
            Direction::Forward => (start + offset) % total,
            Direction::Backward => (start + total - offset % total) % total,
        })
        .filter_map(CellPos::from_linear)
        .find(|pos| {
            rows.get(pos.row)
                .is_some_and(|row| row.field(pos.column).needs_attention())
        })
}
