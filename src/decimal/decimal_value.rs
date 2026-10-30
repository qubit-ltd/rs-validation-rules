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

/// Maximum estimated decimal-digit work for checking a finite endpoint.
///
/// This feasibility-calculation budget is distinct from the raw-literal UTF-8
/// byte limit used by dynamic registration. It is not a runtime input limit.
const MAX_DECIMAL_FEASIBILITY_DIGITS: u64 = 131_072;

/// Rejects a raw finite endpoint whose scale adjustment exceeds the budget.
///
/// The estimate is `coefficient_digits + abs(scale-endpoint_scale) + 1` and is
/// checked before comparison, precision-domain clipping, or grid feasibility
/// work. Each endpoint is checked independently, including a single-sided
/// endpoint. The limit rejects estimates above 131,072.
///
/// # Parameters
/// - `value`: Raw finite endpoint supplied to typed construction.
/// - `scale`: Target decimal scale used by the rule.
/// - `parameter`: Endpoint parameter name (`min` or `max`) for error context.
///
/// # Returns
/// Returns `Ok(())` when the estimate is within the supported budget.
///
/// # Errors
/// Returns `ParameterOutOfRange` with `parameter` when the estimate exceeds
/// 131,072; the endpoint value is not retained in the error.
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

/// Reports whether a decimal satisfies both optional inclusive or exclusive
/// bounds.
///
/// # Parameters
/// - `value`: Decimal value to compare with the bounds.
/// - `min`: Optional lower bound; `None` or `Some(Bound::Unbounded)` imposes no
///   lower restriction.
/// - `max`: Optional upper bound; `None` or `Some(Bound::Unbounded)` imposes no
///   upper restriction.
///
/// # Returns
/// Returns `true` only when `value` satisfies both present bounds, respecting
/// their inclusive or exclusive kind.
#[must_use]
#[inline]
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

/// Determines whether the configured interval contains a value on the scale
/// grid.
///
/// The search first clips the interval to the precision domain, then rounds a
/// candidate up from the lower bound or down from the upper bound and checks it
/// against the original inclusive or exclusive bounds. `Ok(false)` means no
/// representable value exists; `Err` means feasibility calculations exceeded
/// their supported range. With no precision cap and at least one unbounded
/// side, it returns `Ok(true)` immediately because the scale grid extends
/// without bound in that direction.
///
/// # Parameters
/// - `precision`: Optional maximum total precision; `None` leaves the domain
///   unbounded by precision.
/// - `scale`: Scale defining the decimal grid to search.
/// - `min`: Optional inclusive or exclusive lower endpoint; `None` or
///   `Some(Bound::Unbounded)` is unbounded below.
/// - `max`: Optional inclusive or exclusive upper endpoint; `None` or
///   `Some(Bound::Unbounded)` is unbounded above.
///
/// # Returns
/// Returns `Ok(true)` if a value on the scale grid lies within the interval,
/// `Ok(false)` if no such value exists, or an error if feasibility calculations
/// exceed the supported range.
///
/// # Errors
/// Returns `ParameterOutOfRange` with `precision` if constructing the precision
/// domain exceeds the supported calculation range.
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
///
/// # Examples
///
/// ```
/// use qubit_validation_rules::decimal::DecimalValue;
/// use qubit_validator::Validator;
///
/// let rule = DecimalValue::new(Some(3), 2, None, None).expect("valid rule");
/// assert!(rule.validate(&"1.23".parse().expect("decimal"), &()).is_ok());
/// ```
#[must_use]
#[derive(Clone)]
pub struct DecimalValue {
    /// Maximum total precision, or no cap when `None`.
    precision: Option<u16>,
    /// Maximum normalized fractional scale and the accepted decimal grid.
    scale: u16,
    /// Inclusive or exclusive bounds applied after precision and scale checks.
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
    /// This API accepts already-parsed [`BigDecimal`] endpoints. The byte limit
    /// applied to dynamic registration's endpoint strings does not apply here.
    ///
    /// # Parameters
    ///
    /// - `precision`: Maximum total precision, or `None` for no precision cap.
    /// - `scale`: Maximum normalized fractional scale and the decimal grid used
    ///   when determining whether the bounds contain a representable value.
    /// - `min`: Optional inclusive or exclusive lower endpoint; `None` or
    ///   `Some(Bound::Unbounded)` leaves the lower side unbounded.
    /// - `max`: Optional inclusive or exclusive upper endpoint; `None` or
    ///   `Some(Bound::Unbounded)` leaves the upper side unbounded.
    ///
    /// # Returns
    ///
    /// Returns a rule containing the validated precision, scale, and bounds.
    ///
    /// # Errors
    /// Rejects zero precision, scale greater than precision, invalid bounds,
    /// and intervals containing no value on the declared scale grid. After
    /// scalar checks, each raw finite endpoint is independently checked before
    /// bound ordering or precision-domain clipping. The feasibility estimate
    /// rejects endpoint calculations above 131,072 decimal digits; it is not a
    /// general input or runtime resource limit. Errors do not retain endpoint
    /// values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ops::Bound;
    /// use std::str::FromStr;
    ///
    /// use bigdecimal::BigDecimal;
    /// use qubit_validation_rules::decimal::DecimalValue;
    /// use qubit_validator::BindErrorKind;
    ///
    /// let decimal = |s| BigDecimal::from_str(s).expect("decimal literal");
    /// let nonempty = DecimalValue::new(
    ///     Some(3), 2,
    ///     Some(Bound::Excluded(decimal("1.23"))),
    ///     Some(Bound::Included(decimal("1.24"))),
    /// );
    /// assert!(nonempty.is_ok());
    ///
    /// let empty = DecimalValue::new(
    ///     Some(3), 2,
    ///     Some(Bound::Excluded(decimal("1.23"))),
    ///     Some(Bound::Excluded(decimal("1.24"))),
    /// );
    /// assert_eq!(empty.unwrap_err().kind(), BindErrorKind::InvalidBounds);
    /// ```
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
        let _ = Range::new(
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

/// Borrows a bound's endpoint while preserving its bound kind.
///
/// # Type Parameters
/// - `T`: Endpoint type stored by the bound.
///
/// # Parameters
/// - `bound`: Bound whose finite endpoint is borrowed.
///
/// # Returns
/// Returns the corresponding `Bound<&T>`: `Included` and `Excluded` retain
/// their kind and borrow the endpoint, while `Unbounded` remains unbounded.
#[must_use]
#[inline]
fn borrow_bound<T>(bound: &Bound<T>) -> Bound<&T> {
    match bound {
        Bound::Included(value) => Bound::Included(value),
        Bound::Excluded(value) => Bound::Excluded(value),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl Validator<BigDecimal, ()> for DecimalValue {
    type Error = DecimalValueError;

    /// Validates a borrowed decimal against the configured scale, precision,
    /// and range constraints, in that order. The unit context is unused.
    ///
    /// Representation-only trailing fractional zeros do not count toward the
    /// normalized scale or precision, and zero has normalized scale and
    /// precision requirements of zero. Values are checked exactly; this method
    /// does not round them. The endpoint feasibility budget used by
    /// [`DecimalValue::new`] does not limit runtime input values.
    ///
    /// # Parameters
    ///
    /// - `value`: Decimal value to validate.
    /// - `context`: Unit context; unused by this validator.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value satisfies all configured constraints.
    ///
    /// # Errors
    ///
    /// Returns [`DecimalValueError::Scale`] when normalized scale exceeds the
    /// configured scale, [`DecimalValueError::Precision`] when required
    /// precision exceeds the configured precision, or
    /// [`DecimalValueError::Range`] when the value is outside the configured
    /// range. When multiple constraints fail, the first error in the order
    /// above is returned.
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
