use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

use crate::text::TextRuleError;

/// A compiled full-string regular expression rule.
#[cfg(feature = "regex")]
pub struct RegexMatch(regex::Regex);

#[cfg(feature = "regex")]
impl RegexMatch {
    /// Compiles `pattern` as a full-string match.
    /// Returns `InvalidPattern` when the regex syntax is invalid.
    pub fn new(pattern: &str) -> Result<Self, BindError> {
        regex::Regex::new(&format!(r"\A(?:{pattern})\z"))
            .map(Self)
            .map_err(|_| BindError::new(BindErrorKind::InvalidPattern))
    }
}

#[cfg(feature = "regex")]
impl Validator<str, ()> for RegexMatch {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        self.0.is_match(value).then_some(()).ok_or(TextRuleError::Pattern)
    }
}
