// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Failure reported when a collection count violates its bounds.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::collection::ItemCount;
/// use qubit_validation_rules::collection::ItemCountError;
/// use qubit_validator::Validator;
///
/// let rule = ItemCount::new(Some(2), None).expect("valid bound");
/// assert_eq!(rule.validate(&1, &()), Err(ItemCountError::TooSmall { min: 2 }));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum ItemCountError {
    /// The collection contains fewer items than the configured minimum.
    #[error("too few items")]
    TooSmall {
        /// Required inclusive minimum count.
        min: usize,
    },
    /// The collection contains more items than the configured maximum.
    #[error("too many items")]
    TooLarge {
        /// Allowed inclusive maximum count.
        max: usize,
    },
}
