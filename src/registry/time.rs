// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::NaiveTime;
use chrono::Utc;
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
use qubit_validator::prepare_typed_validator;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

use crate::time::TemporalResolution;
use crate::time::TimePrecision;

const EMPTY_DEPS: &[DependencySpec] = &[];

/// Decodes the required temporal resolution from the shared argument reader.
///
/// # Parameters
/// - `reader`: Reader containing the required `precision` parameter.
///
/// # Returns
/// The precision mapped from `second`, `millisecond`, `microsecond`, or
/// `nanosecond`.
///
/// # Errors
/// Returns a bind error when `precision` is missing or has an unsupported
/// value. The binding flow reports duplicate and unknown parameters.
fn read_rule(reader: &mut ArgumentReader<'_>) -> Result<TimePrecision, BindError> {
    let resolution = match reader.required_str("precision")? {
        "second" => TemporalResolution::Second,
        "millisecond" => TemporalResolution::Millisecond,
        "microsecond" => TemporalResolution::Microsecond,
        "nanosecond" => TemporalResolution::Nanosecond,
        _ => {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("precision"));
        }
    };
    Ok(TimePrecision::new(resolution))
}

/// Prepares the validator for UTC timestamps.
///
/// # Parameters
/// - `reader`: Shared reader containing the required `precision` parameter.
///
/// # Returns
/// A prepared validator that checks UTC timestamps at the requested precision.
///
/// # Errors
/// Returns a bind error when the precision is missing or unsupported. The
/// binding flow reports duplicate and unknown parameters.
fn prepare_utc(reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator::<DateTime<Utc>, _, _>(
        read_rule(reader)?,
        |_| ViolationDraft::new(ViolationCode::new("time.precision")),
    ))
}

/// Prepares the validator for date-times without a timezone.
///
/// # Parameters
/// - `reader`: Shared reader containing the required `precision` parameter.
///
/// # Returns
/// A prepared validator that checks naive date-times at the requested
/// precision.
///
/// # Errors
/// Returns a bind error when the precision is missing or unsupported. The
/// binding flow reports duplicate and unknown parameters.
fn prepare_date_time(reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator::<NaiveDateTime, _, _>(
        read_rule(reader)?,
        |_| ViolationDraft::new(ViolationCode::new("time.precision")),
    ))
}

/// Prepares the validator for times of day without a timezone.
///
/// # Parameters
/// - `reader`: Shared reader containing the required `precision` parameter.
///
/// # Returns
/// A prepared validator that checks naive times at the requested precision.
///
/// # Errors
/// Returns a bind error when the precision is missing or unsupported. The
/// binding flow reports duplicate and unknown parameters.
fn prepare_time(reader: &mut ArgumentReader<'_>) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator::<NaiveTime, _, _>(read_rule(reader)?, |_| {
        ViolationDraft::new(ViolationCode::new("time.precision"))
    }))
}

const UTC_SIG: ValidatorSignature = ValidatorSignature::new(InputType::of::<DateTime<Utc>>(), EMPTY_DEPS, prepare_utc);
const DATE_TIME_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::of::<NaiveDateTime>(), EMPTY_DEPS, prepare_date_time);
const TIME_SIG: ValidatorSignature = ValidatorSignature::new(InputType::of::<NaiveTime>(), EMPTY_DEPS, prepare_time);
pub(super) static DESC_TIME: ValidatorDescriptor = ValidatorDescriptor::new(&[UTC_SIG, DATE_TIME_SIG, TIME_SIG]);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_TIME: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TIME_PRECISION),
    &DESC_TIME,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_TIME);
