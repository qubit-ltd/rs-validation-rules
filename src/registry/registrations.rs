// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::ValidatorRegistration;

use super::collection;
#[cfg(feature = "decimal")]
use super::decimal;
#[cfg(feature = "regex")]
use super::regex;
use super::text;
#[cfg(feature = "time")]
use super::time;

/// Returns built-in rule registrations for a local validator registry.
/// Feature-gated rules are included when their features are enabled.
///
/// # Returns
/// Registrations in the crate's stable built-in rule set.
///
/// # Examples
/// Bind a rule with a named argument and inspect its structured rejection:
///
/// ```
/// use qubit_validation_rules::ids;
/// use qubit_validation_rules::registrations;
/// use qubit_validator::BoundValidationContext;
/// use qubit_validator::InputType;
/// use qubit_validator::NamedValidationArgument;
/// use qubit_validator::ValidationArgument;
/// use qubit_validator::ValidationOutcome;
/// use qubit_validator::ValidationValue;
/// use qubit_validator::ValidatorRegistry;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = ValidatorRegistry::from_registrations(registrations())?;
///     let arguments = [NamedValidationArgument::new("set", ValidationArgument::String("code"))];
///     let rule = registry.bind(ids::TEXT_ALLOWED_CHARS, InputType::Text, &arguments)?;
///     let context = BoundValidationContext::new(&[]);
///     assert_eq!(rule.validate(ValidationValue::Text("job_1"), &context)?, ValidationOutcome::Valid);
///     let ValidationOutcome::Invalid(violations) = rule.validate(ValidationValue::Text("job 1"), &context)? else {
///         panic!("space must be rejected by the code profile");
///     };
///     assert_eq!(violations[0].code().as_str(), "text.disallowed_characters");
///     Ok(())
/// }
/// ```
///
/// Bind a dependency-based rule and distinguish a mismatch from an execution
/// error:
///
/// ```
/// use qubit_validation_rules::ids;
/// use qubit_validation_rules::registrations;
/// use qubit_validator::InputType;
/// use qubit_validator::NamedValidationDependency;
/// use qubit_validator::ValidationOutcome;
/// use qubit_validator::ValidationValue;
/// use qubit_validator::ValidatorRegistry;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = ValidatorRegistry::from_registrations(registrations())?;
///     let rule = registry.bind(ids::TEXT_MATCHES_DEPENDENCY, InputType::Text, &[])?;
///     let dependencies = [NamedValidationDependency::new("expected", ValidationValue::Text("same"))];
///     assert_eq!(rule.validate_named(ValidationValue::Text("same"), &dependencies)?, ValidationOutcome::Valid);
///     let ValidationOutcome::Invalid(violations) = rule.validate_named(ValidationValue::Text("different"), &dependencies)? else {
///         panic!("different text must be rejected");
///     };
///     assert_eq!(violations[0].code().as_str(), "text.dependency_mismatch");
///     Ok(())
/// }
/// ```
#[must_use]
pub fn registrations() -> Vec<ValidatorRegistration> {
    #[allow(unused_mut)]
    let mut rules = vec![
        text::REG_NON_BLANK,
        text::REG_CHAR_LENGTH,
        text::REG_BYTE_LENGTH,
        text::REG_ALLOWED_CHARS,
        text::REG_EMAIL,
        text::REG_MATCHES_DEPENDENCY,
        text::REG_MOBILE,
        text::REG_URI,
        text::REG_UUID,
        collection::REG_COUNT,
    ];
    #[cfg(feature = "regex")]
    rules.push(regex::REG_REGEX);
    #[cfg(feature = "decimal")]
    rules.push(decimal::REG_DECIMAL);
    #[cfg(feature = "time")]
    rules.push(time::REG_TIME);
    rules
}
