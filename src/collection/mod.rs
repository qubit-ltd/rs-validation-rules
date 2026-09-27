// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Collection and range validation rules.

mod comparison_limit_exceeded;
mod item_count;
mod item_count_error;
mod range;
mod range_error;
mod unique_items;
mod unique_pairs;

pub use comparison_limit_exceeded::ComparisonLimitExceeded;
pub use item_count::ItemCount;
pub use item_count_error::ItemCountError;
pub use range::Range;
pub use range_error::RangeError;
pub use unique_items::UniqueItems;
pub use unique_pairs::UniquePairs;
