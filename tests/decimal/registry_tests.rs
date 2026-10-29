// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn create_test_registry() -> ValidatorRegistry {
    ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid")
}

fn violation_code(outcome: qubit_validator::ValidationOutcome) -> String {
    let qubit_validator::ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected an invalid outcome, got {outcome:?}");
    };
    violations[0].code().as_str().to_owned()
}

#[cfg(feature = "decimal")]
#[test]
fn test_decimal_registry_endpoint_string_syntax() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::ids;
    use qubit_validator::BindErrorKind;

    let registry = create_test_registry();
    let cases = [
        ("0", true),
        ("-0", true),
        ("001.2300", true),
        ("1.", true),
        ("-1.20", true),
        ("", false),
        ("+1", false),
        ("1e3", false),
        (".5", false),
        (" 1", false),
        ("1 ", false),
        ("1.2.3", false),
        ("１２", false),
    ];
    for parameter in ["min", "max"] {
        for (literal, expected_valid) in cases {
            let arguments = [
                NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
                NamedValidationArgument::new(parameter, ValidationArgument::String(literal)),
            ];
            let result = registry.bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &arguments);
            if expected_valid {
                assert!(result.is_ok(), "{parameter}: {literal:?} should bind");
            } else {
                let error = result.expect_err("invalid spelling must fail binding");
                assert_eq!(
                    error.kind(),
                    BindErrorKind::ParameterOutOfRange,
                    "{parameter}: {literal:?}"
                );
                assert_eq!(error.parameter(), Some(parameter));
            }
        }
    }
}

#[cfg(feature = "decimal")]
#[test]
fn test_decimal_registry_endpoint_cost_and_grid() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::ids;
    use qubit_validation_rules::registrations;
    use qubit_validator::BindErrorKind;
    use qubit_validator::InputType;
    use qubit_validator::NamedValidationArgument;
    use qubit_validator::ValidationArgument;
    use qubit_validator::ValidatorRegistry;

    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    let huge = "9".repeat(131_072);
    for (min, max, parameter) in [
        (Some(huge.as_str()), None, "min"),
        (None, Some(huge.as_str()), "max"),
        (Some("0"), Some(huge.as_str()), "max"),
    ] {
        let mut args = vec![NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0))];
        if let Some(value) = min {
            args.push(NamedValidationArgument::new("min", ValidationArgument::String(value)));
        }
        if let Some(value) = max {
            args.push(NamedValidationArgument::new("max", ValidationArgument::String(value)));
        }
        let error = registry
            .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
            .unwrap_err();
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some(parameter));
        assert!(!format!("{error:?}").contains(&huge));
        assert!(!error.to_string().contains(&huge));
    }
    let args = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("min", ValidationArgument::String("1.23")),
        NamedValidationArgument::new("max", ValidationArgument::String("1.24")),
        NamedValidationArgument::new("min_inclusive", ValidationArgument::Bool(false)),
    ];
    assert!(
        registry
            .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
            .is_ok()
    );
    let invalid = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("precision", ValidationArgument::Unsigned(3)),
        NamedValidationArgument::new("min", ValidationArgument::String("10")),
    ];
    assert_eq!(
        registry
            .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &invalid)
            .unwrap_err()
            .kind(),
        BindErrorKind::InvalidBounds,
    );
}
#[cfg(feature = "decimal")]
#[test]
fn test_decimal_registry_endpoint_literal_length() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::ids;
    use qubit_validation_rules::registrations;
    use qubit_validator::BindErrorKind;
    use qubit_validator::InputType;
    use qubit_validator::NamedValidationArgument;
    use qubit_validator::ValidationArgument;
    use qubit_validator::ValidatorRegistry;

    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    let at_limit = "0".repeat(131_072);
    let oversized = "0".repeat(131_073);
    for parameter in ["min", "max"] {
        let at_limit_args = [
            NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0)),
            NamedValidationArgument::new(parameter, ValidationArgument::String(&at_limit)),
        ];
        assert!(
            registry
                .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &at_limit_args)
                .is_ok(),
            "{parameter} must accept a valid literal at the byte limit",
        );

        let oversized_args = [
            NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0)),
            NamedValidationArgument::new(parameter, ValidationArgument::String(&oversized)),
        ];
        let error = registry
            .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &oversized_args)
            .unwrap_err();
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some(parameter));
        assert!(!format!("{error:?}").contains(&oversized));
        assert!(!error.to_string().contains(&oversized));
    }

    let args = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("min", ValidationArgument::String("1.23")),
    ];
    assert!(
        registry
            .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
            .is_ok()
    );
}
#[cfg(feature = "decimal")]
#[test]
fn test_registered_decimal_max_bound_is_exact_and_exclusive() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::ids;

    let registry = create_test_registry();
    let args = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("max", ValidationArgument::String("1.2300")),
        NamedValidationArgument::new("max_inclusive", ValidationArgument::Bool(false)),
    ];
    let rule = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
        .unwrap();
    let at_max = "1.23".parse::<BigDecimal>().unwrap();
    let below_max = "1.22".parse::<BigDecimal>().unwrap();
    assert_eq!(
        violation_code(
            rule.validate(ValidationValue::Typed(&at_max), &BoundValidationContext::new(&[]))
                .unwrap()
        ),
        "decimal.range"
    );
    assert_eq!(
        rule.validate(ValidationValue::Typed(&below_max), &BoundValidationContext::new(&[]))
            .unwrap(),
        ValidationOutcome::Valid
    );
}
#[cfg(feature = "decimal")]
#[test]
fn test_registered_decimal_optional_min_bound_uses_string_literal() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::ids;

    let registry = create_test_registry();
    let args = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("min", ValidationArgument::String("1.20")),
        NamedValidationArgument::new("min_inclusive", ValidationArgument::Bool(false)),
    ];
    let rule = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
        .unwrap();
    let at_min = "1.20".parse::<BigDecimal>().unwrap();
    let above_min = "1.21".parse::<BigDecimal>().unwrap();
    assert_eq!(
        violation_code(
            rule.validate(ValidationValue::Typed(&at_min), &BoundValidationContext::new(&[]))
                .unwrap()
        ),
        "decimal.range"
    );
    assert_eq!(
        rule.validate(ValidationValue::Typed(&above_min), &BoundValidationContext::new(&[]))
            .unwrap(),
        ValidationOutcome::Valid
    );
}

#[cfg(feature = "decimal")]
#[test]
fn test_registered_decimal_preserves_violation_parameters() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::ids;
    use qubit_validation_rules::registrations;
    use qubit_validator::BoundValidationContext;
    use qubit_validator::InputType;
    use qubit_validator::NamedValidationArgument;
    use qubit_validator::ValidationArgument;
    use qubit_validator::ValidationOutcome;
    use qubit_validator::ValidationValue;
    use qubit_validator::ValidatorRegistry;
    use qubit_validator::ViolationParam;

    let registry = ValidatorRegistry::from_registrations(registrations()).unwrap();
    let context = BoundValidationContext::new(&[]);
    let scalar_args = [
        NamedValidationArgument::new("precision", ValidationArgument::Unsigned(3)),
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
    ];
    let scalar_rule = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &scalar_args)
        .unwrap();
    for (input, code, name, expected) in [
        ("1.234", "decimal.scale", "scale", 2_u128),
        ("12", "decimal.precision", "precision", 3_u128),
    ] {
        let value: BigDecimal = input.parse().unwrap();
        let outcome = scalar_rule.validate(ValidationValue::Typed(&value), &context).unwrap();
        let ValidationOutcome::Invalid(violations) = outcome else {
            panic!("expected a scalar violation");
        };
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_id().as_str(), ids::DECIMAL_VALUE);
        assert_eq!(violations[0].code().as_str(), code);
        assert_eq!(violations[0].params().len(), 1);
        assert_eq!(
            violations[0].params().get(name),
            Some(&ViolationParam::Unsigned(expected))
        );
    }

    for inclusivity in [
        None,
        Some((true, true)),
        Some((true, false)),
        Some((false, true)),
        Some((false, false)),
    ] {
        let mut args = scalar_args.to_vec();
        args.extend([
            NamedValidationArgument::new("min", ValidationArgument::String("1")),
            NamedValidationArgument::new("max", ValidationArgument::String("2")),
        ]);
        if let Some((min, max)) = inclusivity {
            args.extend([
                NamedValidationArgument::new("min_inclusive", ValidationArgument::Bool(min)),
                NamedValidationArgument::new("max_inclusive", ValidationArgument::Bool(max)),
            ]);
        }
        let rule = registry
            .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
            .unwrap();
        let (min, max) = inclusivity.unwrap_or((true, true));
        for input in ["0.99", "2.01"] {
            let value: BigDecimal = input.parse().unwrap();
            let outcome = rule.validate(ValidationValue::Typed(&value), &context).unwrap();
            let ValidationOutcome::Invalid(violations) = outcome else {
                panic!("expected a range violation");
            };
            assert_eq!(violations.len(), 1);
            assert_eq!(violations[0].rule_id().as_str(), ids::DECIMAL_VALUE);
            assert_eq!(violations[0].code().as_str(), "decimal.range");
            assert_eq!(violations[0].params().len(), 2);
            assert_eq!(
                violations[0].params().get("min_inclusive"),
                Some(&ViolationParam::Bool(min))
            );
            assert_eq!(
                violations[0].params().get("max_inclusive"),
                Some(&ViolationParam::Bool(max))
            );
        }
    }
}
