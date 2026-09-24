use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
/// Item-count validation errors.
pub enum ItemCountError {
    #[error("too few items")]
    /// The count is below `min`.
    TooSmall {
        /// Required minimum count.
        min: usize,
    },
    #[error("too many items")]
    /// The count exceeds `max`.
    TooLarge {
        /// Allowed maximum count.
        max: usize,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Bounds a collection item count represented by `usize`.
pub struct ItemCount {
    min: Option<usize>,
    max: Option<usize>,
}
impl ItemCount {
    /// Creates a rule with inclusive optional bounds.
    /// Returns `ParameterOutOfRange` if `min` exceeds `max`.
    pub fn new(min: Option<usize>, max: Option<usize>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<usize, ()> for ItemCount {
    type Error = ItemCountError;
    fn validate(&self, v: &usize, _: &()) -> Result<(), Self::Error> {
        if self.min.is_some_and(|m| *v < m) {
            return Err(ItemCountError::TooSmall { min: self.min.unwrap() });
        }
        if self.max.is_some_and(|m| *v > m) {
            return Err(ItemCountError::TooLarge { max: self.max.unwrap() });
        }
        Ok(())
    }
}
