// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::SkipReason;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationLimits;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationReport;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

const CHAR_LENGTH_ID: &str = "qubit.rules.text.char_length";

/// Binds the standard character-length rule for one password field.
fn bind_password_rule(minimum: u32) -> BoundValidator {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("built-in rules are valid");
    let arguments = [NamedValidationArgument::new(
        "min",
        ValidationArgument::Unsigned(u128::from(minimum)),
    )];
    registry
        .bind(CHAR_LENGTH_ID, InputType::Text, &arguments, &[])
        .expect("character-length rule binds")
}

/// Keeps the original password violation path when confirmation is skipped.
#[test]
fn test_model_flow_keeps_prerequisite_at_its_original_absolute_path() {
    let bound = bind_password_rule(20);
    let context = BoundValidationContext::new(&[]);
    let outcome = bound
        .validate(ValidationValue::Text("private-value"), &context)
        .expect("password validation executes");
    let password_path = ValidationPath::root().with_field("user").with_field("password");
    let confirmation_path = ValidationPath::root().with_field("user").with_field("confirmation");
    let mut report = ValidationReport::new();

    assert!(
        report
            .record_outcome(0, password_path.clone(), outcome)
            .expect("password result fits")
    );
    let evidence = report.violations()[0].clone();
    let skipped = ValidationOutcome::failed_prerequisite(vec![evidence])
        .expect("failed prerequisite includes its original violation");
    assert!(
        report
            .record_outcome(1, confirmation_path.clone(), skipped)
            .expect("skip result fits")
    );

    assert_eq!(report.violations()[0].path(), &password_path);
    assert_eq!(report.skipped()[0].path(), &confirmation_path);
    assert_eq!(report.skipped()[0].prerequisites()[0].path(), &password_path);
    assert_eq!(report.failure_count(), 2);
    assert!(!report.is_valid());
    assert!(!format!("{report:?}").contains("private-value"));
}

/// Applies separate violation and skip limits to a caller-orchestrated flow.
#[test]
fn test_model_flow_marks_truncation_when_prerequisite_evidence_does_not_fit() {
    let bound = bind_password_rule(20);
    let context = BoundValidationContext::new(&[]);
    let outcome = bound
        .validate(ValidationValue::Text("private-value"), &context)
        .expect("password validation executes");
    let password_path = ValidationPath::root().with_field("user").with_field("password");
    let optional_path = ValidationPath::root().with_field("user").with_field("optional_note");
    let confirmation_path = ValidationPath::root().with_field("user").with_field("confirmation");
    let limits = ValidationLimits {
        max_violations: Some(1),
        max_skipped: Some(1),
    };
    let mut report = ValidationReport::with_limits(limits);

    assert!(
        report
            .record_outcome(0, password_path, outcome)
            .expect("first violation fits")
    );
    assert!(
        report
            .record_outcome(1, optional_path, ValidationOutcome::missing_optional())
            .expect("optional skip fits")
    );
    let evidence = report.violations()[0].clone();
    let failed_prerequisite = ValidationOutcome::failed_prerequisite(vec![evidence])
        .expect("failed prerequisite includes its original violation");

    assert!(
        !report
            .record_outcome(2, confirmation_path, failed_prerequisite)
            .expect("capacity exhaustion is a truncated result")
    );
    assert_eq!(report.violations().len(), 1);
    assert_eq!(report.skipped().len(), 1);
    assert!(
        !report
            .skipped()
            .iter()
            .any(|entry| { entry.reason() == SkipReason::FailedPrerequisite })
    );
    assert!(report.is_truncated());
    assert!(!report.is_valid());
}
