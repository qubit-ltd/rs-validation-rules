// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validation_rules::text::ByteLength;
use qubit_validation_rules::text::ByteLengthError;
use qubit_validation_rules::text::CharLength;
use qubit_validation_rules::text::TextLengthError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ViolationParam;

fn expected_char_length_result(value: &str, min: Option<u32>, max: Option<u32>) -> Result<(), TextLengthError> {
    let count = value.chars().count();
    if let Some(min) = min
        && count < min as usize
    {
        return Err(TextLengthError::TooShort { min });
    }
    if let Some(max) = max
        && count > max as usize
    {
        return Err(TextLengthError::TooLong { max });
    }
    Ok(())
}

fn assert_registered_length_violation(
    registry: &ValidatorRegistry,
    bound: u32,
    parameter: &'static str,
    input: &str,
    expected_code: &str,
) {
    let arguments = [NamedValidationArgument::new(
        parameter,
        ValidationArgument::Unsigned(bound.into()),
    )];
    let validator = registry
        .bind("qubit.rules.text.char_length", InputType::Text, &arguments)
        .expect("char-length rule binds with one bound");
    let outcome = validator
        .validate(ValidationValue::Text(input), &BoundValidationContext::new(&[]))
        .expect("char-length rule executes");
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected {expected_code} for {input:?}, got {outcome:?}");
    };
    assert_eq!(violations[0].code().as_str(), expected_code);
    assert_eq!(violations[0].params().len(), 1);
    assert_eq!(
        violations[0].params().get("bound"),
        Some(&ViolationParam::Unsigned(bound.into()))
    );
}

#[test]
fn test_typed_text_length_rules_cover_success_and_domain_errors() {
    let char_length = CharLength::new(Some(2), Some(3)).expect("ordered bounds");
    assert_eq!(char_length.validate("éa", &()), Ok(()));
    assert_eq!(
        char_length.validate("a", &()),
        Err(TextLengthError::TooShort { min: 2 })
    );
    assert_eq!(
        char_length.validate("four", &()),
        Err(TextLengthError::TooLong { max: 3 })
    );
    assert_eq!(
        CharLength::new(Some(4), Some(3)).unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );

    let byte_length = ByteLength::new(Some(2), Some(3)).expect("ordered bounds");
    assert_eq!(byte_length.validate("é", &()), Ok(()));
    assert_eq!(
        byte_length.validate("a", &()),
        Err(ByteLengthError::TooFewBytes { min: 2 })
    );
    assert_eq!(
        byte_length.validate("éé", &()),
        Err(ByteLengthError::TooManyBytes { max: 3 })
    );
    assert_eq!(
        ByteLength::new(Some(4), Some(3)).unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );
}

#[test]
fn test_char_length_matches_full_scalar_count_for_bounds_and_text() {
    let bounds = [
        (Some(0), None),
        (Some(1), None),
        (Some(2), None),
        (None, Some(0)),
        (None, Some(2)),
        (Some(1), Some(2)),
        (Some(2), Some(2)),
        (Some(u32::MAX), None),
        (None, Some(u32::MAX)),
    ];
    let values = ["", "a", "ab", "abc", "汉字", "e\u{301}", "👩\u{200d}💻"];

    for (min, max) in bounds {
        let rule = CharLength::new(min, max).expect("at least one bound is present");
        for value in values {
            assert_eq!(
                rule.validate(value, &()),
                expected_char_length_result(value, min, max),
                "min={min:?}, max={max:?}, value={value:?}",
            );
        }
    }
}

#[test]
fn test_char_length_checks_long_input_with_bounded_prefix() {
    let value = "汉".repeat(4096);
    assert_eq!(
        CharLength::new(None, Some(2)).expect("max bound").validate(&value, &()),
        Err(TextLengthError::TooLong { max: 2 }),
    );
    assert_eq!(
        CharLength::new(Some(2), None).expect("min bound").validate(&value, &()),
        Ok(()),
    );
    assert_eq!(
        CharLength::new(Some(0), None)
            .expect("zero min bound")
            .validate(&value, &()),
        Ok(()),
    );
}

#[test]
fn test_registered_char_length_preserves_bound_violation_parameters() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid");
    assert_registered_length_violation(&registry, 2, "min", "a", "text.too_short");
    assert_registered_length_violation(&registry, 2, "max", "abc", "text.too_long");
}
