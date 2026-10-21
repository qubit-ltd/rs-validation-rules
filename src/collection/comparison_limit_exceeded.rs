// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Indicates that duplicate detection exhausted its comparison budget before
/// it could finish checking the input.
///
/// The error contains no element values or comparison counts. It is returned
/// by [`super::UniqueItems::first_duplicate_with_limit`] when the configured
/// budget is too small for the comparisons needed.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::collection::{ComparisonLimitExceeded, UniqueItems};
///
/// let result = UniqueItems::first_duplicate_with_limit(&[1, 2], 0);
/// assert_eq!(result, Err(ComparisonLimitExceeded));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
#[error("unique item comparison limit exceeded")]
pub struct ComparisonLimitExceeded;
