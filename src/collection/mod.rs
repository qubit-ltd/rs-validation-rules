// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(missing_docs)]

pub use crate::internal::legacy::ItemCount;
pub use crate::internal::legacy::ItemCountError;
pub use crate::internal::legacy::Range;
pub use crate::internal::legacy::RangeError;

pub mod item_count {
    pub use super::ItemCount;
    pub use super::ItemCountError;
}
pub mod range {
    pub use super::Range;
    pub use super::RangeError;
}
