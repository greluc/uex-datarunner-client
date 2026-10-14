// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each measured variant names the mitigations it applies.

use tallyline_ort_telemetry_spike::variant::{EnvironmentOptOut, TelemetryEvents, Variant};

#[test]
fn the_four_variants_parse_from_their_names() {
    assert_eq!("none".parse(), Ok(Variant::NoMitigation));
    assert_eq!("api".parse(), Ok(Variant::ApiOff));
    assert_eq!("env".parse(), Ok(Variant::EnvironmentAndApiOff));
    assert_eq!("env-only".parse(), Ok(Variant::EnvironmentOnly));
}

#[test]
fn an_unknown_name_is_refused() {
    assert!("off".parse::<Variant>().is_err());
    assert!("".parse::<Variant>().is_err());
}

#[test]
fn the_name_round_trips() {
    for variant in [
        Variant::NoMitigation,
        Variant::ApiOff,
        Variant::EnvironmentAndApiOff,
        Variant::EnvironmentOnly,
    ] {
        assert_eq!(variant.name().parse(), Ok(variant));
    }
}

#[test]
fn no_mitigation_keeps_the_defaults_of_ort() {
    assert_eq!(
        Variant::NoMitigation.telemetry_events(),
        TelemetryEvents::OrtDefault
    );
    assert_eq!(
        Variant::NoMitigation.environment_opt_out(),
        EnvironmentOptOut::Unset
    );
}

#[test]
fn api_off_switches_events_off_without_the_environment() {
    assert_eq!(Variant::ApiOff.telemetry_events(), TelemetryEvents::Off);
    assert_eq!(
        Variant::ApiOff.environment_opt_out(),
        EnvironmentOptOut::Unset
    );
}

#[test]
fn environment_and_api_off_applies_both_mitigations() {
    assert_eq!(
        Variant::EnvironmentAndApiOff.telemetry_events(),
        TelemetryEvents::Off
    );
    assert_eq!(
        Variant::EnvironmentAndApiOff.environment_opt_out(),
        EnvironmentOptOut::Set
    );
}

#[test]
fn environment_only_sets_the_variable_and_keeps_the_events_of_ort() {
    assert_eq!(
        Variant::EnvironmentOnly.telemetry_events(),
        TelemetryEvents::OrtDefault
    );
    assert_eq!(
        Variant::EnvironmentOnly.environment_opt_out(),
        EnvironmentOptOut::Set
    );
}
