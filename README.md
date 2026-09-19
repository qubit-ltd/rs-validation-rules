# qubit-validation-rules

Typed validation rules for Qubit Rust services, built on `qubit-validator`.

## Usage

Rules remain usable as ordinary typed Rust validators. Applications that need
dynamic rule selection can build an isolated registry without enabling any
feature:

```rust
use qubit_validation_rules::registrations;
use qubit_validator::ValidatorRegistry;

let registry = ValidatorRegistry::from_registrations(registrations())?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Enable the `inventory` feature when the process-wide `ValidatorRegistry::global`
registry and `register_validator!` discovery are desired. The `regex` and
`china-identity` features add their respective rule families.

The registry binding layer assigns the registered rule ID to every structured
violation. A rule implementation only supplies its violation code and safe
parameters.

## Features

- `inventory` — process-wide static registration.
- `regex` — regular-expression rules.
- `china-identity` — Mainland China identity-card rules.

## License

Apache-2.0.
