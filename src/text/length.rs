use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

fn below_min(count: usize, min: Option<u32>) -> bool {
    min.is_some_and(|bound| count < bound as usize)
}

fn above_max(count: usize, max: Option<u32>) -> bool {
    max.is_some_and(|bound| count > bound as usize)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
/// Character-length validation errors.
pub enum TextLengthError {
    #[error("text is too short")]
    /// The Unicode scalar value count is below `min`.
    TooShort {
        /// Required minimum count.
        min: u32,
    },
    #[error("text is too long")]
    /// The Unicode scalar value count exceeds `max`.
    TooLong {
        /// Allowed maximum count.
        max: u32,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Bounds the number of Unicode scalar values in text, not grapheme clusters.
pub struct CharLength {
    min: Option<u32>,
    max: Option<u32>,
}
impl CharLength {
    /// Creates a rule with inclusive optional bounds.
    /// Returns `ParameterOutOfRange` if `min` exceeds `max`.
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for CharLength {
    type Error = TextLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.chars().count();
        if below_min(n, self.min) {
            return Err(TextLengthError::TooShort { min: self.min.unwrap() });
        }
        if above_max(n, self.max) {
            return Err(TextLengthError::TooLong { max: self.max.unwrap() });
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
/// UTF-8 byte-length validation errors.
pub enum ByteLengthError {
    #[error("text has too few bytes")]
    /// The UTF-8 byte count is below `min`.
    TooFewBytes {
        /// Required minimum count.
        min: u32,
    },
    #[error("text has too many bytes")]
    /// The UTF-8 byte count exceeds `max`.
    TooManyBytes {
        /// Allowed maximum count.
        max: u32,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Bounds the UTF-8 byte length of text.
pub struct ByteLength {
    min: Option<u32>,
    max: Option<u32>,
}
impl ByteLength {
    /// Creates a rule with inclusive optional bounds.
    /// Returns `ParameterOutOfRange` if `min` exceeds `max`.
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for ByteLength {
    type Error = ByteLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.len();
        if below_min(n, self.min) {
            return Err(ByteLengthError::TooFewBytes { min: self.min.unwrap() });
        }
        if above_max(n, self.max) {
            return Err(ByteLengthError::TooManyBytes { max: self.max.unwrap() });
        }
        Ok(())
    }
}

#[cfg(all(test, target_pointer_width = "64"))]
mod tests {
    use super::above_max;
    use super::below_min;

    #[test]
    fn counts_above_u32_max_do_not_wrap() {
        let count = u32::MAX as usize + 1;

        assert!(!below_min(count, Some(1)));
        assert!(above_max(count, Some(u32::MAX)));
        assert!(above_max(count, Some(0)));
    }
}
