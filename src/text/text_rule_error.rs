// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

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
