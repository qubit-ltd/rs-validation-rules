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
use qubit_validator::NamedValidationArgument;
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

const EMPTY_DEPS: &[DependencySpec] = &[];

/// Accepts the decimal literal syntax used by model metadata declarations.
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

/// Reads a named exact endpoint, returning `None` when it was not declared.
/// Invalid syntax or conversion returns a bind error naming the endpoint only.
fn optional_bound(
    reader: &mut ArgumentReader<'_>,
    args: &[NamedValidationArgument<'_>],
    name: &str,
    inclusive: bool,
) -> Result<Option<Bound<BigDecimal>>, BindError> {
    if !args.iter().any(|argument| argument.name() == name) {
        return Ok(None);
    }
    let literal = reader.required_str(name)?;
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

/// Decodes scalar arguments once and prepares the typed decimal adapter.
fn prepare_decimal(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
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
    let min = optional_bound(&mut reader, args, "min", min_inclusive)?;
    let max = optional_bound(&mut reader, args, "max", max_inclusive)?;
    reader.finish()?;
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

const DECIMAL_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::of::<BigDecimal>(), EMPTY_DEPS, prepare_decimal);
pub(super) static DESC_DECIMAL: ValidatorDescriptor = ValidatorDescriptor::new(&[DECIMAL_SIG]);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_DECIMAL: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::DECIMAL_VALUE),
    &DESC_DECIMAL,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_DECIMAL);
