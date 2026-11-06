// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Redacted formatting for [`super::super::Range`].

use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Result;
use std::ops::Bound;

/// Formats a bound variant without exposing its endpoint value.
///
/// # Type Parameters
///
/// * `T` is the endpoint type. Its value is never formatted.
pub(in crate::collection::range) struct RedactedBound<'a, T>(
    /// Endpoint bound whose value is deliberately hidden by the formatter.
    pub(in crate::collection::range) &'a Bound<T>,
);

impl<T> Debug for RedactedBound<'_, T> {
    /// Writes the bound variant and a fixed redaction marker.
    ///
    /// # Parameters
    ///
    /// * `formatter` receives the bound variant name and redaction marker.
    ///
    /// # Returns
    ///
    /// Returns the formatter's result, including any formatting error.
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self.0 {
            Bound::Unbounded => formatter.write_str("Unbounded"),
            Bound::Included(_) => formatter.write_str("Included(<redacted>)"),
            Bound::Excluded(_) => formatter.write_str("Excluded(<redacted>)"),
        }
    }
}
