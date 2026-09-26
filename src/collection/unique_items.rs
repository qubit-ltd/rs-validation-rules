// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed duplicate detection for borrowed sequences.

/// Finds the first duplicate according to [`PartialEq`].
pub struct UniqueItems;

impl UniqueItems {
    /// Returns the first equal pair as `(first_index, second_index)`, or
    /// `None` when all items are distinct. The later index is searched first,
    /// then earlier indices in ascending order. This performs O(n²)
    /// comparisons and never formats or stores element values.
    pub fn first_duplicate<T: PartialEq>(values: &[T]) -> Option<(usize, usize)> {
        for second in 1..values.len() {
            for first in 0..second {
                if values[first] == values[second] {
                    return Some((first, second));
                }
            }
        }
        None
    }
}
