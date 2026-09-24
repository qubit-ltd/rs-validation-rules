use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
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

use crate::text::AllowedChars;
use crate::text::ByteLength;
use crate::text::ByteLengthError;
use crate::text::CharLength;
use crate::text::CharacterSet;
use crate::text::ChinaMobileStructure;
use crate::text::EmailAscii;
use crate::text::MatchesDependency;
use crate::text::NonBlank;
use crate::text::TextLengthError;
use crate::text::Uri;
use crate::text::UuidText;

/// Rejects any arguments for a parameterless rule.
/// Returns a bind error for invalid or unexpected arguments.
pub(super) fn no_args(args: &[NamedValidationArgument<'_>]) -> Result<(), BindError> {
    ArgumentReader::new(args)?.finish()
}

/// Binds a non-blank rule with no arguments.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_non_blank(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(qubit_validator::prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

/// Binds optional Unicode scalar-value length bounds.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_char_length(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = CharLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(qubit_validator::prepare_text_validator(rule, |error| match error {
        TextLengthError::TooShort { min } => ViolationDraft::new(ViolationCode::new("text.too_short"))
            .with_param("bound", ViolationParam::Unsigned(min.into())),
        TextLengthError::TooLong { max } => ViolationDraft::new(ViolationCode::new("text.too_long"))
            .with_param("bound", ViolationParam::Unsigned(max.into())),
    }))
}

/// Binds optional UTF-8 byte-length bounds.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_byte_length(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = ByteLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(qubit_validator::prepare_text_validator(rule, |error| match error {
        ByteLengthError::TooFewBytes { min } => ViolationDraft::new(ViolationCode::new("text.too_few_bytes"))
            .with_param("bound", ViolationParam::Unsigned(min.into())),
        ByteLengthError::TooManyBytes { max } => ViolationDraft::new(ViolationCode::new("text.too_many_bytes"))
            .with_param("bound", ViolationParam::Unsigned(max.into())),
    }))
}

/// Binds a required character-set profile.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_allowed_chars(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let set = match reader.required_str("set")? {
        "unicode" => CharacterSet::Unicode,
        "printable_unicode" => CharacterSet::PrintableUnicode,
        "ascii" => CharacterSet::Ascii,
        "printable_ascii" => CharacterSet::PrintableAscii,
        "code" => CharacterSet::Code,
        _ => {
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange).with_parameter("set"));
        }
    };
    reader.finish()?;
    Ok(qubit_validator::prepare_text_validator(AllowedChars::new(set), |_| {
        ViolationDraft::new(ViolationCode::new("text.disallowed_characters"))
    }))
}

/// Binds an ASCII email profile with no arguments.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_email(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(qubit_validator::prepare_text_validator(EmailAscii, |_| {
        ViolationDraft::new(ViolationCode::new("text.email"))
    }))
}

/// Binds a rule that compares its input with the declared dependency.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_matches_dependency(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(qubit_validator::prepare_contextual_text_validator(
        MatchesDependency,
        |_| ViolationDraft::new(ViolationCode::new("text.dependency_mismatch")),
    ))
}
/// Binds a mainland China mobile-number structure rule.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_mobile(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(qubit_validator::prepare_text_validator(ChinaMobileStructure, |_| {
        ViolationDraft::new(ViolationCode::new("text.mobile"))
    }))
}
/// Binds an absolute URI profile with no arguments.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_uri(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(qubit_validator::prepare_text_validator(Uri, |_| {
        ViolationDraft::new(ViolationCode::new("text.uri"))
    }))
}
/// Binds a canonical UUID text profile with no arguments.
/// Returns a bind error for invalid or unexpected arguments.
fn prepare_uuid(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(qubit_validator::prepare_text_validator(UuidText, |_| {
        ViolationDraft::new(ViolationCode::new("text.uuid"))
    }))
}
const EMPTY_DEPS: &[DependencySpec] = &[];
const TEXT_SIG_NON_BLANK: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_non_blank);
const TEXT_SIG_BOUNDS: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_char_length);
const BYTE_SIG_BOUNDS: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_byte_length);
const ALLOWED_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_allowed_chars);
const EMAIL_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_email);
const MATCHES_DEPENDENCY_DEPS: &[DependencySpec] = &[DependencySpec::new("expected", InputType::Text, false)];
const MATCHES_DEPENDENCY_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, MATCHES_DEPENDENCY_DEPS, prepare_matches_dependency);
const MOBILE_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_mobile);
const URI_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uri);
const UUID_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uuid);
pub(super) static DESC_NON_BLANK: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_NON_BLANK]);
pub(super) static DESC_CHAR_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_BOUNDS]);
pub(super) static DESC_BYTE_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[BYTE_SIG_BOUNDS]);
pub(super) static DESC_ALLOWED: ValidatorDescriptor = ValidatorDescriptor::new(&[ALLOWED_SIG]);
pub(super) static DESC_EMAIL: ValidatorDescriptor = ValidatorDescriptor::new(&[EMAIL_SIG]);
pub(super) static DESC_MATCHES_DEPENDENCY: ValidatorDescriptor = ValidatorDescriptor::new(&[MATCHES_DEPENDENCY_SIG]);
pub(super) static DESC_MOBILE: ValidatorDescriptor = ValidatorDescriptor::new(&[MOBILE_SIG]);
pub(super) static DESC_URI: ValidatorDescriptor = ValidatorDescriptor::new(&[URI_SIG]);
pub(super) static DESC_UUID: ValidatorDescriptor = ValidatorDescriptor::new(&[UUID_SIG]);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.non_blank", descriptor = &DESC_NON_BLANK);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.char_length", descriptor = &DESC_CHAR_LENGTH);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.byte_length", descriptor = &DESC_BYTE_LENGTH);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.allowed_chars", descriptor = &DESC_ALLOWED);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.email_ascii", descriptor = &DESC_EMAIL);
#[cfg(feature = "inventory")]
register_validator!(
    id = "qubit.rules.text.matches_dependency",
    descriptor = &DESC_MATCHES_DEPENDENCY
);
#[cfg(feature = "inventory")]
register_validator!(
    id = "qubit.rules.text.china_mobile_structure",
    descriptor = &DESC_MOBILE
);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.uri", descriptor = &DESC_URI);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.uuid", descriptor = &DESC_UUID);
