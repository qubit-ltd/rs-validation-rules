use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationPath;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

#[test]
fn standard_rules_are_available_to_local_registries_without_inventory() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard rules are valid");
    let arguments = [NamedValidationArgument::new("min", ValidationArgument::Unsigned(3))];
    let bound = registry
        .bind("qubit.rules.text.char_length", InputType::Text, &arguments, &[])
        .expect("length rule binds");
    let outcome = bound
        .validate(ValidationValue::Text("hi"), &BoundValidationContext::new(&[]))
        .expect("length rule executes");
    assert!(matches!(outcome, ValidationOutcome::Invalid(violations)
        if violations.len() == 1
            && violations[0].rule_id().as_str() == "qubit.rules.text.char_length"
            && violations[0].code().as_str() == "text.too_short"));
}

#[test]
fn dependency_text_rule_reads_its_declared_slot_and_redacts_values() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard rules are valid");
    let dependencies = [DependencySpec::new("expected", InputType::Text, false)];
    let bound = registry
        .bind(
            "qubit.rules.text.matches_dependency",
            InputType::Text,
            &[],
            &dependencies,
        )
        .expect("dependency rule is locally registered");
    let expected = "safe-value";
    let values = [ValidationValue::Text(expected)];
    let path = ValidationPath::root().with_field("confirmation");
    let paths = [path.clone()];
    let context =
        BoundValidationContext::new_with_paths(&values, &paths).expect("dependency path is aligned with the value");

    assert_eq!(
        bound
            .validate(ValidationValue::Text(expected), &context)
            .expect("matching value is valid"),
        ValidationOutcome::Valid,
    );
    let outcome = bound
        .validate(ValidationValue::Text("private-mismatch"), &context)
        .expect("mismatch becomes a structured violation");
    assert!(matches!(&outcome, ValidationOutcome::Invalid(violations)
        if violations.len() == 1
            && violations[0].code().as_str() == "text.dependency_mismatch"
            && violations[0].path() == &ValidationPath::root()));
    assert!(!format!("{outcome:?}").contains(expected));
    assert!(!format!("{outcome:?}").contains("private-mismatch"));
}

#[test]
fn dependency_text_rule_reports_missing_slot_before_invocation() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard rules are valid");
    let dependencies = [DependencySpec::new("expected", InputType::Text, false)];
    let bound = registry
        .bind(
            "qubit.rules.text.matches_dependency",
            InputType::Text,
            &[],
            &dependencies,
        )
        .expect("dependency rule is locally registered");
    let values = [ValidationValue::Missing];
    let path = ValidationPath::root().with_field("confirmation");
    let paths = [path.clone()];
    let context =
        BoundValidationContext::new_with_paths(&values, &paths).expect("dependency path is aligned with the value");

    let error = bound
        .validate(ValidationValue::Text("value"), &context)
        .expect_err("required dependency is checked before validator execution");
    assert_eq!(error.dependency(), Some("expected"));
    assert_eq!(error.path(), &path);
}
