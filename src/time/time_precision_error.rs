// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Reports that a temporal value exceeds its declared resolution.
///
/// # Examples
///
/// ```
/// use chrono::NaiveTime;
/// use qubit_validation_rules::time::TemporalResolution;
/// use qubit_validation_rules::time::TimePrecision;
/// use qubit_validation_rules::time::TimePrecisionError;
/// use qubit_validator::Validator;
///
/// let rule = TimePrecision::new(TemporalResolution::Millisecond);
/// let time = NaiveTime::from_hms_nano_opt(12, 30, 0, 123_456_000).unwrap();
/// assert_eq!(rule.validate(&time, &()), Err(TimePrecisionError::Precision));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum TimePrecisionError {
    /// The nanosecond component is not divisible by the declared unit.
    #[error("temporal precision exceeded")]
    Precision,
}
