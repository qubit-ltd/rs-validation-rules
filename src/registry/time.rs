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
use qubit_validator::NamedValidationArgument;
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

/// Decodes the required resolution and rejects other parameter names.
fn read_rule(args: &[NamedValidationArgument<'_>]) -> Result<TimePrecision, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let resolution = match reader.required_str("precision")? {
        "second" => TemporalResolution::Second,
        "millisecond" => TemporalResolution::Millisecond,
        "microsecond" => TemporalResolution::Microsecond,
        "nanosecond" => TemporalResolution::Nanosecond,
        _ => {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("precision"));
        }
    };
    reader.finish()?;
    Ok(TimePrecision::new(resolution))
}

/// Prepares the UTC timestamp signature.
fn prepare_utc(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator::<DateTime<Utc>, _, _>(read_rule(args)?, |_| {
        ViolationDraft::new(ViolationCode::new("time.precision"))
    }))
}

/// Prepares the date-time signature without a timezone.
fn prepare_date_time(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator::<NaiveDateTime, _, _>(read_rule(args)?, |_| {
        ViolationDraft::new(ViolationCode::new("time.precision"))
    }))
}

/// Prepares the time-of-day signature.
fn prepare_time(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    Ok(prepare_typed_validator::<NaiveTime, _, _>(read_rule(args)?, |_| {
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
