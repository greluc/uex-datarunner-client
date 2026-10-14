// SPDX-FileCopyrightText: 2026 Lucas Greuloch
// SPDX-License-Identifier: GPL-3.0-or-later

//! The measured variants and the mitigations each applies.

use std::fmt;
use std::str::FromStr;

/// One measured configuration of the probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// `ort`'s defaults: telemetry events on, no environment variable.
    NoMitigation,
    /// Telemetry events switched off through `EnvironmentBuilder::with_telemetry(false)`.
    ApiOff,
    /// `ORT_DISABLE_TELEMETRY=1` set by the probe before the first `ort` call, and events switched off as in [`Variant::ApiOff`].
    EnvironmentAndApiOff,
}

/// How the environment is built with respect to telemetry events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryEvents {
    /// `with_telemetry` is not called.
    OrtDefault,
    /// `with_telemetry(false)` is called.
    Off,
}

/// Whether the probe sets `ORT_DISABLE_TELEMETRY=1` itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentOptOut {
    /// The variable is left as the process inherited it.
    Unset,
    /// The probe sets the variable before the first `ort` call.
    Set,
}

/// The name is none of `none`, `api` and `env`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownVariant(pub String);

impl fmt::Display for UnknownVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown variant `{}`; expected none, api or env", self.0)
    }
}

impl std::error::Error for UnknownVariant {}

impl Variant {
    /// Returns the name used on the command line and in the result files.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::NoMitigation => "none",
            Self::ApiOff => "api",
            Self::EnvironmentAndApiOff => "env",
        }
    }

    /// Returns how the environment is built.
    #[must_use]
    pub fn telemetry_events(self) -> TelemetryEvents {
        match self {
            Self::NoMitigation => TelemetryEvents::OrtDefault,
            Self::ApiOff | Self::EnvironmentAndApiOff => TelemetryEvents::Off,
        }
    }

    /// Returns whether the probe sets `ORT_DISABLE_TELEMETRY=1`.
    #[must_use]
    pub fn environment_opt_out(self) -> EnvironmentOptOut {
        match self {
            Self::NoMitigation | Self::ApiOff => EnvironmentOptOut::Unset,
            Self::EnvironmentAndApiOff => EnvironmentOptOut::Set,
        }
    }
}

impl FromStr for Variant {
    type Err = UnknownVariant;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        [Self::NoMitigation, Self::ApiOff, Self::EnvironmentAndApiOff]
            .into_iter()
            .find(|variant| variant.name() == name)
            .ok_or_else(|| UnknownVariant(name.to_owned()))
    }
}
