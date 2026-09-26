// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashSet;

use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;
#[cfg(feature = "inventory")]
use qubit_validator::ValidatorRegistry;

#[test]
fn test_public_rule_ids_match_explicit_registrations() {
    let mut expected = vec![
        ids::TEXT_NON_BLANK,
        ids::TEXT_CHAR_LENGTH,
        ids::TEXT_BYTE_LENGTH,
        ids::TEXT_ALLOWED_CHARS,
        ids::TEXT_EMAIL_ASCII,
        ids::TEXT_MATCHES_DEPENDENCY,
        ids::TEXT_CHINA_MOBILE_STRUCTURE,
        ids::TEXT_URI,
        ids::TEXT_UUID,
        ids::COLLECTION_ITEM_COUNT,
    ];
    #[cfg(feature = "regex")]
    expected.push(ids::TEXT_REGEX);
    #[cfg(feature = "decimal")]
    expected.push(ids::DECIMAL_VALUE);
    #[cfg(feature = "time")]
    expected.push(ids::TIME_PRECISION);

    let mut registered = registrations()
        .into_iter()
        .map(|registration| registration.id().as_str())
        .collect::<Vec<_>>();
    expected.sort_unstable();
    registered.sort_unstable();

    assert_eq!(registered, expected);
    assert_eq!(
        registered.len(),
        registered.iter().copied().collect::<HashSet<_>>().len()
    );
}

#[test]
fn test_public_rule_ids_keep_their_protocol_values() {
    assert_eq!(ids::TEXT_NON_BLANK, "qubit.rules.text.non_blank");
    assert_eq!(ids::TEXT_CHAR_LENGTH, "qubit.rules.text.char_length");
    assert_eq!(ids::TEXT_BYTE_LENGTH, "qubit.rules.text.byte_length");
    assert_eq!(ids::TEXT_ALLOWED_CHARS, "qubit.rules.text.allowed_chars");
    assert_eq!(ids::TEXT_EMAIL_ASCII, "qubit.rules.text.email_ascii");
    assert_eq!(ids::TEXT_MATCHES_DEPENDENCY, "qubit.rules.text.matches_dependency");
    assert_eq!(
        ids::TEXT_CHINA_MOBILE_STRUCTURE,
        "qubit.rules.text.china_mobile_structure"
    );
    assert_eq!(ids::TEXT_URI, "qubit.rules.text.uri");
    assert_eq!(ids::TEXT_UUID, "qubit.rules.text.uuid");
    assert_eq!(ids::COLLECTION_ITEM_COUNT, "qubit.rules.collection.item_count");
    #[cfg(feature = "regex")]
    assert_eq!(ids::TEXT_REGEX, "qubit.rules.text.regex");
    #[cfg(feature = "decimal")]
    assert_eq!(ids::DECIMAL_VALUE, "qubit.rules.decimal.value");
    #[cfg(feature = "time")]
    assert_eq!(ids::TIME_PRECISION, "qubit.rules.time.precision");
}

#[cfg(feature = "inventory")]
#[test]
fn test_public_rule_ids_match_inventory_registrations() {
    let mut explicit = registrations()
        .into_iter()
        .map(|registration| registration.id().as_str())
        .collect::<Vec<_>>();
    let mut discovered = ValidatorRegistry::global()
        .registrations()
        .iter()
        .map(|registration| registration.id().as_str())
        .collect::<Vec<_>>();
    explicit.sort_unstable();
    discovered.sort_unstable();
    assert_eq!(discovered, explicit);
}
