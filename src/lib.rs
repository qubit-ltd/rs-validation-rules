// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed validation rules and optional validator-registry adapters.
//!
//! Use typed rules when application code owns the rule choice. Use
//! [`registrations()`] with a local registry when IDs and parameters come from
//! configuration. Both paths share the underlying validation behavior.
//!
//! # Examples
//!
//! ```
//! use qubit_validation_rules::ids;
//! use qubit_validation_rules::registrations;
//! use qubit_validation_rules::text::EmailAscii;
//! use qubit_validator::BoundValidationContext;
//! use qubit_validator::InputType;
//! use qubit_validator::ValidationOutcome;
//! use qubit_validator::ValidationValue;
//! use qubit_validator::Validator;
//! use qubit_validator::ValidatorRegistry;
//!
//! EmailAscii.validate("user@example.com", &())?;
//! let registry = ValidatorRegistry::from_registrations(registrations())?;
//! let rule = registry.bind(ids::TEXT_URI, InputType::Text, &[])?;
//! let outcome = rule.validate(
//!     ValidationValue::Text("https://example.com/callback"),
//!     &BoundValidationContext::new(&[]),
//! )?;
//! assert_eq!(outcome, ValidationOutcome::Valid);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Features
//!
//! No optional features are enabled by default. Local registrations remain
//! available without global discovery.
//!
//! | Feature | Adds |
//! | --- | --- |
//! | `inventory` | Built-in registrations for optional process-wide discovery. |
//! | `regex` | Full-string regex rules and their dynamic registration. |
//! | `china-identity` | The typed China identity structure rule, without dynamic registration. |
//! | `decimal` | Exact decimal rules and their dynamic registration. |
//! | `time` | Temporal resolution rules and their dynamic registration. |
//!
//! # User guides
//!
//! See the [English user guide](https://github.com/qubit-ltd/rs-validation-rules/blob/main/doc/user_guide.md)
//! or [简体中文用户手册](https://github.com/qubit-ltd/rs-validation-rules/blob/main/doc/user_guide.zh_CN.md)
//! for binding parameters, errors, resource limits, and application-policy
//! boundaries.

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod internal;
mod registry;

/// Collection validation rules.
pub mod collection;
/// Exact decimal validation rules.
#[cfg(feature = "decimal")]
pub mod decimal;
/// Mainland China identity-card validation rules.
#[cfg(feature = "china-identity")]
pub mod identity;
/// Stable identifiers for built-in validation rules.
pub mod ids;
/// Regular-expression validation rules.
#[cfg(feature = "regex")]
pub mod regex_rule;
/// Text validation rules.
pub mod text;
/// Temporal resolution validation rules.
#[cfg(feature = "time")]
pub mod time;

pub use registry::registrations;
