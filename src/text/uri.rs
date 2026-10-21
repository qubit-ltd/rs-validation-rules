// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use fluent_uri::Uri as FluentUri;
use qubit_validator::Validator;

use crate::text::text_rule_error::TextRuleError;

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
