// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ops::Bound;

use qubit_validation_rules::collection::Range;
use qubit_validation_rules::collection::RangeError;
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
use qubit_validator::ViolationParam;

const ITEM_COUNT_ID: &str = "qubit.rules.collection.item_count";

/// Rejects values that cannot be ordered even when neither endpoint is bounded.
#[test]
fn test_range_rejects_nan_with_unbounded_endpoints() {
    let range = Range::<f64>::new(Bound::Unbounded, Bound::Unbounded).expect("unbounded range binds");
    assert_eq!(range.validate(&f64::NAN, &()), Err(RangeError::Unordered));

    let integer_range = Range::<i32>::new(Bound::Unbounded, Bound::Unbounded).expect("integer range binds");
    assert_eq!(integer_range.validate(&0, &()), Ok(()));
}

/// Rejects NaN endpoints independently, including when the other end is absent.
#[test]
fn test_range_rejects_nan_single_endpoints() {
    for lower in [Bound::Included(f64::NAN), Bound::Excluded(f64::NAN)] {
        let error = Range::new(lower, Bound::Unbounded).expect_err("NaN lower endpoint is invalid");
        assert_eq!(error.kind(), BindErrorKind::InvalidBounds);
    }
    for upper in [Bound::Included(f64::NAN), Bound::Excluded(f64::NAN)] {
        let error = Range::new(Bound::Unbounded, upper).expect_err("NaN upper endpoint is invalid");
        assert_eq!(error.kind(), BindErrorKind::InvalidBounds);
    }
}

/// Uses the full platform count width for registry parameters and violations.
#[cfg(target_pointer_width = "64")]
#[test]
fn test_item_count_registry_accepts_bounds_above_u32_max() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard rules are valid");
    let large_bound = u32::MAX as usize + 1;
    let min_arguments = [NamedValidationArgument::new(
        "min",
        ValidationArgument::Unsigned(large_bound as u128),
    )];
    let minimum_rule = registry
        .bind(ITEM_COUNT_ID, InputType::of::<usize>(), &min_arguments, &[])
        .expect("large minimum binds");
    assert_eq!(
        minimum_rule
            .validate(ValidationValue::Typed(&large_bound), &BoundValidationContext::new(&[]))
            .expect("count at minimum executes"),
        ValidationOutcome::Valid
    );
    let too_small = minimum_rule
        .validate(
            ValidationValue::Typed(&(large_bound - 1)),
            &BoundValidationContext::new(&[]),
        )
        .expect("count below minimum executes");
    let ValidationOutcome::Invalid(violations) = too_small else {
        panic!("expected too-small violation, got {too_small:?}");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code().as_str(), "collection.too_small");
    assert_eq!(
        violations[0].params().get("bound"),
        Some(&ViolationParam::Unsigned(large_bound as u128))
    );

    let max_arguments = [NamedValidationArgument::new(
        "max",
        ValidationArgument::Unsigned(large_bound as u128),
    )];
    let maximum_rule = registry
        .bind(ITEM_COUNT_ID, InputType::of::<usize>(), &max_arguments, &[])
        .expect("large maximum binds");
    assert_eq!(
        maximum_rule
            .validate(ValidationValue::Typed(&large_bound), &BoundValidationContext::new(&[]))
            .expect("count at maximum executes"),
        ValidationOutcome::Valid
    );
    let too_large = maximum_rule
        .validate(
            ValidationValue::Typed(&(large_bound + 1)),
            &BoundValidationContext::new(&[]),
        )
        .expect("count above maximum executes");
    let ValidationOutcome::Invalid(violations) = too_large else {
        panic!("expected too-large violation, got {too_large:?}");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code().as_str(), "collection.too_large");
    assert_eq!(
        violations[0].params().get("bound"),
        Some(&ViolationParam::Unsigned(large_bound as u128))
    );
}

/// Reports the offending parameter name when a bound exceeds `usize`.
#[test]
fn test_item_count_registry_rejects_bounds_above_usize_max() {
    let registry = ValidatorRegistry::from_registrations(registrations()).expect("standard rules are valid");
    for name in ["min", "max"] {
        let arguments = [NamedValidationArgument::new(
            name,
            ValidationArgument::Unsigned(usize::MAX as u128 + 1),
        )];
        let error = registry
            .bind(ITEM_COUNT_ID, InputType::of::<usize>(), &arguments, &[])
            .expect_err("out-of-range count bound must fail binding");
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some(name));
    }
}
