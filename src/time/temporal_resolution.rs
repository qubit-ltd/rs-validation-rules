// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Supported exact temporal resolutions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemporalResolution {
    /// Whole seconds.
    Second,
    /// Whole milliseconds.
    Millisecond,
    /// Whole microseconds.
    Microsecond,
    /// Any nanosecond value.
    Nanosecond,
}
