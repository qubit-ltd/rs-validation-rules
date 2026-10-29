// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::ops::Bound;
use std::str::FromStr;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::DependencySpec;
use qubit_validator::InputType;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::ViolationParam;
use qubit_validator::prepare_typed_validator;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

use crate::decimal::DecimalValue;
use crate::decimal::DecimalValueError;

/// Dependency list for decimal rules, which do not read other validated fields.
const EMPTY_DEPS: &[DependencySpec] = &[];
/// Maximum UTF-8 byte length of a raw dynamic decimal endpoint literal.
///
/// This limit is checked before syntax scanning and parsing. It does not apply
/// to typed construction with an already-parsed [`BigDecimal`] endpoint.
const MAX_DECIMAL_ENDPOINT_LITERAL_BYTES: usize = 131_072;

/// Checks a dynamic endpoint's UTF-8 byte length before scanning or parsing it.
///
/// # Parameters
/// - `literal`: Raw endpoint text from the dynamic rule declaration.
/// - `name`: Parameter name to identify in a binding error.
///
/// # Returns
/// Returns `Ok(())` when the literal is at most 131,072 UTF-8 bytes.
///
/// # Errors
/// Returns `ParameterOutOfRange` with only `name` when the limit is exceeded;
/// the literal contents are not retained in the error.
fn check_endpoint_literal_length(literal: &str, name: &str) -> Result<(), BindError> {
    if literal.len() > MAX_DECIMAL_ENDPOINT_LITERAL_BYTES {
        return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter(name));
    }
    Ok(())
}

/// Checks the complete ASCII decimal grammar `-?[0-9]+(\.[0-9]*)?`.
///
/// Leading zeros and an empty fractional part are accepted, including `001.20`
/// and `1.`. Whitespace is not trimmed. A plus sign, exponent notation, and a
/// leading decimal point without integer digits, such as `+1`, `1e2`, and `.5`,
/// are rejected. This check does not enforce byte limits or determine whether
/// the parsed number satisfies configured bounds.
///
/// # Parameters
/// - `value`: Complete raw decimal literal to check.
///
/// # Returns
/// Returns `true` if the entire literal matches the grammar, otherwise `false`.
fn canonical_decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let mut parts = unsigned.split('.');
    let Some(integer) = parts.next() else {
        return false;
    };
    let fraction = parts.next();
    parts.next().is_none()
        && !integer.is_empty()
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fraction.is_none_or(|digits| digits.bytes().all(|byte| byte.is_ascii_digit()))
}

/// Reads a named finite endpoint, returning `None` when it was not declared.
///
/// A declared literal is length-checked before syntax scanning and parsing,
/// then converted to an inclusive or exclusive bound according to `inclusive`.
///
/// # Parameters
/// - `reader`: Reader for the dynamic rule parameters.
/// - `name`: Endpoint parameter name, such as `min` or `max`.
/// - `inclusive`: Whether a declared endpoint is inclusive (`true`) or
///   exclusive (`false`).
///
/// # Returns
/// Returns `Ok(None)` when the parameter is absent, or `Ok(Some(bound))` with
/// the parsed finite endpoint and requested inclusivity when present.
///
/// # Errors
/// Propagates parameter-reading errors. A too-long literal, invalid syntax, or
/// failed decimal conversion returns `ParameterOutOfRange` identifying only
/// `name`; the literal contents are not retained.
fn optional_bound(
    reader: &mut ArgumentReader<'_>,
    name: &str,
    inclusive: bool,
) -> Result<Option<Bound<BigDecimal>>, BindError> {
    let Some(literal) = reader.optional_str(name)? else {
        return Ok(None);
    };
    check_endpoint_literal_length(literal, name)?;
    if !canonical_decimal(literal) {
        return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter(name));
    }
    let number = BigDecimal::from_str(literal)
        .map_err(|_| BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter(name))?;
    Ok(Some(if inclusive {
        Bound::Included(number)
    } else {
        Bound::Excluded(number)
    }))
}

/// Decodes decimal parameters and prepares the typed decimal adapter.
///
/// # Parameters
/// - `reader`: Shared reader for scale, precision, and optional bounds.
///
/// # Returns
/// Returns a prepared validator that accepts `BigDecimal` inputs and applies
/// the configured decimal rule.
///
/// # Errors
/// Propagates errors while reading parameters; reports `ParameterOutOfRange`
/// for invalid precision or scale values and invalid dynamic endpoints; and
/// propagates construction errors such as invalid bounds or an interval with
/// no representable value. The prepared validator maps runtime scale,
/// precision, and range violations to their corresponding violation codes.
/// The binding flow reports duplicate and unknown parameters.
fn prepare_decimal(reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let precision = reader
        .optional_u32("precision")?
        .map(|value| {
            u16::try_from(value)
                .map_err(|_| BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("precision"))
        })
        .transpose()?;
    let scale = u16::try_from(reader.required_u32("scale")?)
        .map_err(|_| BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("scale"))?;
    if precision == Some(0) {
        return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("precision"));
    }
    if precision.is_some_and(|value| scale > value) {
        return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("scale"));
    }
    let min_inclusive = reader.optional_bool("min_inclusive")?.unwrap_or(true);
    let max_inclusive = reader.optional_bool("max_inclusive")?.unwrap_or(true);
    let min = optional_bound(reader, "min", min_inclusive)?;
    let max = optional_bound(reader, "max", max_inclusive)?;
    let rule = DecimalValue::new(precision, scale, min, max)?;
    Ok(prepare_typed_validator(rule, move |error| match error {
        DecimalValueError::Scale => ViolationDraft::new(ViolationCode::new("decimal.scale"))
            .with_param("scale", ViolationParam::Unsigned(u128::from(scale))),
        DecimalValueError::Precision => ViolationDraft::new(ViolationCode::new("decimal.precision")).with_param(
            "precision",
            ViolationParam::Unsigned(u128::from(precision.unwrap_or(0))),
        ),
        DecimalValueError::Range => ViolationDraft::new(ViolationCode::new("decimal.range"))
            .with_param("min_inclusive", ViolationParam::Bool(min_inclusive))
            .with_param("max_inclusive", ViolationParam::Bool(max_inclusive)),
    }))
}

/// Signature binding `BigDecimal` input and no dependencies to
/// `prepare_decimal`.
const DECIMAL_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::of::<BigDecimal>(), EMPTY_DEPS, prepare_decimal);
/// Descriptor containing the decimal validator's dynamic binding signature.
pub(super) static DESC_DECIMAL: ValidatorDescriptor = ValidatorDescriptor::new(&[DECIMAL_SIG]);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_DECIMAL: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::DECIMAL_VALUE),
    &DESC_DECIMAL,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_DECIMAL);
