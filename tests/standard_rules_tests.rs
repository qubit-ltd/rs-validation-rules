// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ops::Bound;

use qubit_validation_rules::collection::ItemCount;
use qubit_validation_rules::collection::ItemCountError;
use qubit_validation_rules::collection::Range;
use qubit_validation_rules::collection::RangeError;
use qubit_validation_rules::registrations;
use qubit_validation_rules::text::AllowedChars;
use qubit_validation_rules::text::ByteLength;
use qubit_validation_rules::text::ByteLengthError;
use qubit_validation_rules::text::CharLength;
use qubit_validation_rules::text::CharacterSet;
use qubit_validation_rules::text::ChinaMobileStructure;
use qubit_validation_rules::text::EmailAscii;
use qubit_validation_rules::text::NonBlank;
use qubit_validation_rules::text::NonBlankError;
use qubit_validation_rules::text::TextLengthError;
use qubit_validation_rules::text::TextRuleError;
use qubit_validation_rules::text::Uri;
use qubit_validation_rules::text::UuidText;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::BoundValidator;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::NamedValidationDependency;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

const TEXT_NON_BLANK: &str = "qubit.rules.text.non_blank";
const TEXT_CHAR_LENGTH: &str = "qubit.rules.text.char_length";
const TEXT_BYTE_LENGTH: &str = "qubit.rules.text.byte_length";
const TEXT_ALLOWED_CHARS: &str = "qubit.rules.text.allowed_chars";
const TEXT_EMAIL: &str = "qubit.rules.text.email_ascii";
const TEXT_URI: &str = "qubit.rules.text.uri";
const TEXT_UUID: &str = "qubit.rules.text.uuid";
const TEXT_MOBILE: &str = "qubit.rules.text.china_mobile_structure";
const TEXT_MATCHES_DEPENDENCY: &str = "qubit.rules.text.matches_dependency";
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
        Some(Bound::Included(value.clone())),
    )
    .unwrap();
    assert_eq!(bounded.validate(&value, &()), Ok(()));
    assert_eq!(bounded.validate(&negative, &()), Err(DecimalValueError::Range));
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

#[cfg(feature = "time")]
#[test]
fn test_time_precision_every_resolution_on_each_chrono_type_across_utc_day() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use chrono::Utc;
    use qubit_validation_rules::time::TemporalResolution;
    use qubit_validation_rules::time::TimePrecision;

    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let cases = [
        (TemporalResolution::Second, 0, true),
        (TemporalResolution::Second, 1, false),
        (TemporalResolution::Millisecond, 1_000_000, true),
        (TemporalResolution::Millisecond, 1_000_001, false),
        (TemporalResolution::Microsecond, 1_000, true),
        (TemporalResolution::Microsecond, 1_001, false),
        (TemporalResolution::Nanosecond, 1, true),
    ];
    for (resolution, nanos, accepted) in cases {
        let time = NaiveTime::from_hms_nano_opt(0, 0, 0, nanos).unwrap();
        let naive = date.and_time(time);
        let utc: DateTime<Utc> = naive.and_utc();
        let rule = TimePrecision::new(resolution);
        assert_eq!(rule.validate(&time, &()).is_ok(), accepted);
        assert_eq!(rule.validate(&naive, &()).is_ok(), accepted);
        assert_eq!(rule.validate(&utc, &()).is_ok(), accepted);
        let before_midnight = NaiveTime::from_hms_nano_opt(23, 59, 59, nanos).unwrap();
        let prior_day: DateTime<Utc> = NaiveDate::from_ymd_opt(2025, 12, 31)
            .unwrap()
            .and_time(before_midnight)
            .and_utc();
        assert_eq!(rule.validate(&prior_day, &()).is_ok(), accepted);
    }
}

#[cfg(feature = "time")]
#[test]
fn test_time_precision_on_all_chrono_inputs_and_registration() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use chrono::Utc;
    use qubit_validation_rules::ids;
    use qubit_validation_rules::time::TemporalResolution;
    use qubit_validation_rules::time::TimePrecision;
    use qubit_validation_rules::time::TimePrecisionError;

    let precise = NaiveTime::from_hms_nano_opt(12, 0, 0, 123_000_000).unwrap();
    let second = TimePrecision::new(TemporalResolution::Second);
    let milli = TimePrecision::new(TemporalResolution::Millisecond);
    let micro = TimePrecision::new(TemporalResolution::Microsecond);
    let nano = TimePrecision::new(TemporalResolution::Nanosecond);
    assert_eq!(second.validate(&precise, &()), Err(TimePrecisionError::Precision));
    assert_eq!(milli.validate(&precise, &()), Ok(()));
    assert_eq!(micro.validate(&precise, &()), Ok(()));
    assert_eq!(nano.validate(&precise, &()), Ok(()));
    let date_time = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap().and_time(precise);
    let utc: DateTime<Utc> = date_time.and_utc();
    assert_eq!(second.validate(&date_time, &()), Err(TimePrecisionError::Precision));
    assert_eq!(second.validate(&utc, &()), Err(TimePrecisionError::Precision));
    let micros = NaiveTime::from_hms_nano_opt(12, 0, 0, 123_456_000).unwrap();
    let nanos = NaiveTime::from_hms_nano_opt(12, 0, 0, 123_456_789).unwrap();
    assert_eq!(milli.validate(&micros, &()), Err(TimePrecisionError::Precision));
    assert_eq!(micro.validate(&micros, &()), Ok(()));
    assert_eq!(micro.validate(&nanos, &()), Err(TimePrecisionError::Precision));
    assert_eq!(nano.validate(&nanos, &()), Ok(()));
    let registry = create_test_registry();
    let args = [NamedValidationArgument::new(
        "precision",
        ValidationArgument::String("second"),
    )];
    for input in [
        InputType::of::<NaiveTime>(),
        InputType::of::<chrono::NaiveDateTime>(),
        InputType::of::<DateTime<Utc>>(),
    ] {
        assert!(registry.bind(ids::TIME_PRECISION, input, &args).is_ok());
    }
    let bound = registry
        .bind(ids::TIME_PRECISION, InputType::of::<DateTime<Utc>>(), &args)
        .unwrap();
    let outcome = bound
        .validate(ValidationValue::Typed(&utc), &BoundValidationContext::new(&[]))
        .unwrap();
    assert_eq!(violation_code(outcome), "time.precision");
    let invalid = [NamedValidationArgument::new(
        "precision",
        ValidationArgument::String("minute"),
    )];
    let error = registry
        .bind(ids::TIME_PRECISION, InputType::of::<NaiveTime>(), &invalid)
        .unwrap_err();
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("precision"));
}

#[test]
fn test_typed_text_rules_cover_success_and_domain_errors() {
    assert_eq!(NonBlank.validate(" text ", &()), Ok(()));
    assert_eq!(NonBlank.validate(" \t", &()), Err(NonBlankError::Blank));

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
fn test_character_sets_enforce_each_declared_profile() {
    let cases = [
        (CharacterSet::Unicode, "é\n", true),
        (CharacterSet::PrintableUnicode, "é", true),
        (CharacterSet::PrintableUnicode, "\n", false),
        (CharacterSet::Ascii, "abc\n", true),
        (CharacterSet::Ascii, "é", false),
        (CharacterSet::PrintableAscii, "abc", true),
        (CharacterSet::PrintableAscii, "\n", false),
        (CharacterSet::Code, "a9._-", true),
        (CharacterSet::Code, "a b", false),
    ];

    for (set, value, expected_valid) in cases {
        let result = AllowedChars::new(set).validate(value, &());
        assert_eq!(result.is_ok(), expected_valid, "set {set:?}, value {value:?}");
    }
}

#[test]
fn test_strict_text_profiles_accept_valid_and_reject_invalid_inputs() {
    assert_eq!(EmailAscii.validate("a.b+tag@example-domain.com", &()), Ok(()));
    for invalid in [
        "missing-at",
        "a@b@c",
        "@example.com",
        "a@",
        "a..b@example.com",
        ".a@example.com",
        "a.@example.com",
        "a b@example.com",
        "a@-example.com",
        "a@example-.com",
        "a@example..com",
        "a@例子.测试",
    ] {
        assert_eq!(
            EmailAscii.validate(invalid, &()),
            Err(TextRuleError::Email),
            "{invalid}"
        );
    }

    assert_eq!(Uri.validate("https://example.com/path", &()), Ok(()));
    assert_eq!(Uri.validate("a:", &()), Ok(()));
    for invalid in ["", "relative", "1bad:rest", "a:b c"] {
        assert_eq!(Uri.validate(invalid, &()), Err(TextRuleError::Uri), "{invalid}");
    }

    assert_eq!(UuidText.validate("550e8400-e29b-41d4-a716-446655440000", &()), Ok(()));
    assert_eq!(UuidText.validate("not-a-uuid", &()), Err(TextRuleError::Uuid));

    assert_eq!(ChinaMobileStructure.validate("13800138000", &()), Ok(()));
    for invalid in ["23800138000", "12800138000", "1380013800", "1380013800x"] {
        assert_eq!(
            ChinaMobileStructure.validate(invalid, &()),
            Err(TextRuleError::Mobile),
            "{invalid}"
        );
    }
}

#[test]
fn test_collection_rules_cover_bounds_and_unordered_values() {
    let count = ItemCount::new(Some(2), Some(4)).expect("ordered bounds");
    assert_eq!(count.validate(&3, &()), Ok(()));
    assert_eq!(count.validate(&1, &()), Err(ItemCountError::TooSmall { min: 2 }));
    assert_eq!(count.validate(&5, &()), Err(ItemCountError::TooLarge { max: 4 }));
    assert_eq!(
        ItemCount::new(Some(5), Some(4)).unwrap_err().kind(),
        BindErrorKind::ParameterOutOfRange
    );

    let range = Range::new(Bound::Excluded(2), Bound::Included(5)).expect("valid range");
    assert_eq!(range.validate(&3, &()), Ok(()));
    assert_eq!(range.validate(&2, &()), Err(RangeError::OutOfRange));
    assert_eq!(range.validate(&6, &()), Err(RangeError::OutOfRange));
    assert_eq!(
        Range::new(Bound::Included(5), Bound::Included(2)).unwrap_err().kind(),
        BindErrorKind::InvalidBounds
    );
    assert_eq!(
        Range::new(Bound::Excluded(3), Bound::Included(3)).unwrap_err().kind(),
        BindErrorKind::InvalidBounds
    );
    let unordered = Range::new(Bound::Included(0.0_f64), Bound::Unbounded).expect("unbounded upper end");
    assert_eq!(unordered.validate(&f64::NAN, &()), Err(RangeError::Unordered));
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

    let bound = registry
        .bind(TEXT_MATCHES_DEPENDENCY, InputType::Text, &[])
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

#[cfg(feature = "regex")]
#[test]
fn test_regex_rule_compiles_and_matches_full_input() {
    use qubit_validation_rules::regex_rule::RegexMatch;

    let rule = RegexMatch::new(r"[a-z]+@[a-z]+\.com").expect("valid regex");
    assert_eq!(rule.validate("user@example.com", &()), Ok(()));
    assert_eq!(
        rule.validate("prefix user@example.com", &()),
        Err(TextRuleError::Pattern)
    );
    assert!(matches!(RegexMatch::new("["), Err(error) if error.kind() == BindErrorKind::InvalidPattern));

    let registry = create_test_registry();
    let arguments = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String("[0-9]+"),
    )];
    assert_eq!(
        validate_text_rule(&registry, "qubit.rules.text.regex", &arguments, "123"),
        ValidationOutcome::Valid
    );
    assert_eq!(
        violation_code(validate_text_rule(
            &registry,
            "qubit.rules.text.regex",
            &arguments,
            "12a"
        )),
        "text.pattern"
    );
}

/// Typed and registered regex construction share the pattern byte limit.
#[cfg(feature = "regex")]
#[test]
fn test_regex_rejects_pattern_above_limit() {
    use qubit_validation_rules::ids;
    use qubit_validation_rules::regex_rule::RegexMatch;

    let allowed = "a".repeat(4_096);
    assert!(
        RegexMatch::new(&allowed).is_ok(),
        "a simple pattern at the limit must compile"
    );

    let too_long = "a".repeat(4_097);
    let error = match RegexMatch::new(&too_long) {
        Ok(_) => panic!("typed constructor accepted an oversized pattern"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("pattern"));

    let registry = create_test_registry();
    let args = [NamedValidationArgument::new(
        "pattern",
        ValidationArgument::String(&too_long),
    )];
    let error = match registry.bind(ids::TEXT_REGEX, InputType::Text, &args) {
        Ok(_) => panic!("registry accepted an oversized pattern"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
    assert_eq!(error.parameter(), Some("pattern"));
    assert!(matches!(RegexMatch::new("["), Err(error) if error.kind() == BindErrorKind::InvalidPattern));
}

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_structure_accepts_zero_region_code() {
    use qubit_validation_rules::identity::ChinaIdentity18Structure;

    // This satisfies the structural checks; it does not establish a valid issued
    // credential.
    let structurally_valid = "000000194912310027";
    let facts = ChinaIdentity18Structure::parse(structurally_valid)
        .expect("zero region code does not invalidate the structure");
    assert_eq!(facts.birth_date().to_string(), "1949-12-31");
    assert_eq!(ChinaIdentity18Structure.validate(structurally_valid, &()), Ok(()));
}

#[cfg(feature = "china-identity")]
#[test]
fn test_china_identity_parser_reports_each_validation_failure() {
    use qubit_validation_rules::identity::ChinaIdentity18Structure;
    use qubit_validation_rules::identity::ChinaIdentityError;

    assert_eq!(
        ChinaIdentity18Structure::parse("short"),
        Err(ChinaIdentityError::InvalidLength)
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("é010519491231002X"),
        Err(ChinaIdentityError::NonAsciiBodyDigit { position: 0 })
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("1101051949123100A0"),
        Err(ChinaIdentityError::InvalidBodyDigit { position: 16 })
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("11010519491231002!"),
        Err(ChinaIdentityError::InvalidChecksumCharacter)
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("11010520230230002X"),
        Err(ChinaIdentityError::InvalidBirthDate)
    );
    assert_eq!(
        ChinaIdentity18Structure::parse("110105194912310020"),
        Err(ChinaIdentityError::InvalidChecksum)
    );

    let valid = "11010519491231002X";
    let facts = ChinaIdentity18Structure::parse(valid).expect("known structurally valid identity number");
    assert_eq!(facts.birth_date().to_string(), "1949-12-31");
    assert_eq!(ChinaIdentity18Structure.validate(valid, &()), Ok(()));
}
