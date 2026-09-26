// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed duplicate detection for borrowed sequences.

use super::comparison_limit_exceeded::ComparisonLimitExceeded;

/// Finds the first duplicate according to [`PartialEq`] within a comparison
/// budget.
pub struct UniqueItems;

impl UniqueItems {
    /// Returns the first equal pair as `(first_index, second_index)`, or
    /// `None` when all items are distinct. The later index is searched first,
    /// then earlier indices in ascending order. At most `max_comparisons`
    /// calls to [`PartialEq::eq`] are made. The worst-case work is O(n²),
    /// and neither the result nor the error formats or stores element values.
    pub fn first_duplicate_with_limit<T: PartialEq>(
        values: &[T],
        max_comparisons: usize,
    ) -> Result<Option<(usize, usize)>, ComparisonLimitExceeded> {
        let mut comparisons = 0;
        for second in 1..values.len() {
            for first in 0..second {
                if comparisons >= max_comparisons {
                    return Err(ComparisonLimitExceeded);
                }
                comparisons += 1;
                if values[first] == values[second] {
                    return Ok(Some((first, second)));
                }
            }
        }
        Ok(None)
    }
}
