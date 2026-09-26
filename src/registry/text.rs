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
use qubit_validator::BindErrorKind;
use qubit_validator::DependencySpec;
use qubit_validator::ExecutionError;
use qubit_validator::ExecutionErrorKind;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::PreparedOutcome;
use qubit_validator::PreparedValidator;
use qubit_validator::RegistrationSource;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorDescriptor;
use qubit_validator::ValidatorId;
use qubit_validator::ValidatorRegistration;
use qubit_validator::ValidatorSignature;
use qubit_validator::ViolationCode;
use qubit_validator::ViolationDraft;
use qubit_validator::ViolationParam;
use qubit_validator::prepare_text_validator;
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
use crate::text::MatchesDependencyError;
use crate::text::NonBlank;
use crate::text::TextLengthError;
use crate::text::Uri;
use crate::text::UuidText;

/// Rejects any arguments for a parameterless rule.
///
/// # Parameters
/// - `args`: Named arguments supplied to the rule.
///
/// # Returns
/// `Ok(())` when no arguments remain unread.
///
/// # Errors
/// Returns a bind error when any argument is supplied.
pub(super) fn no_args(args: &[NamedValidationArgument<'_>]) -> Result<(), BindError> {
    ArgumentReader::new(args)?.finish()
}

/// Binds a non-blank rule with no arguments.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the rule.
///
/// # Returns
/// A prepared text validator for the non-blank rule.
///
/// # Errors
/// Returns a bind error when an argument is supplied.
fn prepare_non_blank(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(prepare_text_validator(NonBlank, |_| {
        ViolationDraft::new(ViolationCode::new("text.blank"))
    }))
}

/// Binds at least one Unicode scalar-value length bound.
///
/// # Parameters
/// - `args`: Named arguments containing optional `min` and `max` bounds.
///
/// # Returns
/// A prepared text validator using Unicode scalar-value bounds.
///
/// # Errors
/// Returns a bind error when both bounds are absent, a bound is invalid, or an
/// unexpected argument is supplied.
fn prepare_char_length(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = CharLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(prepare_text_validator(rule, |error| match error {
        TextLengthError::TooShort { min } => ViolationDraft::new(ViolationCode::new("text.too_short"))
            .with_param("bound", ViolationParam::Unsigned(min.into())),
        TextLengthError::TooLong { max } => ViolationDraft::new(ViolationCode::new("text.too_long"))
            .with_param("bound", ViolationParam::Unsigned(max.into())),
    }))
}

/// Binds at least one UTF-8 byte-length bound.
///
/// # Parameters
/// - `args`: Named arguments containing optional `min` and `max` bounds.
///
/// # Returns
/// A prepared text validator using UTF-8 byte bounds.
///
/// # Errors
/// Returns a bind error when both bounds are absent, a bound is invalid, or an
/// unexpected argument is supplied.
fn prepare_byte_length(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = ByteLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(prepare_text_validator(rule, |error| match error {
        ByteLengthError::TooFewBytes { min } => ViolationDraft::new(ViolationCode::new("text.too_few_bytes"))
            .with_param("bound", ViolationParam::Unsigned(min.into())),
        ByteLengthError::TooManyBytes { max } => ViolationDraft::new(ViolationCode::new("text.too_many_bytes"))
            .with_param("bound", ViolationParam::Unsigned(max.into())),
    }))
}

/// Binds a required character-set profile.
///
/// # Parameters
/// - `args`: Named arguments containing the required `set` profile.
///
/// # Returns
/// A prepared text validator using the selected character profile.
///
/// # Errors
/// Returns a bind error when the profile name is unknown or an unexpected
/// argument is supplied.
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
    Ok(prepare_text_validator(AllowedChars::new(set), |_| {
        ViolationDraft::new(ViolationCode::new("text.disallowed_characters"))
    }))
}

/// Binds an ASCII email profile with no arguments.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the rule.
///
/// # Returns
/// A prepared text validator for the ASCII email profile.
///
/// # Errors
/// Returns a bind error when an argument is supplied.
fn prepare_email(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(prepare_text_validator(EmailAscii, |_| {
        ViolationDraft::new(ViolationCode::new("text.email"))
    }))
}

/// Binds a rule that compares its input with the declared dependency.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the rule.
///
/// # Returns
/// A prepared contextual text validator for the first declared dependency.
///
/// # Errors
/// Returns a bind error when an argument is supplied.
fn prepare_matches_dependency(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(Arc::new(MatchesDependencyAdapter))
}

/// Preserves the distinction between a domain mismatch and an invalid context.
struct MatchesDependencyAdapter;

impl PreparedValidator for MatchesDependencyAdapter {
    /// Validates text with slot zero, returning a contract error if it is
    /// unreadable.
    fn validate(
        &self,
        value: ValidationValue<'_>,
        context: &qubit_validator::BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let text = value
            .as_text()
            .ok_or_else(|| ExecutionError::new(ExecutionErrorKind::InputTypeMismatch))?;
        match MatchesDependency.validate(text, context) {
            Ok(()) => Ok(PreparedOutcome::valid()),
            Err(MatchesDependencyError::Mismatch) => Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(
                ViolationCode::new("text.dependency_mismatch"),
            )])),
            Err(MatchesDependencyError::MissingDependency) => {
                Err(ExecutionError::new(ExecutionErrorKind::AdapterContractViolation))
            }
        }
    }
}
/// Binds a mainland China mobile-number structure rule.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the rule.
///
/// # Returns
/// A prepared text validator for the mobile-number structure profile.
///
/// # Errors
/// Returns a bind error when an argument is supplied.
fn prepare_mobile(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(prepare_text_validator(ChinaMobileStructure, |_| {
        ViolationDraft::new(ViolationCode::new("text.mobile"))
    }))
}
/// Binds an absolute URI profile with no arguments.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the rule.
///
/// # Returns
/// A prepared text validator for absolute URI syntax.
///
/// # Errors
/// Returns a bind error when an argument is supplied.
fn prepare_uri(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(prepare_text_validator(Uri, |_| {
        ViolationDraft::new(ViolationCode::new("text.uri"))
    }))
}
/// Binds a canonical UUID text profile with no arguments.
///
/// # Parameters
/// - `args`: Named arguments supplied while binding the rule.
///
/// # Returns
/// A prepared text validator for canonical UUID text.
///
/// # Errors
/// Returns a bind error when an argument is supplied.
fn prepare_uuid(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(prepare_text_validator(UuidText, |_| {
        ViolationDraft::new(ViolationCode::new("text.uuid"))
    }))
}
/// Shared empty dependency list for standalone text rules.
const EMPTY_DEPS: &[DependencySpec] = &[];
/// Binding signature for the non-blank text rule.
const TEXT_SIG_NON_BLANK: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_non_blank);
/// Binding signature for the character-length rule.
const TEXT_SIG_BOUNDS: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_char_length);
/// Binding signature for the UTF-8 byte-length rule.
const BYTE_SIG_BOUNDS: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_byte_length);
/// Binding signature for the allowed-character rule.
const ALLOWED_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_allowed_chars);
/// Binding signature for the ASCII email rule.
const EMAIL_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_email);
/// Required text dependency read by the dependency comparison rule.
const MATCHES_DEPENDENCY_DEPS: &[DependencySpec] = &[DependencySpec::new("expected", InputType::Text, false)];
/// Binding signature for the dependency comparison rule.
const MATCHES_DEPENDENCY_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, MATCHES_DEPENDENCY_DEPS, prepare_matches_dependency);
/// Binding signature for the mainland China mobile-number rule.
const MOBILE_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_mobile);
/// Binding signature for the absolute URI rule.
const URI_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uri);
/// Binding signature for the canonical UUID rule.
const UUID_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uuid);
/// Registry descriptor for the non-blank text rule.
pub(super) static DESC_NON_BLANK: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_NON_BLANK]);
/// Registry descriptor for the Unicode scalar-value length rule.
pub(super) static DESC_CHAR_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_BOUNDS]);
/// Registry descriptor for the UTF-8 byte-length rule.
pub(super) static DESC_BYTE_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[BYTE_SIG_BOUNDS]);
/// Registry descriptor for the allowed-character rule.
pub(super) static DESC_ALLOWED: ValidatorDescriptor = ValidatorDescriptor::new(&[ALLOWED_SIG]);
/// Registry descriptor for the ASCII email rule.
pub(super) static DESC_EMAIL: ValidatorDescriptor = ValidatorDescriptor::new(&[EMAIL_SIG]);
/// Registry descriptor for the dependency comparison rule.
pub(super) static DESC_MATCHES_DEPENDENCY: ValidatorDescriptor = ValidatorDescriptor::new(&[MATCHES_DEPENDENCY_SIG]);
/// Registry descriptor for the mobile-number structure rule.
pub(super) static DESC_MOBILE: ValidatorDescriptor = ValidatorDescriptor::new(&[MOBILE_SIG]);
/// Registry descriptor for the absolute URI rule.
pub(super) static DESC_URI: ValidatorDescriptor = ValidatorDescriptor::new(&[URI_SIG]);
/// Registry descriptor for the canonical UUID rule.
pub(super) static DESC_UUID: ValidatorDescriptor = ValidatorDescriptor::new(&[UUID_SIG]);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_NON_BLANK: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_NON_BLANK),
    &DESC_NON_BLANK,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_CHAR_LENGTH: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_CHAR_LENGTH),
    &DESC_CHAR_LENGTH,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_BYTE_LENGTH: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_BYTE_LENGTH),
    &DESC_BYTE_LENGTH,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_ALLOWED_CHARS: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_ALLOWED_CHARS),
    &DESC_ALLOWED,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_EMAIL: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_EMAIL_ASCII),
    &DESC_EMAIL,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_MATCHES_DEPENDENCY: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_MATCHES_DEPENDENCY),
    &DESC_MATCHES_DEPENDENCY,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_MOBILE: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_CHINA_MOBILE_STRUCTURE),
    &DESC_MOBILE,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_URI: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_URI),
    &DESC_URI,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
/// Registration shared by the local and optional inventory registries.
pub(super) const REG_UUID: ValidatorRegistration = ValidatorRegistration::new(
    ValidatorId::new(crate::ids::TEXT_UUID),
    &DESC_UUID,
    RegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_NON_BLANK);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_CHAR_LENGTH);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_BYTE_LENGTH);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_ALLOWED_CHARS);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_EMAIL);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_MATCHES_DEPENDENCY);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_MOBILE);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_URI);
#[cfg(feature = "inventory")]
register_validator!(registration = REG_UUID);
