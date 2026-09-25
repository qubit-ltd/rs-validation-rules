// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Failure reported when a value cannot satisfy a range rule.
///
/// # Examples
///
/// ```
/// use std::ops::Bound;
/// use qubit_validation_rules::collection::Range;
/// use qubit_validation_rules::collection::RangeError;
/// use qubit_validator::Validator;
///
/// let rule = Range::new(Bound::Included(1), Bound::Included(2)).expect("valid bounds");
/// assert_eq!(rule.validate(&3, &()), Err(RangeError::OutOfRange));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum RangeError {
    /// A value falls outside an inclusive or exclusive endpoint.
    #[error("value is outside the range")]
    OutOfRange,
    /// A value cannot be ordered against itself or a configured endpoint.
    #[error("value cannot be ordered")]
    Unordered,
}
