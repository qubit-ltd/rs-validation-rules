// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validation_rules::text::MatchesDependency;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationDependency;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

fn violation_code(outcome: ValidationOutcome) -> String {
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected an invalid outcome, got {outcome:?}");
    };
    violations[0].code().as_str().to_owned()
}

#[test]
fn test_matches_dependency_accepts_equal_text_and_reports_mismatch() {
    let values = [ValidationValue::Text("same")];
    let context = BoundValidationContext::new(&values);
    assert_eq!(MatchesDependency.validate("same", &context), Ok(()));
    assert!(MatchesDependency.validate("different", &context).is_err());
    assert!(
        MatchesDependency
            .validate("same", &BoundValidationContext::new(&[]))
            .is_err()
    );
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("registrations are valid");
    let bound = registry
        .bind("qubit.rules.text.matches_dependency", InputType::Text, &[])
        .expect("dependency rule binds");
    let dependencies = [NamedValidationDependency::new(
        "expected",
        ValidationValue::Text("expected"),
    )];
    assert_eq!(
        bound
            .validate_named(ValidationValue::Text("expected"), &dependencies)
            .expect("matching text passes"),
        ValidationOutcome::Valid
    );
    assert_eq!(
        violation_code(
            bound
                .validate_named(ValidationValue::Text("different"), &dependencies)
                .expect("mismatch is invalid")
        ),
        "text.dependency_mismatch"
    );
}
