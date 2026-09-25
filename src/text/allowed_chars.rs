use qubit_validator::Validator;
use unicode_general_category::GeneralCategory;
use unicode_general_category::get_general_category;

/// Returns whether `c` belongs to the printable Unicode category allowlist.
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Character profiles accepted by `AllowedChars`.
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
/// Rejection reason for an allowed-character rule.
pub enum AllowedCharsError {
    #[error("invalid characters")]
    /// Input contains a character outside the configured profile.
    Invalid,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Checks every character against a selected character profile.
pub struct AllowedChars {
    set: CharacterSet,
}
impl AllowedChars {
    /// Creates a rule for `set`.
    pub const fn new(set: CharacterSet) -> Self {
        Self { set }
    }
}
impl Validator<str, ()> for AllowedChars {
    type Error = AllowedCharsError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let ok = v.chars().all(|c| match self.set {
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
    fn printable_unicode_matches_the_general_category_allowlist() {
        let rule = AllowedChars::new(CharacterSet::PrintableUnicode);

        for value in ["A", "中", "\u{0301}", "9", "!", "😀", " "] {
            assert!(rule.validate(value, &()).is_ok(), "{value:?}");
        }
        for value in ["\0", "\u{200B}", "\u{E000}", "\u{0378}", "\u{2028}", "\u{2029}"] {
            assert!(rule.validate(value, &()).is_err(), "{value:?}");
        }
    }
}
