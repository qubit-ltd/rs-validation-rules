// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Rejection reason for a non-blank text rule.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::NonBlank;
/// use qubit_validation_rules::text::NonBlankError;
/// use qubit_validator::Validator;
///
/// assert_eq!(NonBlank.validate(" \t", &()), Err(NonBlankError::Blank));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum NonBlankError {
    /// The input contains only whitespace or is empty.
    #[error("text is blank")]
    Blank,
}
