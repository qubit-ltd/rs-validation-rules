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
#[derive(Debug, Clone)]
pub struct UniquePairs {
    len: usize,
    first: usize,
    second: usize,
}

impl UniquePairs {
    /// Creates an iterator over the unique pairs in a sequence of `len` items.
    pub const fn new(len: usize) -> Self {
        Self {
            len,
            first: 0,
            second: 1,
        }
    }
}

impl Iterator for UniquePairs {
    type Item = (usize, usize);

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
