// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#![cfg(feature = "decimal")]

use std::ops::Bound;

use bigdecimal::BigDecimal;
use qubit_validation_rules::decimal::DecimalValue;
use qubit_validator::BindErrorKind;

fn decimal(text: &str) -> BigDecimal {
    text.parse().expect("test decimal")
}

#[test]
fn test_decimal_core_grid_and_capacity() {
    for (precision, scale, min, max, min_closed, max_closed, valid) in [
        (None, 2, "1.23", "1.24", false, true, true),
        (None, 2, "-1.24", "-1.23", true, false, true),
        (None, 2, "1.23", "1.24", false, false, false),
        (Some(3), 2, "10", "11", true, true, false),
        (Some(3), 2, "-11", "-10", true, true, false),
        (Some(3), 2, "9.99", "9.99", true, true, true),
        (Some(2), 2, "0.99", "0.99", true, true, true),
        (Some(2), 2, "1", "2", true, true, false),
        (Some(2), 0, "99", "99", true, true, true),
    ] {
        let lower = if min_closed {
            Bound::Included(decimal(min))
        } else {
            Bound::Excluded(decimal(min))
        };
        let upper = if max_closed {
            Bound::Included(decimal(max))
        } else {
            Bound::Excluded(decimal(max))
        };
        let result = DecimalValue::new(precision, scale, Some(lower), Some(upper));
        assert_eq!(result.is_ok(), valid, "{precision:?}/{scale}: {min}, {max}");
        if !valid {
            assert_eq!(result.unwrap_err().kind(), BindErrorKind::InvalidBounds);
        }
    }
    for (min, max) in [
        (Some(Bound::Included(decimal("10"))), None),
        (None, Some(Bound::Included(decimal("-10")))),
        (Some(Bound::Excluded(decimal("9.99"))), None),
        (None, Some(Bound::Excluded(decimal("-9.99")))),
    ] {
        assert_eq!(
            DecimalValue::new(Some(3), 2, min, max).unwrap_err().kind(),
            BindErrorKind::InvalidBounds,
        );
    }
}

#[test]
fn test_decimal_core_small_grid_oracle() {
    for lower in -3i32..=3 {
        for upper in lower..=3 {
            for lower_closed in [false, true] {
                for upper_closed in [false, true] {
                    let expected = (-99i32..=99).any(|k| {
                        let low = lower * 10;
                        let high = upper * 10;
                        (k > low || lower_closed && k == low) && (k < high || upper_closed && k == high)
                    });
                    let low = if lower_closed {
                        Bound::Included(BigDecimal::from(lower))
                    } else {
                        Bound::Excluded(BigDecimal::from(lower))
                    };
                    let high = if upper_closed {
                        Bound::Included(BigDecimal::from(upper))
                    } else {
                        Bound::Excluded(BigDecimal::from(upper))
                    };
                    assert_eq!(DecimalValue::new(Some(2), 1, Some(low), Some(high)).is_ok(), expected,);
                }
            }
        }
    }
}
