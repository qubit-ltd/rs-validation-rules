use std::sync::Arc;

use qubit_validator::ArgumentReader;
use qubit_validator::BindError;
use qubit_validator::BindErrorKind;
use qubit_validator::BoundValidationContext;
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
#[cfg(feature = "inventory")]
use qubit_validator::register_validator;

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
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for CharLength {
    type Error = TextLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.chars().count() as u32;
        if self.min.is_some_and(|m| n < m) {
            return Err(TextLengthError::TooShort { min: self.min.unwrap() });
        }
        if self.max.is_some_and(|m| n > m) {
            return Err(TextLengthError::TooLong { max: self.max.unwrap() });
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
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<str, ()> for ByteLength {
    type Error = ByteLengthError;
    fn validate(&self, v: &str, _: &()) -> Result<(), Self::Error> {
        let n = v.len() as u32;
        if self.min.is_some_and(|m| n < m) {
            return Err(ByteLengthError::TooFewBytes { min: self.min.unwrap() });
        }
        if self.max.is_some_and(|m| n > m) {
            return Err(ByteLengthError::TooManyBytes { max: self.max.unwrap() });
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
        if ok { Ok(()) } else { Err(AllowedCharsError::Invalid) }
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
            return Err(BindError::new(BindErrorKind::ParameterOutOfRange));
        }
        Ok(Self { min, max })
    }
}
impl Validator<usize, ()> for ItemCount {
    type Error = ItemCountError;
    fn validate(&self, v: &usize, _: &()) -> Result<(), Self::Error> {
        if self.min.is_some_and(|m| *v < m) {
            return Err(ItemCountError::TooSmall { min: self.min.unwrap() });
        }
        if self.max.is_some_and(|m| *v > m) {
            return Err(ItemCountError::TooLarge { max: self.max.unwrap() });
        }
        Ok(())
    }
}

/// Errors produced by text format and character rules.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TextRuleError {
    #[error("text is blank")]
    Blank,
    #[error("text contains disallowed characters")]
    DisallowedCharacters,
    #[error("text does not match the pattern")]
    Pattern,
    #[error("text is not a valid email address")]
    Email,
    #[error("text is not a valid URI")]
    Uri,
    #[error("text is not a valid UUID")]
    Uuid,
    #[error("text is not a valid mobile number")]
    Mobile,
}

/// ASCII email profile used by the standard rules.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailAscii;

impl Validator<str, ()> for EmailAscii {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let mut parts = value.split('@');
        let local = parts.next().ok_or(TextRuleError::Email)?;
        let domain = parts.next().ok_or(TextRuleError::Email)?;
        if parts.next().is_some()
            || value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !value.is_ascii()
            || value.chars().any(char::is_whitespace)
            || local.starts_with('.')
            || local.ends_with('.')
            || local.contains("..")
            || !local
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "!#$%&'*+-/=?^_`{|}~.".contains(c))
            || !domain.split('.').all(|label| {
                !label.is_empty()
                    && label.len() <= 63
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
        {
            return Err(TextRuleError::Email);
        }
        Ok(())
    }
}

/// Absolute URI profile used by the standard rules.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Uri;

impl Validator<str, ()> for Uri {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        let Some((scheme, rest)) = value.split_once(':') else {
            return Err(TextRuleError::Uri);
        };
        if scheme.is_empty()
            || !scheme.chars().enumerate().all(|(i, c)| {
                if i == 0 {
                    c.is_ascii_alphabetic()
                } else {
                    c.is_ascii_alphanumeric() || "+-.".contains(c)
                }
            })
            || rest.is_empty()
            || value.chars().any(char::is_whitespace)
        {
            return Err(TextRuleError::Uri);
        }
        Ok(())
    }
}

/// Canonical 8-4-4-4-12 UUID text profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UuidText;

impl Validator<str, ()> for UuidText {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.len() == 36
            && value.as_bytes().iter().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    *b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
        {
            Ok(())
        } else {
            Err(TextRuleError::Uuid)
        }
    }
}

/// Mainland China mobile number structural profile.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChinaMobileStructure;

impl Validator<str, ()> for ChinaMobileStructure {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        if value.len() == 11
            && value.starts_with('1')
            && value.as_bytes().get(1).is_some_and(|b| (b'3'..=b'9').contains(b))
            && value.chars().all(|c| c.is_ascii_digit())
        {
            Ok(())
        } else {
            Err(TextRuleError::Mobile)
        }
    }
}

/// A compiled full-string regular expression rule.
#[cfg(feature = "regex")]
pub struct RegexMatch(regex::Regex);

#[cfg(feature = "regex")]
impl RegexMatch {
    pub fn new(pattern: &str) -> Result<Self, BindError> {
        regex::Regex::new(&format!(r"\A(?:{pattern})\z"))
            .map(Self)
            .map_err(|_| BindError::new(BindErrorKind::InvalidPattern))
    }
}

#[cfg(feature = "regex")]
impl Validator<str, ()> for RegexMatch {
    type Error = TextRuleError;
    fn validate(&self, value: &str, _: &()) -> Result<(), Self::Error> {
        self.0.is_match(value).then_some(()).ok_or(TextRuleError::Pattern)
    }
}

#[cfg(feature = "regex")]
struct RegexPrepared(RegexMatch);

#[cfg(feature = "regex")]
impl PreparedValidator for RegexPrepared {
    fn validate(
        &self,
        value: ValidationValue<'_>,
        _context: &BoundValidationContext<'_>,
    ) -> Result<PreparedOutcome, ExecutionError> {
        let Some(value) = value.as_text() else {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch));
        };
        if self.0.validate(value, &()).is_ok() {
            Ok(PreparedOutcome::Valid)
        } else {
            Ok(PreparedOutcome::Invalid(vec![ViolationDraft::new(ViolationCode::new(
                "text.pattern",
            ))]))
        }
    }
}

#[cfg(feature = "regex")]
fn prepare_regex(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let pattern = reader.required_str("pattern")?;
    reader.finish()?;
    Ok(Arc::new(RegexPrepared(RegexMatch::new(pattern)?)))
}

/// A comparable inclusive/exclusive range rule.
#[derive(Clone, Debug)]
pub struct Range<T> {
    lower: std::ops::Bound<T>,
    upper: std::ops::Bound<T>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum RangeError {
    #[error("value is outside the range")]
    OutOfRange,
    #[error("value cannot be ordered")]
    Unordered,
}

impl<T: PartialOrd> Range<T> {
    pub fn new(lower: std::ops::Bound<T>, upper: std::ops::Bound<T>) -> Result<Self, BindError> {
        use std::ops::Bound::Excluded;
        use std::ops::Bound::Included;
        use std::ops::Bound::Unbounded;
        let invalid = match (&lower, &upper) {
            (Unbounded, _) | (_, Unbounded) => false,
            (Included(a), Included(b))
            | (Included(a), Excluded(b))
            | (Excluded(a), Included(b))
            | (Excluded(a), Excluded(b)) => match a.partial_cmp(b) {
                Some(std::cmp::Ordering::Greater) => true,
                Some(std::cmp::Ordering::Equal) => {
                    matches!((&lower, &upper), (Excluded(_), _) | (_, Excluded(_)))
                }
                None => true,
                _ => false,
            },
        };
        if invalid {
            return Err(BindError::new(BindErrorKind::InvalidBounds));
        }
        Ok(Self { lower, upper })
    }
}

impl<T: PartialOrd> Validator<T, ()> for Range<T> {
    type Error = RangeError;
    fn validate(&self, value: &T, _: &()) -> Result<(), Self::Error> {
        use std::ops::Bound::Excluded;
        use std::ops::Bound::Included;
        use std::ops::Bound::Unbounded;
        let lower_ok = match &self.lower {
            Unbounded => true,
            Included(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_ge(),
            Excluded(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_gt(),
        };
        let upper_ok = match &self.upper {
            Unbounded => true,
            Included(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_le(),
            Excluded(bound) => value.partial_cmp(bound).ok_or(RangeError::Unordered)?.is_lt(),
        };
        if lower_ok && upper_ok {
            Ok(())
        } else {
            Err(RangeError::OutOfRange)
        }
    }
}

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
    ) -> Result<PreparedOutcome, ExecutionError> {
        let Some(value) = value.as_text() else {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch));
        };
        let result = match self.0 {
            TextRule::NonBlank => NonBlank
                .validate(value, &())
                .map_err(|error| ("text.blank", error.to_string(), None)),
            TextRule::CharLength(rule) => rule.validate(value, &()).map_err(|error| match error {
                TextLengthError::TooShort { min } => (
                    "text.too_short",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(min.into())),
                ),
                TextLengthError::TooLong { max } => (
                    "text.too_long",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(max.into())),
                ),
            }),
            TextRule::ByteLength(rule) => rule.validate(value, &()).map_err(|error| match error {
                ByteLengthError::TooFewBytes { min } => (
                    "text.too_few_bytes",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(min.into())),
                ),
                ByteLengthError::TooManyBytes { max } => (
                    "text.too_many_bytes",
                    error.to_string(),
                    Some(ViolationParam::Unsigned(max.into())),
                ),
            }),
            TextRule::AllowedChars(rule) => rule
                .validate(value, &())
                .map_err(|error| ("text.disallowed_characters", error.to_string(), None)),
            TextRule::Format(format) => format_valid(format, value).then_some(()).ok_or_else(|| {
                (
                    format_code(format),
                    "value does not match the requested format".to_owned(),
                    None,
                )
            }),
        };
        match result {
            Ok(()) => Ok(PreparedOutcome::Valid),
            Err((code, _message, param)) => {
                let mut violation = ViolationDraft::new(ViolationCode::new(code));
                if let Some(param) = param {
                    violation = violation.with_param("bound", param);
                }
                Ok(PreparedOutcome::Invalid(vec![violation]))
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
    ) -> Result<PreparedOutcome, ExecutionError> {
        let Some(value) = value.typed::<usize>() else {
            return Err(ExecutionError::new(ExecutionErrorKind::InputTypeMismatch));
        };
        match self.0.validate(value, &()) {
            Ok(()) => Ok(PreparedOutcome::Valid),
            Err(error) => {
                let (code, bound) = match error {
                    ItemCountError::TooSmall { min } => ("collection.too_small", min),
                    ItemCountError::TooLarge { max } => ("collection.too_large", max),
                };
                Ok(PreparedOutcome::Invalid(vec![
                    ViolationDraft::new(ViolationCode::new(code))
                        .with_param("bound", ViolationParam::Unsigned(bound as u128)),
                ]))
            }
        }
    }
}

fn format_code(format: TextFormat) -> &'static str {
    match format {
        TextFormat::Email => "text.email",
        TextFormat::Mobile => "text.mobile",
        TextFormat::Uri => "text.uri",
        TextFormat::Uuid => "text.uuid",
    }
}

fn format_valid(format: TextFormat, value: &str) -> bool {
    match format {
        TextFormat::Email => {
            let mut parts = value.split('@');
            let local = parts.next().unwrap_or_default();
            let domain = parts.next().unwrap_or_default();
            !local.is_empty() && !domain.is_empty() && parts.next().is_none() && !value.chars().any(char::is_whitespace)
        }
        TextFormat::Mobile => {
            value.len() == 11 && value.starts_with('1') && value.chars().all(|character| character.is_ascii_digit())
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

fn prepare_non_blank(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(Arc::new(TextPrepared(TextRule::NonBlank)))
}

fn prepare_char_length(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = CharLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(Arc::new(TextPrepared(TextRule::CharLength(rule))))
}

fn prepare_byte_length(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let rule = ByteLength::new(reader.optional_u32("min")?, reader.optional_u32("max")?)?;
    reader.finish()?;
    Ok(Arc::new(TextPrepared(TextRule::ByteLength(rule))))
}

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
    Ok(Arc::new(TextPrepared(TextRule::AllowedChars(AllowedChars::new(set)))))
}

fn prepare_format(
    format: TextFormat,
    args: &[NamedValidationArgument<'_>],
) -> Result<Arc<dyn PreparedValidator>, BindError> {
    no_args(args)?;
    Ok(Arc::new(TextPrepared(TextRule::Format(format))))
}

fn prepare_email(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Email, args)
}
fn prepare_mobile(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Mobile, args)
}
fn prepare_uri(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Uri, args)
}
fn prepare_uuid(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    prepare_format(TextFormat::Uuid, args)
}

fn prepare_item_count(args: &[NamedValidationArgument<'_>]) -> Result<Arc<dyn PreparedValidator>, BindError> {
    let mut reader = ArgumentReader::new(args)?;
    let min = reader.optional_u32("min")?.map(|value| value as usize);
    let max = reader.optional_u32("max")?.map(|value| value as usize);
    let rule = ItemCount::new(min, max)?;
    reader.finish()?;
    Ok(Arc::new(CountPrepared(rule)))
}

const EMPTY_DEPS: &[DependencySpec] = &[];
const TEXT_SIG_NON_BLANK: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_non_blank);
const TEXT_SIG_BOUNDS: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_char_length);
const BYTE_SIG_BOUNDS: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_byte_length);
const ALLOWED_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_allowed_chars);
const EMAIL_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_email);
const MOBILE_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_mobile);
const URI_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uri);
const UUID_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_uuid);
const COUNT_SIG: ValidatorSignature = ValidatorSignature::new(InputType::of::<usize>(), EMPTY_DEPS, prepare_item_count);
#[cfg(feature = "regex")]
const REGEX_SIG: ValidatorSignature = ValidatorSignature::new(InputType::Text, EMPTY_DEPS, prepare_regex);

static DESC_NON_BLANK: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_NON_BLANK]);
static DESC_CHAR_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[TEXT_SIG_BOUNDS]);
static DESC_BYTE_LENGTH: ValidatorDescriptor = ValidatorDescriptor::new(&[BYTE_SIG_BOUNDS]);
static DESC_ALLOWED: ValidatorDescriptor = ValidatorDescriptor::new(&[ALLOWED_SIG]);
static DESC_EMAIL: ValidatorDescriptor = ValidatorDescriptor::new(&[EMAIL_SIG]);
static DESC_MOBILE: ValidatorDescriptor = ValidatorDescriptor::new(&[MOBILE_SIG]);
static DESC_URI: ValidatorDescriptor = ValidatorDescriptor::new(&[URI_SIG]);
static DESC_UUID: ValidatorDescriptor = ValidatorDescriptor::new(&[UUID_SIG]);
static DESC_COUNT: ValidatorDescriptor = ValidatorDescriptor::new(&[COUNT_SIG]);
#[cfg(feature = "regex")]
static DESC_REGEX: ValidatorDescriptor = ValidatorDescriptor::new(&[REGEX_SIG]);

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
    id = "qubit.rules.text.china_mobile_structure",
    descriptor = &DESC_MOBILE
);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.uri", descriptor = &DESC_URI);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.text.uuid", descriptor = &DESC_UUID);
#[cfg(feature = "inventory")]
register_validator!(id = "qubit.rules.collection.item_count", descriptor = &DESC_COUNT);
#[cfg(all(feature = "inventory", feature = "regex"))]
register_validator!(id = "qubit.rules.text.regex", descriptor = &DESC_REGEX);

const SOURCE: RegistrationSource = RegistrationSource::new("qubit-validation-rules", module_path!(), file!(), line!());

pub fn registrations() -> Vec<ValidatorRegistration> {
    #[allow(unused_mut)]
    let mut rules = [
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
    .collect::<Vec<_>>();
    #[cfg(feature = "regex")]
    rules.push(ValidatorRegistration::new(
        ValidatorId::new("qubit.rules.text.regex"),
        &DESC_REGEX,
        SOURCE,
    ));
    rules
}

#[cfg(test)]
mod tests {
    use qubit_validator::BoundValidationContext;
    use qubit_validator::ValidationArgument;
    use qubit_validator::ValidationOutcome;
    use qubit_validator::ValidatorRegistry;

    use super::InputType;
    use super::NamedValidationArgument;
    use super::ValidationValue;
    use super::registrations;
    #[cfg(feature = "china-identity")]
    use crate::identity::ChinaIdentity18;
    #[cfg(feature = "china-identity")]
    use crate::identity::ChinaIdentityError;

    #[test]
    fn standard_rules_bind_and_report_structured_violations() {
        let registry = ValidatorRegistry::from_registrations(registrations()).expect("valid rules");
        let arguments = [NamedValidationArgument::new("min", ValidationArgument::Unsigned(3))];
        let bound = registry
            .bind("qubit.rules.text.char_length", InputType::Text, &arguments)
            .expect("length rule binds");
        let outcome = bound
            .validate(ValidationValue::Text("hi"), &BoundValidationContext::new(&[]))
            .expect("length rule executes");
        assert!(
            matches!(outcome, ValidationOutcome::Invalid(violations) if violations[0].code().as_str() == "text.too_short" && violations[0].rule_id().as_str() == "qubit.rules.text.char_length")
        );
    }

    #[cfg(feature = "inventory")]
    #[test]
    fn inventory_registry_contains_standard_rules() {
        let registry = ValidatorRegistry::global();
        assert!(registry.get("qubit.rules.text.non_blank").is_some());
    }

    #[cfg(feature = "china-identity")]
    #[test]
    fn china_identity_parser_rejects_invalid_checksum() {
        assert_eq!(
            ChinaIdentity18::parse("110105194912310020"),
            Err(ChinaIdentityError::InvalidChecksum)
        );
    }
}
