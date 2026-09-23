use qubit_validator::BoundValidationContext;
use qubit_validator::DependencySpec;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;
use qubit_validation_rules as _;

fn main() {
    let registry = ValidatorRegistry::try_global().expect("inventory registry");
    assert!(registry.get("qubit.rules.text.email_ascii").is_some());
    assert!(registry.get("qubit.rules.collection.item_count").is_some());
    let dependencies = [DependencySpec::new("expected", InputType::Text, false)];
    let bound = registry
        .bind(
            "qubit.rules.text.matches_dependency",
            InputType::Text,
            &[],
            &dependencies,
        )
        .expect("context-aware rule is inventory registered");
    let values = [ValidationValue::Text("expected-value")];
    let context = BoundValidationContext::new(&values);
    assert_eq!(
        bound
            .validate(ValidationValue::Text("expected-value"), &context)
            .expect("matching dependency passes"),
        ValidationOutcome::Valid,
    );
    let outcome = bound
        .validate(ValidationValue::Text("other-value"), &context)
        .expect("dependency mismatch is a rule violation");
    assert!(matches!(outcome, ValidationOutcome::Invalid(violations)
        if violations[0].code().as_str() == "text.dependency_mismatch"));
}
