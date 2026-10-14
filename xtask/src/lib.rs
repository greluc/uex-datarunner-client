// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Repository checks and generators for the Tallyline workspace.

#![forbid(unsafe_code)]

pub mod check;
pub mod deny;
pub mod exit;
pub mod guard;
pub mod hashlist;
pub mod listing;
pub mod lockfile;
pub mod registry;
pub mod requirement;
pub mod sources;
pub mod tags;
pub mod vendor;

#[cfg(test)]
mod denygate;
#[cfg(test)]
mod negative;
#[cfg(test)]
mod scratch;
