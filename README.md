# qubit-validation-rules

[![Rust CI](https://github.com/qubit-ltd/rs-validation-rules/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-validation-rules/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-validation-rules/coverage-badge.json)](https://qubit-ltd.github.io/rs-validation-rules/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-validation-rules.svg?color=blue)](https://crates.io/crates/qubit-validation-rules)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-validation-rules` provides typed validation rules and named registrations
for Qubit Rust services. Use it when an application needs the same rule to work
through direct Rust calls and a `qubit-validator` registry, without maintaining
two implementations or losing structured violation details.

For example, a service can validate an email and callback URI directly, then
bind that same URI rule by a stable ID when the rule is selected from
configuration. This crate supplies reusable rule behavior; the application
still owns policy decisions such as which URI schemes and destinations are
allowed.

## Installation

Add both crates to your application's `Cargo.toml` (Rust 1.94 or later):

```toml
[dependencies]
qubit-validation-rules = "0.1"
qubit-validator = "0.1"
```

Enable optional rule families or global discovery with the features described
below.

## Quick Start

Suppose a service accepts an email address and a callback URI. It can check
both values directly, then bind the same URI rule by its stable ID when the
rule comes from configuration:

```rust
use qubit_validation_rules::registrations;
use qubit_validation_rules::ids;
use qubit_validation_rules::text::EmailAscii;
use qubit_validation_rules::text::Uri;
use qubit_validator::BoundValidationContext;
use qubit_validator::InputType;
use qubit_validator::ValidationOutcome;
use qubit_validator::ValidationValue;
use qubit_validator::Validator;
use qubit_validator::ValidatorRegistry;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let email = "user@example.com";
    let callback_uri = "https://example.com/callback";
    EmailAscii.validate(email, &())?;
    Uri.validate(callback_uri, &())?;

    let registry = ValidatorRegistry::from_registrations(registrations())?;
    let rule = registry.bind(ids::TEXT_URI, InputType::Text, &[])?;
    let outcome = rule.validate(
        ValidationValue::Text(callback_uri),
        &BoundValidationContext::new(&[]),
    )?;
    assert_eq!(outcome, ValidationOutcome::Valid);
    Ok(())
}
```

The direct calls return typed errors. A bound registry rule returns a structured
violation for invalid input, including its rule ID and violation code. The
application must still decide which URI schemes and destinations it permits.

## Rules and Boundaries

| Rule | What it checks |
| --- | --- |
| `text::CharLength` | Number of Unicode scalar values (`char`), not grapheme clusters. At least one of `min` or `max` is required. Configured bounds use `u32`, while the measured count remains `usize` and is compared without narrowing or wraparound. |
| `text::ByteLength` | Number of UTF-8 bytes, which can differ from `CharLength` for the same text. At least one of `min` or `max` is required. Configured bounds use `u32`, while the measured count remains `usize` and is compared without narrowing or wraparound. |
| `text::EmailAscii` | An ASCII email shape and length profile; it does not establish mailbox existence or delivery. |
| `text::AllowedChars` | Checks the selected character profile. `PrintableUnicode` allows Unicode letters, marks, numbers, punctuation, symbols, and space separators; it rejects control, format, private-use, unassigned, line-separator, and paragraph-separator characters. |
| `text::Uri` | Generic RFC 3986 absolute URI syntax, including `mailto:` and `urn:`; it does not establish scheme suitability, host existence, or reachability. |
| `collection::ItemCount` | At least one inclusive item-count bound expressed as `usize` is required; the largest accepted bound depends on the target architecture. |
| `collection::Range<T>` | Checks ordered inclusive or exclusive endpoints for comparable values; unordered values such as `NaN` are rejected. Ordered endpoints do not guarantee that a discrete type has a value inside, for example the open integer interval `(1, 2)`. |
| `collection::UniqueItems` | Finds the first duplicate pair with `first_duplicate_with_limit(values, max_comparisons)`; the limit caps actual `PartialEq` calls. Model execution has its own comparison budget. |
| `decimal::DecimalValue` | With `decimal`, checks normalized `BigDecimal` scale, optional `DECIMAL(p,s)` total capacity, and exact inclusive or exclusive bounds. It never rounds the input. |
| `time::TimePrecision` | With `time`, checks exact second, millisecond, microsecond, or nanosecond resolution for `DateTime<Utc>`, `NaiveDateTime`, and `NaiveTime`, without rounding. |
| `identity::ChinaIdentity18Structure` | With `china-identity`, length, body digits, calendar birth date, and checksum of an 18-character mainland China identity number. It does not establish a valid or assigned region code, issuance, or the holder's identity. |

Other built-in rules cover blank text, allowed characters, text dependencies,
canonical UUID text, mainland China mobile-number structure, and optionally
regular expressions. `Range<T>` and `ChinaIdentity18Structure` are typed rules;
the latter is deliberately absent from the built-in dynamic registrations.
With `regex`, `RegexMatch` limits pattern bodies to 4,096 UTF-8 bytes, sets an
approximate compiled-program size limit of 8 MiB, and caps each lazy DFA cache
at 2 MiB. Construction and binding report oversized patterns or programs as
`ParameterOutOfRange(pattern)` and invalid syntax as `InvalidPattern`.
Applications must bound matching input length at their input boundary.

When `qubit-model-metadata` executes declarations, outer Map entry counts reuse
`ItemCount` through a generated length adapter; outer sequence `unique_items`
uses a generated element equality adapter rather than a generic registry rule.
The model plan checks adapters and concrete types at build time. Its `max_nodes`
budget covers reads and rule calls, while `max_comparisons` bounds sequence
pair checks. Standard constraints inside selectors and Map key/value traversal
remain unsupported by that backend. See the metadata [execution support
matrix](../rs-model-metadata/doc/user_guide.md#limitations-execution-support-and-explicit-refusal).

## Registration and Features

`registrations()` lists the built-in dynamic rules for a local
`ValidatorRegistry`, even with no features enabled. Choose this when each
registry should have an explicit lifetime and contents. The `inventory`
feature enables process-wide static discovery through
`ValidatorRegistry::global()` and `register_validator!` for applications that
choose global registration.

When binding `CharLength`, `ByteLength`, or `ItemCount`, supply at least one
bound. Binding without `min` and `max` returns `BindErrorKind::InvalidBounds`.

The `ids` module exposes constants such as `ids::TEXT_URI` for stable built-in
rule identifiers. Use these constants when referring to a built-in rule from
application code.

| Feature | Effect |
| --- | --- |
| Default | No optional features; typed rules and `registrations()` remain available. |
| `inventory` | Registers the built-in dynamic rules for global discovery. |
| `regex` | Adds `regex_rule::RegexMatch` and its dynamic registration. |
| `china-identity` | Adds the typed `identity::ChinaIdentity18Structure` rule; it is not dynamically registered. |
| `decimal` | Adds the typed `decimal::DecimalValue` rule and `ids::DECIMAL_VALUE` registration for `BigDecimal`. |
| `time` | Adds the typed `time::TimePrecision` rule and `ids::TIME_PRECISION` registration for three chrono temporal types. |

`text::MatchesDependency` returns `MatchesDependencyError::MissingDependency`
when dependency slot zero is missing or is not text, and `Mismatch` when two
texts differ. The registry reports a genuine mismatch as
`text.dependency_mismatch`; a missing dependency is an execution error. The
`qubit.rules.text.email_ascii` ID is unchanged; model declarations now spell
the format `email_ascii` and use `TextFormat::EmailAscii`.

## Learn More

- [User guide](doc/user_guide.md) for typed rules, registry bindings, errors,
  and feature choices.
- Generate the API documentation locally with `cargo doc --all-features --no-deps --open`.
- [中文文档](README.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-validation-rules](https://github.com/qubit-ltd/rs-validation-rules)
