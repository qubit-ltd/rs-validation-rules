// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod bounds;
mod byte_length;
mod byte_length_error;
mod char_length;
mod text_length_error;

pub use byte_length::ByteLength;
pub use byte_length_error::ByteLengthError;
pub use char_length::CharLength;
pub use text_length_error::TextLengthError;

#[cfg(all(test, target_pointer_width = "64"))]
mod tests {
    use super::bounds::above_max;
    use super::bounds::below_min;

    #[test]
    fn test_counts_above_u32_max_do_not_wrap() {
        let count = u32::MAX as usize + 1;

        assert!(!below_min(count, Some(1)));
        assert!(above_max(count, Some(u32::MAX)));
        assert!(above_max(count, Some(0)));
    }
}
