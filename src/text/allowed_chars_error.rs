// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Rejection reason returned when an input violates a character profile.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::AllowedChars;
/// use qubit_validation_rules::text::AllowedCharsError;
/// use qubit_validation_rules::text::CharacterSet;
/// use qubit_validator::Validator;
///
/// let rule = AllowedChars::new(CharacterSet::Ascii);
/// assert_eq!(rule.validate("中", &()), Err(AllowedCharsError::Invalid));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum AllowedCharsError {
    /// Input contains a character outside the configured profile.
    #[error("invalid characters")]
    Invalid,
}
