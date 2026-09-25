// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use fluent_uri::Uri as FluentUri;
use qubit_validator::Validator;

/// Errors produced by text format and character rules.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::EmailAscii;
/// use qubit_validation_rules::text::TextRuleError;
/// use qubit_validator::Validator;
///
/// assert_eq!(EmailAscii.validate("invalid", &()), Err(TextRuleError::Email));
/// ```
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
// qubit-style: allow public-type-layout
pub enum TextRuleError {
    /// Text is blank.
    #[error("text is blank")]
    Blank,
    /// Text contains a disallowed character.
    #[error("text contains disallowed characters")]
    DisallowedCharacters,
    /// Text does not match a regular expression.
    #[error("text does not match the pattern")]
    Pattern,
    /// Text does not meet the ASCII email profile.
    #[error("text is not a valid email address")]
    Email,
    /// Text does not meet the absolute URI profile.
    #[error("text is not a valid URI")]
    Uri,
    /// Text does not have canonical UUID form.
    #[error("text is not a valid UUID")]
    Uuid,
    /// Text does not meet the mainland China mobile number structure.
    #[error("text is not a valid mobile number")]
    Mobile,
    /// Text differs from a required text dependency.
    #[error("text does not match its required dependency")]
    DependencyMismatch,
}
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
// qubit-style: allow public-type-layout
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

/// Validates the generic RFC 3986 syntax of an absolute URI.
///
/// This accepts non-Web schemes such as `mailto:` and `urn:`. It does not
/// verify that a host exists or that a scheme is appropriate for an
/// application.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::Uri;
/// use qubit_validator::Validator;
///
/// assert!(Uri.validate("mailto:user@example.com", &()).is_ok());
/// assert!(Uri.validate("relative/path", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
// qubit-style: allow public-type-layout
pub struct Uri;

impl Validator<str, ()> for Uri {
    /// URI syntax failure emitted when parsing rejects the input.
    type Error = TextRuleError;

    /// Parses the input as a generic absolute URI.
    ///
    /// # Parameters
    /// - `value`: URI text to parse.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when parsing accepts the absolute URI syntax.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Uri`] when the text is not valid absolute URI
    /// syntax.
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        FluentUri::parse(value).map(|_| ()).map_err(|_| TextRuleError::Uri)
    }
}

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
// qubit-style: allow public-type-layout
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

/// Checks the structural shape of an 11-digit mainland China mobile number.
///
/// The rule does not verify number assignment or reachability.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::ChinaMobileStructure;
/// use qubit_validator::Validator;
///
/// assert!(ChinaMobileStructure.validate("13800138000", &()).is_ok());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
// qubit-style: allow public-type-layout
pub struct ChinaMobileStructure;

impl Validator<str, ()> for ChinaMobileStructure {
    /// Mobile-number structure failure emitted for a rejected input.
    type Error = TextRuleError;

    /// Checks the length, prefix, and ASCII-digit structure.
    ///
    /// # Parameters
    /// - `value`: Mobile number text to validate.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the input has the expected mobile-number shape.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Mobile`] when the structural profile fails.
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.len() == 11
            && value.starts_with('1')
            && value.as_bytes().get(1).is_some_and(|b| (b'3'..=b'9').contains(b))
            && value.chars().all(|c| c.is_ascii_digit())
        {
            Ok(())
        } else {
            Err(TextRuleError::Mobile)
        }
    }
}
