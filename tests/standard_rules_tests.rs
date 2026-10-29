// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::collection::ItemCount;
use qubit_validation_rules::registrations;
use qubit_validation_rules::text::ByteLength;
use qubit_validation_rules::text::CharLength;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

const TEXT_NON_BLANK: &str = "qubit.rules.text.non_blank";
const TEXT_CHAR_LENGTH: &str = "qubit.rules.text.char_length";
const TEXT_BYTE_LENGTH: &str = "qubit.rules.text.byte_length";
const TEXT_ALLOWED_CHARS: &str = "qubit.rules.text.allowed_chars";
const TEXT_EMAIL: &str = "qubit.rules.text.email_ascii";
const TEXT_URI: &str = "qubit.rules.text.uri";
const TEXT_UUID: &str = "qubit.rules.text.uuid";
const TEXT_MOBILE: &str = "qubit.rules.text.china_mobile_structure";
const COLLECTION_ITEM_COUNT: &str = "qubit.rules.collection.item_count";

fn create_test_registry() -> ValidatorRegistry {
    ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid")
}

fn bind_text_rule(registry: &ValidatorRegistry, id: &str, arguments: &[NamedValidationArgument<'_>]) -> BoundValidator {
    registry
        .bind(id, InputType::Text, arguments)
        .expect("text rule should bind")
}

fn validate_text_rule(
    registry: &ValidatorRegistry,
    id: &str,
    arguments: &[NamedValidationArgument<'_>],
    value: &str,
) -> ValidationOutcome {
    bind_text_rule(registry, id, arguments)
        .validate(ValidationValue::Text(value), &BoundValidationContext::new(&[]))
        .expect("text rule should execute")
}

fn violation_code(outcome: ValidationOutcome) -> String {
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected an invalid outcome, got {outcome:?}");
    };
    violations[0].code().as_str().to_owned()
}

/// Rejects typed length and count rules that would otherwise accept every
/// value.
#[test]
fn test_length_and_item_count_rules_require_at_least_one_bound() {
    assert_eq!(
        CharLength::new(None, None).unwrap_err().kind(),
        BindErrorKind::InvalidBounds
    );
    assert_eq!(
        ByteLength::new(None, None).unwrap_err().kind(),
        BindErrorKind::InvalidBounds
    );
    assert_eq!(
        ItemCount::new(None, None).unwrap_err().kind(),
        BindErrorKind::InvalidBounds
    );

    assert!(CharLength::new(Some(0), None).is_ok());
    assert!(ByteLength::new(Some(0), None).is_ok());
    assert!(ItemCount::new(Some(0), None).is_ok());
}

/// Rejects empty dynamic bounds while preserving zero as a valid bound.
#[test]
fn test_registered_length_and_item_count_rules_require_bounds() {
    let registry = create_test_registry();
    for (id, input_type) in [
        (TEXT_CHAR_LENGTH, InputType::Text),
        (TEXT_BYTE_LENGTH, InputType::Text),
        (COLLECTION_ITEM_COUNT, InputType::of::<usize>()),
    ] {
        let error = match registry.bind(id, input_type, &[]) {
            Ok(_) => panic!("{id} must reject missing bounds"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), BindErrorKind::InvalidBounds, "{id}");
    }

    let zero = [NamedValidationArgument::new("min", ValidationArgument::Unsigned(0))];
    assert!(registry.bind(TEXT_CHAR_LENGTH, InputType::Text, &zero).is_ok());
    assert!(
        registry
            .bind(COLLECTION_ITEM_COUNT, InputType::of::<usize>(), &zero)
            .is_ok()
    );
}

#[test]
fn test_registered_rules_cover_preparation_and_mapping_paths() {
    let registry = create_test_registry();

    assert_eq!(
        validate_text_rule(&registry, TEXT_NON_BLANK, &[], "hello"),
        ValidationOutcome::Valid
    );
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_NON_BLANK, &[], " \t")),
        "text.blank"
    );

    let char_args = [
        NamedValidationArgument::new("min", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("max", ValidationArgument::Unsigned(3)),
    ];
    assert_eq!(
        validate_text_rule(&registry, TEXT_CHAR_LENGTH, &char_args, "ok"),
        ValidationOutcome::Valid
    );
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_CHAR_LENGTH, &char_args, "x")),
        "text.too_short"
    );
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_CHAR_LENGTH, &char_args, "long")),
        "text.too_long"
    );

    let byte_args = [NamedValidationArgument::new("min", ValidationArgument::Unsigned(2))];
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_BYTE_LENGTH, &byte_args, "x")),
        "text.too_few_bytes"
    );
    let byte_max_args = [NamedValidationArgument::new("max", ValidationArgument::Unsigned(1))];
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_BYTE_LENGTH, &byte_max_args, "é")),
        "text.too_many_bytes"
    );

    for (set, value, code) in [
        ("ascii", "é", "text.disallowed_characters"),
        ("code", "a b", "text.disallowed_characters"),
    ] {
        let args = [NamedValidationArgument::new("set", ValidationArgument::String(set))];
        assert_eq!(
            violation_code(validate_text_rule(&registry, TEXT_ALLOWED_CHARS, &args, value)),
            code
        );
    }

    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_EMAIL, &[], "bad")),
        "text.email"
    );
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_URI, &[], "relative")),
        "text.uri"
    );
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_UUID, &[], "bad")),
        "text.uuid"
    );
    assert_eq!(
        violation_code(validate_text_rule(&registry, TEXT_MOBILE, &[], "bad")),
        "text.mobile"
    );

    let count_args = [NamedValidationArgument::new("min", ValidationArgument::Unsigned(2))];
    let count_rule = registry
        .bind(COLLECTION_ITEM_COUNT, InputType::of::<usize>(), &count_args)
        .expect("count rule binds");
    assert_eq!(
        count_rule
            .validate(ValidationValue::Typed(&3_usize), &BoundValidationContext::new(&[]))
            .expect("in-range count passes"),
        ValidationOutcome::Valid
    );
    let invalid_count = count_rule
        .validate(ValidationValue::Typed(&1_usize), &BoundValidationContext::new(&[]))
        .expect("out-of-range count is invalid");
    assert_eq!(violation_code(invalid_count), "collection.too_small");
}

#[test]
fn test_invalid_rule_parameters_are_rejected_during_binding() {
    let registry = create_test_registry();
    let invalid_set = [NamedValidationArgument::new(
        "set",
        ValidationArgument::String("unknown"),
    )];
    assert_eq!(
        registry
            .bind(TEXT_ALLOWED_CHARS, InputType::Text, &invalid_set)
            .unwrap_err()
            .kind(),
        BindErrorKind::ParameterOutOfRange
    );

    let unknown = [NamedValidationArgument::new("unused", ValidationArgument::Bool(true))];
    assert_eq!(
        registry
            .bind(TEXT_NON_BLANK, InputType::Text, &unknown)
            .unwrap_err()
            .kind(),
        BindErrorKind::UnknownParameter
    );
}
