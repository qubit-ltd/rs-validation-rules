// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
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

use crate::collection::ItemCount;
use crate::collection::ItemCountError;
/// Collection rules do not read any validation dependencies.
const EMPTY_DEPS: &[DependencySpec] = &[];

/// Binds at least one collection-size bound.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the count rule.
///
/// # Returns
/// A prepared count validator configured with the supplied optional bounds.
///
/// # Errors
/// Returns a bind error when both bounds are absent, an argument is invalid or
/// out of range, or an unexpected argument is supplied.
fn prepare_item_count(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let min = reader.optional_usize("min")?;
    let max = reader.optional_usize("max")?;
    let rule = ItemCount::new(min, max)?;
    reader.finish()?;
    Ok(prepare_typed_validator(rule, |error| match error {
        ItemCountError::TooSmall { min } => ViolationDraft::new(ViolationCode::new("collection.too_small"))
            .with_param("bound", ViolationParam::Unsigned(min as u128)),
        ItemCountError::TooLarge { max } => ViolationDraft::new(ViolationCode::new("collection.too_large"))
            .with_param("bound", ViolationParam::Unsigned(max as u128)),
    }))
}
/// Binding signature for the collection item-count rule.
const COUNT_SIG: ValidatorSignature = ValidatorSignature::new(InputType::of::<usize>(), EMPTY_DEPS, prepare_item_count);
/// Registry descriptor exposed for local and inventory registration.
pub(super) static DESC_COUNT: ValidatorDescriptor = ValidatorDescriptor::new(&[COUNT_SIG]);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_COUNT: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::COLLECTION_ITEM_COUNT),
    &DESC_COUNT,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_COUNT);
