# User Guide

[简体中文](user_guide.zh_CN.md) | [README](../README.md)

This guide is for Rust application developers using `qubit-validation-rules`
0.1 with `qubit-validator` 0.1. It explains how to choose typed rules for
fixed application logic and registry rules when rule IDs or parameters come
from configuration.

## Purpose and Audience

Use this crate when an application needs common validation behavior without
writing separate implementations for ordinary Rust code and a configurable
validator registry. The rules check input structure and bounds. They do not
replace application policy, external lookups, or identity verification.

## Conceptual Model

There are two ways to use a rule:

| Form | How it is selected | Result |
| --- | --- | --- |
| Typed rule | Rust type such as `text::Uri` or `collection::ItemCount` | A rule-specific Rust error on failure |
| Registered rule | Stable ID such as `ids::TEXT_URI`, then `ValidatorRegistry::bind` | `ValidationOutcome`, including structured violations for invalid values |

`registrations()` returns the built-in dynamic registrations. A registry built
from this list is local to the application code that owns it. The optional
`inventory` feature supports process-wide discovery through
`ValidatorRegistry::global()`. Each dynamically registered built-in rule uses one registration
constant for both the local list and inventory discovery, keeping its ID,
descriptor, and source metadata identical across both paths.
`ids::COLLECTION_UNIQUE` is the stable ID for model-intrinsic sequence
uniqueness. It is reserved by `rs-model-metadata` and is not a dynamic
registration because that backend supplies its own budget-aware adapter.

## Scenario: Validate a Callback URI

A service receives an email address and a callback URI. It wants convenient
direct checks for fixed fields, while allowing the callback rule to be chosen
by a stable ID from configuration. A successful URI syntax check means the
value is an absolute URI; it does not mean that its scheme or destination is
safe for the service.

## Installation and Minimal Configuration

Add both crates to the application manifest. The package requires Rust 1.94 or
later.

```toml
[dependencies]
qubit-validation-rules = "0.1"
qubit-validator = "0.1"
```

## Core Workflow

Call typed rules directly when the code already knows which checks to perform.
The `Validator` trait provides `validate`:

```rust
use qubit_validation_rules::text::EmailAscii;
use qubit_validation_rules::text::Uri;
use qubit_validator::Validator;

fn validate_fixed_fields(email: &str, callback_uri: &str) -> Result<(), Box<dyn std::error::Error>> {
    EmailAscii.validate(email, &())?;
    Uri.validate(callback_uri, &())?;
    Ok(())
}
```

When the rule is selected by ID, create a registry from the crate's
registrations, then bind and run the rule:

```rust
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn validate_configured_uri(value: &str) -> Result<ValidationOutcome, Box<dyn std::error::Error>> {
    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let rule = registry.bind(ids::TEXT_URI, InputType::Text, &[])?;
    Ok(rule.validate(
        ValidationValue::Text(value),
        &BoundValidationContext::new(&[]),
    )?)
}
```

`ValidationOutcome::Valid` indicates the rule accepted the URI syntax.
`ValidationOutcome::Invalid` carries structured violation details. A binding
error, such as an unknown ID or invalid arguments, is returned as an error
before validation runs.

For parameterized rules, provide named arguments at bind time. This example
requires at least two Unicode scalar values:

```rust
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let arguments = [NamedValidationArgument::new(
        "min",
        ValidationArgument::Unsigned(2),
    )];
    let rule = registry.bind(ids::TEXT_CHAR_LENGTH, InputType::Text, &arguments)?;
    let outcome = rule.validate(
        ValidationValue::Text("éa"),
        &BoundValidationContext::new(&[]),
    )?;
    assert!(matches!(outcome, qubit_validator::ValidationOutcome::Valid));
    Ok(())
}
```

## Advanced Usage

### Choose between typed and registered rules

Prefer typed rules when application code owns the choice of rule and can handle
its Rust error type. Use registry rules when configuration or a shared model
refers to rules by ID and needs the validator's structured outcome format.
Both forms use the same underlying rule behavior.

### Select optional rule families

The default feature set includes typed rules and local `registrations()`.
Optional features add:

| Feature | Adds |
| --- | --- |
| `inventory` | Built-in rules to process-wide registry discovery |
| `regex` | `regex_rule::RegexMatch` and its dynamic registration |
| `china-identity` | Typed `identity::ChinaIdentity18Structure` |
| `decimal` | Typed `decimal::DecimalValue` and dynamic `ids::DECIMAL_VALUE` for `BigDecimal` |
| `time` | Typed `time::TimePrecision` and dynamic `ids::TIME_PRECISION` for chrono temporal values |

`Range<T>` is a typed rule only. The China identity rule is also intentionally
not included in dynamic registrations. Enable only the features the application
uses. `rs-model-metadata` builds a local registry from `registrations()` and
does not need the `inventory` feature for that path.

### Inventory integration example

Cargo features select which rules are compiled, while a crate dependency alone
does not guarantee that a rules crate with no Rust references is linked into
the final application. When rules are selected only by ID, add the anonymous
import below. If the application already has another actual Rust reference to
the rules crate, the extra import is unnecessary. Optional rules also require
their corresponding feature.

```toml
[dependencies]
qubit-validation-rules = { version = "0.1", features = ["inventory"] }
qubit-validator = "0.1"
```

```rust
use qubit_validation_rules as _;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ValidatorRegistry::try_global()?;
    assert!(registry.get("qubit.rules.text.email_ascii").is_some());
    let rule = registry.bind("qubit.rules.text.email_ascii", InputType::Text, &[])?;
    let context = BoundValidationContext::new(&[]);
    assert_eq!(
        rule.validate(ValidationValue::Text("user@example.com"), &context)?,
        ValidationOutcome::Valid,
    );
    assert!(matches!(
        rule.validate(ValidationValue::Text("invalid"), &context)?,
        ValidationOutcome::Invalid(_),
    ));
    Ok(())
}
```

If a built-in ID is missing from the global registry, confirm that the `inventory`
feature is enabled and the rules crate is linked. Also enable the feature for any
optional rule family being used.

### Decimal integration example

A payment endpoint can accept decimal text from a caller, then enforce the
application's precision before continuing. The typed rule reports a
`DecimalValueError`; the registry form provides structured violation details
that the application can return as a field error. Neither form rounds the
amount. Registry endpoint strings follow the [decimal endpoint syntax](#decimal-endpoint-string-syntax),
and configured bounds are subject to the [endpoint cost limits](#other-rule-profiles).

Add these dependencies to the application's `Cargo.toml`:

```toml
[dependencies]
qubit-validation-rules = { version = "0.1", features = ["decimal"] }
qubit-validator = "0.1"
bigdecimal = "0.4"
```

This complete program accepts `1.2300` at scale 2 because its trailing zero is
representational, while rejecting `1.234`. A valid outcome is the signal to
continue processing; on rejection, the application should report a precision
problem to the caller rather than silently changing the amount.

```rust
use bigdecimal::BigDecimal;
use qubit_validation_rules::decimal::DecimalValue;
use qubit_validation_rules::decimal::DecimalValueError;
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ViolationParam;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let amount: BigDecimal = "1.2300".parse()?;
    let too_precise: BigDecimal = "1.234".parse()?;
    let typed = DecimalValue::new(Some(3), 2, None, None)?;
    typed.validate(&amount, &())?;
    assert_eq!(typed.validate(&too_precise, &()), Err(DecimalValueError::Scale));

    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let args = [
        NamedValidationArgument::new("precision", ValidationArgument::Unsigned(3)),
        NamedValidationArgument::new("scale", ValidationArgument::Unsigned(2)),
    ];
    let rule = registry.bind(ids::DECIMAL_VALUE, InputType::of::<BigDecimal>(), &args)?;
    let context = BoundValidationContext::new(&[]);
    assert_eq!(rule.validate(ValidationValue::Typed(&amount), &context)?, ValidationOutcome::Valid);
    let ValidationOutcome::Invalid(violations) = rule.validate(ValidationValue::Typed(&too_precise), &context)? else {
        panic!("fractional precision must be rejected");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code().as_str(), "decimal.scale");
    assert_eq!(violations[0].params().get("scale"), Some(&ViolationParam::Unsigned(2)));
    Ok(())
}
```

### Time precision integration example

For a booking time that must align to milliseconds, `TimePrecision` checks the
value without changing it. After a precision error, the application decides
whether to return the input for correction or explicitly convert it under its
own policy; this rule does not round automatically.

Add these dependencies to `Cargo.toml`:

```toml
[dependencies]
qubit-validation-rules = { version = "0.1", features = ["time"] }
qubit-validator = "0.1"
chrono = { version = "0.4", default-features = false, features = ["std"] }
```

The following complete program accepts a millisecond-aligned time and rejects
one with an additional nanosecond. `Valid` permits the booking workflow to
continue; on rejection, the application chooses how to ask the caller to
correct or explicitly convert the value.

```rust
use chrono::NaiveTime;
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validation_rules::time::TemporalResolution;
use qubit_validation_rules::time::TimePrecision;
use qubit_validation_rules::time::TimePrecisionError;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;
use qubit_validator::ViolationParam;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let accepted = NaiveTime::from_hms_nano_opt(12, 0, 0, 123_000_000).unwrap();
    let rejected = NaiveTime::from_hms_nano_opt(12, 0, 0, 123_000_001).unwrap();
    let typed = TimePrecision::new(TemporalResolution::Millisecond);
    typed.validate(&accepted, &())?;
    assert_eq!(typed.validate(&rejected, &()), Err(TimePrecisionError::Precision));

    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let args = [NamedValidationArgument::new("precision", ValidationArgument::String("millisecond"))];
    let rule = registry.bind(ids::TIME_PRECISION, InputType::of::<NaiveTime>(), &args)?;
    let context = BoundValidationContext::new(&[]);
    assert_eq!(rule.validate(ValidationValue::Typed(&accepted), &context)?, ValidationOutcome::Valid);
    let ValidationOutcome::Invalid(violations) = rule.validate(ValidationValue::Typed(&rejected), &context)? else {
        panic!("sub-millisecond precision must be rejected");
    };
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].code().as_str(), "time.precision");
    assert_eq!(violations[0].params().get("precision"), Some(&ViolationParam::Token("millisecond")));
    Ok(())
}
```

### Identity facts integration example

An application may need facts encoded in an 18-character mainland China
identity number, such as its birth date. A successful parse exposes those
encoded facts only; the application remains responsible for identity checks
and comparing the date with its business rules. A structural pass does not
establish that the number was issued, that its region code is assigned, or that
it belongs to the current holder. Inputs shorter or longer than 18 characters
fail immediately with a structure error.

Add this dependency to `Cargo.toml`:

```toml
[dependencies]
qubit-validation-rules = { version = "0.1", features = ["china-identity"] }
```

This complete program reads the encoded birth date and demonstrates the
short-input error. The application can use the parsed date in its next business
step, but must perform any identity verification separately.

```rust
use qubit_validation_rules::identity::ChinaIdentity18Structure;
use qubit_validation_rules::identity::ChinaIdentityError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let facts = ChinaIdentity18Structure::parse("11010519491231002X")?;
    assert_eq!(facts.birth_date().to_string(), "1949-12-31");
    assert_eq!(ChinaIdentity18Structure::parse("short"), Err(ChinaIdentityError::InvalidLength));
    Ok(())
}
```

### Other rule profiles

- `CharLength` counts Unicode scalar values, while `ByteLength` counts UTF-8
  bytes. Neither counts grapheme clusters.
- `AllowedChars` supports `Unicode`, `PrintableUnicode`, `Ascii`,
  `PrintableAscii`, and `Code`. `PrintableUnicode` rejects control and format
  characters, including zero-width joiners.
- `EmailAscii` checks the bounded ASCII profile below. It does not confirm
  mailbox existence or delivery.
- `Uri` accepts absolute RFC 3986 URI syntax, including non-Web schemes such
  as `mailto:` and `urn:`.
- `CharLength`, `ByteLength`, and `ItemCount` require at least one configured
  bound. Binding with neither `min` nor `max` returns `InvalidBounds`.
- `ItemCount` uses `usize` bounds. `Range<T>` checks endpoint order and supports
  included, excluded, and unbounded endpoints for partially ordered values;
  endpoint ordering does not prove that a discrete type has an interior value.
  Its `Debug` output shows endpoint kinds and redacts endpoint values.
- `UniqueItems::first_duplicate_with_limit(values, max_comparisons)` compares
  elements with `PartialEq` and returns the first `(first_index, second_index)`
  pair. The budget caps actual equality calls; zero succeeds for fewer than two
  items and otherwise returns `ComparisonLimitExceeded`. `collection::UniquePairs`
  exposes the same index-pair order for consumers that own their comparison
  behavior and budgets. Worst-case work is O(n²).
- `DecimalValue::new(precision, scale, min, max)` validates `BigDecimal` using
  `DECIMAL(p,s)` capacity: its grid step is `10^(-s)` and its value domain is
  `[-(10^p-1)×10^(-s), (10^p-1)×10^(-s)]`. Thus `(3,2)` accepts `1.2300`,
  rejects `12`, and has the domain `[-9.99,9.99]`. It normalizes
  representation-only trailing zeros before checking input scale and precision;
  validation order is scale, precision, then range. Bounds can be inclusive or
  exclusive, and input is never rounded. This intentionally changes the old
  `(1,0)` behavior that accepted `1e3`. At scale 2, `(1.23,1.24]` is nonempty,
  while `(1.23,1.24)` is empty. Construction checks every original finite
  endpoint independently using `coefficient_digits + abs(s-endpoint_scale) + 1`.
  A cost above 131,072 returns `ParameterOutOfRange(min/max)` before endpoint
  ordering, precision-domain clipping, or feasibility checks; this also applies
  to one-sided bounds. The estimate does not cover allocations before literal
  parsing and does not limit the size of validation inputs. For dynamic registry
  bindings, each declared `min` or `max` string is first limited to 131,072 UTF-8
  bytes: longer literals fail with `ParameterOutOfRange(min/max)` before syntax
  scanning or `BigDecimal` parsing. This raw-string limit is separate from the
  131,072 endpoint-cost limit above and does not apply to the typed
  `DecimalValue::new` constructor. A literal within the byte limit can still fail
  syntax validation, endpoint-cost validation, bound ordering, or grid
  feasibility checks. Construction then
  rejects bounds containing no value on the declared scale grid. Its diagnostic
  order is scalar relationships, `min` cost, `max` cost, endpoint ordering, then
  mathematical feasibility. Errors do not retain endpoint values.
- `TimePrecision::new` supports `Second`, `Millisecond`, `Microsecond`, and
  `Nanosecond` for `DateTime<Utc>`, `NaiveDateTime`, and `NaiveTime`. The
  nanosecond component must divide evenly by the selected unit; no rounding or
  date adjustment occurs.
- `RegexMatch::new` and the `ids::TEXT_REGEX` registration accept a pattern body
  of at most 4,096 UTF-8 bytes. The approximate compiled-program size limit is 8 MiB.
  The lazy DFA transition cache is configured with an approximate capacity of 2 MiB.
  This is not a total memory limit for a rule or process: concurrent searches may
  use multiple caches, and other regex engine allocations are separate. Body AST
  nesting depth is limited to 250; deeper bodies return `InvalidPattern`. The internal full-match wrapper
  has a separate allowance and does not consume that body budget. AST depth is
  not a count of pattern parentheses. The rule does not impose a matching
  input length limit; applications must enforce one at their input boundary.
- The `qubit.rules.text.email_ascii` registration ID remains stable. In model
  declarations, replace `format = email` with `format = email_ascii` and
  `TextFormat::Email` with `TextFormat::EmailAscii`.

### EmailAscii profile

`EmailAscii` accepts exactly one `@`, ASCII characters only, and no whitespace.
It does not trim or normalize the input. The complete address is at most 254
bytes; the local part must be non-empty and at most 64 bytes.

The local part contains only ASCII letters, digits, and
``!#$%&'*+-/=?^_`{|}~.``. A dot cannot be first, last, or repeated. Quoted local
parts (for example, `"a"`) and comment syntax are not accepted.

The domain is a sequence of dot-separated labels. Each label is 1 to 63 bytes
and contains only ASCII letters, digits, or hyphens; a hyphen cannot be first
or last. Empty labels, a trailing dot, underscores, domain literals such as
`[127.0.0.1]`, and Unicode domain text are rejected. A single-label domain is
valid, so `a@b` is accepted. A syntactically permitted ASCII punycode label is
treated like any other ASCII label; IDNA validity is not checked.

This profile does not require a public suffix or DNS-resolvable domain and does
not check mailbox existence, delivery capability, or application-specific
domain policy. It is a bounded application profile, not a complete
implementation of every RFC email syntax form.

### Dynamic binding reference

The table lists the built-in dynamic registrations returned by [`registrations()`](https://docs.rs/qubit-validation-rules/latest/qubit_validation_rules/fn.registrations.html). “Default” means no optional crate feature is needed; every rule takes no named dependency except `TEXT_MATCHES_DEPENDENCY`.

| Rule ID | Feature | `InputType` | Named arguments | Named dependencies |
| --- | --- | --- | --- | --- |
| `TEXT_NON_BLANK`, `TEXT_EMAIL_ASCII`, `TEXT_CHINA_MOBILE_STRUCTURE`, `TEXT_URI`, `TEXT_UUID` | Default | `Text` | None | None |
| `TEXT_CHAR_LENGTH`, `TEXT_BYTE_LENGTH` | Default | `Text` | `min`, `max`: optional `Unsigned` values converted to `u32`; at least one is required and `min <= max` | None |
| `TEXT_ALLOWED_CHARS` | Default | `Text` | Required `set: String`; one of `unicode`, `printable_unicode`, `ascii`, `printable_ascii`, `code` | None |
| `TEXT_MATCHES_DEPENDENCY` | Default | `Text` | None | Required text `expected`; positional dependency slot 0 is also supported |
| `COLLECTION_ITEM_COUNT` | Default | Typed `usize` | `min`, `max`: optional `Unsigned` values converted to `usize`; at least one is required and `min <= max` | None |
| `TEXT_REGEX` | `regex` | `Text` | Required `pattern: String`, at most 4,096 UTF-8 bytes | None |
| `DECIMAL_VALUE` | `decimal` | Typed `BigDecimal` | Required `scale: Unsigned`; optional `precision: Unsigned`; both convert to `u16`. Optional `min`/`max: String` ([syntax](#decimal-endpoint-string-syntax)) and `min_inclusive`/`max_inclusive: Bool`, each inclusivity defaulting to `true` | None |
| `TIME_PRECISION` | `time` | Typed `DateTime<Utc>`, `NaiveDateTime`, or `NaiveTime` | Required `precision: String`: `second`, `millisecond`, `microsecond`, or `nanosecond` | None |

`Range<T>` and `ChinaIdentity18Structure` are typed rules without dynamic registrations. `COLLECTION_UNIQUE` is an intrinsic `rs-model-metadata` sequence rule, not a registration from this crate. Decimal parameters that pass type and range checks can still fail construction: retain the decimal syntax, endpoint-cost, input privacy, and scale-grid limits described in [Decimal rules](#other-rule-profiles).

### Decimal endpoint string syntax

Dynamic `min` and `max` strings must match the complete ASCII grammar `-?[0-9]+(\.[0-9]*)?`. A leading minus sign is optional; the integer part is required and the fractional part may be empty. Leading zeros and trailing fractional zeros are accepted. The parser does not trim whitespace. A plus sign, exponent notation, or a missing integer part is rejected.

Valid spellings include `0`, `-0`, `001.2300`, `1.`, and `-1.20`. Invalid spellings include an empty string, `+1`, `1e3`, `.5`, ` 1`, `1 `, `1.2.3`, and `１２`.

A permitted spelling can still fail scale/precision, endpoint-cost, ordering, or scale-grid checks. Each dynamic endpoint is also limited to 131,072 UTF-8 bytes before syntax scanning and parsing; this differs from the endpoint-cost limit. Syntax failures return `ParameterOutOfRange` with parameter `min` or `max` without retaining the literal. These string restrictions do not apply to typed `DecimalValue::new`, which receives already parsed `BigDecimal` values.

### Run a bound rule and inspect its outcome

The following examples are also executable doctests on [`registrations()`](https://docs.rs/qubit-validation-rules/latest/qubit_validation_rules/fn.registrations.html). The application can continue when it receives `Valid`; for `Invalid`, it can handle the structured violation code and parameters. Binding or execution errors are separate from a validation rejection.

```rust
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::NamedValidationArgument;
use qubit_validator::ValidationArgument;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let arguments = [NamedValidationArgument::new("set", ValidationArgument::String("code"))];
    let rule = registry.bind(ids::TEXT_ALLOWED_CHARS, InputType::Text, &arguments)?;
    let context = BoundValidationContext::new(&[]);
    assert_eq!(rule.validate(ValidationValue::Text("job_1"), &context)?, ValidationOutcome::Valid);
    let ValidationOutcome::Invalid(violations) = rule.validate(ValidationValue::Text("job 1"), &context)? else {
        panic!("space must be rejected by the code profile");
    };
    assert_eq!(violations[0].code().as_str(), "text.disallowed_characters");
    Ok(())
}
```

This example binds the `code` character profile: underscore passes, while a space produces `text.disallowed_characters`.

```rust
use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
use qubit_validator::InputType;
use qubit_validator::NamedValidationDependency;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let rule = registry.bind(ids::TEXT_MATCHES_DEPENDENCY, InputType::Text, &[])?;
    let dependencies = [NamedValidationDependency::new("expected", ValidationValue::Text("same"))];
    assert_eq!(rule.validate_named(ValidationValue::Text("same"), &dependencies)?, ValidationOutcome::Valid);
    let ValidationOutcome::Invalid(violations) = rule.validate_named(ValidationValue::Text("different"), &dependencies)? else {
        panic!("different text must be rejected");
    };
    assert_eq!(violations[0].code().as_str(), "text.dependency_mismatch");
    Ok(())
}
```

Here, unequal text produces `text.dependency_mismatch`. Omitting `expected` is an execution error because a required dependency is missing; it does not mean the text differs.

### Model metadata execution boundary

`qubit-model-metadata` binds the registered `ItemCount` rule to an outer
`HashMap` or `BTreeMap` entry count through a generated `map_len` adapter.
Outer `#[sequence(unique_items)]` instead uses an element `PartialEq` adapter
for a supported `Vec<T>` or array with a borrowed-slice getter; it is not a
general-purpose dynamic registration. Missing adapters and incompatible types
fail plan construction. A duplicate produces one violation at the later element
index and includes `first_index`. Its stable ID is `ids::COLLECTION_UNIQUE`,
which cannot be claimed by a custom registration; Map count violations use the
field path.

The model plan's `max_nodes` budget counts actual reads and rule invocations.
`max_comparisons` counts each uniqueness pair comparison and selector element
rule call; element reads also cost nodes. The limits are checked before work,
and exhaustion returns `TraversalLimit` with a partial report. Uniqueness can
require O(n²) comparisons. These budgets belong to model execution, not direct
typed-rule calls. Generated borrowed adapters also execute applicable selector
rules for borrowed `Vec<T>` and array elements and for `HashMap<K, V>` /
`BTreeMap<K, V>` keys and values. Selector rules cannot bind dependencies;
unknown/custom containers, owned collection getters, implicit member unwrapping,
and enum-payload selectors remain unsupported. Consult the metadata
[execution support matrix](https://github.com/qubit-ltd/rs-model-metadata/blob/main/doc/user_guide.md)
before building a plan.

## Errors and Diagnostics

Typed rules return domain-specific errors such as `TextRuleError::Uri`,
`TextLengthError::TooShort`, or `RangeError::Unordered`. Constructors with
configurable bounds return `qubit_validator::BindError` when bounds are invalid.

Direct `MatchesDependency` calls return `MatchesDependencyError::MissingDependency`
when slot zero is absent or not text, and `Mismatch` when its text differs from
the input. The registered rule emits `text.dependency_mismatch` only for unequal
texts; a missing dependency is an execution error. Neither error prints either
input value. Decimal and time rules report `decimal.scale`,
`decimal.precision`, `decimal.range`, or `time.precision` without the input value.
Decimal binding requires `scale`, accepts optional `precision`, canonical string
`min`/`max`, and optional `min_inclusive`/`max_inclusive` (both default true).
Time binding requires `precision = second|millisecond|microsecond|nanosecond`.
A `time.precision` violation includes a `precision` parameter whose static
token is the configured value (`second`, `millisecond`, `microsecond`, or
`nanosecond`). Unknown or out-of-range arguments fail at binding.
An oversized regex pattern or compiled program returns
`BindErrorKind::ParameterOutOfRange` with parameter `pattern` during construction
or binding. Invalid regex syntax returns `BindErrorKind::InvalidPattern`.

Registry binding checks the ID, input type, arguments, dependencies, and
feature availability before a bound rule can execute. Length and item-count
rules reject two missing bounds with `BindErrorKind::InvalidBounds`; bounds
with `min` greater than `max` return `ParameterOutOfRange`. During execution, a rejected
value is represented by `ValidationOutcome::Invalid`; it is distinct from a
binding error. Applications should use violation codes and parameters for
programmatic handling rather than parsing display strings.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Registry binding returns an error | Confirm the ID, `InputType`, argument names and types, and that the required feature is enabled. |
| A built-in ID is missing from the global registry | Confirm the `inventory` feature, ensure `qubit-validation-rules` is linked with `use qubit_validation_rules as _;` when no other Rust reference exists, and enable any optional rule feature. |
| Text rejected by `CharLength` | Check scalar-value count; a visible character may contain multiple scalars. |
| Text rejected by `ByteLength` | Check UTF-8 byte count; non-ASCII text commonly uses multiple bytes per scalar. |
| URI accepted but unsuitable for a callback | Apply application-level scheme, host, and destination policy after syntax validation. |
| `PrintableUnicode` rejects text that looks visible | Inspect for format characters such as zero-width joiners. |

## Limitations and Best Practices

- These rules validate syntax, structure, or bounds only. They do not perform
  DNS, network, mailbox, issuance, or identity checks.
- URI acceptance is not an authorization or network safety decision. Restrict
  schemes and destinations in the application before using a URI for callbacks.
- Character length is measured in Unicode scalar values, not user-perceived
  grapheme clusters. Use byte length when the external contract is defined in
  bytes.
- `Range<T>` uses partial ordering. Values that cannot be ordered, including
  `NaN`, are rejected. Constructor checks endpoint ordering but cannot know
  whether every generic `T` domain contains a value between them; `(1_i32, 2_i32)`
  is an ordered open interval with no integer member.
- `ItemCount` bounds use the platform's `usize`; architecture can affect the
  largest representable count.

## Further Reading

- [README](../README.md) and [简体中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-validation-rules)
- [qubit-validator README](https://github.com/qubit-ltd/rs-validator/blob/main/README.md) for registry and
  validation model details
