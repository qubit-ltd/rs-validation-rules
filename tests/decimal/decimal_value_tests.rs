// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ops::Bound;

use bigdecimal::BigDecimal;
use qubit_validation_rules::decimal::DecimalValue;
use qubit_validation_rules::registrations;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

fn decimal(text: &str) -> BigDecimal {
    text.parse().expect("test decimal")
}

fn violation_code(outcome: ValidationOutcome) -> String {
    let ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected an invalid outcome, got {outcome:?}");
    };
    violations[0].code().as_str().to_owned()
}

#[test]
fn test_decimal_public_endpoint_construction_cost() {
    use std::ops::Bound;

    use qubit_validation_rules::decimal::DecimalValue;

    let huge = BigDecimal::from_bigint(1.into(), -131_071);
    for (precision, min, max, parameter) in [
        (None, Some(Bound::Included(huge.clone())), None, "min"),
        (None, None, Some(Bound::Excluded(huge.clone())), "max"),
        (
            None,
            Some(Bound::Included(decimal("0"))),
            Some(Bound::Included(huge.clone())),
            "max",
        ),
        (
            None,
            Some(Bound::Included(huge.clone())),
            Some(Bound::Included(huge.clone())),
            "min",
        ),
        (Some(3), None, Some(Bound::Included(huge.clone())), "max"),
        (Some(3), Some(Bound::Included(-huge.clone())), None, "min"),
    ] {
        let error = DecimalValue::new(precision, 0, min, max).unwrap_err();
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some(parameter));
    }
    assert!(DecimalValue::new(None, 2, Some(Bound::Unbounded), None).is_ok());
    assert!(DecimalValue::new(None, 2, None, Some(Bound::Included(decimal("1.23")))).is_ok());
}

fn create_test_registry() -> ValidatorRegistry {
    ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid")
}

#[cfg(feature = "decimal")]
#[test]
fn test_decimal_empty_domain_is_rejected_at_construction() {
    use std::ops::Bound;
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use qubit_validation_rules::decimal::DecimalValue;
    use qubit_validation_rules::ids;
    use qubit_validator::BindErrorKind;
    use qubit_validator::InputType;

    let decimal = |value| BigDecimal::from_str(value).unwrap();
    let invalid = [
        (Some(1), 0, Some(Bound::Included(decimal("100"))), None),
        (Some(1), 0, None, Some(Bound::Included(decimal("-100")))),
        (
            None,
            0,
            Some(Bound::Excluded(decimal("1"))),
            Some(Bound::Excluded(decimal("2"))),
        ),
        (
            None,
            2,
            Some(Bound::Excluded(decimal("1.230"))),
            Some(Bound::Excluded(decimal("1.231"))),
        ),
    ];
    for (precision, scale, min, max) in invalid {
        assert_eq!(
            DecimalValue::new(precision, scale, min, max).unwrap_err().kind(),
            BindErrorKind::InvalidBounds
        );
    }
    assert!(
        DecimalValue::new(
            None,
            2,
            Some(Bound::Included(decimal("1.230"))),
            Some(Bound::Included(decimal("1.230"))),
        )
        .is_ok()
    );
    assert!(
        DecimalValue::new(
            None,
            0,
            Some(Bound::Included(decimal("-1"))),
            Some(Bound::Included(decimal("1")))
        )
        .is_ok()
    );
    assert!(DecimalValue::new(None, 2, None, Some(Bound::Excluded(decimal("0")))).is_ok());

    let registry = create_test_registry();
    let args = [
        NamedValidationArgument::new("precision", ValidationArgument::Unsigned(1)),
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0)),
        NamedValidationArgument::new("min", ValidationArgument::String("100")),
    ];
    let error = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
        .unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::InvalidBounds);
    assert!(!format!("{error:?}").contains("100"));

    let oversized_min = format!("1{}", "0".repeat(131_072));
    let oversized_max = format!("2{}", "0".repeat(131_072));
    let oversized_args = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0)),
        NamedValidationArgument::new("min", ValidationArgument::String(&oversized_min)),
        NamedValidationArgument::new("max", ValidationArgument::String(&oversized_max)),
    ];
    let oversized_error = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &oversized_args)
        .unwrap_err();
    assert_eq!(oversized_error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(oversized_error.parameter(), Some("min"));
    assert!(!format!("{oversized_error:?}").contains(&oversized_min));
}
#[cfg(feature = "decimal")]
#[test]
fn test_decimal_value_normalization_bounds_and_registration() {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use qubit_validation_rules::decimal::DecimalValue;
    use qubit_validation_rules::decimal::DecimalValueError;
    use qubit_validation_rules::ids;

    let decimal = |value| BigDecimal::from_str(value).unwrap();
    let rule = DecimalValue::new(
        Some(3),
        2,
        Some(Bound::Included(decimal("-1.23"))),
        Some(Bound::Excluded(decimal("1.23"))),
    )
    .unwrap();
    assert_eq!(rule.validate(&decimal("1.2300"), &()), Err(DecimalValueError::Range));
    assert_eq!(rule.validate(&decimal("-1.2300"), &()), Ok(()));
    assert_eq!(rule.validate(&decimal("-1.2301"), &()), Err(DecimalValueError::Scale));
    assert_eq!(rule.validate(&decimal("1.234"), &()), Err(DecimalValueError::Scale));
    assert_eq!(rule.validate(&decimal("12.34"), &()), Err(DecimalValueError::Precision));
    assert_eq!(
        DecimalValue::new(Some(1), 0, None, None)
            .unwrap()
            .validate(&decimal("0.000"), &()),
        Ok(())
    );
    assert_eq!(
        DecimalValue::new(Some(1), 0, None, None)
            .unwrap()
            .validate(&decimal("1e3"), &()),
        Err(DecimalValueError::Precision)
    );
    for (precision, scale, literal, expected) in [
        (1, 0, "1000", Err(DecimalValueError::Precision)),
        (3, 2, "12", Err(DecimalValueError::Precision)),
        (3, 2, "1.2300", Ok(())),
        (4, 2, "12.00", Ok(())),
        (5, 5, "0.00001", Ok(())),
    ] {
        let rule = DecimalValue::new(Some(precision), scale, None, None).unwrap();
        assert_eq!(rule.validate(&decimal(literal), &()), expected, "{literal}");
    }
    assert_eq!(
        DecimalValue::new(
            Some(3),
            2,
            Some(Bound::Included(decimal("1.2300"))),
            Some(Bound::Included(decimal("2")))
        )
        .unwrap()
        .validate(&decimal("1.23"), &()),
        Ok(())
    );
    assert_eq!(
        DecimalValue::new(Some(0), 0, None, None).unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );
    assert_eq!(
        DecimalValue::new(Some(2), 3, None, None).unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );
    assert_eq!(
        DecimalValue::new(
            None,
            2,
            Some(Bound::Included(decimal("2"))),
            Some(Bound::Included(decimal("1")))
        )
        .unwrap_err()
        .kind(),
        BindErrorKind::InvalidBounds
    );
    assert!(!format!("{:?}", rule.validate(&decimal("1.234"), &())).contains("1.234"));

    let registry = create_test_registry();
    let args = [NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2))];
    let bound = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
        .unwrap();
    let outcome = bound
        .validate(
            ValidationValue::Typed(&decimal("1.234")),
            &BoundValidationContext::new(&[]),
        )
        .unwrap();
    assert_eq!(violation_code(outcome), "decimal.scale");
    let precision_args = [
        NamedValidationArgument::new("precision", ValidationArgument::Unsigned(3)),
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
    ];
    let bound = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &precision_args)
        .unwrap();
    assert_eq!(
        violation_code(
            bound
                .validate(
                    ValidationValue::Typed(&decimal("12.3")),
                    &BoundValidationContext::new(&[])
                )
                .unwrap()
        ),
        "decimal.precision"
    );
    assert_eq!(
        violation_code(
            bound
                .validate(
                    ValidationValue::Typed(&decimal("12")),
                    &BoundValidationContext::new(&[])
                )
                .unwrap()
        ),
        "decimal.precision"
    );
    let range_args = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("min", ValidationArgument::String("1.23")),
        NamedValidationArgument::new("min_inclusive", ValidationArgument::Bool(false)),
    ];
    let bound = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &range_args)
        .unwrap();
    assert_eq!(
        violation_code(
            bound
                .validate(
                    ValidationValue::Typed(&decimal("1.2300")),
                    &BoundValidationContext::new(&[])
                )
                .unwrap()
        ),
        "decimal.range"
    );
    let invalid = [
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
        NamedValidationArgument::new("min", ValidationArgument::String("private-bad-bound")),
    ];
    let error = registry
        .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &invalid)
        .unwrap_err();
    assert_eq!(error.parameter(), Some("min"));
    assert!(!format!("{error:?} {error}").contains("private-bad-bound"));
    for (args, parameter) in [
        (
            vec![NamedValidationArgument::new(
                "scale",
                ValidationArgument::Unsigned(65_536),
            )],
            "scale",
        ),
        (
            vec![
                NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0)),
                NamedValidationArgument::new("precision", ValidationArgument::Unsigned(65_536)),
            ],
            "precision",
        ),
        (
            vec![
                NamedValidationArgument::new("scale", ValidationArgument::Unsigned(0)),
                NamedValidationArgument::new("precision", ValidationArgument::Unsigned(0)),
            ],
            "precision",
        ),
        (
            vec![
                NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
                NamedValidationArgument::new("precision", ValidationArgument::Unsigned(1)),
            ],
            "scale",
        ),
    ] {
        assert_eq!(
            registry
                .bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)
                .unwrap_err()
                .parameter(),
            Some(parameter)
        );
    }
}
#[cfg(feature = "decimal")]
#[test]
fn test_decimal_value_debug_redacts_endpoint_values() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::decimal::DecimalValue;

    let min = "98765.4321".parse::<BigDecimal>().unwrap();
    let max = "98766.4321".parse::<BigDecimal>().unwrap();
    let rule = DecimalValue::new(None, 4, Some(Bound::Included(min)), Some(Bound::Excluded(max))).unwrap();
    let debug = format!("{rule:?}");
    assert!(!debug.contains("98765.4321"), "{debug}");
    assert!(!debug.contains("98766.4321"), "{debug}");
    assert!(!debug.contains("987654321"), "{debug}");
    assert!(!debug.contains("987664321"), "{debug}");
}
#[cfg(feature = "decimal")]
#[test]
fn test_decimal_value_extreme_exponent_does_not_overflow_normalization() {
    use bigdecimal::BigDecimal;
    use qubit_validation_rules::decimal::DecimalValue;
    use qubit_validation_rules::decimal::DecimalValueError;

    let value = BigDecimal::new(10.into(), i64::MIN);
    let one_digit = DecimalValue::new(Some(1), 0, None, None).unwrap();
    assert_eq!(one_digit.validate(&value, &()), Err(DecimalValueError::Precision));
    let negative = BigDecimal::new((-10).into(), i64::MIN);
    assert_eq!(one_digit.validate(&negative, &()), Err(DecimalValueError::Precision));
    let two_digits = BigDecimal::new(11.into(), i64::MIN);
    assert_eq!(
        one_digit.validate(&two_digits, &()),
        Err(qubit_validation_rules::decimal::DecimalValueError::Precision)
    );
    let zero = BigDecimal::new(0.into(), i64::MIN);
    assert_eq!(one_digit.validate(&zero, &()), Ok(()));
    let tiny_fraction = BigDecimal::new(1.into(), i64::MAX);
    assert_eq!(one_digit.validate(&tiny_fraction, &()), Err(DecimalValueError::Scale));
    let fractional_zero = BigDecimal::new(0.into(), i64::MAX);
    assert_eq!(one_digit.validate(&fractional_zero, &()), Ok(()));

    let bounded = DecimalValue::new(
        None,
        0,
        Some(Bound::Included(BigDecimal::from(0))),
        Some(Bound::Included(BigDecimal::from(1))),
    )
    .unwrap();
    assert_eq!(bounded.validate(&BigDecimal::from(1), &()), Ok(()));
    assert_eq!(
        bounded.validate(&BigDecimal::from(-1), &()),
        Err(DecimalValueError::Range)
    );
    for (min, max, parameter) in [
        (None, Some(Bound::Included(value.clone())), "max"),
        (
            Some(Bound::Included(BigDecimal::from(0))),
            Some(Bound::Included(value.clone())),
            "max",
        ),
    ] {
        let error = DecimalValue::new(None, 0, min, max).unwrap_err();
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some(parameter));
    }
    let error = DecimalValue::new(
        None,
        0,
        Some(Bound::Included(value)),
        Some(Bound::Included(BigDecimal::new(20.into(), i64::MIN))),
    )
    .expect_err("two extreme endpoints exceed the feasibility work limit");
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("min"));
    let upper_error = DecimalValue::new(Some(1), 0, None, Some(Bound::Included(tiny_fraction)))
        .expect_err("an extreme selected upper endpoint exceeds the feasibility work limit");
    assert_eq!(upper_error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(upper_error.parameter(), Some("max"));
}

#[test]
fn test_decimal_small_grid_feasibility() {
    let endpoints = [
        -901, -900, -899, -101, -100, -99, -91, -90, -89, -1, 0, 1, 89, 90, 91, 99, 100, 101, 899, 900, 901,
    ];
    let mut bounds = vec![Bound::Unbounded];
    for endpoint in endpoints {
        bounds.push(Bound::Included(endpoint));
        bounds.push(Bound::Excluded(endpoint));
    }
    let decimal_bound = |bound: &Bound<i32>| match bound {
        Bound::Unbounded => Some(Bound::Unbounded),
        Bound::Included(value) => Some(Bound::Included(BigDecimal::from_bigint((*value).into(), 2))),
        Bound::Excluded(value) => Some(Bound::Excluded(BigDecimal::from_bigint((*value).into(), 2))),
    };
    for scale in [0_u16, 1] {
        let factor = 100 / 10_i32.pow(u32::from(scale));
        for lower in &bounds {
            for upper in &bounds {
                let expected = (-9..=9).any(|coefficient| {
                    let value = coefficient * factor;
                    let above = match lower {
                        Bound::Unbounded => true,
                        Bound::Included(min) => value >= *min,
                        Bound::Excluded(min) => value > *min,
                    };
                    let below = match upper {
                        Bound::Unbounded => true,
                        Bound::Included(max) => value <= *max,
                        Bound::Excluded(max) => value < *max,
                    };
                    above && below
                });
                let actual = DecimalValue::new(Some(1), scale, decimal_bound(lower), decimal_bound(upper));
                assert_eq!(
                    actual.is_ok(),
                    expected,
                    "scale={scale}, lower={lower:?}, upper={upper:?}, result={actual:?}",
                );
            }
        }
    }
}
