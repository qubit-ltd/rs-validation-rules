// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Collection and range validation rules.

mod item_count;
mod item_count_error;
mod range;
mod range_error;

pub use item_count::ItemCount;
pub use item_count_error::ItemCountError;
pub use range::Range;
pub use range_error::RangeError;
