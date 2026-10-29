// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Errors produced by text format rules and the optional regex rule.
///
/// The `Blank`, `DisallowedCharacters`, and `DependencyMismatch` variants are
/// retained for API compatibility. The corresponding built-in rules return
/// dedicated error types instead:
///
/// | Rule | Error returned by the rule |
/// | --- | --- |
/// | [`crate::text::NonBlank`] | [`crate::text::NonBlankError::Blank`] |
/// | [`crate::text::AllowedChars`] | [`crate::text::AllowedCharsError::Invalid`] |
/// | [`crate::text::MatchesDependency`] | [`crate::text::MatchesDependencyError::MissingDependency`], [`crate::text::MatchesDependencyError::Mismatch`] |
///
/// Dependency comparison distinguishes a missing dependency from a mismatch.
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
    /// Compatibility variant for blank text. The built-in
    /// [`crate::text::NonBlank`] rule returns [`crate::text::NonBlankError`]
    /// instead.
    #[error("text is blank")]
    Blank,
    /// Compatibility variant for disallowed characters. The built-in
    /// [`crate::text::AllowedChars`] rule returns
    /// [`crate::text::AllowedCharsError`] instead.
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
    /// Compatibility variant for a dependency mismatch. The built-in
    /// [`crate::text::MatchesDependency`] rule returns
    /// [`crate::text::MatchesDependencyError`] instead.
    #[error("text does not match its required dependency")]
    DependencyMismatch,
}
