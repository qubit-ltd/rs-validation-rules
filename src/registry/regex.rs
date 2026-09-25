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
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::prepare_text_validator;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

use crate::regex_rule::RegexMatch;
/// Regex rules do not read validation dependencies.
const EMPTY_DEPS: &[DependencySpec] = &[];

/// Compiles the required full-string regex pattern.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the regex rule.
///
/// # Returns
/// A prepared full-string regex validator.
///
/// # Errors
/// Returns a bind error for a missing, invalid, or unexpected argument.
#[cfg(feature = "regex")]
fn prepare_regex(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let pattern = reader.required_str("pattern")?;
    reader.finish()?;
    Ok(prepare_text_validator(RegexMatch::new(pattern)?, |_| {
        ViolationDraft::new(ViolationCode::new("text.pattern"))
    }))
}
/// Binding signature for the regular-expression rule.
#[cfg(feature = "regex")]
const REGEX_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_regex);
/// Registry descriptor exposed for local and inventory registration.
#[cfg(feature = "regex")]
pub(super) static DESC_REGEX: ValidatorDescriptor = ValidatorDescriptor::new(&[REGEX_SIG]);
#[cfg(all(feature = "inventory", feature = "regex"))]
register_validator!(id = "qubit.rules.text.regex", descriptor = &DESC_REGEX);
