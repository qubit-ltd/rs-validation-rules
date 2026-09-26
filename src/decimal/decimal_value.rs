// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ops::Bound;

use bigdecimal::BigDecimal;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::Validator;

use super::decimal_value_error::DecimalValueError;
use crate::collection::Range;

/// Validates normalized decimal scale, significant digits, and exact bounds.
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
    /// `precision` counts digits in the normalized coefficient; zero counts as
    /// one digit. `scale` limits its positive decimal exponent.
    ///
    /// # Errors
    /// Rejects zero precision, scale greater than precision, and invalid or
    /// contradictory bounds without retaining endpoint values in the error.
    pub fn new(
        precision: Option<u16>,
        scale: u16,
        min: Option<Bound<BigDecimal>>,
        max: Option<Bound<BigDecimal>>,
    ) -> Result<Self, BindError> {
        if precision.is_some_and(|p| p == 0 || scale > p) {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        let range = Range::new(min.unwrap_or(Bound::Unbounded), max.unwrap_or(Bound::Unbounded))?;
        Ok(Self {
            precision,
            scale,
            range,
        })
    }
}

impl Validator<BigDecimal, ()> for DecimalValue {
    type Error = DecimalValueError;

    fn validate(&self, value: &BigDecimal, _: &()) -> Result<(), Self::Error> {
        let (coefficient, exponent) = value.as_bigint_and_scale();
        let decimal_digits = coefficient.to_str_radix(10);
        let unsigned_digits = decimal_digits.strip_prefix('-').unwrap_or(&decimal_digits);
        let significant_digits = unsigned_digits.trim_end_matches('0');
        let (normalized_scale, precision) = if significant_digits.is_empty() {
            (0, 1)
        } else {
            let trailing_zeros = unsigned_digits.len() - significant_digits.len();
            let trailing_zeros = i64::try_from(trailing_zeros).unwrap_or(i64::MAX);
            // An underflow here still means an integral value. Saturating
            // preserves that classification without BigDecimal::normalized's panic.
            (
                exponent.saturating_sub(trailing_zeros),
                u64::try_from(significant_digits.len()).unwrap_or(u64::MAX),
            )
        };
        if normalized_scale > i64::from(self.scale) {
            return Err(DecimalValueError::Scale);
        }
        if self.precision.is_some_and(|p| precision > u64::from(p)) {
            return Err(DecimalValueError::Precision);
        }
        self.range.validate(value, &()).map_err(|_| DecimalValueError::Range)
    }
}
