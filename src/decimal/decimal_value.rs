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
            let max_value = max_coefficient.with_scale(i64::from(scale));
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

    if let Some((bound, _)) = effective_lower {
        check_endpoint_cost(bound, scale, "min")?;
    } else if let Some((bound, _)) = effective_upper {
        check_endpoint_cost(bound, scale, "max")?;
    }

    let mut candidate = if let Some((bound, exclusive)) = effective_lower {
        let mut value = bound.with_scale_round(i64::from(scale), RoundingMode::Ceiling);
        if exclusive && &value == bound {
            let next = BigDecimal::from(1).with_scale(i64::from(scale));
            value += next;
        }
        value
    } else if let Some((bound, exclusive)) = effective_upper {
        let mut value = bound.with_scale_round(i64::from(scale), RoundingMode::Floor);
        if exclusive && &value == bound {
            let previous = BigDecimal::from(1).with_scale(i64::from(scale));
            value -= previous;
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
    /// `precision` limits the total capacity at the declared `scale`: at most
    /// `precision - scale` integer digits and `scale` fractional digits.
    /// Representation-only trailing fractional zeros do not consume scale.
    ///
    /// # Errors
    /// Rejects zero precision, scale greater than precision, invalid bounds,
    /// and intervals containing no value on the declared scale grid. The
    /// feasibility check limits endpoint intermediates to 131,072 decimal
    /// digits. Errors do not retain endpoint values.
    pub fn new(
        precision: Option<u16>,
        scale: u16,
        min: Option<Bound<BigDecimal>>,
        max: Option<Bound<BigDecimal>>,
    ) -> Result<Self, BindError> {
        if precision.is_some_and(|p| p == 0 || scale > p) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
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
