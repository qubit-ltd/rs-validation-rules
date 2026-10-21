// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::Validator;

use super::non_blank_error::NonBlankError;

/// Rejects empty text and text containing only Unicode whitespace.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::NonBlank;
/// use qubit_validator::Validator;
///
/// assert!(NonBlank.validate("hello", &()).is_ok());
/// assert!(NonBlank.validate(" \t", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonBlank;
impl Validator<str, ()> for NonBlank {
    /// Blank-input failure emitted when no non-whitespace character is present.
    type Error = NonBlankError;

    /// Rejects inputs that contain no non-whitespace Unicode scalar value.
    ///
    /// # Parameters
    /// - `value`: Text to inspect.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when at least one non-whitespace character is present.
    ///
    /// # Errors
    /// Returns [`NonBlankError::Blank`] for empty or all-whitespace text.
    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        if value.trim().is_empty() {
            Err(NonBlankError::Blank)
        } else {
            Ok(())
        }
    }
}
