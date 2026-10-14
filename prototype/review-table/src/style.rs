// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Tallyline cell states as colours, borders, icons and badges, from the dark theme of `ui/design-tokens/`.

use egui::Color32;

use crate::model::{Confidence, Deviation, Field};

/// The colour tokens the review table uses, resolved from one theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// `surface-1`.
    pub surface: Color32,
    /// `surface-2`.
    pub surface_raised: Color32,
    /// `text-primary`.
    pub text: Color32,
    /// `text-secondary`.
    pub text_secondary: Color32,
    /// `focus-ring`.
    pub focus_ring: Color32,
    /// `selection-row-bg`.
    pub selection_row_bg: Color32,
    /// `confidence-confirm-fg` and `-border`.
    pub confidence_confirm: Color32,
    /// `confidence-select-fg` and `-border`.
    pub confidence_select: Color32,
    /// `confidence-correct-fg` and `-border`.
    pub confidence_correct: Color32,
    /// `user-entered-fg` and `-border`.
    pub user_entered: Color32,
    /// `confirmed-fg` and `-border`.
    pub confirmed: Color32,
    /// `double-confirmed-fg` and `-border`.
    pub double_confirmed: Color32,
    /// `deviation-minor-bg`.
    pub deviation_minor_bg: Color32,
    /// `deviation-minor-fg` and `-border`.
    pub deviation_minor_fg: Color32,
    /// `deviation-major-bg`.
    pub deviation_major_bg: Color32,
    /// `deviation-major-fg`.
    pub deviation_major_fg: Color32,
    /// `deviation-major-badge-bg`.
    pub deviation_major_badge_bg: Color32,
    /// `deviation-major-badge-fg`.
    pub deviation_major_badge_fg: Color32,
    /// `no-reference-bg`.
    pub no_reference_bg: Color32,
    /// `no-reference-fg`.
    pub no_reference_fg: Color32,
    /// `no-reference-border`.
    pub no_reference_border: Color32,
}

impl Palette {
    /// The dark theme, design-system version 1791542897-a5d3.
    pub const DARK: Self = Self {
        surface: Color32::from_rgb(0x17, 0x26, 0x35),
        surface_raised: Color32::from_rgb(0x1d, 0x2e, 0x3f),
        text: Color32::from_rgb(0xe8, 0xee, 0xf4),
        text_secondary: Color32::from_rgb(0xb3, 0xc2, 0xd1),
        focus_ring: Color32::from_rgb(0xb4, 0xe0, 0xfa),
        selection_row_bg: Color32::from_rgb(0x1f, 0x3a, 0x52),
        confidence_confirm: Color32::from_rgb(0xe5, 0xc2, 0x5c),
        confidence_select: Color32::from_rgb(0xe5, 0xc2, 0x5c),
        confidence_correct: Color32::from_rgb(0xff, 0x80, 0x78),
        user_entered: Color32::from_rgb(0xc9, 0xd3, 0xdf),
        confirmed: Color32::from_rgb(0x52, 0xcf, 0xb2),
        double_confirmed: Color32::from_rgb(0x52, 0xcf, 0xb2),
        deviation_minor_bg: Color32::from_rgb(0x2a, 0x29, 0x13),
        deviation_minor_fg: Color32::from_rgb(0xe6, 0xd6, 0x7a),
        deviation_major_bg: Color32::from_rgb(0x62, 0x30, 0x0c),
        deviation_major_fg: Color32::from_rgb(0xff, 0x94, 0x40),
        deviation_major_badge_bg: Color32::from_rgb(0xff, 0x94, 0x40),
        deviation_major_badge_fg: Color32::from_rgb(0x1a, 0x0d, 0x02),
        no_reference_bg: Color32::from_rgb(0x26, 0x2d, 0x37),
        no_reference_fg: Color32::from_rgb(0xc2, 0xcc, 0xd8),
        no_reference_border: Color32::from_rgb(0x71, 0x89, 0xa2),
    };
}

/// `width-hairline` in points.
pub const WIDTH_HAIRLINE: f32 = 1.0;
/// `width-state` in points.
pub const WIDTH_STATE: f32 = 2.0;
/// `size-row-compact` in points.
pub const ROW_HEIGHT: f32 = 28.0;
/// `size-icon-sm` in points.
pub const ICON_SIZE: f32 = 16.0;
/// `radius-sm` in points.
pub const RADIUS: u8 = 2;

/// A state icon, drawn as a stand-in glyph from egui's default fonts for the Tallyline icon font (O-112).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    /// `help`.
    Help,
    /// `alt_route`.
    AltRoute,
    /// `edit_square`.
    EditSquare,
    /// `edit`.
    Edit,
    /// `check_circle`.
    CheckCircle,
    /// `done_all`.
    DoneAll,
    /// `compare_arrows`.
    CompareArrows,
    /// `crisis_alert`.
    CrisisAlert,
    /// `fiber_new`.
    FiberNew,
}

impl Icon {
    /// Every icon.
    pub const ALL: [Self; 9] = [
        Self::Help,
        Self::AltRoute,
        Self::EditSquare,
        Self::Edit,
        Self::CheckCircle,
        Self::DoneAll,
        Self::CompareArrows,
        Self::CrisisAlert,
        Self::FiberNew,
    ];

    /// The stand-in glyph.
    #[must_use]
    pub const fn glyph(self) -> char {
        match self {
            Self::Help => '❓',
            Self::AltRoute => '🔀',
            Self::EditSquare => '📝',
            Self::Edit => '✏',
            Self::CheckCircle => '✔',
            Self::DoneAll => '☑',
            Self::CompareArrows => '↔',
            Self::CrisisAlert => '⚠',
            Self::FiberNew => '🆕',
        }
    }

    /// The Material Symbols name of the Tallyline icon it stands in for.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::AltRoute => "alt_route",
            Self::EditSquare => "edit_square",
            Self::Edit => "edit",
            Self::CheckCircle => "check_circle",
            Self::DoneAll => "done_all",
            Self::CompareArrows => "compare_arrows",
            Self::CrisisAlert => "crisis_alert",
            Self::FiberNew => "fiber_new",
        }
    }
}

/// How a border is stroked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorderStyle {
    /// A continuous line.
    Solid,
    /// A dashed line.
    Dashed,
}

/// The border of a cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    /// The stroke colour.
    pub colour: Color32,
    /// The stroke width in points.
    pub width: f32,
    /// The stroke style.
    pub style: BorderStyle,
}

/// An icon in its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateIcon {
    /// The icon.
    pub icon: Icon,
    /// Its colour.
    pub colour: Color32,
}

/// The Δ badge of a deviating cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Badge {
    /// The badge text, for example `Δ +12 %`.
    pub text: String,
    /// The fill of a filled badge; `None` draws an outlined badge.
    pub fill: Option<Color32>,
    /// The text colour, also the outline colour of an outlined badge.
    pub colour: Color32,
}

/// Everything the table draws for one cell besides its value.
#[derive(Clone, Debug, PartialEq)]
pub struct CellLook {
    /// The deviation background.
    pub background: Option<Color32>,
    /// The confidence border, or the dashed no-reference border.
    pub border: Option<Border>,
    /// The confidence icon.
    pub confidence_icon: Option<StateIcon>,
    /// The deviation icon.
    pub deviation_icon: Option<StateIcon>,
    /// The Δ badge.
    pub badge: Option<Badge>,
    /// The state as text, so that no state is shown by colour alone (R-UI-4, R-UI-11).
    pub reason: String,
}

fn delta_text(delta_percent: i32) -> String {
    let sign = if delta_percent < 0 { '−' } else { '+' };
    format!("Δ {sign}{} %", delta_percent.unsigned_abs())
}

fn confidence_part(
    confidence: Confidence,
    palette: &Palette,
) -> Option<(StateIcon, Border, &'static str)> {
    let (icon, colour, width, label) = match confidence {
        Confidence::Ok => return None,
        Confidence::Confirm => (
            Icon::Help,
            palette.confidence_confirm,
            WIDTH_STATE,
            "Confirm: below send threshold",
        ),
        Confidence::Select => (
            Icon::AltRoute,
            palette.confidence_select,
            WIDTH_STATE,
            "Select: n candidates",
        ),
        Confidence::Correct => (
            Icon::EditSquare,
            palette.confidence_correct,
            WIDTH_STATE,
            "Correct: unreadable",
        ),
        Confidence::UserEntered => (
            Icon::Edit,
            palette.user_entered,
            WIDTH_STATE,
            "Entered by you",
        ),
        Confidence::Confirmed => (
            Icon::CheckCircle,
            palette.confirmed,
            WIDTH_STATE,
            "Confirmed",
        ),
        Confidence::DoubleConfirmed => (
            Icon::DoneAll,
            palette.double_confirmed,
            WIDTH_HAIRLINE,
            "OCR and AI agree",
        ),
    };
    Some((
        StateIcon { icon, colour },
        Border {
            colour,
            width,
            style: BorderStyle::Solid,
        },
        label,
    ))
}

/// The look of `field`: confidence as icon and border, deviation as background, icon and Δ badge (R-UI-4, R-UI-10).
#[must_use]
pub fn look(field: &Field, palette: &Palette) -> CellLook {
    let confidence = confidence_part(field.confidence, palette);
    let (background, deviation_icon, badge, deviation_label) = match field.deviation {
        Deviation::Equal => (None, None, None, None),
        Deviation::Minor { delta_percent } => (
            Some(palette.deviation_minor_bg),
            Some(StateIcon {
                icon: Icon::CompareArrows,
                colour: palette.deviation_minor_fg,
            }),
            Some(Badge {
                text: delta_text(delta_percent),
                fill: None,
                colour: palette.deviation_minor_fg,
            }),
            Some("Minor deviation – please cross-check"),
        ),
        Deviation::Major { delta_percent } => (
            Some(palette.deviation_major_bg),
            Some(StateIcon {
                icon: Icon::CrisisAlert,
                colour: palette.deviation_major_fg,
            }),
            Some(Badge {
                text: delta_text(delta_percent),
                fill: Some(palette.deviation_major_badge_bg),
                colour: palette.deviation_major_badge_fg,
            }),
            Some("Major deviation – confirmation required"),
        ),
        Deviation::NoReference => (
            Some(palette.no_reference_bg),
            Some(StateIcon {
                icon: Icon::FiberNew,
                colour: palette.no_reference_fg,
            }),
            None,
            Some("No reference"),
        ),
    };
    let border = match (&confidence, &field.deviation) {
        (Some((_, border, _)), _) => Some(*border),
        (None, Deviation::NoReference) => Some(Border {
            colour: palette.no_reference_border,
            width: WIDTH_HAIRLINE,
            style: BorderStyle::Dashed,
        }),
        (None, Deviation::Equal | Deviation::Minor { .. } | Deviation::Major { .. }) => None,
    };
    let reason = confidence
        .as_ref()
        .map(|(_, _, label)| *label)
        .into_iter()
        .chain(deviation_label)
        .collect::<Vec<_>>()
        .join(" · ");
    CellLook {
        background,
        border,
        confidence_icon: confidence.map(|(icon, _, _)| icon),
        deviation_icon,
        badge,
        reason,
    }
}
