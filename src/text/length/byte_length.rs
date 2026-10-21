// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

use super::ByteLengthError;
use super::above_max;
use super::below_min;

/// Bounds the UTF-8 byte length of text.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::ByteLength;
/// use qubit_validator::Validator;
///
/// let rule = ByteLength::new(Some(2), Some(3)).expect("ordered bounds");
/// assert!(rule.validate("é", &()).is_ok());
/// assert!(rule.validate("éé", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteLength {
    /// Inclusive minimum UTF-8 byte count, when configured.
    min: Option<u32>,
    /// Inclusive maximum UTF-8 byte count, when configured.
    max: Option<u32>,
}

impl ByteLength {
    /// Creates a rule with inclusive optional bounds.
    ///
    /// # Parameters
    /// - `min`: Optional minimum UTF-8 byte count.
    /// - `max`: Optional maximum UTF-8 byte count.
    ///
    /// # Returns
    /// A rule when at least one bound is present and the bounds are ordered.
    ///
    /// # Errors
    /// Returns `InvalidBounds` when both bounds are absent, or
    /// `ParameterOutOfRange` when both are present and `min` exceeds `max`.
    #[inline]
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.is_none() && max.is_none() {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}

impl Validator<str, ()> for ByteLength {
    /// UTF-8 byte-count failure emitted when a bound is violated.
    type Error = ByteLengthError;

    /// Counts UTF-8 bytes and checks the configured bounds.
    ///
    /// # Parameters
    /// - `value`: Text whose encoded bytes are counted.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the encoded byte count satisfies both bounds.
    ///
    /// # Errors
    /// Returns [`ByteLengthError::TooFewBytes`] or
    /// [`ByteLengthError::TooManyBytes`] when a configured bound is violated.
    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        let count = value.len();
        if let Some(min) = self.min
            && below_min(count, Some(min))
        {
            return Err(ByteLengthError::TooFewBytes { min });
        }
        if let Some(max) = self.max
            && above_max(count, Some(max))
        {
            return Err(ByteLengthError::TooManyBytes { max });
        }
        Ok(())
    }
}
