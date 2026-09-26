// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Timelike;
use chrono::Utc;
use qubit_validator::Validator;

use super::temporal_resolution::TemporalResolution;
use super::time_precision_error::TimePrecisionError;

/// Checks the nanosecond component without rounding or changing dates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimePrecision {
    resolution: TemporalResolution,
}

impl TimePrecision {
    /// Creates a rule for the declared resolution.
    #[must_use]
    pub const fn new(resolution: TemporalResolution) -> Self {
        Self { resolution }
    }

    /// Checks that the nanosecond component divides evenly by the unit.
    fn check(&self, nanos: u32) -> Result<(), TimePrecisionError> {
        let unit = match self.resolution {
            TemporalResolution::Second => 1_000_000_000,
            TemporalResolution::Millisecond => 1_000_000,
            TemporalResolution::Microsecond => 1_000,
            TemporalResolution::Nanosecond => 1,
        };
        if nanos.is_multiple_of(unit) {
            Ok(())
        } else {
            Err(TimePrecisionError::Precision)
        }
    }
}

impl Validator<NaiveTime, ()> for TimePrecision {
    type Error = TimePrecisionError;

    fn validate(&self, value: &NaiveTime, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}

impl Validator<NaiveDateTime, ()> for TimePrecision {
    type Error = TimePrecisionError;

    fn validate(&self, value: &NaiveDateTime, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}

impl Validator<DateTime<Utc>, ()> for TimePrecision {
    type Error = TimePrecisionError;

    fn validate(&self, value: &DateTime<Utc>, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}
