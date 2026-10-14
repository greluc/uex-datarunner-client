// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Rows, fields and their states, and the deterministic sample data of the spike.

/// A column of the review table; every column is an editable field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Column {
    /// The commodity, chosen from the vocabulary.
    Commodity,
    /// The price per SCU.
    Price,
    /// The stock or demand in SCU.
    Scu,
    /// The inventory status level, chosen from the status vocabulary.
    Status,
    /// The container sizes as a set.
    ContainerSizes,
}

/// How a column is edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorKind {
    /// A single-line text field.
    Text,
    /// A searchable combo box over a fixed vocabulary.
    Choice,
}

impl Column {
    /// The columns in display order.
    pub const ALL: [Self; 5] = [
        Self::Commodity,
        Self::Price,
        Self::Scu,
        Self::Status,
        Self::ContainerSizes,
    ];

    /// The position of the column in [`Self::ALL`].
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Commodity => 0,
            Self::Price => 1,
            Self::Scu => 2,
            Self::Status => 3,
            Self::ContainerSizes => 4,
        }
    }

    /// The column at `index` in [`Self::ALL`], if there is one.
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    /// The header text of the column.
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Commodity => "Commodity",
            Self::Price => "Price / SCU",
            Self::Scu => "SCU",
            Self::Status => "Status",
            Self::ContainerSizes => "Containers",
        }
    }

    /// The editor the column uses.
    #[must_use]
    pub const fn editor(self) -> EditorKind {
        match self {
            Self::Commodity | Self::Status => EditorKind::Choice,
            Self::Price | Self::Scu | Self::ContainerSizes => EditorKind::Text,
        }
    }

    /// The vocabulary a choice column picks from; empty for text columns.
    #[must_use]
    pub const fn vocabulary(self) -> &'static [&'static str] {
        match self {
            Self::Commodity => COMMODITIES,
            Self::Status => STATUS_LEVELS,
            Self::Price | Self::Scu | Self::ContainerSizes => &[],
        }
    }
}

/// The recognition confidence of a field (R-UI-4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    /// At or above the send threshold.
    Ok,
    /// One candidate below the send threshold.
    Confirm,
    /// Several candidates.
    Select,
    /// Unreadable or implausible.
    Correct,
    /// Typed or corrected by the user.
    UserEntered,
    /// Confirmed by the user.
    Confirmed,
    /// OCR and AI agree.
    DoubleConfirmed,
}

/// The deviation of a field from the previous UEX value (R-UI-10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Deviation {
    /// Equal to the reference.
    Equal,
    /// Within tolerance.
    Minor {
        /// The signed difference in percent.
        delta_percent: i32,
    },
    /// Beyond tolerance; needs confirmation.
    Major {
        /// The signed difference in percent.
        delta_percent: i32,
    },
    /// No reference value at this terminal.
    NoReference,
}

/// One cell of the review table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// The displayed value.
    pub value: String,
    /// The recognition confidence.
    pub confidence: Confidence,
    /// The deviation from the reference.
    pub deviation: Deviation,
}

impl Field {
    /// Whether F8 stops at this field: an open recognition decision, or a deviation the user has not confirmed.
    #[must_use]
    pub const fn needs_attention(&self) -> bool {
        let (open_decision, confirmed) = match self.confidence {
            Confidence::Confirm | Confidence::Select | Confidence::Correct => (true, false),
            Confidence::Confirmed => (false, true),
            Confidence::Ok | Confidence::UserEntered | Confidence::DoubleConfirmed => {
                (false, false)
            }
        };
        let deviates = match self.deviation {
            Deviation::Minor { .. } | Deviation::Major { .. } => true,
            Deviation::Equal | Deviation::NoReference => false,
        };
        open_decision || (deviates && !confirmed)
    }
}

/// One commodity row of the review table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    commodity: Field,
    price: Field,
    scu: Field,
    status: Field,
    container_sizes: Field,
}

impl Row {
    /// The field of `column`.
    #[must_use]
    pub const fn field(&self, column: Column) -> &Field {
        match column {
            Column::Commodity => &self.commodity,
            Column::Price => &self.price,
            Column::Scu => &self.scu,
            Column::Status => &self.status,
            Column::ContainerSizes => &self.container_sizes,
        }
    }

    /// The field of `column`, mutable.
    pub const fn field_mut(&mut self, column: Column) -> &mut Field {
        match column {
            Column::Commodity => &mut self.commodity,
            Column::Price => &mut self.price,
            Column::Scu => &mut self.scu,
            Column::Status => &mut self.status,
            Column::ContainerSizes => &mut self.container_sizes,
        }
    }
}

/// Commodity names used as the sample vocabulary.
pub const COMMODITIES: &[&str] = &[
    "Agricium",
    "Agricultural Supplies",
    "Aluminum",
    "Astatine",
    "Beryl",
    "Bexalite",
    "Borase",
    "Chlorine",
    "Compboard",
    "Copper",
    "Corundum",
    "Diamond",
    "Distilled Spirits",
    "Fluorine",
    "Gold",
    "Hadanite",
    "Hephaestanite",
    "Hydrogen",
    "Iodine",
    "Laranite",
    "Medical Supplies",
    "Processed Food",
    "Quantanium",
    "Quartz",
    "Raw Tin",
    "Recycled Material Composite",
    "Scrap",
    "Stims",
    "Taranite",
    "Tin",
    "Titanium",
    "Tungsten",
    "Waste",
];

/// Status levels used as the sample vocabulary.
pub const STATUS_LEVELS: &[&str] = &[
    "Out of stock",
    "Very low inventory",
    "Low inventory",
    "Medium inventory",
    "High inventory",
    "Very high inventory",
    "Maximum inventory",
];

const CONFIDENCE_CYCLE: [Confidence; 11] = [
    Confidence::Ok,
    Confidence::Ok,
    Confidence::Confirm,
    Confidence::Ok,
    Confidence::Select,
    Confidence::Ok,
    Confidence::Correct,
    Confidence::UserEntered,
    Confidence::Confirmed,
    Confidence::Ok,
    Confidence::DoubleConfirmed,
];

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound.max(1)
    }
}

fn pick<'a>(rng: &mut Lcg, items: &[&'a str]) -> &'a str {
    let index = usize::try_from(rng.below(items.len() as u64)).unwrap_or(0);
    items.get(index).copied().unwrap_or("")
}

fn confidence(rng: &mut Lcg) -> Confidence {
    let index = usize::try_from(rng.below(CONFIDENCE_CYCLE.len() as u64)).unwrap_or(0);
    CONFIDENCE_CYCLE
        .get(index)
        .copied()
        .unwrap_or(Confidence::Ok)
}

fn deviation(rng: &mut Lcg) -> Deviation {
    let sign = if rng.below(2) == 0 { 1 } else { -1 };
    match rng.below(10) {
        0..=5 => Deviation::Equal,
        6 | 7 => Deviation::Minor {
            delta_percent: sign * i32::try_from(rng.below(9) + 1).unwrap_or(1),
        },
        8 => Deviation::Major {
            delta_percent: sign * i32::try_from(rng.below(80) + 20).unwrap_or(20),
        },
        _ => Deviation::NoReference,
    }
}

fn sample_field(rng: &mut Lcg, value: String) -> Field {
    Field {
        value,
        confidence: confidence(rng),
        deviation: deviation(rng),
    }
}

/// Builds `count` rows of deterministic sample data that cover every cell state.
#[must_use]
pub fn sample_rows(count: usize) -> Vec<Row> {
    let mut rng = Lcg(0x5EED_7A11);
    (0..count)
        .map(|_| {
            let commodity = pick(&mut rng, COMMODITIES).to_owned();
            let price = format!("{}", rng.below(30_000) + 1);
            let scu = format!("{}", rng.below(12_000));
            let status = pick(&mut rng, STATUS_LEVELS).to_owned();
            let sizes = pick(
                &mut rng,
                &[
                    "1, 2, 4",
                    "1, 2, 4, 8",
                    "1, 2, 4, 8, 16",
                    "2, 4, 8, 16, 24, 32",
                ],
            )
            .to_owned();
            Row {
                commodity: Field {
                    value: commodity,
                    confidence: confidence(&mut rng),
                    deviation: Deviation::Equal,
                },
                price: sample_field(&mut rng, price),
                scu: sample_field(&mut rng, scu),
                status: sample_field(&mut rng, status),
                container_sizes: Field {
                    value: sizes,
                    confidence: confidence(&mut rng),
                    deviation: match deviation(&mut rng) {
                        Deviation::Major { delta_percent } => Deviation::Minor { delta_percent },
                        other @ (Deviation::Equal
                        | Deviation::Minor { .. }
                        | Deviation::NoReference) => other,
                    },
                },
            }
        })
        .collect()
}
