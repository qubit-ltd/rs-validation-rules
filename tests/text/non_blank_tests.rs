// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_validation_rules::text::NonBlank;
use qubit_validation_rules::text::NonBlankError;
use qubit_validator::Validator;

#[test]
fn test_non_blank_accepts_text_and_rejects_whitespace() {
    assert_eq!(NonBlank.validate(" text ", &()), Ok(()));
    assert_eq!(NonBlank.validate(" \t", &()), Err(NonBlankError::Blank));
}
