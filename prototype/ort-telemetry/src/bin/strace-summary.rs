// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Prints the IPv4 and IPv6 destinations of each strace log given on the command line.

use std::process::ExitCode;

use tallyline_ort_telemetry_spike::strace::internet_destinations;

fn main() -> ExitCode {
    let mut status = ExitCode::SUCCESS;
    for path in std::env::args().skip(1) {
        let log = match std::fs::read_to_string(&path) {
            Ok(log) => log,
            Err(error) => {
                eprintln!("cannot read {path}: {error}");
                status = ExitCode::FAILURE;
                continue;
            }
        };
        let destinations = internet_destinations(&log);
        let remote = destinations
            .keys()
            .filter(|destination| !destination.is_loopback())
            .count();
        println!("{path}: {remote} non-loopback destination(s)");
        for (destination, calls) in &destinations {
            let scope = if destination.is_loopback() {
                "loopback"
            } else {
                "remote"
            };
            println!(
                "  {:?} {} port {} ({scope}): {calls} call(s)",
                destination.family, destination.address, destination.port
            );
        }
    }
    status
}
