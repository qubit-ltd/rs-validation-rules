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

use super::TextLengthError;
use super::above_max;
use super::below_min;

/// Bounds the number of Unicode scalar values in text, not grapheme clusters.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::CharLength;
/// use qubit_validator::Validator;
///
/// let rule = CharLength::new(Some(1), Some(2)).expect("ordered bounds");
/// assert!(rule.validate("é", &()).is_ok());
/// assert!(rule.validate("", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharLength {
    /// Inclusive minimum scalar-value count, when configured.
    min: Option<u32>,
    /// Inclusive maximum scalar-value count, when configured.
    max: Option<u32>,
}

impl CharLength {
    /// Creates a rule with inclusive optional bounds.
    ///
    /// # Parameters
    /// - `min`: Optional minimum Unicode scalar-value count.
    /// - `max`: Optional maximum Unicode scalar-value count.
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

impl Validator<str, ()> for CharLength {
    /// Scalar-value count failure emitted when a bound is violated.
    type Error = TextLengthError;

    /// Counts Unicode scalar values and checks the configured bounds.
    ///
    /// # Parameters
    /// - `value`: Text whose scalar values are counted.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the scalar-value count satisfies both bounds.
    ///
    /// # Errors
    /// Returns [`TextLengthError::TooShort`] or [`TextLengthError::TooLong`]
    /// when a configured bound is violated.
    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        let count = value.chars().count();
        if below_min(count, self.min) {
            return Err(TextLengthError::TooShort { min: self.min.unwrap() });
        }
        if above_max(count, self.max) {
            return Err(TextLengthError::TooLong { max: self.max.unwrap() });
        }
        Ok(())
    }
}
