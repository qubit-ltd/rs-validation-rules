// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validation_rules::text::TextRuleError;
use qubit_validation_rules::text::Uri;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

#[test]
fn test_uri_accepts_absolute_rfc3986_examples() {
    for value in [
        "mailto:user@example.com",
        "a:b",
        "https://example.com",
        "urn:isbn:0451450523",
    ] {
        assert_eq!(Uri.validate(value, &()), Ok(()), "{value}");
    }
}

#[test]
fn test_uri_rejects_invalid_syntax_and_relative_references() {
    for value in [
        "https://example.com/%GG",
        "a:b\0",
        "https://例子.example/",
        "relative/path",
    ] {
        assert_eq!(Uri.validate(value, &()), Err(TextRuleError::Uri), "{value:?}");
    }
}

#[test]
fn test_registered_uri_rejects_invalid_percent_escape() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid");
    let bound = registry
        .bind("qubit.rules.text.uri", InputType::Text, &[])
        .expect("URI rule binds without arguments");
    let outcome = bound
        .validate(
            ValidationValue::Text("https://example.com/%GG"),
            &BoundValidationContext::new(&[]),
        )
        .expect("URI rule executes");
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("invalid percent escape must produce a violation: {outcome:?}");
    };
    assert_eq!(violations[0].code().as_str(), "text.uri");
}
