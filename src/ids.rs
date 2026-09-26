// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable identifiers for built-in validation rules.

/// Stable identifier for the non-blank text rule.
pub const TEXT_NON_BLANK: &str = "qubit.rules.text.non_blank";
/// Stable identifier for the Unicode scalar-value text-length rule.
pub const TEXT_CHAR_LENGTH: &str = "qubit.rules.text.char_length";
/// Stable identifier for the UTF-8 byte-length rule.
pub const TEXT_BYTE_LENGTH: &str = "qubit.rules.text.byte_length";
/// Stable identifier for the allowed-character text rule.
pub const TEXT_ALLOWED_CHARS: &str = "qubit.rules.text.allowed_chars";
/// Stable identifier for the ASCII email text rule.
pub const TEXT_EMAIL_ASCII: &str = "qubit.rules.text.email_ascii";
/// Stable identifier for the text-to-dependency comparison rule.
pub const TEXT_MATCHES_DEPENDENCY: &str = "qubit.rules.text.matches_dependency";
/// Stable identifier for the mainland China mobile-number rule.
pub const TEXT_CHINA_MOBILE_STRUCTURE: &str = "qubit.rules.text.china_mobile_structure";
/// Stable identifier for the absolute URI rule.
pub const TEXT_URI: &str = "qubit.rules.text.uri";
/// Stable identifier for the canonical UUID text rule.
pub const TEXT_UUID: &str = "qubit.rules.text.uuid";
/// Stable identifier for the collection item-count rule.
pub const COLLECTION_ITEM_COUNT: &str = "qubit.rules.collection.item_count";
/// Stable identifier for the exact decimal value rule.
#[cfg(feature = "decimal")]
pub const DECIMAL_VALUE: &str = "qubit.rules.decimal.value";
/// Stable identifier for the temporal precision rule.
#[cfg(feature = "time")]
pub const TIME_PRECISION: &str = "qubit.rules.time.precision";
/// Stable identifier for the full-string regular-expression rule.
#[cfg(feature = "regex")]
pub const TEXT_REGEX: &str = "qubit.rules.text.regex";
