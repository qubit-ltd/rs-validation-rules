// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![allow(missing_docs)]

pub use TextRuleError as RegexMatchError;

pub use crate::internal::legacy::RegexMatch;
pub use crate::internal::legacy::TextRuleError;
pub mod regex_match {
    pub use super::RegexMatch;
    pub use super::RegexMatchError;
}
