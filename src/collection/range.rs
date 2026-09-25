// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cmp::Ordering;
use std::ops::Bound;

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

use super::range_error::RangeError;

/// Checks inclusive or exclusive bounds for partially ordered values.
///
/// Unordered values such as `NaN` fail validation, including when both bounds
/// are unbounded.
///
/// # Type Parameters
/// - `T`: `PartialOrd` value type compared with the range endpoints.
///
/// # Examples
///
/// ```
/// use std::ops::Bound;
/// use qubit_validation_rules::collection::Range;
/// use qubit_validator::Validator;
///
/// let rule = Range::new(Bound::Excluded(1), Bound::Included(4))
///     .expect("ordered non-empty bounds");
/// assert!(rule.validate(&2, &()).is_ok());
/// assert!(rule.validate(&1, &()).is_err());
/// ```
#[derive(Clone, Debug)]
pub struct Range<T> {
    /// Lower endpoint, including whether equality is accepted.
    lower: Bound<T>,
    /// Upper endpoint, including whether equality is accepted.
    upper: Bound<T>,
}

impl<T: PartialOrd> Range<T> {
    /// Creates a range from `lower` and `upper`.
    ///
    /// # Parameters
    /// - `lower`: Inclusive, exclusive, or unbounded lower endpoint.
    /// - `upper`: Inclusive, exclusive, or unbounded upper endpoint.
    ///
    /// # Returns
    /// A rule whose endpoints are ordered and describe a non-empty range.
    ///
    /// # Errors
    /// Returns `InvalidBounds` for reversed, empty, or unordered bounds,
    /// including an endpoint that cannot be compared with itself.
    pub fn new(lower: Bound<T>, upper: Bound<T>) -> Result<Self, BindError> {
        let unordered_bound = [&lower, &upper].into_iter().any(|bound| match bound {
            Bound::Included(value) | Bound::Excluded(value) => value.partial_cmp(value).is_none(),
            Bound::Unbounded => false,
        });
        if unordered_bound {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        let invalid = match (&lower, &upper) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => false,
            (Bound::Included(a), Bound::Included(b))
            | (Bound::Included(a), Bound::Excluded(b))
            | (Bound::Excluded(a), Bound::Included(b))
            | (Bound::Excluded(a), Bound::Excluded(b)) => match a.partial_cmp(b) {
                Some(Ordering::Greater) => true,
                Some(Ordering::Equal) => {
                    matches!((&lower, &upper), (Bound::Excluded(_), _) | (_, Bound::Excluded(_)))
                }
                None => true,
                _ => false,
            },
        };
        if invalid {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        Ok(Self { lower, upper })
    }
}

impl<T: PartialOrd> Validator<T, ()> for Range<T> {
    /// Range-bound failure emitted by this rule.
    type Error = RangeError;

    /// Checks that `value` lies inside both endpoints.
    ///
    /// # Parameters
    /// - `value`: Value to compare against the endpoints.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when `value` satisfies both endpoints.
    ///
    /// # Errors
    /// Returns [`RangeError::Unordered`] if a comparison is unordered, or
    /// [`RangeError::OutOfRange`] when an endpoint is not satisfied.
    fn validate(&self, value: &T, _: &()) -> Result<(), Self::Error> {
        if value.partial_cmp(value).is_none() {
            return Err(RangeError::Unordered);
        }
        let lower_ok = match &self.lower {
            Bound::Unbounded => true,
            Bound::Included(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_ge(),
            Bound::Excluded(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_gt(),
        };
        let upper_ok = match &self.upper {
            Bound::Unbounded => true,
            Bound::Included(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_le(),
            Bound::Excluded(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_lt(),
        };
        if lower_ok && upper_ok {
            Ok(())
        } else {
            Err(RangeError::OutOfRange)
        }
    }
}
