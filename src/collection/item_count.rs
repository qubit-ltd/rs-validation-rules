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

use super::item_count_error::ItemCountError;

/// Checks inclusive optional bounds on a collection's `usize` item count.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::collection::ItemCount;
/// use qubit_validator::Validator;
///
/// let rule = ItemCount::new(Some(2), Some(4)).expect("ordered bounds");
/// assert!(rule.validate(&3, &()).is_ok());
/// assert!(rule.validate(&1, &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemCount {
    /// Inclusive lower count bound, when configured.
    min: Option<usize>,
    /// Inclusive upper count bound, when configured.
    max: Option<usize>,
}
impl ItemCount {
    /// Creates a rule with inclusive optional bounds.
    ///
    /// # Parameters
    /// - `min`: Optional inclusive lower count bound.
    /// - `max`: Optional inclusive upper count bound.
    ///
    /// # Returns
    /// The rule when at least one bound is present and the bounds are ordered.
    ///
    /// # Errors
    /// Returns `InvalidBounds` when both bounds are absent, or
    /// `ParameterOutOfRange` when both are present and `min` exceeds `max`.
    #[inline]
    pub fn new(min: Option<usize>, max: Option<usize>) -> Result<Self, BindError> {
        if min.is_none() && max.is_none() {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<usize, ()> for ItemCount {
    /// Count-bound failure emitted by this rule.
    type Error = ItemCountError;

    /// Accepts counts inside the configured inclusive bounds.
    ///
    /// # Parameters
    /// - `value`: Collection item count to check.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the count satisfies every configured bound.
    ///
    /// # Errors
    /// Returns [`ItemCountError::TooSmall`] or [`ItemCountError::TooLarge`]
    /// when the count falls outside a configured bound.
    fn validate(&self, value: &usize, _context: &()) -> Result<(), Self::Error> {
        if self.min.is_some_and(|m| *value < m) {
            return Err(ItemCountError::TooSmall { min: self.min.unwrap() });
        }
        if self.max.is_some_and(|m| *value > m) {
            return Err(ItemCountError::TooLarge { max: self.max.unwrap() });
        }
        Ok(())
    }
}
