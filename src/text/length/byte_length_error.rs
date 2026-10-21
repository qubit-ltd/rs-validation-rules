// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// UTF-8 byte-length validation errors.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::ByteLength;
/// use qubit_validation_rules::text::ByteLengthError;
/// use qubit_validator::Validator;
///
/// let rule = ByteLength::new(Some(2), None).expect("valid bound");
/// assert_eq!(rule.validate("x", &()), Err(ByteLengthError::TooFewBytes { min: 2 }));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
pub enum ByteLengthError {
    /// The UTF-8 byte count is below `min`.
    #[error("text has too few bytes")]
    TooFewBytes {
        /// Required minimum count.
        min: u32,
    },
    /// The UTF-8 byte count exceeds `max`.
    #[error("text has too many bytes")]
    TooManyBytes {
        /// Allowed maximum count.
        max: u32,
    },
}
