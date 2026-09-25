// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;
#[cfg(feature = "regex")]
use regex::Regex;

use crate::text::TextRuleError;

/// Matches the entire input against a compiled regular expression.
///
/// Anchors are added by the constructor, so callers provide only the pattern
/// body. The match is Unicode-aware according to the regex crate's defaults.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::regex_rule::RegexMatch;
/// use qubit_validator::Validator;
///
/// let rule = RegexMatch::new(r"[a-z]+@[a-z]+\.com").expect("valid pattern");
/// assert!(rule.validate("user@example.com", &()).is_ok());
/// assert!(rule.validate("prefix user@example.com", &()).is_err());
/// ```
#[cfg(feature = "regex")]
pub struct RegexMatch(
    /// Compiled pattern anchored to match the entire input.
    Regex,
);

#[cfg(feature = "regex")]
impl RegexMatch {
    /// Compiles `pattern` as a full-string match.
    ///
    /// # Parameters
    /// - `pattern`: Regular-expression body to match against the complete
    ///   input.
    ///
    /// # Returns
    /// A compiled rule that succeeds only when the entire input matches.
    ///
    /// # Errors
    /// Returns `InvalidPattern` when the regex syntax is invalid.
    pub fn new(pattern: &str) -> Result<Self, BindError> {
        Regex::new(&format!(r"\A(?:{pattern})\z"))
            .map(Self)
            .map_err(|_| BindError::new(BindErrorKind::InvalidPattern))
    }
}

#[cfg(feature = "regex")]
impl Validator<str, ()> for RegexMatch {
    /// Pattern mismatch emitted when the complete input does not match.
    type Error = TextRuleError;

    /// Tests whether the compiled pattern matches the complete input.
    ///
    /// # Parameters
    /// - `value`: Text to match.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the entire input matches the compiled pattern.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Pattern`] when the pattern does not match.
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        self.0.is_match(value).then_some(()).ok_or(TextRuleError::Pattern)
    }
}
