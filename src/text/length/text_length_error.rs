// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Character-length validation errors.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::CharLength;
/// use qubit_validation_rules::text::TextLengthError;
/// use qubit_validator::Validator;
///
/// let rule = CharLength::new(Some(2), None).expect("valid bound");
/// assert_eq!(rule.validate("x", &()), Err(TextLengthError::TooShort { min: 2 }));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum TextLengthError {
    /// The Unicode scalar value count is below `min`.
    #[error("text is too short")]
    TooShort {
        /// Required minimum count.
        min: u32,
    },
    /// The Unicode scalar value count exceeds `max`.
    #[error("text is too long")]
    TooLong {
        /// Allowed maximum count.
        max: u32,
    },
}
