// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Typed validation rules and optional validator-registry adapters.

#![deny(missing_docs)]
#![deny(unsafe_code)]

mod internal;

/// Collection validation rules.
pub mod collection;
/// Mainland China identity-card validation rules.
#[cfg(feature = "china-identity")]
pub mod identity;
/// Regular-expression validation rules.
#[cfg(feature = "regex")]
pub mod regex_rule;
/// Text validation rules.
pub mod text;

pub use internal::legacy::registrations;
