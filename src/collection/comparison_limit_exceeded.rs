// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Indicates that duplicate detection exhausted its comparison budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("unique item comparison limit exceeded")]
pub struct ComparisonLimitExceeded;
