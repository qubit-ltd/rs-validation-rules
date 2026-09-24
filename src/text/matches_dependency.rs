use qubit_validator::BoundValidationContext;
use qubit_validator::Validator;

use super::TextRuleError;

/// Requires target text to match the first declared text dependency.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MatchesDependency;

impl<'a> Validator<str, BoundValidationContext<'a>> for MatchesDependency {
    type Error = TextRuleError;

    fn validate(&self, value: &str, context: &BoundValidationContext<'a>) -> Result<(), Self::Error> {
        let expected = context.text(0).map_err(|_| TextRuleError::DependencyMismatch)?;
        if value == expected {
            Ok(())
        } else {
            Err(TextRuleError::DependencyMismatch)
        }
    }
}
