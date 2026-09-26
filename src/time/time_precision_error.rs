// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A temporal value has finer resolution than declared.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TimePrecisionError {
    /// The nanosecond component is not divisible by the declared unit.
    #[error("temporal precision exceeded")]
    Precision,
}
