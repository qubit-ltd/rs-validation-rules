// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// A failed decimal constraint, without the input or endpoint value.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use bigdecimal::BigDecimal;
/// use qubit_validation_rules::decimal::DecimalValue;
/// use qubit_validation_rules::decimal::DecimalValueError;
/// use qubit_validator::Validator;
///
/// let validator = DecimalValue::new(Some(2), 0, None, None).expect("valid decimal rule");
/// let value = BigDecimal::from_str("1.2").expect("decimal literal");
/// assert_eq!(validator.validate(&value, &()), Err(DecimalValueError::Scale));
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DecimalValueError {
    /// The normalized value has too many fractional places.
    #[error("decimal scale exceeded")]
    Scale,
    /// The normalized value exceeds the declared decimal precision capacity.
    #[error("decimal precision exceeded")]
    Precision,
    /// The value falls outside its declared interval.
    #[error("decimal range exceeded")]
    Range,
}
