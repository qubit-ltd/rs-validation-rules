use qubit_validator::Validator;
/// Errors produced by text format and character rules.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TextRuleError {
    #[error("text is blank")]
    /// Text is blank.
    Blank,
    #[error("text contains disallowed characters")]
    /// Text contains a disallowed character.
    DisallowedCharacters,
    #[error("text does not match the pattern")]
    /// Text does not match a regular expression.
    Pattern,
    #[error("text is not a valid email address")]
    /// Text does not meet the ASCII email profile.
    Email,
    #[error("text is not a valid URI")]
    /// Text does not meet the absolute URI profile.
    Uri,
    #[error("text is not a valid UUID")]
    /// Text does not have canonical UUID form.
    Uuid,
    #[error("text is not a valid mobile number")]
    /// Text does not meet the mainland China mobile number structure.
    Mobile,
    /// Text differs from a required text dependency.
    #[error("text does not match its required dependency")]
    DependencyMismatch,
}
/// ASCII email profile used by the standard rules.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailAscii;

impl Validator<str, ()> for EmailAscii {
    type Error = TextRuleError;
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
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Uri;

impl Validator<str, ()> for Uri {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        fluent_uri::Uri::parse(value)
            .map(|_| ())
            .map_err(|_| TextRuleError::Uri)
    }
}

/// Canonical 8-4-4-4-12 UUID text profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UuidText;

impl Validator<str, ()> for UuidText {
    type Error = TextRuleError;
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

/// Mainland China mobile number structural profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChinaMobileStructure;

impl Validator<str, ()> for ChinaMobileStructure {
    type Error = TextRuleError;
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
