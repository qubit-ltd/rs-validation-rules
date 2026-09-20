use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
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
