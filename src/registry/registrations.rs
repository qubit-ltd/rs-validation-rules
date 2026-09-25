use qubit_validator::RegistrationSource;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;

use super::collection;
#[cfg(feature = "regex")]
use super::regex;
use super::text;
use crate::ids;

const SOURCE: RegistrationSource = RegistrationSource::new("qubit-validation-rules", module_path!(), file!(), line!());

/// Returns built-in rule registrations for a local validator registry.
/// Feature-gated rules are included when their features are enabled.
pub fn registrations() -> Vec<ValidatorRegistration> {
    #[allow(unused_mut)]
    let mut rules = [
        (ids::TEXT_NON_BLANK, &text::DESC_NON_BLANK),
        (ids::TEXT_CHAR_LENGTH, &text::DESC_CHAR_LENGTH),
        (ids::TEXT_BYTE_LENGTH, &text::DESC_BYTE_LENGTH),
        (ids::TEXT_ALLOWED_CHARS, &text::DESC_ALLOWED),
        (ids::TEXT_EMAIL_ASCII, &text::DESC_EMAIL),
        (ids::TEXT_MATCHES_DEPENDENCY, &text::DESC_MATCHES_DEPENDENCY),
        (ids::TEXT_CHINA_MOBILE_STRUCTURE, &text::DESC_MOBILE),
        (ids::TEXT_URI, &text::DESC_URI),
        (ids::TEXT_UUID, &text::DESC_UUID),
    ]
    .into_iter()
    .map(|(id, descriptor)| ValidatorRegistration::new(ValidatorId::new(id), descriptor, SOURCE))
    .chain(std::iter::once(ValidatorRegistration::new(
        ValidatorId::new(ids::COLLECTION_ITEM_COUNT),
        &collection::DESC_COUNT,
        SOURCE,
    )))
    .collect::<Vec<_>>();
    #[cfg(feature = "regex")]
    rules.push(ValidatorRegistration::new(
        ValidatorId::new(ids::TEXT_REGEX),
        &regex::DESC_REGEX,
        SOURCE,
    ));
    rules
}
