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
/// The value must contain exactly one `@`, contain only ASCII characters, and
/// contain no whitespace; it is neither trimmed nor normalized. The complete
/// address is limited to 254 bytes, and the non-empty local part is limited to
/// 64 bytes. The local part accepts ASCII letters, digits, and
/// ``!#$%&'*+-/=?^_`{|}~.``; dots cannot lead, trail, or repeat. Quoted local
/// parts and comments are not supported.
///
/// The domain consists of dot-separated labels of 1 to 63 bytes, containing
/// only ASCII letters, digits, and hyphens. A label cannot start or end with a
/// hyphen. Empty labels, trailing dots, underscores, domain literals, and
/// Unicode domain text are rejected. A single-label domain such as `a@b` is
/// accepted. ASCII punycode labels are treated as ordinary ASCII labels; IDNA
/// validity, public suffixes, DNS resolution, mailbox existence, delivery, and
/// application-specific domain policy are not checked. This profile does not
/// implement all RFC email syntax.
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
