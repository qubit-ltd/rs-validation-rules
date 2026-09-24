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
use qubit_validator::ViolationParam;
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

use crate::collection::ItemCount;
use crate::collection::ItemCountError;
const EMPTY_DEPS: &[DependencySpec] = &[];

/// Binds optional collection-size bounds.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_item_count(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let min = reader.optional_usize("min")?;
    let max = reader.optional_usize("max")?;
    let rule = ItemCount::new(min, max)?;
    reader.finish()?;
    Ok(qubit_validator::prepare_typed_validator(rule, |error| match error {
        ItemCountError::TooSmall { min } => ViolationDraft::new(ViolationCode::new("collection.too_small"))
            .with_param("bound", ViolationParam::Unsigned(min as u128)),
        ItemCountError::TooLarge { max } => ViolationDraft::new(ViolationCode::new("collection.too_large"))
            .with_param("bound", ViolationParam::Unsigned(max as u128)),
    }))
}
const COUNT_SIG: ValidatorSignature = ValidatorSignature::new(InputType::of::<usize>(), EMPTY_DEPS, prepare_item_count);
pub(super) static DESC_COUNT: ValidatorDescriptor = ValidatorDescriptor::new(&[COUNT_SIG]);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.collection.item_count", descriptor = &DESC_COUNT);
