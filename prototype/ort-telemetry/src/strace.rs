// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Summary of an `strace -f -e trace=connect,sendto,sendmsg,sendmmsg` log.

use std::collections::BTreeMap;
use std::net::IpAddr;

const ADDRESSED_CALLS: [&str; 4] = ["connect(", "sendto(", "sendmsg(", "sendmmsg("];

/// Address family of a destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    /// `AF_INET`.
    Ipv4,
    /// `AF_INET6`.
    Ipv6,
}

/// One address and port that the traced process addressed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Destination {
    /// Address family.
    pub family: Family,
    /// The address as strace printed it.
    pub address: String,
    /// The port.
    pub port: u16,
}

impl Destination {
    /// Returns whether the address is a loopback address; an address that does not parse is not.
    #[must_use]
    pub fn is_loopback(&self) -> bool {
        self.address
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.to_canonical().is_loopback())
    }
}

/// Returns every IPv4 and IPv6 destination of a `connect`, `sendto`, `sendmsg` or `sendmmsg` call in `log`, with the number of calls.
#[must_use]
pub fn internet_destinations(log: &str) -> BTreeMap<Destination, usize> {
    let mut found = BTreeMap::new();
    for destination in log.lines().filter_map(destination_of) {
        *found.entry(destination).or_insert(0) += 1;
    }
    found
}

fn destination_of(line: &str) -> Option<Destination> {
    let token = line
        .split_whitespace()
        .find(|token| ADDRESSED_CALLS.iter().any(|call| token.starts_with(call)))?;
    let call = line.get(line.find(token)?..)?;
    if let Some(rest) = after(call, "sa_family=AF_INET6,") {
        Some(Destination {
            family: Family::Ipv6,
            address: between(rest, "inet_pton(AF_INET6, \"", "\"")?.to_owned(),
            port: between(rest, "sin6_port=htons(", ")")?.parse().ok()?,
        })
    } else if let Some(rest) = after(call, "sa_family=AF_INET,") {
        Some(Destination {
            family: Family::Ipv4,
            address: between(rest, "sin_addr=inet_addr(\"", "\"")?.to_owned(),
            port: between(rest, "sin_port=htons(", ")")?.parse().ok()?,
        })
    } else {
        None
    }
}

fn after<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    text.find(marker)
        .and_then(|index| text.get(index + marker.len()..))
}

fn between<'a>(text: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let rest = after(text, prefix)?;
    rest.get(..rest.find(suffix)?)
}
