// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cell::Cell;
use std::cmp::Ordering;
use std::ops::Bound;
use std::rc::Rc;

use qubit_validation_rules::collection::ComparisonLimitExceeded;
use qubit_validation_rules::collection::ItemCount;
use qubit_validation_rules::collection::ItemCountError;
use qubit_validation_rules::collection::Range;
use qubit_validation_rules::collection::RangeError;
use qubit_validation_rules::collection::UniqueItems;
use qubit_validation_rules::collection::UniquePairs;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DisjointOrder {
    branch: u8,
    value: u8,
}

impl PartialOrd for DisjointOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.branch == other.branch {
            self.value.partial_cmp(&other.value)
        } else {
            None
        }
    }
}

#[test]
fn test_range_rejects_distinct_unordered_endpoints() {
    let lower = DisjointOrder { branch: 0, value: 1 };
    let upper = DisjointOrder { branch: 1, value: 3 };
    assert_eq!(lower.partial_cmp(&lower), Some(Ordering::Equal));
    assert_eq!(upper.partial_cmp(&upper), Some(Ordering::Equal));
    assert_eq!(lower.partial_cmp(&upper), None);
    for lower in [Bound::Included(lower), Bound::Excluded(lower)] {
        for upper in [Bound::Included(upper), Bound::Excluded(upper)] {
            let error = Range::new(lower, upper).expect_err("distinct branches are unordered");
            assert_eq!(error.kind(), BindErrorKind::InvalidBounds);
        }
    }
}

#[test]
fn test_range_rejects_distinct_unordered_values() {
    let endpoint = DisjointOrder { branch: 0, value: 2 };
    let value = DisjointOrder { branch: 1, value: 2 };
    assert_eq!(value.partial_cmp(&value), Some(Ordering::Equal));
    for endpoint in [Bound::Included(endpoint), Bound::Excluded(endpoint)] {
        let lower = Range::new(endpoint, Bound::Unbounded).expect("valid lower endpoint");
        assert_eq!(lower.validate(&value, &()), Err(RangeError::Unordered));
        let upper = Range::new(Bound::Unbounded, endpoint).expect("valid upper endpoint");
        assert_eq!(upper.validate(&value, &()), Err(RangeError::Unordered));
    }
    let lower = DisjointOrder { branch: 0, value: 1 };
    let upper = DisjointOrder { branch: 0, value: 3 };
    let range = Range::new(Bound::Included(lower), Bound::Included(upper)).expect("one branch is ordered");
    assert_eq!(range.validate(&endpoint, &()), Ok(()));
}

#[test]
fn test_unique_pairs_emits_each_pair_in_duplicate_search_order() {
    assert_eq!(
        UniquePairs::new(4).collect::<Vec<_>>(),
        vec![(0, 1), (0, 2), (1, 2), (0, 3), (1, 3), (2, 3)],
    );
    assert_eq!(UniquePairs::new(0).collect::<Vec<_>>(), Vec::<(usize, usize)>::new());
    assert_eq!(UniquePairs::new(1).collect::<Vec<_>>(), Vec::<(usize, usize)>::new());
    assert_eq!(UniquePairs::new(2).collect::<Vec<_>>(), vec![(0, 1)]);
}

#[test]
fn test_range_debug_redacts_endpoints() {
    let range =
        Range::new(Bound::Included("sensitive-lower"), Bound::Excluded("sensitive-upper")).expect("ordered bounds");
    let debug = format!("{range:?}");

    assert_eq!(
        debug,
        "Range { lower: Included(<redacted>), upper: Excluded(<redacted>) }"
    );
    assert!(!debug.contains("sensitive-lower"));
    assert!(!debug.contains("sensitive-upper"));

    let unbounded = Range::<i32>::new(Bound::Unbounded, Bound::Unbounded).expect("unbounded range");
    assert_eq!(format!("{unbounded:?}"), "Range { lower: Unbounded, upper: Unbounded }");
}

#[test]
fn test_range_debug_does_not_require_debug_endpoints() {
    #[derive(PartialEq, PartialOrd)]
    struct PartialOrdOnly(i32);

    let range = Range::new(Bound::Included(PartialOrdOnly(1)), Bound::Unbounded).expect("ordered bounds");
    assert_eq!(
        format!("{range:?}"),
        "Range { lower: Included(<redacted>), upper: Unbounded }"
    );
}

#[test]
fn test_unique_items_finds_first_partial_eq_duplicate() {
    assert_eq!(UniqueItems::first_duplicate_with_limit(&[1, 2, 1], 2), Ok(Some((0, 2))));
    assert_eq!(
        UniqueItems::first_duplicate_with_limit(&[1, 2, 2, 1], 3),
        Ok(Some((1, 2)))
    );
    assert_eq!(UniqueItems::first_duplicate_with_limit::<i32>(&[], 0), Ok(None));
    assert_eq!(UniqueItems::first_duplicate_with_limit(&[1], 0), Ok(None));
    assert_eq!(
        UniqueItems::first_duplicate_with_limit(&[1, 2, 1], 1),
        Err(ComparisonLimitExceeded)
    );
    assert_eq!(
        UniqueItems::first_duplicate_with_limit(&[1, 2], 0),
        Err(ComparisonLimitExceeded)
    );
    assert_eq!(
        UniqueItems::first_duplicate_with_limit(&[f64::NAN, f64::NAN], 1),
        Ok(None)
    );
    assert_eq!(
        ComparisonLimitExceeded.to_string(),
        "unique item comparison limit exceeded"
    );
}

#[test]
fn test_unique_items_does_not_exceed_comparison_budget() {
    struct CountedEq {
        value: i32,
        comparisons: Rc<Cell<usize>>,
    }
    impl PartialEq for CountedEq {
        fn eq(&self, other: &Self) -> bool {
            self.comparisons.set(self.comparisons.get() + 1);
            self.value == other.value
        }
    }

    let comparisons = Rc::new(Cell::new(0));
    let values = [1, 2, 3].map(|value| CountedEq {
        value,
        comparisons: Rc::clone(&comparisons),
    });
    assert_eq!(
        UniqueItems::first_duplicate_with_limit(&values, 2),
        Err(ComparisonLimitExceeded)
    );
    assert_eq!(comparisons.get(), 2);
}

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

/// Shows that ordered open endpoints can contain no value for a discrete type.
#[test]
fn test_discrete_open_interval_can_be_empty() {
    let range = Range::new(Bound::Excluded(1_i32), Bound::Excluded(2_i32))
        .expect("ordered endpoints need not contain an integer");
    for value in 0..=3 {
        assert_eq!(range.validate(&value, &()), Err(RangeError::OutOfRange));
    }
    let error = Range::new(Bound::Excluded(1_i32), Bound::Included(1_i32))
        .expect_err("equal endpoints with exclusion are directly contradictory");
    assert_eq!(error.kind(), BindErrorKind::InvalidBounds);
}

#[test]
fn test_range_open_closed_endpoint_matrix() {
    for lower_inclusive in [true, false] {
        for upper_inclusive in [true, false] {
            let lower = if lower_inclusive {
                Bound::Included(2)
            } else {
                Bound::Excluded(2)
            };
            let upper = if upper_inclusive {
                Bound::Included(5)
            } else {
                Bound::Excluded(5)
            };
            let rule = Range::new(lower, upper).expect("ordered endpoints form a valid range");
            for (value, accepted) in [
                (1, false),
                (2, lower_inclusive),
                (3, true),
                (4, true),
                (5, upper_inclusive),
                (6, false),
            ] {
                let expected = if accepted { Ok(()) } else { Err(RangeError::OutOfRange) };
                assert_eq!(
                    rule.validate(&value, &()),
                    expected,
                    "lower_inclusive={lower_inclusive}, upper_inclusive={upper_inclusive}, value={value}"
                );
            }
        }
    }
}

#[test]
fn test_range_included_singleton_accepts_only_endpoint() {
    let rule = Range::new(Bound::Included(3), Bound::Included(3)).expect("an included singleton is nonempty");
    assert_eq!(rule.validate(&3, &()), Ok(()));
    for value in [2, 4] {
        assert_eq!(rule.validate(&value, &()), Err(RangeError::OutOfRange));
    }
}

#[test]
fn test_range_equal_endpoints_with_exclusion_are_invalid() {
    for (lower, upper) in [
        (Bound::Included(3), Bound::Excluded(3)),
        (Bound::Excluded(3), Bound::Excluded(3)),
    ] {
        let error = Range::new(lower, upper).expect_err("an excluded singleton is empty");
        assert_eq!(error.kind(), BindErrorKind::InvalidBounds);
    }
    // The existing collection test covers (Excluded(3), Included(3)).
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
        .bind(ITEM_COUNT_ID, InputType::of::<usize>(), &min_arguments)
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
        .bind(ITEM_COUNT_ID, InputType::of::<usize>(), &max_arguments)
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
            .bind(ITEM_COUNT_ID, InputType::of::<usize>(), &arguments)
            .expect_err("out-of-range count bound must fail binding");
        assert_eq!(error.kind(), BindErrorKind::ParameterOutOfRange);
        assert_eq!(error.parameter(), Some(name));
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
