// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validator::ValidatorRegistration;

use super::collection;
#[cfg(feature = "decimal")]
use super::decimal;
#[cfg(feature = "regex")]
use super::regex;
use super::text;
#[cfg(feature = "time")]
use super::time;

/// Returns built-in rule registrations for a local validator registry.
/// Feature-gated rules are included when their features are enabled.
///
/// # Returns
/// Registrations in the crate's stable built-in rule set.
pub fn registrations() -> Vec<ValidatorRegistration> {
    #[allow(unused_mut)]
    let mut rules = vec![
        text::REG_NON_BLANK,
        text::REG_CHAR_LENGTH,
        text::REG_BYTE_LENGTH,
        text::REG_ALLOWED_CHARS,
        text::REG_EMAIL,
        text::REG_MATCHES_DEPENDENCY,
        text::REG_MOBILE,
        text::REG_URI,
        text::REG_UUID,
        collection::REG_COUNT,
    ];
    #[cfg(feature = "regex")]
    rules.push(regex::REG_REGEX);
    #[cfg(feature = "decimal")]
    rules.push(decimal::REG_DECIMAL);
    #[cfg(feature = "time")]
    rules.push(time::REG_TIME);
    rules
}
