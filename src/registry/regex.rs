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
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

use crate::regex_rule::RegexMatch;
const EMPTY_DEPS: &[DependencySpec] = &[];

#[cfg(feature = "regex")]
/// Compiles the required full-string regex pattern.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_regex(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let pattern = reader.required_str("pattern")?;
    reader.finish()?;
    Ok(qubit_validator::prepare_text_validator(
        RegexMatch::new(pattern)?,
        |_| ViolationDraft::new(ViolationCode::new("text.pattern")),
    ))
}
#[cfg(feature = "regex")]
const REGEX_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_regex);
#[cfg(feature = "regex")]
pub(super) static DESC_REGEX: ValidatorDescriptor = ValidatorDescriptor::new(&[REGEX_SIG]);
#[cfg(all(feature = "inventory", feature = "regex"))]
register_validator!(id = "qubit.rules.text.regex", descriptor = &DESC_REGEX);
