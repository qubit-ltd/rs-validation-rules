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

/// Validates that a time value matches its declared temporal resolution.
///
/// Values are checked exactly: validation does not round or alter the input.
///
/// # Examples
///
/// ```
/// use chrono::NaiveTime;
/// use qubit_validation_rules::time::TemporalResolution;
/// use qubit_validation_rules::time::TimePrecision;
/// use qubit_validator::Validator;
///
/// let rule = TimePrecision::new(TemporalResolution::Millisecond);
/// let time = NaiveTime::from_hms_nano_opt(12, 30, 0, 123_000_000).unwrap();
/// assert!(rule.validate(&time, &()).is_ok());
/// ```
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

    /// Validates the fractional-second precision of a time value.
    fn validate(&self, value: &NaiveTime, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}

impl Validator<NaiveDateTime, ()> for TimePrecision {
    type Error = TimePrecisionError;

    /// Validates the fractional-second precision of a date-time value.
    fn validate(&self, value: &NaiveDateTime, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}

impl Validator<DateTime<Utc>, ()> for TimePrecision {
    type Error = TimePrecisionError;

    /// Validates the fractional-second precision of a UTC date-time value.
    fn validate(&self, value: &DateTime<Utc>, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}
