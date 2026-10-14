// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The candidate filter of the searchable combo box.

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rank {
    Prefix,
    WordStart,
    Substring,
}

fn rank(item: &str, query: &str) -> Option<Rank> {
    let item = item.to_lowercase();
    if item.starts_with(query) {
        return Some(Rank::Prefix);
    }
    if item.split_whitespace().any(|word| word.starts_with(query)) {
        return Some(Rank::WordStart);
    }
    item.contains(query).then_some(Rank::Substring)
}

/// The indexes of `items` that match `query`, prefix matches first, then word-start, then substring matches, each in vocabulary order.
#[must_use]
pub fn filter_candidates(items: &[&str], query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    let mut ranked: Vec<(Rank, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| rank(item, &query).map(|r| (r, index)))
        .collect();
    ranked.sort_unstable();
    ranked.into_iter().map(|(_, index)| index).collect()
}
