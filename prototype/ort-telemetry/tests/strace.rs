// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The strace summary finds every IPv4 and IPv6 destination the traced process addressed.

use tallyline_ort_telemetry_spike::strace::{Destination, Family, internet_destinations};

fn v4(address: &str, port: u16) -> Destination {
    Destination {
        family: Family::Ipv4,
        address: address.to_owned(),
        port,
    }
}

fn v6(address: &str, port: u16) -> Destination {
    Destination {
        family: Family::Ipv6,
        address: address.to_owned(),
        port,
    }
}

#[test]
fn an_ipv4_connect_is_a_destination() {
    let log = r#"4711 10:00:00.123456 connect(5, {sa_family=AF_INET, sin_port=htons(443), sin_addr=inet_addr("13.69.109.130")}, 16) = -1 EINPROGRESS (Operation now in progress)"#;

    let found = internet_destinations(log);

    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        vec![(v4("13.69.109.130", 443), 1)]
    );
}

#[test]
fn an_ipv6_connect_is_a_destination() {
    let log = r#"4712 connect(7, {sa_family=AF_INET6, sin6_port=htons(443), sin6_flowinfo=htonl(0), inet_pton(AF_INET6, "2603:1030:b:3::152", &sin6_addr), sin6_scope_id=0}, 28) = 0"#;

    let found = internet_destinations(log);

    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        vec![(v6("2603:1030:b:3::152", 443), 1)]
    );
}

#[test]
fn unix_sockets_and_calls_without_an_address_are_not_destinations() {
    let log = [
        r#"4711 connect(3, {sa_family=AF_UNIX, sun_path="/var/run/nscd/socket"}, 110) = -1 ENOENT (No such file or directory)"#,
        r#"4711 sendto(5, "\26\3\1", 517, MSG_NOSIGNAL, NULL, 0) = 517"#,
        r#"4711 bind(6, {sa_family=AF_INET, sin_port=htons(0), sin_addr=inet_addr("0.0.0.0")}, 16) = 0"#,
        "4711 +++ exited with 0 +++",
    ]
    .join("\n");

    assert!(internet_destinations(&log).is_empty());
}

#[test]
fn a_dns_query_by_sendto_and_sendmsg_counts_each_call() {
    let log = [
        r#"4713 sendto(3, "\x12\x34", 40, MSG_NOSIGNAL, {sa_family=AF_INET, sin_port=htons(53), sin_addr=inet_addr("127.0.0.53")}, 16) = 40"#,
        r#"4713 sendmsg(3, {msg_name={sa_family=AF_INET, sin_port=htons(53), sin_addr=inet_addr("127.0.0.53")}, msg_namelen=16, msg_iov=[{iov_base="\x12", iov_len=40}], msg_iovlen=1, msg_controllen=0, msg_flags=0}, MSG_NOSIGNAL) = 40"#,
    ]
    .join("\n");

    let found = internet_destinations(&log);

    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        vec![(v4("127.0.0.53", 53), 2)]
    );
}

#[test]
fn a_sendmmsg_with_an_address_is_a_destination() {
    let log = r#"4716 sendmmsg(3, [{msg_hdr={msg_name={sa_family=AF_INET6, sin6_port=htons(53), sin6_flowinfo=htonl(0), inet_pton(AF_INET6, "::1", &sin6_addr), sin6_scope_id=0}, msg_namelen=28, msg_iov=[{iov_base="\x56", iov_len=48}], msg_iovlen=1, msg_controllen=0, msg_flags=0}, msg_len=48}], 1, MSG_NOSIGNAL) = 1"#;

    let found = internet_destinations(log);

    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        vec![(v6("::1", 53), 1)]
    );
}

#[test]
fn an_unfinished_call_counts_once_and_its_resumption_not_at_all() {
    let log = [
        r#"4714 connect(5, {sa_family=AF_INET, sin_port=htons(443), sin_addr=inet_addr("20.42.65.84")}, 16 <unfinished ...>"#,
        "4714 <... connect resumed>) = 0",
        r#"4715 connect(6, {sa_family=AF_INET, sin_port=htons(443), sin_addr=inet_addr("20.42.65.84")}, 16) = 0"#,
    ]
    .join("\n");

    let found = internet_destinations(&log);

    assert_eq!(
        found.into_iter().collect::<Vec<_>>(),
        vec![(v4("20.42.65.84", 443), 2)]
    );
}

#[test]
fn loopback_destinations_are_recognised() {
    assert!(v4("127.0.0.53", 53).is_loopback());
    assert!(v6("::1", 53).is_loopback());
    assert!(!v4("20.42.65.84", 443).is_loopback());
    assert!(!v6("2603:1030:b:3::152", 443).is_loopback());
}
