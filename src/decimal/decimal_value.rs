// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ops::Bound;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use bigdecimal::RoundingMode;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

use super::decimal_value_error::DecimalValueError;
use crate::collection::Range;

const MAX_DECIMAL_FEASIBILITY_DIGITS: u64 = 131_072;

fn check_endpoint_cost(value: &BigDecimal, scale: u16, parameter: &str) -> Result<(), BindError> {
    let source_scale = i128::from(value.fractional_digit_count());
    let scale_delta = (i128::from(scale) - source_scale).unsigned_abs();
    if value
        .decimal_digit_count()
        .saturating_add(u64::try_from(scale_delta).unwrap_or(u64::MAX))
        .saturating_add(1)
        > MAX_DECIMAL_FEASIBILITY_DIGITS
    {
        return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter(parameter));
    }
    Ok(())
}

fn satisfies_bounds(value: &BigDecimal, min: &Option<Bound<BigDecimal>>, max: &Option<Bound<BigDecimal>>) -> bool {
    let above_min = match min {
        None | Some(Bound::Unbounded) => true,
        Some(Bound::Included(bound)) => value >= bound,
        Some(Bound::Excluded(bound)) => value > bound,
    };
    let below_max = match max {
        None | Some(Bound::Unbounded) => true,
        Some(Bound::Included(bound)) => value <= bound,
        Some(Bound::Excluded(bound)) => value < bound,
    };
    above_min && below_max
}

fn has_representable_value(
    precision: Option<u16>,
    scale: u16,
    min: &Option<Bound<BigDecimal>>,
    max: &Option<Bound<BigDecimal>>,
) -> Result<bool, BindError> {
    let lower = min.as_ref().and_then(|bound| match bound {
        Bound::Included(value) | Bound::Excluded(value) => Some((value, matches!(bound, Bound::Excluded(_)))),
        Bound::Unbounded => None,
    });
    let upper = max.as_ref().and_then(|bound| match bound {
        Bound::Included(value) | Bound::Excluded(value) => Some((value, matches!(bound, Bound::Excluded(_)))),
        Bound::Unbounded => None,
    });

    if precision.is_none() && (lower.is_none() || upper.is_none()) {
        return Ok(true);
    }

    let precision_domain = precision
        .map(|precision| {
            let digits = "9".repeat(usize::from(precision));
            let max_coefficient = BigDecimal::from_str(&digits)
                .map_err(|_| BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("precision"))?;
            let (coefficient, _) = max_coefficient.into_bigint_and_scale();
            let max_value = BigDecimal::from_bigint(coefficient, i64::from(scale));
            Ok::<_, BindError>((-max_value.clone(), max_value))
        })
        .transpose()?;

    if let (Some((bound, exclusive)), Some((_, domain_max))) = (lower, &precision_domain)
        && (bound > domain_max || (exclusive && bound == domain_max))
    {
        return Ok(false);
    }
    if let (Some((bound, exclusive)), Some((domain_min, _))) = (upper, &precision_domain)
        && (bound < domain_min || (exclusive && bound == domain_min))
    {
        return Ok(false);
    }

    let effective_lower = match (lower, &precision_domain) {
        (Some((bound, _exclusive)), Some((domain_min, _))) if bound < domain_min => Some((domain_min, false)),
        _ => lower,
    };
    let effective_upper = match (upper, &precision_domain) {
        (Some((bound, _exclusive)), Some((_, domain_max))) if bound > domain_max => Some((domain_max, false)),
        _ => upper,
    };

    let quantum = BigDecimal::from_bigint(1.into(), i64::from(scale));
    let mut candidate = if let Some((bound, exclusive)) = effective_lower {
        let mut value = bound.with_scale_round(i64::from(scale), RoundingMode::Ceiling);
        if exclusive && &value == bound {
            value += &quantum;
        }
        value
    } else if let Some((bound, exclusive)) = effective_upper {
        let mut value = bound.with_scale_round(i64::from(scale), RoundingMode::Floor);
        if exclusive && &value == bound {
            value -= &quantum;
        }
        value
    } else {
        BigDecimal::from(0)
    };

    if let Some((min_value, max_value)) = precision_domain {
        if candidate < min_value {
            candidate = min_value;
        } else if candidate > max_value {
            candidate = max_value;
        }
    }
    Ok(satisfies_bounds(&candidate, min, max))
}

/// Validates normalized decimal scale, total precision capacity, and exact
/// bounds.
#[derive(Clone)]
pub struct DecimalValue {
    precision: Option<u16>,
    scale: u16,
    range: Range<BigDecimal>,
}

impl std::fmt::Debug for DecimalValue {
    /// Reports public numeric limits without formatting boundary values.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DecimalValue")
            .field("precision", &self.precision)
            .field("scale", &self.scale)
            .finish_non_exhaustive()
    }
}

impl DecimalValue {
    /// Creates a decimal rule with optional inclusive or exclusive bounds.
    /// `precision` limits values to `[-(10^p - 1) * 10^-s,
    /// (10^p - 1) * 10^-s]`, where `p` is precision and `s` is scale.
    /// Representation-only trailing fractional zeros do not consume scale.
    ///
    /// # Errors
    /// Rejects zero precision, scale greater than precision, invalid bounds,
    /// and intervals containing no value on the declared scale grid. After
    /// scalar checks, each raw finite endpoint is independently checked before
    /// bound ordering or precision-domain clipping. The feasibility estimate
    /// rejects endpoint calculations above 131,072 decimal digits; it is not a
    /// general input or runtime resource limit. Errors do not retain endpoint
    /// values.
    pub fn new(
        precision: Option<u16>,
        scale: u16,
        min: Option<Bound<BigDecimal>>,
        max: Option<Bound<BigDecimal>>,
    ) -> Result<Self, BindError> {
        if precision.is_some_and(|p| p == 0 || scale > p) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        for (bound, parameter) in [(&min, "min"), (&max, "max")] {
            if let Some(Bound::Included(value) | Bound::Excluded(value)) = bound {
                check_endpoint_cost(value, scale, parameter)?;
            }
        }
        Range::new(
            min.as_ref().map_or(Bound::Unbounded, borrow_bound),
            max.as_ref().map_or(Bound::Unbounded, borrow_bound),
        )?;
        if !has_representable_value(precision, scale, &min, &max)? {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        let range = Range::new(min.unwrap_or(Bound::Unbounded), max.unwrap_or(Bound::Unbounded))?;
        Ok(Self {
            precision,
            scale,
            range,
        })
    }
}

fn borrow_bound<T>(bound: &Bound<T>) -> Bound<&T> {
    match bound {
        Bound::Included(value) => Bound::Included(value),
        Bound::Excluded(value) => Bound::Excluded(value),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl Validator<BigDecimal, ()> for DecimalValue {
    type Error = DecimalValueError;

    fn validate(&self, value: &BigDecimal, _: &()) -> Result<(), Self::Error> {
        let (coefficient, exponent) = value.as_bigint_and_scale();
        let decimal_digits = coefficient.to_str_radix(10);
        let unsigned_digits = decimal_digits.strip_prefix('-').unwrap_or(&decimal_digits);
        let significant_digits = unsigned_digits.trim_end_matches('0');
        let normalized_scale = if significant_digits.is_empty() {
            0
        } else {
            let trailing_zeros = unsigned_digits.len() - significant_digits.len();
            let trailing_zeros = i128::try_from(trailing_zeros).unwrap_or(i128::MAX);
            i128::from(exponent) - trailing_zeros
        };
        if normalized_scale > i128::from(self.scale) {
            return Err(DecimalValueError::Scale);
        }
        if let Some(precision) = self.precision {
            let integer_digits = if significant_digits.is_empty() {
                0
            } else {
                i128::try_from(significant_digits.len())
                    .unwrap_or(i128::MAX)
                    .saturating_sub(normalized_scale)
                    .max(0)
            };
            let required_precision = integer_digits.saturating_add(i128::from(self.scale));
            if required_precision > i128::from(precision) {
                return Err(DecimalValueError::Precision);
            }
        }
        self.range.validate(value, &()).map_err(|_| DecimalValueError::Range)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Bound;

    use bigdecimal::BigDecimal;
    use qubit_validator::BindErrorKind;

    use super::DecimalValue;
    use super::check_endpoint_cost;

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
    fn test_decimal_core_raw_endpoint_cost() {
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
        assert!(check_endpoint_cost(&BigDecimal::from_bigint(1.into(), -131_070), 0, "min").is_ok());
        for exponent in [i64::MIN, i64::MAX] {
            assert!(check_endpoint_cost(&BigDecimal::from_bigint(1.into(), exponent), 0, "max").is_err());
        }
        assert!(DecimalValue::new(None, 2, Some(Bound::Unbounded), None).is_ok());
        assert!(DecimalValue::new(None, 2, None, Some(Bound::Included(decimal("1.23")))).is_ok());
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
                        assert_eq!(DecimalValue::new(Some(2), 1, Some(low), Some(high)).is_ok(), expected);
                    }
                }
            }
        }
    }
}
