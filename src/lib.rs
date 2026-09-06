use qubit_validator::{
    ArgumentReader, BindError, BoundValidationContext, ExecutionError, ExecutionErrorKind,
    InputType, NamedValidationArgument, PreparedValidator, RegistrationSource, RuleOutcome,
    ValidationArgument, ValidationValue, Validator, ValidatorDescriptor, ValidatorId,
    ValidatorRegistration, ValidatorSignature, Violation, ViolationCode, ViolationParam,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NonBlankError {
    #[error("text is blank")]
    Blank,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonBlank;
impl Validator<str, ()> for NonBlank {
    type Error = NonBlankError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        if v.trim().is_empty() {
            Err(NonBlankError::Blank)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TextLengthError {
    #[error("text is too short")]
    TooShort { min: u32 },
    #[error("text is too long")]
    TooLong { max: u32 },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharLength {
    min: Option<u32>,
    max: Option<u32>,
}
impl CharLength {
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(
                qubit_validator::BindErrorKind::ParameterOutOfRange,
            ));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for CharLength {
    type Error = TextLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.chars().count() as u32;
        if self.min.is_some_and(|m| n < m) {
            return Err(TextLengthError::TooShort {
                min: self.min.unwrap(),
            });
        }
        if self.max.is_some_and(|m| n > m) {
            return Err(TextLengthError::TooLong {
                max: self.max.unwrap(),
            });
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ByteLengthError {
    #[error("text has too few bytes")]
    TooFewBytes { min: u32 },
    #[error("text has too many bytes")]
    TooManyBytes { max: u32 },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteLength {
    min: Option<u32>,
    max: Option<u32>,
}
impl ByteLength {
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(
                qubit_validator::BindErrorKind::ParameterOutOfRange,
            ));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for ByteLength {
    type Error = ByteLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.len() as u32;
        if self.min.is_some_and(|m| n < m) {
            return Err(ByteLengthError::TooFewBytes {
                min: self.min.unwrap(),
            });
        }
        if self.max.is_some_and(|m| n > m) {
            return Err(ByteLengthError::TooManyBytes {
                max: self.max.unwrap(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSet {
    Unicode,
    PrintableUnicode,
    Ascii,
    PrintableAscii,
    Code,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AllowedCharsError {
    #[error("invalid characters")]
    Invalid,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllowedChars {
    set: CharacterSet,
}
impl AllowedChars {
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
        if ok {
            Ok(())
        } else {
            Err(AllowedCharsError::Invalid)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ItemCountError {
    #[error("too few items")]
    TooSmall { min: usize },
    #[error("too many items")]
    TooLarge { max: usize },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemCount {
    min: Option<usize>,
    max: Option<usize>,
}
impl ItemCount {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(
                qubit_validator::BindErrorKind::ParameterOutOfRange,
            ));
        }
        Ok(Self { min, max })
    }
}
impl Validator<usize, ()> for ItemCount {
    type Error = ItemCountError;
    fn validate(&self, v: &usize, _: &()) -> Result<(), Self::Error> {
        if self.min.is_some_and(|m| *v < m) {
            return Err(ItemCountError::TooSmall {
                min: self.min.unwrap(),
            });
        }
        if self.max.is_some_and(|m| *v > m) {
            return Err(ItemCountError::TooLarge {
                max: self.max.unwrap(),
            });
        }
        Ok(())
    }
}

#[cfg(feature = "china-identity")]
mod china_identity;
#[cfg(feature = "china-identity")]
pub use china_identity::{ChinaIdentity18, ChinaIdentityError, ChinaIdentityFacts};

pub fn registrations() -> Vec<ValidatorRegistration> {
    Vec::new()
}

#[allow(dead_code)]
fn _adapter(
    _: ValidationValue<'_>,
    _: &BoundValidationContext<'_>,
) -> Result<RuleOutcome, ExecutionError> {
    Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))
}
