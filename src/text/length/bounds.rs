// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Returns whether `count` is below a configured minimum.
///
/// # Parameters
/// - `count`: Measured scalar or byte count.
/// - `min`: Optional inclusive lower bound.
///
/// # Returns
/// `true` when `min` exists and `count` is smaller.
#[must_use]
#[inline]
pub(super) fn below_min(count: usize, min: Option<u32>) -> bool {
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
#[must_use]
#[inline]
pub(super) fn above_max(count: usize, max: Option<u32>) -> bool {
    max.is_some_and(|bound| count > bound as usize)
}
