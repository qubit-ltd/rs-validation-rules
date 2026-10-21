// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::Validator;

use crate::text::text_rule_error::TextRuleError;

/// Checks the structural shape of an 11-digit mainland China mobile number.
///
/// The rule does not verify number assignment or reachability.
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::text::ChinaMobileStructure;
/// use qubit_validator::Validator;
///
/// assert!(ChinaMobileStructure.validate("13800138000", &()).is_ok());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChinaMobileStructure;

impl Validator<str, ()> for ChinaMobileStructure {
    /// Mobile-number structure failure emitted for a rejected input.
    type Error = TextRuleError;

    /// Checks the length, prefix, and ASCII-digit structure.
    ///
    /// # Parameters
    /// - `value`: Mobile number text to validate.
    /// - `context`: Unused unit context.
    ///
    /// # Returns
    /// Returns `Ok(())` when the input has the expected mobile-number shape.
    ///
    /// # Errors
    /// Returns [`TextRuleError::Mobile`] when the structural profile fails.
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
