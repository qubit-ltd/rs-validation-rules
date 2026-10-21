// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::Validator;

use crate::text::text_rule_error::TextRuleError;

/// Checks the canonical 8-4-4-4-12 UUID hexadecimal text shape.
///
/// This rule does not interpret UUID version or variant bits.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::UuidText;
/// use qubit_validator::Validator;
///
/// assert!(UuidText.validate("550e8400-e29b-41d4-a716-446655440000", &()).is_ok());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UuidText;

impl Validator<str, ()> for UuidText {
    /// UUID-shape failure emitted when the input does not match the profile.
    type Error = TextRuleError;

    /// Checks UUID punctuation positions and ASCII hexadecimal digits.
    ///
    /// # Parameters
    /// - `value`: Text to check.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the text matches the canonical UUID shape.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Uuid`] when length, separators, or digits do
    /// not match the canonical shape.
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.len() == 36
            && value.as_bytes().iter().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    *b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            Ok(())
        } else {
            Err(TextRuleError::Uuid)
        }
    }
}
