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

/// Returns whether `count` is below a configured minimum.
///
/// # Parameters
/// - `count`: Measured scalar or byte count.
/// - `min`: Optional inclusive lower bound.
///
/// # Returns
/// `true` when `min` exists and `count` is smaller.
fn below_min(count: usize, min: Option<u32>) -> bool {
    min.is_some_and(|bound| count < bound as usize)
}

/// Returns whether `count` exceeds a configured maximum.
///
/// # Parameters
/// - `count`: Measured scalar or byte count.
/// - `max`: Optional inclusive upper bound.
///
/// # Returns
/// `true` when `max` exists and `count` is larger.
fn above_max(count: usize, max: Option<u32>) -> bool {
    max.is_some_and(|bound| count > bound as usize)
}
/// Character-length validation errors.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::CharLength;
/// use qubit_validation_rules::text::TextLengthError;
/// use qubit_validator::Validator;
///
/// let rule = CharLength::new(Some(2), None).expect("valid bound");
/// assert_eq!(rule.validate("x", &()), Err(TextLengthError::TooShort { min: 2 }));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
// qubit-style: allow public-type-layout
pub enum TextLengthError {
    /// The Unicode scalar value count is below `min`.
    #[error("text is too short")]
    TooShort {
        /// Required minimum count.
        min: u32,
    },
    /// The Unicode scalar value count exceeds `max`.
    #[error("text is too long")]
    TooLong {
        /// Allowed maximum count.
        max: u32,
    },
}
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
// qubit-style: allow public-type-layout
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
    /// A rule when the bounds are ordered.
    ///
    /// # Errors
    /// Returns `ParameterOutOfRange` when both bounds are present and `min`
    /// exceeds `max`.
    #[inline]
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
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
        let n = value.chars().count();
        if below_min(n, self.min) {
            return Err(TextLengthError::TooShort { min: self.min.unwrap() });
        }
        if above_max(n, self.max) {
            return Err(TextLengthError::TooLong { max: self.max.unwrap() });
        }
        Ok(())
    }
}
/// UTF-8 byte-length validation errors.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::ByteLength;
/// use qubit_validation_rules::text::ByteLengthError;
/// use qubit_validator::Validator;
///
/// let rule = ByteLength::new(Some(2), None).expect("valid bound");
/// assert_eq!(rule.validate("x", &()), Err(ByteLengthError::TooFewBytes { min: 2 }));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
// qubit-style: allow public-type-layout
pub enum ByteLengthError {
    /// The UTF-8 byte count is below `min`.
    #[error("text has too few bytes")]
    TooFewBytes {
        /// Required minimum count.
        min: u32,
    },
    /// The UTF-8 byte count exceeds `max`.
    #[error("text has too many bytes")]
    TooManyBytes {
        /// Allowed maximum count.
        max: u32,
    },
}
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
// qubit-style: allow public-type-layout
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
    /// A rule when the bounds are ordered.
    ///
    /// # Errors
    /// Returns `ParameterOutOfRange` when both bounds are present and `min`
    /// exceeds `max`.
    #[inline]
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
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
        let n = value.len();
        if below_min(n, self.min) {
            return Err(ByteLengthError::TooFewBytes { min: self.min.unwrap() });
        }
        if above_max(n, self.max) {
            return Err(ByteLengthError::TooManyBytes { max: self.max.unwrap() });
        }
        Ok(())
    }
}

#[cfg(all(test, target_pointer_width = "64"))]
mod tests {
    use super::above_max;
    use super::below_min;

    #[test]
    fn test_counts_above_u32_max_do_not_wrap() {
        let count = u32::MAX as usize + 1;

        assert!(!below_min(count, Some(1)));
        assert!(above_max(count, Some(u32::MAX)));
        assert!(above_max(count, Some(0)));
    }
}
