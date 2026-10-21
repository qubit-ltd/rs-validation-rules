// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::text::AllowedChars;
use qubit_validation_rules::text::CharacterSet;
use qubit_validator::Validator;

#[test]
fn test_printable_unicode_matches_the_general_category_allowlist() {
    let rule = AllowedChars::new(CharacterSet::PrintableUnicode);

    for value in ["A", "中", "\u{0301}", "9", "!", "😀", " "] {
        assert!(rule.validate(value, &()).is_ok(), "{value:?}");
    }
    for value in ["\0", "\u{200B}", "\u{E000}", "\u{0378}", "\u{2028}", "\u{2029}"] {
        assert!(rule.validate(value, &()).is_err(), "{value:?}");
    }
}
