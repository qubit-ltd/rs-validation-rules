// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::Validator;

use crate::text::text_rule_error::TextRuleError;

/// Checks a bounded ASCII email address profile without network access.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::EmailAscii;
/// use qubit_validator::Validator;
///
/// assert!(EmailAscii.validate("user@example.com", &()).is_ok());
/// assert!(EmailAscii.validate("user@例子.测试", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailAscii;

impl Validator<str, ()> for EmailAscii {
    /// Format failure emitted when the input does not meet the profile.
    type Error = TextRuleError;

    /// Validates the local and domain parts against the crate's ASCII profile.
    ///
    /// # Parameters
    /// - `value`: Email text to validate.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the input satisfies the ASCII email profile.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Email`] when syntax, character, or length
    /// constraints fail.
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let mut parts = value.split('@');
        let local = parts.next().ok_or(TextRuleError::Email)?;
        let domain = parts.next().ok_or(TextRuleError::Email)?;
        if parts.next().is_some()
            || value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !value.is_ascii()
            || value.chars().any(char::is_whitespace)
            || local.starts_with('.')
            || local.ends_with('.')
            || local.contains("..")
            || !local
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-/=?^_`{|}~.".contains(c))
            || !domain.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
        {
            return Err(TextRuleError::Email);
        }
        Ok(())
    }
}
