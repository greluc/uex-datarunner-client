// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The UEX API adapter: a read-only HTTP client for the UEX API 2.0 and its anti-corruption layer.

#![forbid(unsafe_code)]

pub mod api;
#[cfg(test)]
#[expect(
    non_snake_case,
    reason = "tagged test names use a double underscore (11 B4)"
)]
mod contract_tests;
mod envelope;
mod transport;
mod wire;
