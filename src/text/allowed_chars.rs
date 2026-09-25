// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::Validator;
use unicode_general_category::GeneralCategory;
use unicode_general_category::get_general_category;

/// Returns whether `c` belongs to the printable Unicode category allowlist.
///
/// # Parameters
/// - `c`: Unicode scalar value whose category is checked.
///
/// # Returns
/// `true` when the category belongs to the printable profile.
fn is_printable_unicode(c: char) -> bool {
    matches!(
        get_general_category(c),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
            | GeneralCategory::NonspacingMark
            | GeneralCategory::SpacingMark
            | GeneralCategory::EnclosingMark
            | GeneralCategory::DecimalNumber
            | GeneralCategory::LetterNumber
            | GeneralCategory::OtherNumber
            | GeneralCategory::ConnectorPunctuation
            | GeneralCategory::DashPunctuation
            | GeneralCategory::OpenPunctuation
            | GeneralCategory::ClosePunctuation
            | GeneralCategory::InitialPunctuation
            | GeneralCategory::FinalPunctuation
            | GeneralCategory::OtherPunctuation
            | GeneralCategory::MathSymbol
            | GeneralCategory::CurrencySymbol
            | GeneralCategory::ModifierSymbol
            | GeneralCategory::OtherSymbol
            | GeneralCategory::SpaceSeparator
    )
}
/// Character profiles accepted by [`AllowedChars`].
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
// qubit-style: allow public-type-layout
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
/// Rejection reason returned when an input violates a character profile.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::AllowedChars;
/// use qubit_validation_rules::text::AllowedCharsError;
/// use qubit_validation_rules::text::CharacterSet;
/// use qubit_validator::Validator;
///
/// let rule = AllowedChars::new(CharacterSet::Ascii);
/// assert_eq!(rule.validate("中", &()), Err(AllowedCharsError::Invalid));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[must_use]
// qubit-style: allow public-type-layout
pub enum AllowedCharsError {
    /// Input contains a character outside the configured profile.
    #[error("invalid characters")]
    Invalid,
}
/// Checks every character against a selected character profile.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::AllowedChars;
/// use qubit_validation_rules::text::CharacterSet;
/// use qubit_validator::Validator;
///
/// let rule = AllowedChars::new(CharacterSet::PrintableAscii);
/// assert!(rule.validate("hello", &()).is_ok());
/// assert!(rule.validate("hello\n", &()).is_err());
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// qubit-style: allow public-type-layout
pub struct AllowedChars {
    /// Character profile applied to every scalar value in the input.
    set: CharacterSet,
}
impl AllowedChars {
    /// Creates a rule for `set`.
    ///
    /// # Parameters
    /// - `set`: Character profile applied to each input scalar value.
    ///
    /// # Returns
    /// A rule configured with the selected profile.
    #[must_use]
    #[inline]
    pub const fn new(set: CharacterSet) -> Self {
        Self { set }
    }
}
impl Validator<str, ()> for AllowedChars {
    /// Character-profile violation emitted for a rejected input.
    type Error = AllowedCharsError;

    /// Applies the configured character profile to every Unicode scalar value.
    ///
    /// # Parameters
    /// - `value`: Text whose characters are checked.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when every character is accepted.
    ///
    /// # Errors
    /// Returns [`AllowedCharsError::Invalid`] when any character is rejected.
    fn validate(&self, value: &str, _context: &()) -> Result<(), Self::Error> {
        let ok = value.chars().all(|c| match self.set {
            CharacterSet::Unicode => true,
            CharacterSet::PrintableUnicode => is_printable_unicode(c),
            CharacterSet::Ascii => c.is_ascii(),
            CharacterSet::PrintableAscii => c.is_ascii() && !c.is_ascii_control(),
            CharacterSet::Code => c.is_ascii_alphanumeric() || "._-".contains(c),
        });
        if ok { Ok(()) } else { Err(AllowedCharsError::Invalid) }
    }
}

#[cfg(test)]
mod tests {
    use qubit_validator::Validator;

    use super::AllowedChars;
    use super::CharacterSet;

    #[test]
    fn test_printable_unicode_matches_the_general_category_allowlist() {
        let rule = AllowedChars::new(CharacterSet::PrintableUnicode);

        for value in ["A", "中", "\u{0301}", "9", "!", "😀", " "] {
            assert!(rule.validate(value, &()).is_ok(), "{value:?}");
        }
        for value in ["\0", "\u{200B}", "\u{E000}", "\u{0378}", "\u{2028}", "\u{2029}"] {
            assert!(rule.validate(value, &()).is_err(), "{value:?}");
        }
    }
}
