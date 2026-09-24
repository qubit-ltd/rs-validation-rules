use qubit_validator::Validator;
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
/// Rejection reason for a non-blank text rule.
pub enum NonBlankError {
    #[error("text is blank")]
    /// The input contains only whitespace or is empty.
    Blank,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Rejects empty text and text containing only Unicode whitespace.
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
