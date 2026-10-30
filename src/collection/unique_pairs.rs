// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Index-pair iteration shared by collection uniqueness consumers.

/// Iterates over all unique index pairs in duplicate-search order.
///
/// Pairs are emitted by increasing second index, then increasing first index.
/// Iterating a sequence of `len` items takes O(len²) time and O(1) space. This
/// iterator only produces indices; it does not access or compare elements.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::collection::UniquePairs;
///
/// let pairs = UniquePairs::new(3).collect::<Vec<_>>();
/// assert_eq!(pairs, [(0, 1), (0, 2), (1, 2)]);
/// ```
#[derive(Debug, Clone)]
pub struct UniquePairs {
    /// Number of items whose index pairs are being visited.
    len: usize,
    /// First index in the next pair to emit.
    first: usize,
    /// Second index in the next pair to emit.
    second: usize,
}

impl UniquePairs {
    /// Creates an iterator over the unique pairs in a sequence of `len` items.
    ///
    /// # Parameters
    /// - `len`: Number of items whose index pairs should be visited.
    ///
    /// # Returns
    /// An iterator that emits each index pair in duplicate-search order.
    #[must_use]
    #[inline]
    pub const fn new(len: usize) -> Self {
        Self {
            len,
            first: 0,
            second: 1,
        }
    }
}

impl Iterator for UniquePairs {
    /// The next pair of indices in duplicate-search order.
    type Item = (usize, usize);

    /// Returns the next pair in duplicate-search order.
    ///
    /// # Returns
    /// - `Some((first, second))` when another unique pair is available.
    /// - `None` after all pairs have been emitted.
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.second >= self.len {
            return None;
        }
        let pair = (self.first, self.second);
        self.first += 1;
        if self.first >= self.second {
            self.second += 1;
            self.first = 0;
        }
        Some(pair)
    }
}
