// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod byte_length;
mod byte_length_error;
mod char_length;
mod text_length_error;

pub use byte_length::ByteLength;
pub use byte_length_error::ByteLengthError;
pub use char_length::CharLength;
pub use text_length_error::TextLengthError;

/// Returns whether `count` is below a configured minimum.
///
/// # Parameters
/// - `count`: Measured scalar or byte count.
/// - `min`: Optional inclusive lower bound.
///
/// # Returns
/// `true` when `min` exists and `count` is smaller.
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
pub(super) fn above_max(count: usize, max: Option<u32>) -> bool {
    max.is_some_and(|bound| count > bound as usize)
}
#[cfg(all(test, target_pointer_width = "64"))]
mod tests {
    use super::above_max;
    use super::below_min;

    #[test]
    fn test_counts_above_u32_max_do_not_wrap() {
        let count = u32::MAX as usize + 1;

        assert!(!below_min(count, Some(1)));
        assert!(above_max(count, Some(u32::MAX)));
        assert!(above_max(count, Some(0)));
    }
}
