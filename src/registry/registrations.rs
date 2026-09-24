use qubit_validator::RegistrationSource;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;

use super::collection;
#[cfg(feature = "regex")]
use super::regex;
use super::text;

const SOURCE: RegistrationSource = RegistrationSource::new("qubit-validation-rules", module_path!(), file!(), line!());

/// Returns built-in rule registrations for a local validator registry.
/// Feature-gated rules are included when their features are enabled.
pub fn registrations() -> Vec<ValidatorRegistration> {
    #[allow(unused_mut)]
    let mut rules = [
        ("qubit.rules.text.non_blank", &text::DESC_NON_BLANK),
        ("qubit.rules.text.char_length", &text::DESC_CHAR_LENGTH),
        ("qubit.rules.text.byte_length", &text::DESC_BYTE_LENGTH),
        ("qubit.rules.text.allowed_chars", &text::DESC_ALLOWED),
        ("qubit.rules.text.email_ascii", &text::DESC_EMAIL),
        ("qubit.rules.text.matches_dependency", &text::DESC_MATCHES_DEPENDENCY),
        ("qubit.rules.text.china_mobile_structure", &text::DESC_MOBILE),
        ("qubit.rules.text.uri", &text::DESC_URI),
        ("qubit.rules.text.uuid", &text::DESC_UUID),
    ]
    .into_iter()
    .map(|(id, descriptor)| ValidatorRegistration::new(ValidatorId::new(id), descriptor, SOURCE))
    .chain(std::iter::once(ValidatorRegistration::new(
        ValidatorId::new("qubit.rules.collection.item_count"),
        &collection::DESC_COUNT,
        SOURCE,
    )))
    .collect::<Vec<_>>();
    #[cfg(feature = "regex")]
    rules.push(ValidatorRegistration::new(
        ValidatorId::new("qubit.rules.text.regex"),
        &regex::DESC_REGEX,
        SOURCE,
    ));
    rules
}
