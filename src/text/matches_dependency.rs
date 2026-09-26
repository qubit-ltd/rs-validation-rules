// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::BoundValidationContext;
use qubit_validator::Validator;

use super::matches_dependency_error::MatchesDependencyError;

/// Requires target text to match the first declared text dependency.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::{MatchesDependency, MatchesDependencyError};
/// use qubit_validator::BoundValidationContext;
/// use qubit_validator::ValidationValue;
/// use qubit_validator::Validator;
///
/// let values = [ValidationValue::Text("confirmation")];
/// let context = BoundValidationContext::new(&values);
/// let rule = MatchesDependency;
/// assert!(rule.validate("confirmation", &context).is_ok());
/// assert_eq!(rule.validate("different", &context), Err(MatchesDependencyError::Mismatch));
/// assert_eq!(
///     rule.validate("confirmation", &BoundValidationContext::new(&[])),
///     Err(MatchesDependencyError::MissingDependency),
/// );
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MatchesDependency;

impl<'a> Validator<str, BoundValidationContext<'a>> for MatchesDependency {
    /// Distinguishes an unreadable dependency from unequal text.
    type Error = MatchesDependencyError;

    /// Compares the input with dependency slot zero without including either
    /// value in the returned error.
    ///
    /// # Parameters
    /// - `value`: Text being validated.
    /// - `context`: Bound context whose first dependency must be text.
    ///
    /// # Returns
    /// Returns `Ok(())` when the input equals the first dependency text.
    ///
    /// # Errors
    /// Returns [`MatchesDependencyError::MissingDependency`] when slot zero is
    /// absent or not text, and [`MatchesDependencyError::Mismatch`] when the
    /// texts differ.
    fn validate(&self, value: &str, context: &BoundValidationContext<'a>) -> Result<(), Self::Error> {
        let expected = context.text(0).map_err(|_| MatchesDependencyError::MissingDependency)?;
        if value == expected {
            Ok(())
        } else {
            Err(MatchesDependencyError::Mismatch)
        }
    }
}
