// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Character profiles accepted by [`crate::text::AllowedChars`].
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::CharacterSet;
/// use qubit_validation_rules::text::AllowedChars;
/// use qubit_validator::Validator;
///
/// let rule = AllowedChars::new(CharacterSet::PrintableAscii);
/// assert!(rule.validate("key_1", &()).is_ok());
/// assert!(rule.validate("key\n1", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSet {
    /// Any Unicode scalar value.
    Unicode,
    /// Unicode letters, marks, numbers, punctuation, symbols, and space
    /// separators.
    ///
    /// Control, format, private-use, unassigned, line-separator, and
    /// paragraph-separator characters are rejected. This also rejects format
    /// characters such as zero-width joiners in emoji sequences.
    PrintableUnicode,
    /// Any ASCII character.
    Ascii,
    /// ASCII characters except control characters.
    PrintableAscii,
    /// ASCII letters and digits, period, underscore, and hyphen.
    Code,
}
