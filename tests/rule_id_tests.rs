// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::ids;
use qubit_validation_rules::registrations;

#[test]
fn public_rule_ids_match_explicit_registrations() {
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

    let mut registered = registrations()
        .into_iter()
        .map(|registration| registration.id().as_str())
        .collect::<Vec<_>>();
    expected.sort_unstable();
    registered.sort_unstable();

    assert_eq!(registered, expected);
    assert_eq!(
        registered.len(),
        registered
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len()
    );
}

#[test]
fn public_rule_ids_keep_their_protocol_values() {
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
}
