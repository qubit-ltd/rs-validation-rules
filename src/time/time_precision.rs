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
    /// Resolution unit used to check whether the nanosecond component divides
    /// evenly.
    resolution: TemporalResolution,
}

impl TimePrecision {
    /// Creates a rule for the declared resolution.
    ///
    /// # Parameters
    ///
    /// - `resolution`: Resolution unit used by subsequent validation.
    ///
    /// # Returns
    ///
    /// Returns a rule configured with the given resolution.
    #[must_use]
    pub const fn new(resolution: TemporalResolution) -> Self {
        Self { resolution }
    }

    /// Checks whether the nanosecond component is divisible by the resolution
    /// unit.
    ///
    /// # Parameters
    ///
    /// - `nanos`: Nanosecond component to check.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when `nanos` is divisible by the unit, or
    /// [`TimePrecisionError::Precision`] otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`TimePrecisionError::Precision`] when the component has finer
    /// precision than the configured resolution.
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

    /// Validates a time's fractional-second precision using its nanosecond
    /// component.
    ///
    /// The unit context is unused. Validation succeeds when that component is
    /// divisible by the configured resolution unit; the value is not rounded
    /// or changed.
    ///
    /// # Parameters
    ///
    /// - `value`: Time value to validate.
    /// - `context`: Unit context; unused by this validator.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the time matches the configured resolution.
    ///
    /// # Errors
    ///
    /// Returns [`TimePrecisionError::Precision`] when the nanosecond component
    /// is not divisible by the resolution unit.
    fn validate(&self, value: &NaiveTime, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}

impl Validator<NaiveDateTime, ()> for TimePrecision {
    type Error = TimePrecisionError;

    /// Validates a date-time's fractional-second precision using its nanosecond
    /// component.
    ///
    /// The unit context is unused. Validation succeeds when that component is
    /// divisible by the configured resolution unit; the value is not rounded
    /// or changed.
    ///
    /// # Parameters
    ///
    /// - `value`: Date-time value to validate.
    /// - `context`: Unit context; unused by this validator.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the date-time matches the configured resolution.
    ///
    /// # Errors
    ///
    /// Returns [`TimePrecisionError::Precision`] when the nanosecond component
    /// is not divisible by the resolution unit.
    fn validate(&self, value: &NaiveDateTime, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}

impl Validator<DateTime<Utc>, ()> for TimePrecision {
    type Error = TimePrecisionError;

    /// Validates a UTC date-time's fractional-second precision using its
    /// nanosecond component.
    ///
    /// The unit context is unused. Validation succeeds when that component is
    /// divisible by the configured resolution unit; the value is not rounded
    /// or changed.
    ///
    /// # Parameters
    ///
    /// - `value`: UTC date-time value to validate.
    /// - `context`: Unit context; unused by this validator.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the date-time matches the configured resolution.
    ///
    /// # Errors
    ///
    /// Returns [`TimePrecisionError::Precision`] when the nanosecond component
    /// is not divisible by the resolution unit.
    fn validate(&self, value: &DateTime<Utc>, _: &()) -> Result<(), Self::Error> {
        self.check(value.nanosecond())
    }
}
