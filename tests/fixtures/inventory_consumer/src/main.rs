use qubit_validator::ValidatorRegistry;
use qubit_validation_rules as _;

fn main() {
    let registry = ValidatorRegistry::try_global().expect("inventory registry");
    assert!(registry.get("qubit.rules.text.email_ascii").is_some());
    assert!(registry.get("qubit.rules.collection.item_count").is_some());
}
