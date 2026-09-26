// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Error returned when a direct dependency comparison cannot succeed.
///
/// Neither variant includes the input or dependency text in its diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
// qubit-style: allow public-type-layout
pub enum MatchesDependencyError {
    /// Slot zero is absent, missing, or not text.
    #[error("required text dependency is missing")]
    MissingDependency,
    /// The input differs from the text in slot zero.
    #[error("text does not match its required dependency")]
    Mismatch,
}
