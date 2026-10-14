// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The values the read-only UEX client returns, translated from the UEX wire format.

/// The current game versions of `GET /game_versions`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameVersions {
    /// The LIVE version, not always three-part (`4.9`, `3.24.2a`).
    pub live: String,
    /// The PTU version, `None` when UEX sends an empty string or `null`.
    pub ptu: Option<String>,
}

/// A UEX terminal ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TerminalId(pub u64);

/// A UEX commodity ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CommodityId(pub u64);

/// The status levels of `GET /commodities_status`, one list per side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLevels {
    /// The buy-side levels in the order UEX sends them.
    pub buy: Vec<StatusLevel>,
    /// The sell-side levels in the order UEX sends them.
    pub sell: Vec<StatusLevel>,
}

/// One inventory status level of one side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLevel {
    /// The level as UEX numbers it, starting at 1 (out of stock).
    pub code: u8,
    /// The long name, which may differ between the sides.
    pub name: String,
    /// The short name.
    pub name_short: String,
    /// The abbreviation.
    pub name_abbr: String,
}

/// The side a terminal trades a commodity on, taken from the side with a price above zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradedSide {
    /// Only the buy price is above zero.
    Buy,
    /// Only the sell price is above zero.
    Sell,
    /// Both prices are above zero, which the documented schema does not allow.
    Both,
    /// Neither price is above zero.
    Neither,
}

/// A container size of the documented set {1, 2, 4, 8, 16, 24, 32} SCU.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContainerSize {
    /// 1 SCU.
    Scu1,
    /// 2 SCU.
    Scu2,
    /// 4 SCU.
    Scu4,
    /// 8 SCU.
    Scu8,
    /// 16 SCU.
    Scu16,
    /// 24 SCU.
    Scu24,
    /// 32 SCU.
    Scu32,
}

impl ContainerSize {
    /// Returns the size for `scu`, `None` outside the documented set.
    #[must_use]
    pub fn from_scu(scu: u32) -> Option<Self> {
        match scu {
            1 => Some(Self::Scu1),
            2 => Some(Self::Scu2),
            4 => Some(Self::Scu4),
            8 => Some(Self::Scu8),
            16 => Some(Self::Scu16),
            24 => Some(Self::Scu24),
            32 => Some(Self::Scu32),
            _ => None,
        }
    }

    /// Returns the size in SCU.
    #[must_use]
    pub fn scu(self) -> u32 {
        match self {
            Self::Scu1 => 1,
            Self::Scu2 => 2,
            Self::Scu4 => 4,
            Self::Scu8 => 8,
            Self::Scu16 => 16,
            Self::Scu24 => 24,
            Self::Scu32 => 32,
        }
    }
}

/// The side, status and container sizes of one row of `GET /commodities_prices`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PriceRow {
    /// The terminal of the row.
    pub terminal: TerminalId,
    /// The commodity of the row.
    pub commodity: CommodityId,
    /// The traded side.
    pub side: TradedSide,
    /// The status level of the traded side, `None` when it is 0 or the side is not single.
    pub status: Option<u8>,
    /// The container sizes of this row, in the order UEX sends them; empty when none are known.
    pub container_sizes: Vec<ContainerSize>,
}

/// The account state of `GET /user`, without the personal data UEX also sends (O-97).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UexAccount {
    /// The UEX username.
    pub username: String,
    /// Whether the account is a DataRunner.
    pub is_datarunner: bool,
    /// Whether the account is banned from data running.
    pub is_datarunner_banned: bool,
    /// Whether the account belongs to UEX staff.
    pub is_staff: bool,
}

/// The answer of a `GET /data_info` access check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReportAccess {
    /// The number of report rows returned, at most the requested limit.
    pub rows: usize,
}
