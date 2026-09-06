use qubit_validator::{
    ArgumentReader, BindError, BoundValidationContext, ExecutionError, ExecutionErrorKind,
    InputType, NamedValidationArgument, PreparedValidator, RegistrationSource, RuleOutcome,
    ValidationValue, Validator, ValidatorDescriptor, ValidatorId, ValidatorRegistration,
    ValidatorSignature, Violation, ViolationCode, ViolationParam,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NonBlankError {
    #[error("text is blank")]
    Blank,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonBlank;
impl Validator<str, ()> for NonBlank {
    type Error = NonBlankError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        if v.trim().is_empty() {
            Err(NonBlankError::Blank)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TextLengthError {
    #[error("text is too short")]
    TooShort { min: u32 },
    #[error("text is too long")]
    TooLong { max: u32 },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharLength {
    min: Option<u32>,
    max: Option<u32>,
}
impl CharLength {
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(
                qubit_validator::BindErrorKind::ParameterOutOfRange,
            ));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for CharLength {
    type Error = TextLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.chars().count() as u32;
        if self.min.is_some_and(|m| n < m) {
            return Err(TextLengthError::TooShort {
                min: self.min.unwrap(),
            });
        }
        if self.max.is_some_and(|m| n > m) {
            return Err(TextLengthError::TooLong {
                max: self.max.unwrap(),
            });
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ByteLengthError {
    #[error("text has too few bytes")]
    TooFewBytes { min: u32 },
    #[error("text has too many bytes")]
    TooManyBytes { max: u32 },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteLength {
    min: Option<u32>,
    max: Option<u32>,
}
impl ByteLength {
    pub fn new(min: Option<u32>, max: Option<u32>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(
                qubit_validator::BindErrorKind::ParameterOutOfRange,
            ));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for ByteLength {
    type Error = ByteLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.len() as u32;
        if self.min.is_some_and(|m| n < m) {
            return Err(ByteLengthError::TooFewBytes {
                min: self.min.unwrap(),
            });
        }
        if self.max.is_some_and(|m| n > m) {
            return Err(ByteLengthError::TooManyBytes {
                max: self.max.unwrap(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSet {
    Unicode,
    PrintableUnicode,
    Ascii,
    PrintableAscii,
    Code,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AllowedCharsError {
    #[error("invalid characters")]
    Invalid,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AllowedChars {
    set: CharacterSet,
}
impl AllowedChars {
    pub const fn new(set: CharacterSet) -> Self {
        Self { set }
    }
}
impl Validator<str, ()> for AllowedChars {
    type Error = AllowedCharsError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let ok = v.chars().all(|c| match self.set {
            CharacterSet::Unicode => true,
            CharacterSet::PrintableUnicode => !c.is_control(),
            CharacterSet::Ascii => c.is_ascii(),
            CharacterSet::PrintableAscii => c.is_ascii() && !c.is_ascii_control(),
            CharacterSet::Code => c.is_ascii_alphanumeric() || "._-".contains(c),
        });
        if ok {
            Ok(())
        } else {
            Err(AllowedCharsError::Invalid)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ItemCountError {
    #[error("too few items")]
    TooSmall { min: usize },
    #[error("too many items")]
    TooLarge { max: usize },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemCount {
    min: Option<usize>,
    max: Option<usize>,
}
impl ItemCount {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Result<Self, BindError> {
        if min.zip(max).is_some_and(|(a, b)| a > b) {
            return Err(BindError::new(
                qubit_validator::BindErrorKind::ParameterOutOfRange,
            ));
        }
        Ok(Self { min, max })
    }
}
impl Validator<usize, ()> for ItemCount {
    type Error = ItemCountError;
    fn validate(&self, v: &usize, _: &()) -> Result<(), Self::Error> {
        if self.min.is_some_and(|m| *v < m) {
            return Err(ItemCountError::TooSmall {
                min: self.min.unwrap(),
            });
        }
        if self.max.is_some_and(|m| *v > m) {
            return Err(ItemCountError::TooLarge {
                max: self.max.unwrap(),
            });
        }
        Ok(())
    }
}

#[cfg(feature = "china-identity")]
mod china_identity;
#[cfg(feature = "china-identity")]
pub use china_identity::{ChinaIdentity18, ChinaIdentityError, ChinaIdentityFacts};

#[derive(Clone, Copy)]
enum TextRule {
    NonBlank,
    CharLength(CharLength),
    ByteLength(ByteLength),
    AllowedChars(AllowedChars),
    Format(TextFormat),
}

#[derive(Clone, Copy)]
enum TextFormat {
    Email,
    Mobile,
    Uri,
    Uuid,
}

#[derive(Clone, Copy)]
struct TextPrepared(TextRule);

impl PreparedValidator for TextPrepared {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _context: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        let Some(value) = value.as_text() else {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch));
        };
        let result = match self.0 {
            TextRule::NonBlank => NonBlank
                .validate(value, &())
                .map_err(|error| ("blank", error.to_string(), None)),
            TextRule::CharLength(rule) => rule.validate(value, &()).map_err(|error| match error {
                TextLengthError::TooShort { min } => (
                    "too_short",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(min.into())),
                ),
                TextLengthError::TooLong { max } => (
                    "too_long",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(max.into())),
                ),
            }),
            TextRule::ByteLength(rule) => rule.validate(value, &()).map_err(|error| match error {
                ByteLengthError::TooFewBytes { min } => (
                    "too_few_bytes",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(min.into())),
                ),
                ByteLengthError::TooManyBytes { max } => (
                    "too_many_bytes",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(max.into())),
                ),
            }),
            TextRule::AllowedChars(rule) => rule
                .validate(value, &())
                .map_err(|error| ("invalid_chars", error.to_string(), None)),
            TextRule::Format(format) => {
                format_valid(format, value).then_some(()).ok_or_else(|| {
                    (
                        "invalid_format",
                        "value does not match the requested format".to_owned(),
                        None,
                    )
                })
            }
        };
        match result {
            Ok(()) => Ok(RuleOutcome::Valid),
            Err((code, _message, param)) => {
                let rule_id = ValidatorId::new(text_rule_id(self.0));
                let mut violation = Violation::new(rule_id, ViolationCode::new(code));
                if let Some(param) = param {
                    violation = violation.with_param("bound", param);
                }
                Ok(RuleOutcome::Invalid(vec![violation]))
            }
        }
    }
}

#[derive(Clone, Copy)]
struct CountPrepared(ItemCount);

impl PreparedValidator for CountPrepared {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _context: &BoundValidationContext<'_>,
    ) -> Result<RuleOutcome, ExecutionError> {
        let Some(value) = value.typed::<usize>() else {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch));
        };
        match self.0.validate(value, &()) {
            Ok(()) => Ok(RuleOutcome::Valid),
            Err(error) => {
                let (code, bound) = match error {
                    ItemCountError::TooSmall { min } => ("too_few_items", min),
                    ItemCountError::TooLarge { max } => ("too_many_items", max),
                };
                Ok(RuleOutcome::Invalid(vec![
                    Violation::new(
                        ValidatorId::new("qubit.rules.collection.item_count"),
                        ViolationCode::new(code),
                    )
                    .with_param("bound", ViolationParam::Unsigned(bound as u128)),
                ]))
            }
        }
    }
}

fn text_rule_id(rule: TextRule) -> &'static str {
    match rule {
        TextRule::NonBlank => "qubit.rules.text.non_blank",
        TextRule::CharLength(_) => "qubit.rules.text.char_length",
        TextRule::ByteLength(_) => "qubit.rules.text.byte_length",
        TextRule::AllowedChars(_) => "qubit.rules.text.allowed_chars",
        TextRule::Format(TextFormat::Email) => "qubit.rules.text.email_ascii",
        TextRule::Format(TextFormat::Mobile) => "qubit.rules.text.china_mobile_structure",
        TextRule::Format(TextFormat::Uri) => "qubit.rules.text.uri",
        TextRule::Format(TextFormat::Uuid) => "qubit.rules.text.uuid",
    }
}

fn format_valid(format: TextFormat, value: &str) -> bool {
    match format {
        TextFormat::Email => {
            let mut parts = value.split('@');
            let local = parts.next().unwrap_or_default();
            let domain = parts.next().unwrap_or_default();
            !local.is_empty()
                && !domain.is_empty()
                && parts.next().is_none()
                && !value.chars().any(char::is_whitespace)
        }
        TextFormat::Mobile => {
            value.len() == 11
                && value.starts_with('1')
                && value.chars().all(|character| character.is_ascii_digit())
        }
        TextFormat::Uri => value.contains(':') && !value.chars().any(char::is_whitespace),
        TextFormat::Uuid => {
            value.len() == 36
                && value.as_bytes().iter().enumerate().all(|(index, byte)| {
                    if [8, 13, 18, 23].contains(&index) {
                        *byte == b'-'
                    } else {
                        byte.is_ascii_hexdigit()
                    }
                })
        }
    }
}

fn no_args(args: &[NamedValidationArgument<'_>]) -> Result<(), BindError> {
    ArgumentReader::new(args)?.finish()
}

fn prepare_non_blank(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(Arc::new(TextPrepared(TextRule::NonBlank)))
}

fn prepare_char_length(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = CharLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(Arc::new(TextPrepared(TextRule::CharLength(rule))))
}

fn prepare_byte_length(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = ByteLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(Arc::new(TextPrepared(TextRule::ByteLength(rule))))
}

fn prepare_allowed_chars(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let set = match reader.required_str("set")? {
        "unicode" => CharacterSet::Unicode,
        "printable_unicode" => CharacterSet::PrintableUnicode,
        "ascii" => CharacterSet::Ascii,
        "printable_ascii" => CharacterSet::PrintableAscii,
        "code" => CharacterSet::Code,
        _ => {
            return Err(
                BindError::new(qubit_validator::BindErrorKind::ParameterOutOfRange)
                    .with_parameter("set"),
            );
        }
    };
    reader.finish()?;
    Ok(Arc::new(TextPrepared(TextRule::AllowedChars(
        AllowedChars::new(set),
    ))))
}

fn prepare_format(
    format: TextFormat,
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(Arc::new(TextPrepared(TextRule::Format(format))))
}

fn prepare_email(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Email, args)
}
fn prepare_mobile(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Mobile, args)
}
fn prepare_uri(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Uri, args)
}
fn prepare_uuid(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Uuid, args)
}

fn prepare_item_count(
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let min = reader.optional_u32("min")?.map(|value| value as usize);
    let max = reader.optional_u32("max")?.map(|value| value as usize);
    let rule = ItemCount::new(min, max)?;
    reader.finish()?;
    Ok(Arc::new(CountPrepared(rule)))
}

const EMPTY_DEPS: &[qubit_validator::DependencySpec] = &[];
const TEXT_SIG_NON_BLANK: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_non_blank);
const TEXT_SIG_BOUNDS: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_char_length);
const BYTE_SIG_BOUNDS: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_byte_length);
const ALLOWED_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_allowed_chars);
const EMAIL_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_email);
const MOBILE_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_mobile);
const URI_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uri);
const UUID_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uuid);
const COUNT_SIG: ValidatorSignature =
    ValidatorSignature::new(InputType::of::<usize>(), EMPTY_DEPS, prepare_item_count);

static DESC_NON_BLANK: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_NON_BLANK]);
static DESC_CHAR_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_BOUNDS]);
static DESC_BYTE_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[BYTE_SIG_BOUNDS]);
static DESC_ALLOWED: ValidatorDescriptor = ValidatorDescriptor::new(&[ALLOWED_SIG]);
static DESC_EMAIL: ValidatorDescriptor = ValidatorDescriptor::new(&[EMAIL_SIG]);
static DESC_MOBILE: ValidatorDescriptor = ValidatorDescriptor::new(&[MOBILE_SIG]);
static DESC_URI: ValidatorDescriptor = ValidatorDescriptor::new(&[URI_SIG]);
static DESC_UUID: ValidatorDescriptor = ValidatorDescriptor::new(&[UUID_SIG]);
static DESC_COUNT: ValidatorDescriptor = ValidatorDescriptor::new(&[COUNT_SIG]);

const SOURCE: RegistrationSource =
    RegistrationSource::new("qubit-validation-rules", module_path!(), file!(), line!());

pub fn registrations() -> Vec<ValidatorRegistration> {
    [
        ("qubit.rules.text.non_blank", &DESC_NON_BLANK),
        ("qubit.rules.text.char_length", &DESC_CHAR_LENGTH),
        ("qubit.rules.text.byte_length", &DESC_BYTE_LENGTH),
        ("qubit.rules.text.allowed_chars", &DESC_ALLOWED),
        ("qubit.rules.text.email_ascii", &DESC_EMAIL),
        ("qubit.rules.text.china_mobile_structure", &DESC_MOBILE),
        ("qubit.rules.text.uri", &DESC_URI),
        ("qubit.rules.text.uuid", &DESC_UUID),
    ]
    .into_iter()
    .map(|(id, descriptor)| ValidatorRegistration::new(ValidatorId::new(id), descriptor, SOURCE))
    .chain(std::iter::once(ValidatorRegistration::new(
        ValidatorId::new("qubit.rules.collection.item_count"),
        &DESC_COUNT,
        SOURCE,
    )))
    .collect()
}
