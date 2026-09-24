use qubit_validator::Validator;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Character profiles accepted by `AllowedChars`.
pub enum CharacterSet {
    /// Any Unicode scalar value.
    Unicode,
    /// Unicode scalar values except control characters.
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
            CharacterSet::PrintableUnicode => !c.is_control(),
            CharacterSet::Ascii => c.is_ascii(),
            CharacterSet::PrintableAscii => c.is_ascii() && !c.is_ascii_control(),
            CharacterSet::Code => c.is_ascii_alphanumeric() || "._-".contains(c),
        });
        if ok { Ok(()) } else { Err(AllowedCharsError::Invalid) }
    }
}
