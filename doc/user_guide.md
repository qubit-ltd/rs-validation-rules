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
`ValidatorRegistry::global()`.

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
# Ok::<(), Box<dyn std::error::Error>>(())
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

`Range<T>` is a typed rule only. The China identity rule is also intentionally
not included in dynamic registrations. Enable only the features the application
uses.

### Other rule profiles

- `CharLength` counts Unicode scalar values, while `ByteLength` counts UTF-8
  bytes. Neither counts grapheme clusters.
- `AllowedChars` supports `Unicode`, `PrintableUnicode`, `Ascii`,
  `PrintableAscii`, and `Code`. `PrintableUnicode` rejects control and format
  characters, including zero-width joiners.
- `EmailAscii` checks an ASCII email shape and length profile. It does not
  confirm mailbox existence or delivery.
- `Uri` accepts absolute RFC 3986 URI syntax, including non-Web schemes such
  as `mailto:` and `urn:`.
- `ItemCount` uses `usize` bounds. `Range<T>` supports included, excluded, and
  unbounded endpoints for partially ordered values.

## Errors and Diagnostics

Typed rules return domain-specific errors such as `TextRuleError::Uri`,
`TextLengthError::TooShort`, or `RangeError::Unordered`. Constructors with
configurable bounds return `qubit_validator::BindError` when bounds are invalid.

Registry binding checks the ID, input type, arguments, dependencies, and
feature availability before a bound rule can execute. For example, bounds
with `min` greater than `max` cannot be bound. During execution, a rejected
value is represented by `ValidationOutcome::Invalid`; it is distinct from a
binding error. Applications should use violation codes and parameters for
programmatic handling rather than parsing display strings.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Registry binding returns an error | Confirm the ID, `InputType`, argument names and types, and that the required feature is enabled. |
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
  `NaN`, are rejected.
- `ItemCount` bounds use the platform's `usize`; architecture can affect the
  largest representable count.

## Further Reading

- [README](../README.md) and [简体中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-validation-rules)
- [qubit-validator README](../../rs-validator/README.md) for registry and
  validation model details
