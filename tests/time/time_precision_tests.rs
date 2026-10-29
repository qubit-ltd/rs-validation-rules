// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::registrations;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ViolationParam;

fn create_test_registry() -> ValidatorRegistry {
    ValidatorRegistry::from_registrations(registrations()).expect("standard registrations are valid")
}

fn violation_code(outcome: qubit_validator::ValidationOutcome) -> String {
    let qubit_validator::ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected an invalid outcome, got {outcome:?}");
    };
    violations[0].code().as_str().to_owned()
}

#[cfg(feature = "time")]
fn assert_precision_violation(input_type: InputType, value: ValidationValue<'_>, precision: &'static str) {
    let registry = create_test_registry();
    let args = [NamedValidationArgument::new(
        "precision",
        ValidationArgument::String(precision),
    )];
    let bound = registry
        .bind(qubit_validation_rules::ids::TIME_PRECISION, input_type, &args)
        .expect("precision rule binds");
    let outcome = bound
        .validate(value, &BoundValidationContext::new(&[]))
        .expect("time value validates");
    let qubit_validator::ValidationOutcome::Invalid(violations) = outcome else {
        panic!("expected a precision violation, got {outcome:?}");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code().as_str(), "time.precision");
    assert_eq!(violations[0].params().len(), 1);
    assert_eq!(
        violations[0].params().get("precision"),
        Some(&ViolationParam::Token(precision))
    );
}

#[cfg(feature = "time")]
#[test]
fn test_time_precision_violations_report_configured_precision() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use chrono::Utc;
    use qubit_validation_rules::ids;

    let time = NaiveTime::from_hms_nano_opt(12, 0, 0, 123_456_789).unwrap();
    let date_time = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(time);
    let utc: DateTime<Utc> = date_time.and_utc();

    for precision in ["second", "millisecond", "microsecond"] {
        assert_precision_violation(InputType::of::<NaiveTime>(), ValidationValue::Typed(&time), precision);
        assert_precision_violation(
            InputType::of::<chrono::NaiveDateTime>(),
            ValidationValue::Typed(&date_time),
            precision,
        );
        assert_precision_violation(
            InputType::of::<DateTime<Utc>>(),
            ValidationValue::Typed(&utc),
            precision,
        );
    }

    let registry = create_test_registry();
    let args = [NamedValidationArgument::new(
        "precision",
        ValidationArgument::String("nanosecond"),
    )];
    for (input_type, value) in [
        (InputType::of::<NaiveTime>(), ValidationValue::Typed(&time)),
        (
            InputType::of::<chrono::NaiveDateTime>(),
            ValidationValue::Typed(&date_time),
        ),
        (InputType::of::<DateTime<Utc>>(), ValidationValue::Typed(&utc)),
    ] {
        let bound = registry
            .bind(ids::TIME_PRECISION, input_type, &args)
            .expect("nanosecond precision binds");
        assert_eq!(
            bound
                .validate(value, &BoundValidationContext::new(&[]))
                .expect("nanosecond precision validates"),
            qubit_validator::ValidationOutcome::Valid
        );
    }
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
